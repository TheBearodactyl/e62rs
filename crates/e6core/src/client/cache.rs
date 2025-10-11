use {
    crate::client::E6Client,
    anyhow::{Context, Result},
    serde::{Deserialize, Serialize},
    std::{
        collections::HashMap,
        sync::atomic::{AtomicU64, Ordering},
        time::{Instant, SystemTime, UNIX_EPOCH},
    },
    tracing::*,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheEntry {
    data: Vec<u8>,
    timestamp: u64,
    last_accessed: u64,
    etag: Option<String>,
    access_count: u64,
    compressed: bool,
}

#[derive(Debug, Default)]
pub struct CacheStats {
    pub hits: AtomicU64,
    pub misses: AtomicU64,
    pub evictions: AtomicU64,
    pub expired: AtomicU64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let total = hits + self.misses.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            (hits / total) as f64
        }
    }

    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.evictions.store(0, Ordering::Relaxed);
        self.expired.store(0, Ordering::Relaxed);
    }
}

impl E6Client {
    pub async fn get_cached_or_fetch(&self, url: &str) -> Result<Vec<u8>> {
        let cache_key = url.to_string();

        if self.cache_config.enabled.unwrap_or(true) {
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

            {
                let cache = self.cache.read().await;
                if let Some(entry) = cache.get(&cache_key) {
                    let ttl = self.cache_config.ttl_secs.unwrap_or_default();
                    let tti = self.cache_config.tti_secs.unwrap_or_default();

                    let age = now - entry.timestamp;
                    let idle = now - entry.last_accessed;

                    if age < ttl && idle < tti {
                        if self.cache_config.enable_stats.unwrap_or(true) {
                            self.cache_stats.hits.fetch_add(1, Ordering::Relaxed);
                        }

                        debug!("Cache hit for {} (age: {}s, idle: {}s)", url, age, idle);

                        drop(cache);
                        let mut cache = self.cache.write().await;
                        if let Some(entry) = cache.get_mut(&cache_key) {
                            entry.last_accessed = now;
                            entry.access_count += 1;
                        }

                        let data = cache.get(&cache_key).unwrap().data.clone();

                        if cache.get(&cache_key).unwrap().compressed {
                            return self.decompress_data(&data);
                        }

                        return Ok(data);
                    } else {
                        debug!(
                            "Cache entry expired for {} (age: {}s, idle: {}s)",
                            url, age, idle
                        );
                        if self.cache_config.enable_stats.unwrap_or(true) {
                            self.cache_stats.expired.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            }

            if self.cache_config.enable_stats.unwrap_or(true) {
                self.cache_stats.misses.fetch_add(1, Ordering::Relaxed);
            }
        }

        debug!("Cache miss, fetching from network: {}", url);
        let start = Instant::now();

        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to fetch from network")?;

        let status = response.status();
        if !status.is_success() {
            warn!("API returned error status: {} for {}", status, url);
            anyhow::bail!("API returned error status: {} for {}", status, url);
        }

        let etag = response
            .headers()
            .get("etag")
            .and_then(|h| h.to_str().ok())
            .map(String::from);

        let bytes = response
            .bytes()
            .await
            .context("Failed to read response body")?
            .to_vec();

        let elapsed = start.elapsed();
        debug!("Network fetch completed in {:?} for {}", elapsed, url);

        if self.cache_config.enabled.unwrap_or(true) {
            self.insert_into_cache(cache_key, bytes.clone(), etag)
                .await?;
        }

        Ok(bytes)
    }

    async fn insert_into_cache(
        &self,
        cache_key: String,
        data: Vec<u8>,
        etag: Option<String>,
    ) -> Result<()> {
        let mut cache = self.cache.write().await;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let should_compress = self.cache_config.enable_compression.unwrap_or(false);
        let (final_data, compressed) = if should_compress && data.len() > 1024 {
            match self.compress_data(&data) {
                Ok(compressed_data) => {
                    if compressed_data.len() < data.len() {
                        debug!(
                            "Compressed cache entry from {} to {} bytes",
                            data.len(),
                            compressed_data.len()
                        );
                        (compressed_data, true)
                    } else {
                        (data, false)
                    }
                }
                Err(e) => {
                    warn!("Compression failed: {}, storing uncompressed", e);
                    (data, false)
                }
            }
        } else {
            (data, false)
        };

        cache.insert(
            cache_key,
            CacheEntry {
                data: final_data,
                timestamp: now,
                last_accessed: now,
                etag,
                access_count: 0,
                compressed,
            },
        );

        let max_entries = self.cache_config.max_entries.unwrap_or(10_000);
        if cache.len() > max_entries {
            self.evict_entries(&mut cache, max_entries).await;
        }

        Ok(())
    }

    async fn evict_entries(&self, cache: &mut HashMap<String, CacheEntry>, target_size: usize) {
        let to_remove = cache.len() - (target_size * 3 / 4);

        if self.cache_config.use_lru_policy.unwrap_or(false) {
            let mut entries: Vec<_> = cache.iter().collect();
            entries.sort_by_key(|(_, entry)| entry.last_accessed);

            let keys_to_remove: Vec<String> = entries
                .iter()
                .take(to_remove)
                .map(|(k, _)| k.to_string())
                .collect();

            for key in keys_to_remove {
                cache.remove(&key);
            }
        } else {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let mut entries: Vec<_> = cache
                .iter()
                .map(|(k, entry)| {
                    let recency_score = 1.0 / (1.0 + (now - entry.last_accessed) as f64);
                    let frequency_score = (entry.access_count as f64).log2().max(0.0);
                    let score = recency_score * 0.7 + frequency_score * 0.3;
                    (k.clone(), score)
                })
                .collect();

            entries.sort_by(|(_, score_a), (_, score_b)| {
                score_a
                    .partial_cmp(score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let keys_to_remove: Vec<String> = entries
                .iter()
                .take(to_remove)
                .map(|(k, _)| k.clone())
                .collect();

            for key in keys_to_remove {
                cache.remove(&key);
            }
        }

        if self.cache_config.enable_stats.unwrap_or(true) {
            self.cache_stats
                .evictions
                .fetch_add(to_remove as u64, Ordering::Relaxed);
        }

        debug!("Evicted {} cache entries", to_remove);
    }

    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use std::io::Write;

        let level = self.cache_config.compression_level.unwrap_or(6);
        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level as u32));
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        if self.cache_config.enable_stats.unwrap_or(true) {
            self.cache_stats.reset();
        }
        info!("Cache cleared");
    }

    pub async fn get_cache_stats(&self) -> (usize, u64) {
        let cache = self.cache.read().await;
        let size = cache.len();
        let total_bytes: u64 = cache.values().map(|entry| entry.data.len() as u64).sum();
        (size, total_bytes)
    }

    pub async fn get_detailed_cache_stats(&self) -> String {
        let (size, bytes) = self.get_cache_stats().await;

        if self.cache_config.enable_stats.unwrap_or(true) {
            let hits = self.cache_stats.hits.load(Ordering::Relaxed);
            let misses = self.cache_stats.misses.load(Ordering::Relaxed);
            let evictions = self.cache_stats.evictions.load(Ordering::Relaxed);
            let expired = self.cache_stats.expired.load(Ordering::Relaxed);
            let hit_rate = self.cache_stats.hit_rate();

            format!(
                "HTTP Cache Statistics:\n\
                 - Entries: {}\n\
                 - Size: {:.2} MB\n\
                 - Hits: {}\n\
                 - Misses: {}\n\
                 - Hit Rate: {:.2}%\n\
                 - Evictions: {}\n\
                 - Expired: {}",
                size,
                bytes as f64 / (1024.0 * 1024.0),
                hits,
                misses,
                hit_rate * 100.0,
                evictions,
                expired
            )
        } else {
            format!(
                "HTTP Cache: {} entries, {:.2} MB",
                size,
                bytes as f64 / (1024.0 * 1024.0)
            )
        }
    }

    pub async fn clear_all_caches(&self) -> Result<()> {
        self.clear_cache().await;
        self.clear_post_cache().await?;
        info!("All caches cleared");
        Ok(())
    }

    pub async fn get_all_cache_stats(&self) -> String {
        let http_stats = self.get_detailed_cache_stats().await;
        let post_stats = self
            .get_post_cache_stats()
            .await
            .unwrap_or_else(|_| "Post Cache: unavailable".to_string());

        format!("{}\n\n{}", http_stats, post_stats)
    }

    pub async fn cleanup_expired_entries(&self) -> Result<()> {
        if !self.cache_config.enabled.unwrap_or(true) {
            return Ok(());
        }

        let mut cache = self.cache.write().await;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let ttl = self.cache_config.ttl_secs.unwrap_or(3600);
        let tti = self.cache_config.tti_secs.unwrap_or(1800);

        let keys_to_remove: Vec<String> = cache
            .iter()
            .filter(|(_, entry)| {
                let age = now - entry.timestamp;
                let idle = now - entry.last_accessed;
                age >= ttl || idle >= tti
            })
            .map(|(k, _)| k.clone())
            .collect();

        let count = keys_to_remove.len();
        for key in keys_to_remove {
            cache.remove(&key);
        }

        if count > 0 {
            if self.cache_config.enable_stats.unwrap_or(true) {
                self.cache_stats
                    .expired
                    .fetch_add(count as u64, Ordering::Relaxed);
            }
            debug!("Cleaned up {} expired cache entries", count);
        }

        Ok(())
    }
}

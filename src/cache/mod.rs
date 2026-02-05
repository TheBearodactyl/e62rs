//! post and http cache stuff
use {
    crate::{bail, cache::posts::CacheEntry, client::E6Client, error::*, getopt},
    color_eyre::eyre::Context,
    flate2::{Compression, read::GzDecoder, write::GzEncoder},
    hashbrown::HashMap,
    std::{
        io::{Read, Write},
        sync::atomic::Ordering,
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    },
    tracing::{debug, info, warn},
};

pub mod posts;
pub mod stats;

impl E6Client {
    /// get an entry from the cache, fetch if no entry found
    ///
    /// # Arguments
    ///
    /// * `url` - the url of the post to get
    ///
    /// # Errors
    ///
    /// returns an error if it fails to fetch the post
    ///
    /// # Returns
    ///
    /// * `Err(color_eyre::Report)` - an error describing why it failed
    /// * `Ok(Vec<u8>)` - the data of the cache entry or fetched post
    pub async fn get_cached_or_fetch(&self, url: &str) -> Result<Vec<u8>> {
        let cache_key = url.to_string();

        if getopt!(cache.enabled) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .wrap_err("couldn't get current time")
                .map_err(Report::new)
                .unwrap_or(0);

            let cached_data = {
                let cache = self.cache.read().await;

                cache.get(&cache_key).and_then(|entry| {
                    let ttl = getopt!(cache.ttl_secs);
                    let tti = getopt!(cache.tti_secs);
                    let age = now.saturating_sub(entry.timestamp);
                    let idle = now.saturating_sub(entry.last_accessed);

                    if age < ttl && idle < tti {
                        if getopt!(cache.enable_stats) {
                            self.cache_stats.hits.fetch_add(1, Ordering::Relaxed);
                        }

                        debug!("cache hit for {} (age: {}s, idle: {}s)", url, age, idle);
                        Some((entry.data.clone(), entry.compressed))
                    } else {
                        debug!(
                            "cache entry expired for {} (age: {}s, idle: {}s)",
                            url, age, idle
                        );

                        if getopt!(cache.enable_stats) {
                            self.cache_stats.expired.fetch_add(1, Ordering::Relaxed);
                        }

                        None
                    }
                })
            };

            if let Some((data, compressed)) = cached_data {
                {
                    let mut cache = self.cache.write().await;

                    if let Some(entry) = cache.get_mut(&cache_key) {
                        entry.last_accessed = now;
                        entry.access_count = entry.access_count.saturating_add(1);
                    }
                }

                return if compressed {
                    self.decompress_data(&data)
                } else {
                    Ok(data)
                };
            }

            if getopt!(cache.enable_stats) {
                self.cache_stats.misses.fetch_add(1, Ordering::Relaxed);
            }
        }

        debug!("cache miss, fetching: {}", url);

        let start = Instant::now();
        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("failed to fetch")?;
        let status = response.status();

        if !status.is_success() {
            warn!("API returned error status: {} for {}", status, url);
            bail!("API returned error status: {} for {}", status, url);
        }

        let etag = response
            .headers()
            .get("etag")
            .and_then(|h| h.to_str().ok())
            .map(String::from);
        let bytes = response
            .bytes()
            .await
            .context("failed to read response body")?
            .to_vec();
        let elapsed = start.elapsed();

        debug!("network fetch completed in {:?} for {}", elapsed, url);

        if getopt!(cache.enabled)
            && let Err(e) = self.insert_into_cache(cache_key, bytes.clone(), etag).await
        {
            warn!("failed to insert entry into cache: {}", e);
        }

        Ok(bytes)
    }

    /// make a new cache entry
    ///
    /// # Arguments
    ///
    /// * `cache_key` - the key of the new entry to insert into the cache
    /// * `data` - the data of the new entry
    /// * `etag` - an optional etag for the new entry
    async fn insert_into_cache(
        &self,
        cache_key: String,
        data: Vec<u8>,
        etag: Option<String>,
    ) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let should_compress = getopt!(cache.enable_compression);
        let (final_data, compressed) = if should_compress && data.len() > 1024 {
            match self.compress_data(&data) {
                Ok(compressed_data) if compressed_data.len() < data.len() => {
                    debug!(
                        "Compressed cache entry from {} to {} bytes",
                        data.len(),
                        compressed_data.len()
                    );
                    (compressed_data, true)
                }
                Ok(_) => (data, false),
                Err(e) => {
                    warn!("Compression failed: {}, storing uncompressed", e);
                    (data, false)
                }
            }
        } else {
            (data, false)
        };

        let entry = CacheEntry {
            data: final_data,
            timestamp: now,
            last_accessed: now,
            etag,
            access_count: 0,
            compressed,
        };

        let mut cache = self.cache.write().await;

        cache.insert(cache_key, entry);

        let max_entries = getopt!(cache.max_entries);

        if cache.len() > max_entries {
            self.evict_entries_inner(&mut cache, max_entries);
        }

        Ok(())
    }

    /// evict n entries from the cache
    ///
    /// # Arguments
    ///
    /// * `cache` - the current state of the cache (mutable)
    /// * `target_size` - the size to attempt to achieve by evicting entries
    fn evict_entries_inner(&self, cache: &mut HashMap<String, CacheEntry>, target_size: usize) {
        let to_remove = cache.len().saturating_sub(target_size * 3 / 4);

        if to_remove == 0 {
            return;
        }

        let keys_to_remove: Vec<String> = if getopt!(cache.use_lru_policy) {
            let mut entries: Vec<_> = cache.iter().collect();

            entries.sort_by_key(|(_, entry)| entry.last_accessed);
            entries
                .iter()
                .take(to_remove)
                .map(|(k, _)| (*k).clone())
                .collect()
        } else {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let mut entries: Vec<_> = cache
                .iter()
                .map(|(k, entry)| {
                    let age = now.saturating_sub(entry.last_accessed) as f64;
                    let recency_score = 1.0 / (1.0 + age);
                    let frequency_score = (entry.access_count as f64 + 1.0).log2();
                    let score = recency_score * 0.7 + frequency_score * 0.3;
                    (k.clone(), score)
                })
                .collect();

            entries.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            entries
                .into_iter()
                .take(to_remove)
                .map(|(k, _)| k)
                .collect()
        };

        for key in &keys_to_remove {
            cache.remove(key);
        }

        if getopt!(cache.enable_stats) {
            self.cache_stats
                .evictions
                .fetch_add(keys_to_remove.len() as u64, Ordering::Relaxed);
        }

        debug!("Evicted {} cache entries", keys_to_remove.len());
    }

    /// compress bytes
    ///
    /// # Arguments
    ///
    /// * `data` - the data to compress
    ///
    /// # Errors
    ///
    /// returns an error if it fails to move the given data into a compressor
    /// returns an error if it fails to compress the given data
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        let level = getopt!(cache.compression_level);
        let level = level.clamp(0, 9) as u32;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));
        encoder
            .write_all(data)
            .context("Failed to write data to compressor")?;
        encoder
            .finish()
            .context("Failed to finish compression")
            .map_err(Report::new)
    }

    /// decompress bytes
    ///
    /// # Arguments
    ///
    /// * `data` - the data to decompress
    ///
    /// # Errors
    ///
    /// returns an error if it fails to decompress (obviously lol)
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .context("Failed to decompress data")?;
        Ok(decompressed)
    }

    /// clear the cache
    ///
    /// removes all entries from the cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();

        if getopt!(cache.enable_stats) {
            self.cache_stats.reset();
        }

        info!("Cache cleared");
        drop(cache);
    }

    /// get stats for the http cache
    pub async fn get_cache_stats(&self) -> (usize, u64) {
        let cache = self.cache.read().await;
        let size = cache.len();
        let total_bytes: u64 = cache.values().map(|entry| entry.data.len() as u64).sum();
        drop(cache);

        (size, total_bytes)
    }

    /// get detailed stats about the http cache
    pub async fn get_detailed_cache_stats(&self) -> String {
        let (size, bytes) = self.get_cache_stats().await;

        if getopt!(cache.enable_stats) {
            let hits = self.cache_stats.hits.load(Ordering::Relaxed);
            let misses = self.cache_stats.misses.load(Ordering::Relaxed);
            let evictions = self.cache_stats.evictions.load(Ordering::Relaxed);
            let expired = self.cache_stats.expired.load(Ordering::Relaxed);
            let hit_rate = self.cache_stats.hit_rate();

            format!(
                "HTTP Cache Statistics:\n- Entries: {}\n- Size: {:.2} MB\n- Hits: {}\n- Misses: \
                 {}\n- Hit Rate: {:.2}%\n- Evictions: {}\n- Expired: {}",
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

    /// clear the http and post caches
    ///
    /// # Errors
    ///
    /// returns an error if it fails to clear the post cache
    pub async fn clear_all_caches(&self) -> Result<()> {
        self.clear_cache().await;
        self.post_cache.clear().await?;
        info!("All caches cleared");
        Ok(())
    }

    /// get the stats for the http and post caches
    ///
    /// # Errors
    ///
    /// returns an error if it fails to get the stats of the post cache
    pub async fn get_all_cache_stats(&self) -> Result<String> {
        let http_stats = self.get_detailed_cache_stats().await;
        let post_stats = self
            .post_cache
            .get_stats()
            .await
            .context("failed to get post cache stats")?;

        Ok(format!("{}\n\n{}", http_stats, post_stats))
    }

    /// cleanup expired entries from the cache
    ///
    /// # Errors
    ///
    /// returns an error if it fails to get the system time in seconds
    pub async fn cleanup_expired_entries(&self) -> Result<usize> {
        if !getopt!(cache.enabled) {
            return Ok(0);
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|a| Duration::as_secs(&a))
            .map_err(Report::new)?;
        let ttl = getopt!(cache.ttl_secs);
        let tti = getopt!(cache.tti_secs);
        let mut cache = self.cache.write().await;
        let keys_to_remove: Vec<String> = cache
            .iter()
            .filter(|(_, entry)| {
                let age = now.saturating_sub(entry.timestamp);
                let idle = now.saturating_sub(entry.last_accessed);
                age >= ttl || idle >= tti
            })
            .map(|(k, _)| k.clone())
            .collect();
        let count = keys_to_remove.len();

        for key in keys_to_remove {
            cache.remove(&key);
        }

        drop(cache);

        if count > 0 {
            if getopt!(cache.enable_stats) {
                self.cache_stats
                    .expired
                    .fetch_add(count as u64, Ordering::Relaxed);
            }

            debug!("Cleaned up {} expired cache entries", count);
        }

        Ok(count)
    }
}

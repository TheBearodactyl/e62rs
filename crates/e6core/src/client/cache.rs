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

        if self.cache_config.enabled {
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

            {
                let cache = self.cache.read().await;
                if let Some(entry) = cache.get(&cache_key) {
                    let ttl = self.cache_config.ttl_secs;
                    let tti = self.cache_config.tti_secs;

                    let age = now - entry.timestamp;
                    let idle = now - entry.last_accessed;

                    if age < ttl && idle < tti {
                        if self.cache_config.enable_stats {
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
                        if self.cache_config.enable_stats {
                            self.cache_stats.expired.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            }

            if self.cache_config.enable_stats {
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

        if self.cache_config.enabled {
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

        let should_compress = self.cache_config.enable_compression;
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

        let max_entries = self.cache_config.max_entries;
        if cache.len() > max_entries {
            self.evict_entries(&mut cache, max_entries).await;
        }

        Ok(())
    }

    async fn evict_entries(&self, cache: &mut HashMap<String, CacheEntry>, target_size: usize) {
        let to_remove = cache.len() - (target_size * 3 / 4);

        if self.cache_config.use_lru_policy {
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

        if self.cache_config.enable_stats {
            self.cache_stats
                .evictions
                .fetch_add(to_remove as u64, Ordering::Relaxed);
        }

        debug!("Evicted {} cache entries", to_remove);
    }

    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        use {
            flate2::{Compression, write::GzEncoder},
            std::io::Write,
        };

        let level = self.cache_config.compression_level;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level as u32));
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        use {flate2::read::GzDecoder, std::io::Read};

        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        if self.cache_config.enable_stats {
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

        if self.cache_config.enable_stats {
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
        if !self.cache_config.enabled {
            return Ok(());
        }

        let mut cache = self.cache.write().await;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let ttl = self.cache_config.ttl_secs;
        let tti = self.cache_config.tti_secs;

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
            if self.cache_config.enable_stats {
                self.cache_stats
                    .expired
                    .fetch_add(count as u64, Ordering::Relaxed);
            }
            debug!("Cleaned up {} expired cache entries", count);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use {super::*, e6cfg::CacheConfig};

    fn create_test_client(name: &str) -> E6Client {
        let cache_dir = format!(".cache_test_{}", name);
        let mut client = E6Client::new("https://e621.net").expect("Failed to create test client");
        client.cache_config = CacheConfig {
            cache_dir,
            ..Default::default()
        };

        client
    }

    fn create_test_config(enabled: bool) -> CacheConfig {
        let mut cfg = CacheConfig::default();
        cfg.enabled = enabled;
        cfg
    }

    #[test]
    fn test_cache_entry_creation() {
        let entry = CacheEntry {
            data: vec![1, 2, 3, 4],
            timestamp: 1234567890,
            last_accessed: 1234567890,
            etag: Some("test-etag".to_string()),
            access_count: 0,
            compressed: false,
        };

        assert_eq!(entry.data.len(), 4);
        assert_eq!(entry.etag, Some("test-etag".to_string()));
        assert!(!entry.compressed);
    }

    #[test]
    fn test_cache_stats_default() {
        let stats = CacheStats::default();
        assert_eq!(stats.hits.load(Ordering::Relaxed), 0);
        assert_eq!(stats.misses.load(Ordering::Relaxed), 0);
        assert_eq!(stats.evictions.load(Ordering::Relaxed), 0);
        assert_eq!(stats.expired.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_cache_stats_hit_rate_empty() {
        let stats = CacheStats::default();
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_cache_stats_reset() {
        let stats = CacheStats::default();
        stats.hits.store(10, Ordering::Relaxed);
        stats.misses.store(5, Ordering::Relaxed);
        stats.evictions.store(2, Ordering::Relaxed);
        stats.expired.store(3, Ordering::Relaxed);

        stats.reset();

        assert_eq!(stats.hits.load(Ordering::Relaxed), 0);
        assert_eq!(stats.misses.load(Ordering::Relaxed), 0);
        assert_eq!(stats.evictions.load(Ordering::Relaxed), 0);
        assert_eq!(stats.expired.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let client = create_test_client("clear_cache");

        let cache_key = "test_key".to_string();
        let data = vec![1, 2, 3, 4, 5];
        client
            .insert_into_cache(cache_key.clone(), data, None)
            .await
            .unwrap();

        let (size, _) = client.get_cache_stats().await;
        assert!(size > 0);

        client.clear_cache().await;

        let (size, _) = client.get_cache_stats().await;
        assert_eq!(size, 0);
    }

    #[tokio::test]
    async fn test_get_cache_stats() {
        let client = create_test_client("get_cache_stats");

        let data1 = vec![1; 100];
        let data2 = vec![2; 200];

        client
            .insert_into_cache("key1".to_string(), data1, None)
            .await
            .unwrap();
        client
            .insert_into_cache("key2".to_string(), data2, None)
            .await
            .unwrap();

        let (size, bytes) = client.get_cache_stats().await;
        assert_eq!(size, 2);
        assert_eq!(bytes, 300);
    }

    #[tokio::test]
    async fn test_get_detailed_cache_stats() {
        let client = create_test_client("get_detailed_cache_stats");

        let stats = client.get_detailed_cache_stats().await;
        assert!(stats.contains("HTTP Cache Statistics"));
        assert!(stats.contains("Entries:"));
        assert!(stats.contains("Size:"));
    }

    #[tokio::test]
    async fn test_compress_data() {
        let client = create_test_client("compress_data");
        let data = vec![0u8; 2048];

        let compressed = client.compress_data(&data).unwrap();
        assert!(compressed.len() < data.len());
    }

    #[tokio::test]
    async fn test_decompress_data() {
        let client = create_test_client("decompress_data");
        let original_data = b"Hello, World! This is test data.".to_vec();

        let compressed = client.compress_data(&original_data).unwrap();
        let decompressed = client.decompress_data(&compressed).unwrap();

        assert_eq!(original_data, decompressed);
    }

    #[tokio::test]
    async fn test_compress_decompress_roundtrip() {
        let client = create_test_client("compress_decompress_roundtrip");
        let original = vec![42u8; 5000];

        let compressed = client.compress_data(&original).unwrap();
        let decompressed = client.decompress_data(&compressed).unwrap();

        assert_eq!(original, decompressed);
    }

    #[tokio::test]
    async fn test_insert_into_cache_without_compression() {
        let client = create_test_client("insert_into_cache_without_compression");
        let data = vec![1, 2, 3];

        client
            .insert_into_cache(
                "test".to_string(),
                data.clone(),
                Some("etag123".to_string()),
            )
            .await
            .unwrap();

        let cache = client.cache.read().await;
        let entry = cache.get("test").unwrap();
        assert_eq!(entry.data, data);
        assert_eq!(entry.etag, Some("etag123".to_string()));
        assert!(!entry.compressed);
    }

    #[tokio::test]
    async fn test_cleanup_expired_entries() {
        let client = create_test_client("cleanup_expired_entries");

        client
            .insert_into_cache("test".to_string(), vec![1, 2, 3], None)
            .await
            .unwrap();

        {
            let mut cache = client.cache.write().await;
            if let Some(entry) = cache.get_mut("test") {
                entry.timestamp = 0;
                entry.last_accessed = 0;
            }
        }

        client.cleanup_expired_entries().await.unwrap();

        let cache = client.cache.read().await;
        assert!(cache.get("test").is_none());
    }

    #[tokio::test]
    async fn test_cleanup_expired_entries_respects_ttl() {
        let client = create_test_client("cleanup_expired_entries_respects_ttl");

        client
            .insert_into_cache("recent".to_string(), vec![1, 2, 3], None)
            .await
            .unwrap();

        client.cleanup_expired_entries().await.unwrap();

        let cache = client.cache.read().await;
        assert!(cache.get("recent").is_some());
    }

    #[tokio::test]
    async fn test_evict_entries_lru_policy() {
        let client = create_test_client("evict_entries_lru_policy");
        let mut cache_config = create_test_config(true);
        cache_config.use_lru_policy = true;

        let mut cache = HashMap::new();

        for i in 0..5 {
            cache.insert(
                format!("key{}", i),
                CacheEntry {
                    data: vec![i as u8],
                    timestamp: 1000,
                    last_accessed: 1000 + i * 100,
                    etag: None,
                    access_count: 1,
                    compressed: false,
                },
            );
        }

        client.evict_entries(&mut cache, 3).await;

        assert!(cache.len() <= 4);
    }

    #[tokio::test]
    async fn test_clear_all_caches() {
        let client = create_test_client("clear_all_caches");

        client
            .insert_into_cache("http_key".to_string(), vec![1, 2, 3], None)
            .await
            .unwrap();

        client.clear_all_caches().await.unwrap();

        let (size, _) = client.get_cache_stats().await;
        assert_eq!(size, 0);
    }

    #[tokio::test]
    async fn test_get_all_cache_stats() {
        let client = create_test_client("get_all_cache_stats");

        let stats = client.get_all_cache_stats().await;
        assert!(stats.contains("HTTP Cache"));
        assert!(stats.contains("Post Cache"));
    }

    #[test]
    fn test_cache_entry_serialization() {
        let entry = CacheEntry {
            data: vec![1, 2, 3],
            timestamp: 123456,
            last_accessed: 123456,
            etag: Some("test".to_string()),
            access_count: 5,
            compressed: true,
        };

        let serialized = serde_json::to_string(&entry).unwrap();
        let deserialized: CacheEntry = serde_json::from_str(&serialized).unwrap();

        assert_eq!(entry.data, deserialized.data);
        assert_eq!(entry.timestamp, deserialized.timestamp);
        assert_eq!(entry.compressed, deserialized.compressed);
    }

    #[tokio::test]
    async fn test_cache_max_entries_eviction() {
        let client = create_test_client("cache_max_entries_evicition");

        for i in 0..150 {
            client
                .insert_into_cache(format!("key{}", i), vec![i as u8], None)
                .await
                .unwrap();
        }

        let (size, _) = client.get_cache_stats().await;
        assert!(size < 150);
    }
}

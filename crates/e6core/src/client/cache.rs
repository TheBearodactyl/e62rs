use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};

use crate::client::E6Client;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheEntry {
    data: Vec<u8>,
    timestamp: u64,
    etag: Option<String>,
}

impl E6Client {
    pub async fn get_cached_or_fetch(&self, url: &str) -> Result<Vec<u8>> {
        let cache_key = url.to_string();

        if self.cache_config.enabled.unwrap_or(true) {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(&cache_key) {
                let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
                let ttl = self.cache_config.ttl_secs.unwrap_or(3600);

                if now - entry.timestamp < ttl {
                    debug!("Cache hit for {}", url);
                    return Ok(entry.data.clone());
                }
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
            let mut cache = self.cache.write().await;
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

            cache.insert(
                cache_key,
                CacheEntry {
                    data: bytes.clone(),
                    timestamp: now,
                    etag,
                },
            );

            if cache.len() > 1000 {
                let mut entries: Vec<_> = cache.iter().collect();
                entries.sort_by_key(|(_, entry)| entry.timestamp);
                let to_remove = entries.len() / 4;
                let keys_to_remove: Vec<String> = entries
                    .iter()
                    .take(to_remove)
                    .map(|(k, _)| k.to_string())
                    .collect();

                for key in keys_to_remove {
                    cache.remove(&key);
                }
                debug!("Cleaned up {} old cache entries", to_remove);
            }
        }

        Ok(bytes)
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("Cache cleared");
    }

    pub async fn get_cache_stats(&self) -> (usize, u64) {
        let cache = self.cache.read().await;
        let size = cache.len();
        let total_bytes: u64 = cache.values().map(|entry| entry.data.len() as u64).sum();
        (size, total_bytes)
    }
}

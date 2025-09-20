use crate::config::{get_config, CacheConfig};
use crate::models::{E6PostResponse, E6PostsResponse};
use anyhow::{Context, Result};
use futures::future::join_all;
use log::{debug, info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

const BASE_URL: &str = "https://e621.net";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_LIMIT: u64 = 20;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CacheEntry {
    data: Vec<u8>,
    timestamp: u64,
    etag: Option<String>,
}

#[derive(Debug, Clone)]
pub struct E6Client {
    client: Client,
    base_url: String,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    cache_config: CacheConfig,
}

impl Default for E6Client {
    fn default() -> Self {
        Self::new(BASE_URL).expect("Failed to create default E6Client")
    }
}

impl E6Client {
    pub fn new(base_url: &str) -> Result<Self> {
        let config = get_config().unwrap_or_default();
        let http_config = config.http.unwrap_or_default();
        let cache_config = config.cache.unwrap_or_default();

        let mut client_builder = Client::builder()
            .user_agent(
                http_config
                    .user_agent
                    .as_deref()
                    .unwrap_or(crate::USER_AGENT),
            )
            .timeout(Duration::from_secs(http_config.timeout_secs.unwrap_or(30)))
            .connect_timeout(Duration::from_secs(
                http_config.connect_timeout_secs.unwrap_or(10),
            ))
            .pool_max_idle_per_host(http_config.pool_max_idle_per_host.unwrap_or(32))
            .pool_idle_timeout(Duration::from_secs(
                http_config.pool_idle_timeout_secs.unwrap_or(90),
            ))
            .http2_prior_knowledge()
            .tcp_keepalive(Duration::from_secs(60));

        if http_config.tcp_keepalive.unwrap_or(true) {
            client_builder = client_builder.tcp_keepalive(Duration::from_secs(60));
        }

        let client = client_builder
            .build()
            .context("Failed to build HTTP client")?;

        info!(
            "Initialized HTTP client with {} max idle connections per host",
            http_config.pool_max_idle_per_host.unwrap_or(32)
        );

        Ok(Self {
            client,
            base_url: base_url.to_string(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_config,
        })
    }

    pub fn with_client(client: Client, base_url: &str) -> Self {
        let config = get_config().unwrap_or_default();
        let cache_config = config.cache.unwrap_or_default();

        Self {
            client,
            base_url: base_url.to_string(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_config,
        }
    }

    async fn get_cached_or_fetch(&self, url: &str) -> Result<Vec<u8>> {
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
                let to_remove = entries.len() / 4; // Remove oldest 25%
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

    pub async fn get_latest_posts(&self) -> Result<E6PostsResponse> {
        let url = format!("{}/posts.json", self.base_url);
        let bytes = self.get_cached_or_fetch(&url).await?;

        let posts: E6PostsResponse =
            serde_json::from_slice(&bytes).context("Failed to deserialize posts response")?;

        debug!("Successfully fetched {} posts", posts.posts.len());
        Ok(posts)
    }

    pub async fn search_posts(
        &self,
        tags: Vec<String>,
        limit: Option<u64>,
    ) -> Result<E6PostsResponse> {
        let url = format!("{}/posts.json", self.base_url);
        let limit = limit.unwrap_or(DEFAULT_LIMIT);

        let query_url = format!(
            "{}?tags={}&limit={}",
            url,
            urlencoding::encode(&tags.join(" ")),
            limit
        );

        let bytes = self.get_cached_or_fetch(&query_url).await?;

        let posts: E6PostsResponse =
            serde_json::from_slice(&bytes).context("Failed to deserialize search response")?;

        debug!(
            "Successfully searched and found {} posts",
            posts.posts.len()
        );
        Ok(posts)
    }

    pub async fn get_post_by_id(&self, id: i64) -> Result<E6PostResponse> {
        let url = format!("{}/posts/{}.json", self.base_url, id);
        let bytes = self.get_cached_or_fetch(&url).await?;

        let post: E6PostResponse = serde_json::from_slice(&bytes)
            .with_context(|| format!("Failed to deserialize post {}", id))?;

        Ok(post)
    }

    pub async fn get_posts_by_ids(&self, ids: Vec<i64>) -> Result<Vec<E6PostResponse>> {
        let config = get_config().unwrap_or_default();
        let concurrent_limit = config
            .performance
            .as_ref()
            .and_then(|p| p.concurrent_downloads)
            .unwrap_or(8);

        let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrent_limit));
        let futures: Vec<_> = ids
            .into_iter()
            .map(|id| {
                let client = self.clone();
                let semaphore = semaphore.clone();

                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    client.get_post_by_id(id).await
                })
            })
            .collect();

        let results = join_all(futures).await;
        let mut posts = Vec::new();

        for result in results {
            match result {
                Ok(Ok(post)) => posts.push(post),
                Ok(Err(e)) => warn!("Failed to fetch post: {}", e),
                Err(e) => warn!("Task failed: {}", e),
            }
        }

        Ok(posts)
    }

    async fn execute_with_retry<T, F, Fut>(&self, max_retries: u32, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts <= max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    attempts += 1;

                    if attempts <= max_retries {
                        let delay = Duration::from_millis(200 * 2_u64.pow(attempts.min(5)));
                        debug!("Retry attempt {} after {:?}", attempts, delay);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Retry failed without error")))
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

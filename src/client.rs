use crate::config::{CacheConfig, HttpConfig, get_config};
use crate::models::{E6PoolResponse, E6PoolsResponse, E6PostResponse, E6PostsResponse};
use anyhow::{Context, Result, bail};
use chrono::{Datelike, Local, Utc};
use flate2::read::GzDecoder;
use futures::future::join_all;
use log::{debug, info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio::io::AsyncWriteExt;
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
    disk_cache_path: Option<std::path::PathBuf>,
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

        let client = Self::build_http_client(&http_config)?;

        let disk_cache_path = if cache_config.enabled.unwrap_or(true) {
            let cache_dir = cache_config.cache_dir.as_deref().unwrap_or(".cache");
            let path = std::path::PathBuf::from(cache_dir);

            std::fs::create_dir_all(&path)
                .with_context(|| format!("Failed to create cache directory: {:?}", path))?;

            Some(path)
        } else {
            None
        };

        info!(
            "Initialized HTTP client with {} max connections",
            http_config.max_connections.unwrap_or(100)
        );

        Ok(Self {
            client,
            base_url: base_url.to_string(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_config,
            disk_cache_path,
        })
    }

    fn build_http_client(http_config: &HttpConfig) -> Result<Client> {
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
            ));

        if http_config.http2_prior_knowledge.unwrap_or(true) {
            client_builder = client_builder.http2_prior_knowledge();
        }

        if http_config.tcp_keepalive.unwrap_or(true) {
            client_builder = client_builder.tcp_keepalive(Duration::from_secs(60));
        }

        client_builder
            .http1_only()
            .build()
            .context("Failed to build HTTP client")
    }

    pub fn with_client(client: Client, base_url: &str) -> Self {
        let config = get_config().unwrap_or_default();
        let cache_config = config.cache.unwrap_or_default();

        Self {
            client,
            base_url: base_url.to_string(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_config,
            disk_cache_path: None,
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

    async fn fetch_from_network(&self, url: &str) -> Result<Vec<u8>> {
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

        let bytes = response
            .bytes()
            .await
            .context("Failed to read response body")?
            .to_vec();

        let elapsed = start.elapsed();
        debug!("Network fetch completed in {:?} for {}", elapsed, url);

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

    pub async fn update_tags(&self) -> Result<()> {
        let local_file: &str = "data/tags.csv";
        let local_hash_file: &str = "data/tags.csv.hash";

        let now = Local::now();
        let url = format!(
            "https://e621.net/db_export/tags-{:04}-{:02}-{:02}.csv.gz",
            now.year(),
            now.month(),
            now.day()
        );

        info!("{}", url);

        let response = self.client.get(&url).send().await?;
        if !response.status().clone().is_success() {
            bail!("Failed to download tags: {}", response.status());
        }

        let mut remote_bytes = response.bytes().await?;
        let mut hasher = Sha256::new();

        hasher.update(&remote_bytes);

        let remote_hash = hasher.finalize();
        let remote_hash_hex = hex::encode(remote_hash);

        let update_needed = if Path::new(local_hash_file).exists() {
            let local_hash_hex = fs::read_to_string(local_hash_file).await?;
            local_hash_hex.trim() != remote_hash_hex
        } else {
            true
        };

        if update_needed {
            info!("Updating local tags snapshot...");

            let mut gz = GzDecoder::new(&remote_bytes[..]);
            let mut decompressed_data = Vec::new();
            gz.read_to_end(&mut decompressed_data)?;

            fs::create_dir_all("data").await?;
            let mut file = fs::File::create(local_file).await?;
            file.write_all(&decompressed_data).await?;

            let mut hash_file = fs::File::create(local_hash_file).await?;
            hash_file.write_all(remote_hash_hex.as_bytes()).await?;

            info!("Updated local tags snapshot at {}", local_file);
        } else {
            info!("Local snapshot of `tags.csv` is up to date, continuing");
        }

        Ok(())
    }

    pub async fn search_pools(&self, query: String, limit: Option<u64>) -> Result<E6PoolsResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT);

        let query_url = format!(
            "{}/pools.json?search[name_matches]={}&limit={}",
            self.base_url,
            urlencoding::encode(&query),
            limit
        );

        debug!("Searching pools with URL: {}", query_url);
        let bytes = self.get_cached_or_fetch(&query_url).await?;

        let pools: E6PoolsResponse =
            serde_json::from_slice(&bytes).context("Failed to deserialize pool search response")?;

        debug!(
            "Successfully searched and found {} pools",
            pools.pools.len()
        );
        Ok(pools)
    }

    pub async fn search_pools_by_description(
        &self,
        query: String,
        limit: Option<u64>,
    ) -> Result<E6PoolsResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT);

        let query_url = format!(
            "{}/pools.json?search[description_matches]={}&limit={}",
            self.base_url,
            urlencoding::encode(&query),
            limit
        );

        debug!("Searching pools by description with URL: {}", query_url);
        let bytes = self.get_cached_or_fetch(&query_url).await?;

        let pools: E6PoolsResponse =
            serde_json::from_slice(&bytes).context("Failed to deserialize pool search response")?;

        Ok(pools)
    }

    pub async fn search_pools_by_creator(
        &self,
        creator_name: String,
        limit: Option<u64>,
    ) -> Result<E6PoolsResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT);

        let query_url = format!(
            "{}/pools.json?search[creator_name]={}&limit={}",
            self.base_url,
            urlencoding::encode(&creator_name),
            limit
        );

        debug!("Searching pools by creator with URL: {}", query_url);
        let bytes = self.get_cached_or_fetch(&query_url).await?;

        let pools: E6PoolsResponse =
            serde_json::from_slice(&bytes).context("Failed to deserialize pool search response")?;

        Ok(pools)
    }

    pub async fn get_pool_by_id(&self, id: i64) -> Result<E6PoolResponse> {
        let url = format!("{}/pools/{}.json", self.base_url, id);
        let bytes = self.get_cached_or_fetch(&url).await?;

        let pool: E6PoolResponse = serde_json::from_slice(&bytes)
            .with_context(|| format!("Failed to deserialize pool {}", id))?;

        Ok(pool)
    }

    pub async fn get_pool_posts(&self, pool_id: i64) -> Result<E6PostsResponse> {
        let url = format!("{}/posts.json?tags=pool:{}", self.base_url, pool_id);
        let bytes = self.get_cached_or_fetch(&url).await?;

        let posts: E6PostsResponse =
            serde_json::from_slice(&bytes).context("Failed to deserialize pool posts response")?;

        debug!(
            "Successfully fetched {} posts from pool {}",
            posts.posts.len(),
            pool_id
        );
        Ok(posts)
    }

    pub async fn update_pools(&self) -> Result<()> {
        let local_file: &str = "data/pools.csv";
        let local_hash_file: &str = "data/pools.csv.hash";

        let now = Local::now();
        let url = format!(
            "https://e621.net/db_export/pools-{:04}-{:02}-{:02}.csv.gz",
            now.year(),
            now.month(),
            now.day()
        );

        let response = self.client.get(&url).send().await?;
        if !response.status().clone().is_success() {
            bail!("Failed to download pools: {}", response.status());
        }

        let remote_bytes = response.bytes().await?;
        let mut hasher = Sha256::new();

        hasher.update(&remote_bytes);

        let remote_hash = hasher.finalize();
        let remote_hash_hex = hex::encode(remote_hash);

        let update_needed = if Path::new(local_hash_file).exists() {
            let local_hash_hex = fs::read_to_string(local_hash_file).await?;
            local_hash_hex.trim() != remote_hash_hex
        } else {
            true
        };

        if update_needed {
            info!("Updating local pools snapshot...");

            let mut gz = GzDecoder::new(&remote_bytes[..]);
            let mut decompressed_data = Vec::new();
            gz.read_to_end(&mut decompressed_data)?;

            fs::create_dir_all("data").await?;
            let mut file = fs::File::create(local_file).await?;
            file.write_all(&decompressed_data).await?;

            let mut hash_file = fs::File::create(local_hash_file).await?;
            hash_file.write_all(remote_hash_hex.as_bytes()).await?;

            info!("Updated local pools snapshot at {}", local_file);
        } else {
            info!("Local snapshot of `pools.csv` is up to date, continuing");
        }

        Ok(())
    }

    pub async fn get_pools(&self, limit: Option<u64>) -> Result<E6PoolsResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT);
        let url = format!("{}/pools.json?limit={}", self.base_url, limit);
        let bytes = self.get_cached_or_fetch(&url).await?;

        let pools: E6PoolsResponse =
            serde_json::from_slice(&bytes).context("Failed to deserialize pools response")?;

        debug!("Successfully fetched {} pools", pools.pools.len());
        Ok(pools)
    }
}

use crate::{client::cache::CacheEntry, utils::create_auth_header};
use anyhow::{Context, Result};
use e6cfg::{CacheConfig, E62Rs, HttpConfig};
use reqwest::Client;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tracing::*;

pub mod cache;
pub mod pools;
pub mod post_cache;
pub mod posts;

use cache::CacheStats;
use post_cache::PostCache;

const DEFAULT_LIMIT: u64 = 20;

#[derive(Debug, Clone)]
pub struct E6Client {
    client: Client,
    base_url: String,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    cache_config: CacheConfig,
    cache_stats: Arc<CacheStats>,
    _disk_cache_path: Option<std::path::PathBuf>,
    post_cache: Arc<PostCache>,
}

impl Default for E6Client {
    fn default() -> Self {
        let cfg = E62Rs::get().unwrap_or_default();
        Self::new(&cfg.base_url.unwrap_or_default()).expect("Failed to create default E6Client")
    }
}

impl E6Client {
    pub fn new(base_url: &str) -> Result<Self> {
        let cfg = E62Rs::get().unwrap_or_default();
        let http_config = cfg.http.unwrap_or_default();
        let cache_config = cfg.cache.unwrap_or_default();
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

        let post_cache = PostCache::new(
            cache_config.cache_dir.as_deref().unwrap_or(".cache"),
            &cache_config,
        )?;

        info!(
            "Initialized HTTP client with {} max connections",
            http_config.max_connections.unwrap_or(2)
        );

        let client = Self {
            client,
            base_url: base_url.to_string(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_config: cache_config.clone(),
            cache_stats: Arc::new(CacheStats::default()),
            _disk_cache_path: disk_cache_path,
            post_cache: Arc::new(post_cache),
        };

        if cache_config.enabled.unwrap_or(true) {
            let cleanup_interval = cache_config.cleanup_interval_secs.unwrap_or(300);
            let client_clone = client.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(cleanup_interval));
                loop {
                    interval.tick().await;
                    if let Err(e) = client_clone.cleanup_expired_entries().await {
                        warn!("Cache cleanup failed: {}", e);
                    }
                }
            });
        }

        Ok(client)
    }

    fn build_http_client(http_config: &HttpConfig) -> Result<Client> {
        let login_cfg = E62Rs::get().unwrap_or_default().login;
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

        if let Some(login_creds) = login_cfg {
            client_builder = client_builder.default_headers(create_auth_header(&login_creds)?);
        }

        if http_config.tcp_keepalive.unwrap_or(true) {
            client_builder = client_builder.tcp_keepalive(Duration::from_secs(60));
        }

        client_builder
            .http1_only()
            .build()
            .context("Failed to build HTTP client")
    }

    pub async fn execute_with_retry<T, F, Fut>(&self, max_retries: u32, operation: F) -> Result<T>
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
}

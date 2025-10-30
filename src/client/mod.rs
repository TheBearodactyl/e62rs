use {
    crate::{
        cache::{CacheEntry, posts::PostCache, stats::CacheStats},
        config::{
            messages::{config::LogMessageConfig, context::TemplateContext},
            options::{CacheConfig, E62Rs, HttpConfig},
        },
        utils::create_auth_header,
    },
    color_eyre::eyre::{Context, Result, eyre},
    reqwest::Client,
    std::{collections::HashMap, sync::Arc, time::Duration},
    strum::{Display, VariantArray, VariantNames},
    tokio::sync::RwLock,
    tracing::{debug, info, warn},
};

pub mod pools;
pub mod posts;

#[derive(
    Display, Default, VariantNames, VariantArray, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum E621Base {
    #[default]
    #[strum(to_string = "https://e621.net")]
    E621,
    #[strum(to_string = "https://e926.net")]
    E926,
}

#[derive(Debug, Clone)]
pub struct E6Client {
    pub client: Client,
    pub update_client: Client,
    pub base_url: E621Base,
    pub cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    pub cache_config: CacheConfig,
    pub cache_stats: Arc<CacheStats>,
    pub post_cache: Arc<PostCache>,
}

impl Default for E6Client {
    fn default() -> Self {
        Self::new(E621Base::default()).expect("Failed to build default client")
    }
}

impl E6Client {
    pub fn new(base_url: E621Base) -> Result<Self> {
        let cfg = E62Rs::get()?;
        let http_config = cfg.http;
        let cache_config = cfg.cache;
        let client = Self::build_http_client(&http_config, false)?;
        let update_client = Self::build_http_client(&http_config, true)?;
        let post_cache = PostCache::new(&cache_config.cache_dir, &cache_config)?;

        info!(
            "Initialized HTTP client with {} max connections",
            http_config.max_connections
        );

        let client = Self {
            client,
            update_client,
            base_url: E621Base::default(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_config: cache_config.clone(),
            cache_stats: Arc::new(CacheStats::default()),
            post_cache: Arc::new(post_cache),
        };

        if cache_config.enabled {
            let cleanup_interval = cache_config.cleanup_interval_secs;
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

    pub fn get_base_url(&self) -> String {
        self.base_url.to_string()
    }

    fn build_http_client(http_config: &HttpConfig, autoupdate_mode: bool) -> Result<Client> {
        let login_cfg = E62Rs::get()?.login;
        let mut client_builder = Client::builder()
            .user_agent(http_config.clone().user_agent)
            .timeout(Duration::from_secs(http_config.timeout_secs))
            .connect_timeout(Duration::from_secs(http_config.connect_timeout_secs))
            .pool_max_idle_per_host(http_config.pool_max_idle_per_host)
            .pool_idle_timeout(Duration::from_secs(http_config.pool_idle_timeout_secs));

        if http_config.http2_prior_knowledge {
            client_builder = client_builder.http2_prior_knowledge();
        }

        if login_cfg.login {
            client_builder = client_builder.default_headers(create_auth_header(&login_cfg)?);
        }

        if http_config.tcp_keepalive {
            client_builder =
                client_builder.tcp_keepalive(Duration::from_secs(http_config.tcp_keepalive_secs));
        }

        if autoupdate_mode {
            client_builder = client_builder.http1_only();
        }

        client_builder
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

        Err(last_error.unwrap_or_else(|| eyre!("Retry failed without error")))
    }
}

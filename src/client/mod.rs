//! e621 api stuff
use {
    crate::{
        cache::{
            posts::{CacheEntry, PostCache},
            stats::CacheStats,
        },
        config::options::CacheConfig,
        error::*,
    },
    color_eyre::eyre::Context,
    hashbrown::HashMap,
    reqwest::Client,
    std::{sync::Arc, time::Duration},
    tokio::sync::RwLock,
    tracing::{debug, info, warn},
};

pub mod pools;
pub mod posts;

/// configuration for constructing an [`E6Client`]
#[derive(Clone, Debug)]
pub struct E6ClientConfig {
    /// the base url for api requests
    pub base_url: String,
    /// the user agent string
    pub user_agent: String,
    /// request timeout in seconds
    pub timeout: u64,
    /// connection timeout in seconds
    pub connect_timeout: u64,
    /// max idle connections per host
    pub pool_max_idle_per_host: usize,
    /// pool idle timeout in seconds
    pub pool_idle_timeout: u64,
    /// enable http2
    pub http2: bool,
    /// enable tcp keepalive
    pub tcp_keepalive: bool,
    /// tcp keepalive interval in seconds
    pub tcp_keepalive_secs: u64,
    /// optional login credentials (username, api_key)
    pub login: Option<(String, String)>,
    /// cache configuration
    pub cache_config: CacheConfig,
}

impl Default for E6ClientConfig {
    fn default() -> Self {
        let defaults = CacheConfig::default();
        Self {
            base_url: "https://e621.net".to_string(),
            user_agent: format!(
                "{}/v{} (by {} on e621)",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                "bearodactyl"
            ),
            timeout: 30,
            connect_timeout: 10,
            pool_max_idle_per_host: 32,
            pool_idle_timeout: 90,
            http2: false,
            tcp_keepalive: true,
            tcp_keepalive_secs: 60,
            login: None,
            cache_config: defaults,
        }
    }
}

/// the client
#[derive(Clone, Debug)]
pub struct E6Client {
    /// the http client
    pub client: Client,
    /// the base url for api requests
    pub base_url: String,
    /// the http cache
    pub cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// the cache configuration
    pub cache_config: CacheConfig,
    /// the stats for the cache
    pub cache_stats: Arc<CacheStats>,
    /// the post cache
    pub post_cache: Arc<PostCache>,
}

impl Default for E6Client {
    fn default() -> Self {
        Self::with_config(E6ClientConfig::default()).expect("failed to init e6 client")
    }
}

impl E6Client {
    /// make a new e621 api client from explicit configuration
    pub fn with_config(config: E6ClientConfig) -> Result<Self> {
        let client = Self::build_http_client_with(&config)?;

        let cache_dir = config.cache_config.cache_dir.clone()
            .unwrap_or_else(|| ".cache".to_string());
        let posts_enabled = config.cache_config.posts.as_ref()
            .and_then(|p| p.enabled)
            .unwrap_or(true);
        let max_size_mb = config.cache_config.max_size_mb.unwrap_or(500);
        let max_posts = config.cache_config.posts.as_ref()
            .and_then(|p| p.max_posts)
            .unwrap_or(50000000);
        let auto_compact = config.cache_config.posts.as_ref()
            .and_then(|p| p.auto_compact)
            .unwrap_or(true);
        let compact_threshold = config.cache_config.posts.as_ref()
            .and_then(|p| p.compact_threshold)
            .unwrap_or(25);

        let post_cache = PostCache::with_config(
            &cache_dir,
            posts_enabled,
            max_size_mb,
            max_posts,
            auto_compact,
            compact_threshold,
        )?;

        info!("initialized http client");

        let cache_enabled = config.cache_config.enabled.unwrap_or(true);
        let cleanup_int = config.cache_config.cleanup_interval.unwrap_or(300);

        let client = Self {
            client,
            base_url: config.base_url,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_config: config.cache_config,
            cache_stats: Arc::new(CacheStats::default()),
            post_cache: Arc::new(post_cache),
        };

        if cache_enabled {
            let client_clone = client.clone();

            tokio::spawn(async move {
                let mut int = tokio::time::interval(Duration::from_secs(cleanup_int));

                loop {
                    int.tick().await;
                    if let Err(e) = client_clone.cleanup_expired_entries().await {
                        warn!(error = %e, "cache cleanup failed");
                    }
                }
            });
        }

        Ok(client)
    }

    /// make a new e621 api client from the loaded configuration
    #[cfg(feature = "cli")]
    pub fn new() -> Result<Self> {
        use crate::getopt;

        let config = E6ClientConfig {
            base_url: "https://e621.net".to_string(),
            user_agent: getopt!(http.user_agent),
            timeout: getopt!(http.timeout),
            connect_timeout: getopt!(http.connect_timeout),
            pool_max_idle_per_host: getopt!(http.pool_max_idle_per_host),
            pool_idle_timeout: getopt!(http.pool_idle_timeout),
            http2: getopt!(http.http2),
            tcp_keepalive: getopt!(http.tcp_keepalive),
            tcp_keepalive_secs: getopt!(http.tcp_keepalive_secs),
            login: if getopt!(login.login) {
                Some((getopt!(login.username), getopt!(login.api_key)))
            } else {
                None
            },
            cache_config: getopt!(cache).clone(),
        };

        Self::with_config(config)
    }

    /// build an http client from explicit configuration
    fn build_http_client_with(config: &E6ClientConfig) -> Result<Client> {
        let mut client_builder = Client::builder()
            .user_agent(&config.user_agent)
            .timeout(Duration::from_secs(config.timeout))
            .connect_timeout(Duration::from_secs(config.connect_timeout))
            .pool_max_idle_per_host(config.pool_max_idle_per_host)
            .pool_idle_timeout(Duration::from_secs(config.pool_idle_timeout));

        if config.http2 {
            client_builder = client_builder.http2_prior_knowledge();
        }

        if let Some((ref username, ref api_key)) = config.login {
            client_builder = client_builder
                .default_headers(crate::utils::create_auth_header(username, api_key)?);
        }

        if config.tcp_keepalive {
            client_builder = client_builder
                .tcp_keepalive(Duration::from_secs(config.tcp_keepalive_secs));
        }

        if !config.http2 {
            client_builder = client_builder.http1_only();
        }

        client_builder
            .build()
            .context("failed to build http client")
            .map_err(Report::new)
    }

    /// run an operation, retrying n times
    pub async fn execute_with_retry<T, F, Fut>(&self, max_retries: u32, op: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempts = 0;
        let mut last_err = None;

        while attempts <= max_retries {
            match op().await {
                Ok(res) => return Ok(res),
                Err(e) => {
                    last_err = Some(e);
                    attempts += 1;

                    if attempts <= max_retries {
                        let delay = Duration::from_millis(200 * 2_u64.pow(attempts.min(5)));
                        debug!("retry attempt {} after {:?}", attempts, delay);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_err.unwrap_or_else(|| E6Error::from("retry failed without error".to_string())))
    }
}

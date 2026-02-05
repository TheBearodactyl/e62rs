//! e621 api stuff
use {
    crate::{
        cache::{
            posts::{CacheEntry, PostCache},
            stats::CacheStats,
        },
        config::options::CacheConfig,
        error::*,
        getopt, opt_and,
        utils::create_auth_header,
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
        Self::new().expect("failed to init e6 client")
    }
}

impl E6Client {
    /// make a new e621 api client
    pub fn new() -> Result<Self> {
        let client = Self::build_http_client()?;
        let post_cache = PostCache::new(&getopt!(cache.cache_dir))?;

        info!(
            "initialized http client ({} max connections)",
            getopt!(http.max_connections)
        );

        let client = Self {
            client,
            base_url: "https://e621.net".to_string(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_config: getopt!(cache).clone(),
            cache_stats: Arc::new(CacheStats::default()),
            post_cache: Arc::new(post_cache),
        };

        if getopt!(cache.enabled) {
            let cleanup_int = getopt!(cache.cleanup_interval);
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

    /// build an http client based on the loaded configuration
    fn build_http_client() -> Result<Client> {
        let mut client_builder = Client::builder()
            .user_agent(getopt!(http.user_agent))
            .timeout(Duration::from_secs(getopt!(http.timeout)))
            .connect_timeout(Duration::from_secs(getopt!(http.connect_timeout)))
            .pool_max_idle_per_host(getopt!(http.pool_max_idle_per_host))
            .pool_idle_timeout(Duration::from_secs(getopt!(http.pool_idle_timeout)));

        opt_and!(
            http.http2,
            client_builder = client_builder.http2_prior_knowledge()
        );

        opt_and!(
            login.login,
            client_builder = client_builder.default_headers(create_auth_header()?)
        );

        opt_and!(
            http.tcp_keepalive,
            client_builder =
                client_builder.tcp_keepalive(Duration::from_secs(getopt!(http.tcp_keepalive_secs)))
        );

        if !getopt!(http.http2) {
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

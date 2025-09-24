use {
    anyhow::{Context, Result},
    config::Config,
    serde::{Deserialize, Serialize},
    std::{
        fs::{self},
        path::Path,
    },
};

pub mod blacklist;
pub mod defaults;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HttpConfig {
    /// Connection pool size per host
    pub pool_max_idle_per_host: Option<usize>,
    /// Connection pool idle timeout in seconds
    pub pool_idle_timeout_secs: Option<u64>,
    /// Request timeout in seconds
    pub timeout_secs: Option<u64>,
    /// Connection timeout in seconds
    pub connect_timeout_secs: Option<u64>,
    /// Max concurrent connections
    pub max_connections: Option<usize>,
    /// Enable HTTP/2
    pub http2_prior_knowledge: Option<bool>,
    /// Enable keep-alive
    pub tcp_keepalive: Option<bool>,
    /// User agent string
    pub user_agent: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CacheConfig {
    /// Enable response caching
    pub enabled: Option<bool>,
    /// Cache directory
    pub cache_dir: Option<String>,
    /// Cache TTL in seconds
    pub ttl_secs: Option<u64>,
    /// Max cache size in MB
    pub max_size_mb: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PerformanceConfig {
    /// Number of concurrent downloads
    pub concurrent_downloads: Option<usize>,
    /// Prefetch next batch of posts
    pub prefetch_enabled: Option<bool>,
    /// Prefetch batch size
    pub prefetch_batch_size: Option<usize>,
    /// Enable image preloading
    pub preload_images: Option<bool>,
    /// Max image preload size in MB
    pub max_preload_size_mb: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UiConfig {
    /// Progress bar refresh rate (Hz)
    pub progress_refresh_rate: Option<u64>,
    /// Show detailed progress info
    pub detailed_progress: Option<bool>,
    /// Auto-clear completed progress bars
    pub auto_clear_progress: Option<bool>,
    /// Pagination size for post listings
    pub pagination_size: Option<usize>,
    /// Enable colored output
    pub colored_output: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ImageDisplay {
    /// The max width of displayed images
    pub width: Option<u64>,
    /// The max height of displayed images
    pub height: Option<u64>,
    /// Whether to display the image when showing post info
    pub image_when_info: Option<bool>,
    /// Image quality for sixel conversion (1-100)
    pub sixel_quality: Option<u8>,
    /// Resize method (nearest, linear, cubic, gaussian, lanczos3)
    pub resize_method: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchCfg {
    /// The minimum amount of posts on a tag for it to show up in tag selection
    pub min_posts_on_tag: Option<u64>,
    /// The minimum amount of posts on a pool for it to show up in pool selection
    pub min_posts_on_pool: Option<u64>,
    /// Whether or not to show inactive pools
    pub show_inactive_pools: Option<bool>,
    /// Whether or not to sort pools by how many posts they contain
    pub sort_pools_by_post_count: Option<bool>,
    /// Whether or not to sort tags by their post count
    pub sort_tags_by_post_count: Option<bool>,
    /// The minimum score a post should have to show up in search
    pub min_post_score: Option<i64>,
    /// The maximum score a post should have to show up in search
    pub max_post_score: Option<i64>,
    /// Sort tags in reverse alphabetic order
    pub reverse_tags_order: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CompletionCfg {
    /// The similarity threshold to complete a tag
    pub tag_similarity_threshold: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Cfg {
    /// The directory to download posts to
    pub download_dir: Option<String>,
    /// The output format for downloaded files
    pub output_format: Option<String>,
    /// The amount of posts to show in a search
    pub post_count: Option<u64>,
    /// The base URL of the API (defaults to https://e621.net)
    pub base_url: Option<String>,
    /// Post viewing settings
    pub display: Option<ImageDisplay>,
    /// The path to `tags.csv` that's used for tag searching/autocompletion
    pub tags: Option<String>,
    /// The path to `pools.csv` that's used for pool searching/autocompletion
    pub pools: Option<String>,
    /// HTTP client configuration
    pub http: Option<HttpConfig>,
    /// Cache configuration
    pub cache: Option<CacheConfig>,
    /// Performance settings
    pub performance: Option<PerformanceConfig>,
    /// UI settings
    pub ui: Option<UiConfig>,
    /// Search settings
    pub search: Option<SearchCfg>,
    /// Completion Settings
    pub completion: Option<CompletionCfg>,
    /// Blacklisted tags to filter out from all operations
    pub blacklist: Option<Vec<String>>,
}

fn get_config_dir() -> String {
    match dirs::config_dir() {
        Some(path) => path.to_string_lossy().into_owned(),
        None => format!(
            "{}/.config/",
            std::env::var("HOME").expect("Failed to get home dir")
        ),
    }
}

fn find_local_config_file() -> Option<String> {
    let extensions = ["toml", "yaml", "yml", "json"];

    for ext in &extensions {
        let filename = format!("e62rs.{}", ext);
        if Path::new(&filename).exists() {
            return Some(filename);
        }
    }

    None
}

impl Cfg {
    pub fn get() -> Result<Self> {
        let global_config_path = format!("{}/e62rs.toml", get_config_dir());

        let mut builder = Config::builder();

        builder = builder.add_source(config::File::with_name(&global_config_path).required(false));

        if let Some(local_config) = find_local_config_file() {
            let local_config_name = local_config
                .strip_suffix(&format!(
                    ".{}",
                    local_config.split('.').next_back().unwrap_or("toml")
                ))
                .unwrap_or(&local_config);

            builder =
                builder.add_source(config::File::with_name(local_config_name).required(false));
        }

        builder = builder.add_source(config::Environment::with_prefix("E62RS"));

        let settings = builder.build()?;

        let mut cfg = settings
            .try_deserialize::<Cfg>()
            .unwrap_or_else(|_| Cfg::default());

        if cfg.http.is_none() {
            cfg.http = Some(HttpConfig::default());
        }

        if cfg.cache.is_none() {
            cfg.cache = Some(CacheConfig::default());
        }

        if cfg.performance.is_none() {
            cfg.performance = Some(PerformanceConfig::default());
        }

        if cfg.ui.is_none() {
            cfg.ui = Some(UiConfig::default());
        }

        if cfg.display.is_none() {
            cfg.display = Some(ImageDisplay::default());
        }

        if cfg.search.is_none() {
            cfg.search = Some(SearchCfg::default());
        }

        if cfg.blacklist.is_none() {
            cfg.blacklist = Some(
                vec!["young", "rape", "feral", "bestiality"]
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>(),
            );
        }

        if !Path::new(&global_config_path).exists() && find_local_config_file().is_none() {
            log::info!(
                "Creating default configuration file at {}",
                global_config_path
            );
            if let Err(e) = cfg.save_to_file(&global_config_path) {
                log::warn!("Failed to create default config file: {}", e);
            }
        }

        Ok(cfg)
    }

    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let toml_str =
            toml::to_string_pretty(self).context("Failed to serialize config to TOML")?;

        fs::write(path, toml_str)
            .with_context(|| format!("Failed to write configuration to file: {}", path))?;

        Ok(())
    }
}

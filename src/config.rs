use config::Config;
use serde::{Deserialize, Serialize};
use std::time::Duration;

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
    /// HTTP client configuration
    pub http: Option<HttpConfig>,
    /// Cache configuration
    pub cache: Option<CacheConfig>,
    /// Performance settings
    pub performance: Option<PerformanceConfig>,
    /// UI settings
    pub ui: Option<UiConfig>,
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            download_dir: Some("downloads".to_string()),
            output_format: Some("$id.$ext".to_string()),
            post_count: Some(32),
            base_url: Some("https://e621.net".to_string()),
            display: Some(ImageDisplay::default()),
            tags: Some("data/tags.csv".to_string()),
            http: Some(HttpConfig::default()),
            cache: Some(CacheConfig::default()),
            performance: Some(PerformanceConfig::default()),
            ui: Some(UiConfig::default()),
        }
    }
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            pool_max_idle_per_host: Some(32),
            pool_idle_timeout_secs: Some(90),
            timeout_secs: Some(30),
            connect_timeout_secs: Some(10),
            max_connections: Some(100),
            http2_prior_knowledge: Some(true),
            tcp_keepalive: Some(true),
            user_agent: None,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: Some(true),
            cache_dir: Some(".cache".to_string()),
            ttl_secs: Some(3600),
            max_size_mb: Some(500),
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            concurrent_downloads: Some(8),
            prefetch_enabled: Some(true),
            prefetch_batch_size: Some(10),
            preload_images: Some(false),
            max_preload_size_mb: Some(100),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            progress_refresh_rate: Some(20),
            detailed_progress: Some(true),
            auto_clear_progress: Some(true),
            pagination_size: Some(20),
            colored_output: Some(true),
        }
    }
}

impl Default for ImageDisplay {
    fn default() -> Self {
        Self {
            width: Some(800),
            height: Some(600),
            image_when_info: Some(false),
            sixel_quality: Some(75),
            resize_method: Some("lanczos3".to_string()),
        }
    }
}

pub fn get_config() -> anyhow::Result<Cfg> {
    let settings = Config::builder()
        .add_source(config::File::with_name("e62rs").required(false))
        .add_source(config::Environment::with_prefix("E62RS"))
        .build()?;

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

    Ok(cfg)
}

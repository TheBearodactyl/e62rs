use {
    anyhow::{Context, Result},
    config::Config,
    schemars::JsonSchema,
    serde::{Deserialize, Serialize},
    std::{
        fs::{self},
        path::Path,
    },
};

pub mod blacklist;
pub mod defaults;

//static PATTERN_REGEX: &str = r"^[^/]+/(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:-(?:(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?:[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))? \(by [a-zA-Z0-9]{7,20} on e(?:621|926)\)$";

static PATTERN_REGEX: &str = "";

/// Configuration options for making HTTP requests
#[derive(Serialize, Deserialize, Clone, Copy, Debug, JsonSchema, Default)]
#[schemars(bound = "T: JsonSchema + Default")]
pub enum SizeFormat {
    /// Show download progress in bits
    Bits,

    /// Show download progress in bytes
    Bytes,

    /// Show download progress in kilobytes
    KiloBytes,

    /// Show download progress in kilobits
    KiloBits,

    /// Show download progress in megabytes
    #[default]
    MegaBytes,

    /// Show download progress in megabits
    MegaBits,
}

impl SizeFormat {
    pub fn format_size(&self, bytes: u64) -> String {
        let bits = bytes * 8;
        match self {
            SizeFormat::Bits => format!("{} b", bits),
            SizeFormat::Bytes => format!("{} B", bytes),
            SizeFormat::KiloBits => format!("{:.1} Kb", bits as f64 / 1000.0),
            SizeFormat::KiloBytes => format!("{:.1} KB", bytes as f64 / 1024.0),
            SizeFormat::MegaBits => format!("{:.2} Mb", bits as f64 / 1_000_000.0),
            SizeFormat::MegaBytes => format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0)),
        }
    }
}

/// Configuration options for making HTTP requests
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct HttpConfig {
    /// Connection pool size per host
    pub pool_max_idle_per_host: Option<usize>,

    /// Connection pool idle timeout in seconds
    pub pool_idle_timeout_secs: Option<u64>,

    /// Request timeout in seconds
    pub timeout_secs: Option<u64>,

    /// Connection timeout in seconds
    pub connect_timeout_secs: Option<u64>,

    #[schemars(range(min = 1, max = 15))]
    /// Max concurrent connections
    pub max_connections: Option<usize>,

    /// Enable HTTP/2
    pub http2_prior_knowledge: Option<bool>,

    /// Enable keep-alive
    pub tcp_keepalive: Option<bool>,

    #[schemars(inner(regex(pattern = PATTERN_REGEX)))]
    /// User agent string in the format:
    /// `<project name>/<project version> (by <valid e6 username> on <e621/e926>)`
    ///
    /// Some examples of valid User Agents:
    /// - `my-project/1.2.3 (by username123 on e621)`
    /// - `another/2.0.0-beta.1 (by user7890 on e926)`
    /// - `test-proj/0.1.0+build.123 (by myuser12345 on e621)`
    pub user_agent: Option<String>,
}

/// Configuration options for the response cache
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
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

/// Configuration options for performance
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct PerformanceConfig {
    #[schemars(range(min = 1, max = 15))]
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

/// Configuration options for the UI
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
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

/// Configuration options for displaying images
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
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

/// Configuration options for searching posts/pools
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
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

    /// The number of threads to use when fetching post data
    pub fetch_threads: Option<usize>,
}

/// Configuration options for completion in menus
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct CompletionCfg {
    /// The similarity threshold to complete a tag
    pub tag_similarity_threshold: Option<f64>,
}

/// Your login credentials
#[derive(Default, Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct LoginCfg {
    /// Your username
    pub username: String,

    /// Your API key
    pub api_key: String,
}

/// Settings for automatically updating data snapshots
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct AutoUpdateCfg {
    /// Whether or not to auto-update tags
    pub tags: Option<bool>,
    /// Whether or not to auto-update pools
    pub pools: Option<bool>,
}

/// Configuration options for general stuff
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct Cfg {
    /// The directory to download posts to
    pub download_dir: Option<String>,

    /// The output format for downloaded files
    ///
    /// Valid placeholders are as follows:
    /// - `$artists[N]`: The first N artists
    /// - `$tags[N]`: The first N general tags
    /// - `$id`: The ID of the post
    /// - `$rating`: The full content rating of the post (safe, questionable, explicit)
    /// - `$rating_first`: The first letter of the posts content rating (s, q, e)
    /// - `$score`: The total score of the post
    /// - `$fav_count`: The amount of people who favorited the post
    /// - `$comment_count`: The amount of comments on the post
    /// - `$md5`: The MD5 hash of the post file
    /// - `$ext`: The file extension of the post
    /// - `$width`: The original width of the post media in pixels
    /// - `$height`: The original height of the post media in pixels
    /// - `$size`: The file size of the post media
    /// - `$artist`: The first listed artist of the post
    /// - `$uploader`: The name of the person who uploaded the post
    /// - `$uploader_id`: The user ID of the person who uploaded the post
    /// - `$year`: The year when the post was uploaded
    /// - `$month`: The month when the post was uploaded
    /// - `$day`: The day when the post was uploaded
    /// - `$hour`: The hour the post was uploaded at
    /// - `$minute`: The minute the post was uploaded at
    /// - `$second`: The second the post was uploaded at
    /// - `$date`: Shorthand for "$year-$month-$day"
    /// - `$time`: Shorthand for "$hour-$minute-$second"
    /// - `$datetime`: Shorthand for "$year-$month-$day $hour-$minute-$second"
    /// - `$now_year`: The year at the time of downloading the post
    /// - `$now_month`: The month at the time of downloading the post
    /// - `$now_day`: The day at the time of downloading the post
    /// - `$now_hour`: The hour of the day at the time of downloading the post
    /// - `$now_minute`: The minute of the hour at the time of downloading the post
    /// - `$now_second`: The second of the minute at the time of downloading the post
    /// - `$now_date`: Shorthand for "$now_year-$now_month-$now_day"
    /// - `$now_time`: Shorthand for "$now_hour-$now_minute-$now_second"
    /// - `$now_datetime`: Shorthand for "$now_year-$now_month-$now_day $now_hour-$now_minute-$now_second"
    pub output_format: Option<String>,

    /// The format to display download progress in
    pub progress_format: Option<SizeFormat>,

    /// The amount of posts to show in a search
    pub post_count: Option<u64>,

    #[schemars(url)]
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

    /// Login settings
    pub login: Option<LoginCfg>,

    /// Completion settings
    pub completion: Option<CompletionCfg>,

    /// Autoupdate settings
    pub autoupdate: Option<AutoUpdateCfg>,

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

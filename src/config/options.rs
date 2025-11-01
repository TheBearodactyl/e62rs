use {
    color_eyre::eyre::{Context, Result},
    config::Config,
    schemars::JsonSchema,
    serde::{Deserialize, Serialize},
    smart_default::SmartDefault,
    std::{fs, path::Path},
    tracing::{info, warn},
};

/// Configuration options for making HTTP requests
#[derive(Serialize, Deserialize, Clone, Copy, Debug, JsonSchema, SmartDefault)]
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
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
pub struct HttpConfig {
    /// Connection pool size per host
    #[default(32)]
    pub pool_max_idle_per_host: usize,

    /// Connection pool idle timeout in seconds
    #[default(90)]
    pub pool_idle_timeout_secs: u64,

    /// Request timeout in seconds
    #[default(30)]
    pub timeout_secs: u64,

    /// Connection timeout in seconds
    #[default(10)]
    pub connect_timeout_secs: u64,

    #[schemars(range(min = 1, max = 15))]
    /// Max concurrent connections
    #[default(15)]
    pub max_connections: usize,

    /// Enable HTTP/2
    #[default(true)]
    pub http2_prior_knowledge: bool,

    /// Enable keep-alive
    #[default(true)]
    pub tcp_keepalive: bool,

    /// How many seconds to keep TCP alive for
    #[default(60)]
    pub tcp_keepalive_secs: u64,

    /// User agent string in the format:
    /// `<project name>/<project version> (by <valid e6 username> on <e621/e926>)`
    ///
    /// Some examples of valid User Agents:
    /// - `my-project/1.2.3 (by username123 on e621)`
    /// - `another/2.0.0-beta.1 (by user7890 on e926)`
    /// - `test-proj/0.1.0+build.123 (by myuser12345 on e621)`
    #[default(format!(
        "{}/v{} (by {} on e621)",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        "bearodactyl"
    ))]
    pub user_agent: String,
}

/// Configuration options for the response cache
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct CacheConfig {
    /// Enable response caching
    #[default(true)]
    pub enabled: bool,

    /// Cache directory
    #[default(".cache".to_owned())]
    pub cache_dir: String,

    /// Cache TTL in seconds
    #[default(3600)]
    pub ttl_secs: u64,

    /// Cache TTI in seconds
    #[default(1800)]
    pub tti_secs: u64,

    /// Max cache size in MB
    #[default(500)]
    pub max_size_mb: u64,

    /// Maximum number of entries in memory cache
    #[default(10000)]
    pub max_entries: usize,

    /// Enable LRU eviction policy (when false, uses TinyLFU for better hit rates)
    #[default(false)]
    pub use_lru_policy: bool,

    /// Enable cache statistics tracking
    #[default(true)]
    pub enable_stats: bool,

    /// Auto-cleanup interval in seconds (for removing expired entries)
    #[default(300)]
    pub cleanup_interval_secs: u64,

    /// Enable compression for cached data (reduces size but adds CPU overhead)
    #[default(false)]
    pub enable_compression: bool,

    /// Compression level (1-9, where 9 is maximum compression)
    #[default(6)]
    pub compression_level: u8,

    /// Post cache specific settings
    #[default(PostCacheConfig::default())]
    pub post_cache: PostCacheConfig,
}

/// Configuration options for post-specific caching
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct PostCacheConfig {
    /// Enable post cache
    #[default(true)]
    pub enabled: bool,

    /// Maximum number of posts to cache
    #[default(50000000)]
    pub max_posts: usize,

    /// Enable write-ahead logging for better crash recovery
    #[default(true)]
    pub enable_wal: bool,

    /// Database page size in bytes (affects performance and size)
    #[default(4)]
    pub page_size_kb: usize,

    /// Enable automatic compaction to reclaim space
    #[default(true)]
    pub auto_compact: bool,

    /// Compaction threshold (compact when wasted space exceeds this percentage)
    #[default(25)]
    pub compact_threshold_percent: u8,
}

/// Configuration options for performance
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct PerformanceConfig {
    #[schemars(range(min = 1, max = 15))]
    /// Number of concurrent downloads
    #[default(15)]
    pub concurrent_downloads: usize,

    /// Prefetch next batch of posts
    #[default(true)]
    pub prefetch_enabled: bool,

    /// Prefetch batch size
    #[default(10)]
    pub prefetch_batch_size: usize,

    /// Enable image preloading
    #[default(true)]
    pub preload_images: bool,

    /// Max image preload size in MB
    #[default(100)]
    pub max_preload_size_mb: u64,
}

/// Configuration options for the UI
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct UiConfig {
    /// Progress bar refresh rate (Hz)
    #[default(20)]
    pub progress_refresh_rate: u64,

    /// Show detailed progress info
    #[default(true)]
    pub detailed_progress: bool,

    /// Auto-clear completed progress bars
    #[default(true)]
    pub auto_clear_progress: bool,

    /// Pagination size for post listings
    #[default(20)]
    pub pagination_size: usize,

    /// Enable colored output
    #[default(true)]
    pub colored_output: bool,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
pub enum LoggingFormat {
    /// Use the compact output format
    Compact,

    /// Use an excessively pretty output format
    #[default]
    Pretty,
}

/// Settings for logging
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct LoggingConfig {
    /// Enable logging (HIGHLY RECCOMMEND TO KEEP ON)
    #[default(true)]
    pub enable: bool,

    /// The max level to log at
    #[default("info".to_string())]
    pub level: String,

    /// Use extra pretty logging
    #[default(LoggingFormat::Pretty)]
    pub log_format: LoggingFormat,

    /// Enable ANSI escape codes for colors and stuff
    #[default(true)]
    pub asni: bool,

    /// Display event targets in log messages
    #[default(false)]
    pub event_targets: bool,

    /// Display line numbers in log messages
    #[default(false)]
    pub line_numbers: bool,
}

/// Configuration options for displaying images
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct ImageDisplay {
    /// The max width of displayed images
    #[default(800)]
    pub width: u64,

    /// The max height of displayed images
    #[default(600)]
    pub height: u64,

    /// Whether to display the image when showing post info
    #[default(true)]
    pub image_when_info: bool,

    /// Image quality for sixel conversion (1-100)
    #[default(100)]
    pub sixel_quality: u8,

    /// Resize method (nearest, linear, cubic, gaussian, lanczos3)
    #[default("lanczos3".to_string())]
    pub resize_method: String,
}

/// Configuration options for searching posts/pools
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct SearchCfg {
    /// The minimum amount of posts on a tag for it to show up in tag selection
    #[default(2)]
    pub min_posts_on_tag: u64,

    /// The minimum amount of posts on a pool for it to show up in pool selection
    #[default(2)]
    pub min_posts_on_pool: u64,

    /// Whether or not to show inactive pools
    #[default(true)]
    pub show_inactive_pools: bool,

    /// Whether or not to sort pools by how many posts they contain
    #[default(false)]
    pub sort_pools_by_post_count: bool,

    /// Whether or not to sort tags by their post count
    #[default(true)]
    pub sort_tags_by_post_count: bool,

    /// The minimum score a post should have to show up in search
    #[default(0)]
    pub min_post_score: i64,

    /// The maximum score a post should have to show up in search
    #[default(i64::MAX)]
    pub max_post_score: i64,

    /// Sort tags in reverse alphabetic order
    #[default(false)]
    pub reverse_tags_order: bool,

    /// The number of threads to use when fetching post data
    #[default(8)]
    pub fetch_threads: usize,
}

/// Configuration options for completion in menus
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct CompletionCfg {
    /// The similarity threshold to complete a tag
    #[default(0.8)]
    pub tag_similarity_threshold: f64,

    /// The path to `tags.csv` that's used for tag searching/autocompletion
    #[default("data/tags.csv".to_string())]
    pub tags: String,

    /// The path to `pools.csv` that's used for pool searching/autocompletion
    #[default("data/pools.csv".to_string())]
    pub pools: String,
}

/// Your login credentials
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct LoginCfg {
    #[default(true)]
    /// Whether to login or not
    pub login: bool,

    #[default(String::new())]
    /// Your username
    pub username: String,

    #[default(String::new())]
    /// Your API key
    pub api_key: String,
}

/// Settings for automatically updating data snapshots
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct AutoUpdateCfg {
    /// Whether or not to auto-update tags
    #[default(true)]
    pub tags: bool,

    /// Whether or not to auto-update pools
    #[default(true)]
    pub pools: bool,
}

/// Settings for the downloads explorer
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct ExplorerCfg {
    /// Enable recursive directory scanning
    #[default(true)]
    pub recursive_scan: bool,

    /// Show scanning progress for directories with many files
    #[default(true)]
    pub show_scan_progress: bool,

    /// Minimum number of files before showing progress (0 = always show)
    #[default(100)]
    pub progress_threshold: usize,

    /// Default sort order for explorer
    #[default("date_newest".to_string())]
    pub default_sort: String,

    /// Number of posts to display per page in explorer
    #[default(20)]
    pub posts_per_page: usize,

    /// Cache scanned metadata in memory for faster subsequent access
    #[default(true)]
    pub cache_metadata: bool,

    /// Automatically display image when viewing post details
    #[default(true)]
    pub auto_display_image: bool,

    /// The amount of time to wait between slideshow images
    #[default(5)]
    pub slideshow_wait_seconds: u64,
}

/// Settings for post downloading
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct DownloadCfg {
    /// The directory to download posts to
    #[default("downloads".to_string())]
    pub download_dir: String,

    /// Save the data of downloaded posts
    ///
    /// This is used in the auto-reorganizer to enable
    /// fully offline metadata queries
    ///
    /// Unix systems: Will save the JSON data to `<imagepath>.json`
    ///   To read it: Run `cat <imagepath>.json`
    ///
    /// Windows systems: Will save the JSON data to `<imagepath>:metadata`
    ///      To read it: Run `cat <imagepath>:metadata`
    #[default(true)]
    pub save_metadata: bool,

    /// The output format for downloaded files
    ///
    /// Valid placeholders are as follows:
    /// - `$artists[N]`: The first N artists
    /// - `$tags[N]`: The first N general tags
    /// - `$characters[N]`: The first N character tags
    /// - `$species[N]`: The first N species tags
    /// - `$copyright[N]`: The first N copyright tags
    /// - `$sources[N]`: The first N sources (domain names)
    /// - `$id`: The ID of the post
    /// - `$rating`: The full content rating of the post (safe, questionable, explicit)
    /// - `$rating_first`: The first letter of the posts content rating (s, q, e)
    /// - `$score`: The total score of the post
    /// - `$score_up`: The upvote score of the post
    /// - `$score_down`: The downvote score of the post
    /// - `$fav_count`: The amount of people who favorited the post
    /// - `$comment_count`: The amount of comments on the post
    /// - `$md5`: The MD5 hash of the post file
    /// - `$ext`: The file extension of the post
    /// - `$width`: The original width of the post media in pixels
    /// - `$height`: The original height of the post media in pixels
    /// - `$aspect_ratio`: The aspect ratio (width/height)
    /// - `$orientation`: Portrait, landscape, or square
    /// - `$resolution`: Resolution category (SD, HD, FHD, QHD, 4K, 8K)
    /// - `$megapixels`: Megapixel count (rounded to 1 decimal)
    /// - `$size`: The file size of the post media in bytes
    /// - `$size_mb`: The file size in megabytes (rounded to 2 decimals)
    /// - `$size_kb`: The file size in kilobytes (rounded to 2 decimals)
    /// - `$artist`: The first listed artist of the post
    /// - `$artist_count`: Number of artists tagged
    /// - `$tag_count`: Total number of tags
    /// - `$tag_count_general`: Number of general tags
    /// - `$tag_count_character`: Number of character tags
    /// - `$tag_count_species`: Number of species tags
    /// - `$tag_count_copyright`: Number of copyright tags
    /// - `$pool_ids`: Comma-separated pool IDs
    /// - `$pool_count`: Number of pools the post is in
    /// - `$uploader`: The name of the person who uploaded the post
    /// - `$uploader_id`: The user ID of the person who uploaded the post
    /// - `$approver`: The name of the approver (if approved)
    /// - `$approver_id`: The ID of the approver (if approved)
    /// - `$has_children`: "yes" if post has children, "no" otherwise
    /// - `$parent_id`: The parent post ID (if exists)
    /// - `$year`: The year when the post was uploaded
    /// - `$month`: The month when the post was uploaded
    /// - `$day`: The day when the post was uploaded
    /// - `$hour`: The hour the post was uploaded at
    /// - `$minute`: The minute the post was uploaded at
    /// - `$second`: The second the post was uploaded at
    /// - `$date`: Shorthand for "$year-$month-$day"
    /// - `$time`: Shorthand for "$hour-$minute-$second"
    /// - `$datetime`: Shorthand for "$year-$month-$day $hour-$minute-$second"
    /// - `$timestamp`: Unix timestamp of upload
    /// - `$year_updated`: The year when the post was last updated
    /// - `$month_updated`: The month when the post was last updated
    /// - `$day_updated`: The day when the post was last updated
    /// - `$date_updated`: Shorthand for "$year_updated-$month_updated-$day_updated"
    /// - `$now_year`: The year at the time of downloading the post
    /// - `$now_month`: The month at the time of downloading the post
    /// - `$now_day`: The day at the time of downloading the post
    /// - `$now_hour`: The hour of the day at the time of downloading the post
    /// - `$now_minute`: The minute of the hour at the time of downloading the post
    /// - `$now_second`: The second of the minute at the time of downloading the post
    /// - `$now_date`: Shorthand for "$now_year-$now_month-$now_day"
    /// - `$now_time`: Shorthand for "$now_hour-$now_minute-$now_second"
    /// - `$now_datetime`: Shorthand for "$now_year-$now_month-$now_day $now_hour-$now_minute-$now_second"
    /// - `$now_timestamp`: Unix timestamp at download time
    /// - `$is_pending`: "yes" if pending approval, "no" otherwise
    /// - `$is_flagged`: "yes" if flagged, "no" otherwise
    /// - `$is_deleted`: "yes" if deleted, "no" otherwise
    /// - `$has_notes`: "yes" if has notes, "no" otherwise
    /// - `$duration`: Video duration in seconds (if applicable)
    /// - `$duration_formatted`: Video duration as MM:SS or HH:MM:SS
    /// - `$file_type`: Media type (image, video, flash, etc.)
    #[default("$artists[3]/$rating/$tags[3] - $id - $date $time - $score.$ext".to_string())]
    pub output_format: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct GalleryCfg {
    /// Enable the media gallery server
    #[default(true)]
    pub enabled: bool,

    /// Port to run the gallery server on
    #[default(23794)]
    pub port: u16,

    /// Enable metadata-based filtering (requires saved post metadata)
    #[default(true)]
    pub enable_metadata_filtering: bool,

    /// Cache metadata in memory for faster filtering
    #[default(true)]
    pub cache_metadata: bool,

    /// Automatically open browser when starting server
    #[default(false)]
    pub auto_open_browser: bool,

    /// The number of threads to use for loading your downloads
    #[default(8)]
    pub load_threads: usize,

    /// The colorscheme to use for the gallery
    ///
    /// Possible values:
    /// - rose-pine (default)
    /// - rose-pine-moon
    /// - rose-pine-dawn
    /// - catppuccin-latte
    /// - catppuccin-frappe
    /// - catppuccin-macchiato
    /// - catppuccin-mocha
    #[default("catppuccin-frappe".to_string())]
    pub theme: String,
}

/// E62RS configuration options
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct E62Rs {
    /// The format to display download progress in
    #[default(SizeFormat::default())]
    pub progress_format: SizeFormat,

    /// The amount of posts to show in a search
    #[default(320)]
    pub post_count: u64,

    #[schemars(url)]
    /// The base URL of the API (defaults to https://e621.net)
    #[default("https://e621.net".to_string())]
    pub base_url: String,

    /// Post viewing settings
    #[default(ImageDisplay::default())]
    pub display: ImageDisplay,

    /// HTTP client configuration
    #[default(HttpConfig::default())]
    pub http: HttpConfig,

    /// Cache configuration
    #[default(CacheConfig::default())]
    pub cache: CacheConfig,

    /// Performance settings
    #[default(PerformanceConfig::default())]
    pub performance: PerformanceConfig,

    /// UI settings
    #[default(UiConfig::default())]
    pub ui: UiConfig,

    /// Search settings
    #[default(SearchCfg::default())]
    pub search: SearchCfg,

    /// Login settings
    #[default(LoginCfg::default())]
    pub login: LoginCfg,

    /// Completion settings
    #[default(CompletionCfg::default())]
    pub completion: CompletionCfg,

    /// Autoupdate settings
    #[default(AutoUpdateCfg::default())]
    pub autoupdate: AutoUpdateCfg,

    /// Post download settings
    #[default(DownloadCfg::default())]
    pub download: DownloadCfg,

    /// Blacklisted tags to filter out from all operations
    #[default(vec!["young".to_string(), "rape".to_string(), "feral".to_string(), "bestiality".to_string()])]
    pub blacklist: Vec<String>,

    /// Downloads explorer settings
    #[default(ExplorerCfg::default())]
    pub explorer: ExplorerCfg,

    /// Media server settings
    #[default(GalleryCfg::default())]
    pub gallery: GalleryCfg,

    /// Logging settings
    #[default(LoggingConfig::default())]
    pub logging: LoggingConfig,
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

impl E62Rs {
    pub fn get() -> Result<Self> {
        let global_config_path = format!("{}/e62rs.toml", get_config_dir());

        let defaults = E62Rs::default();

        let mut builder = Config::builder();
        builder = builder.add_source(
            config::Config::try_from(&defaults).context("Failed to build default config source")?,
        );

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

        let cfg = settings.try_deserialize::<E62Rs>().unwrap_or_else(|e| {
            warn!("Failed to deserialize config, using defaults: {}", e);
            defaults.clone()
        });

        if !Path::new(&global_config_path).exists() && find_local_config_file().is_none() {
            info!(
                "Creating default configuration file at {}",
                global_config_path
            );
            if let Err(e) = defaults.save_to_file(&global_config_path) {
                warn!("Failed to create default config file: {}", e);
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

    pub fn get_unsafe() -> Self {
        Self::get().expect("Failed to get config")
    }
}

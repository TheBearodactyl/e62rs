//! every single available configuration option and its type is listed in this file
use {
    crate::config::validate::{Validate, format_validation_errors},
    color_eyre::{
        Section, SectionExt,
        eyre::{Context, OptionExt, Result, eyre},
    },
    config::{Config, ConfigBuilder},
    schemars::JsonSchema,
    serde::{Deserialize, Serialize},
    smart_default::SmartDefault,
    std::path::{Path, PathBuf},
    tracing::info,
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
    /// format a given number of bytes into a format
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
    #[default(Some(32))]
    pub pool_max_idle_per_host: Option<usize>,

    /// Connection pool idle timeout in seconds
    #[default(Some(90))]
    pub pool_idle_timeout_secs: Option<u64>,

    /// Request timeout in seconds
    #[default(Some(30))]
    pub timeout_secs: Option<u64>,

    /// Connection timeout in seconds
    #[default(Some(10))]
    pub connect_timeout_secs: Option<u64>,

    #[schemars(range(min = 1, max = 15))]
    /// Max concurrent connections
    #[default(Some(15))]
    pub max_connections: Option<usize>,

    /// Enable HTTP/2
    #[default(Some(true))]
    pub http2_prior_knowledge: Option<bool>,

    /// Enable keep-alive
    #[default(Some(true))]
    pub tcp_keepalive: Option<bool>,

    /// How many seconds to keep TCP alive for
    #[default(Some(60))]
    pub tcp_keepalive_secs: Option<u64>,

    /// User agent string in the format:
    /// `<project name>/<project version> (by <valid e6 username> on <e621/e926>)`
    ///
    /// Some examples of valid User Agents:
    /// - `my-project/1.2.3 (by username123 on e621)`
    /// - `another/2.0.0-beta.1 (by user7890 on e926)`
    /// - `test-proj/0.1.0+build.123 (by myuser12345 on e621)`
    #[default(Some(format!(
        "{}/v{} (by {} on e621)",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        "bearodactyl"
    )))]
    pub user_agent: Option<String>,
}

/// Configuration options for the response cache
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct CacheConfig {
    /// Enable response caching
    #[default(Some(true))]
    pub enabled: Option<bool>,

    /// Cache directory
    #[default(Some(".cache".to_owned()))]
    pub cache_dir: Option<String>,

    /// Cache TTL in seconds
    #[default(Some(3600))]
    pub ttl_secs: Option<u64>,

    /// Cache TTI in seconds
    #[default(Some(1800))]
    pub tti_secs: Option<u64>,

    /// Max cache size in MB
    #[default(Some(500))]
    pub max_size_mb: Option<u64>,

    /// Maximum number of entries in memory cache
    #[default(Some(10000))]
    pub max_entries: Option<usize>,

    /// Enable LRU eviction policy (when false, uses TinyLFU for better hit rates)
    #[default(Some(false))]
    pub use_lru_policy: Option<bool>,

    /// Enable cache statistics tracking
    #[default(Some(true))]
    pub enable_stats: Option<bool>,

    /// Auto-cleanup interval in seconds (for removing expired entries)
    #[default(Some(300))]
    pub cleanup_interval: Option<u64>,

    /// Enable compression for cached data (reduces size but adds CPU overhead)
    #[default(Some(false))]
    pub enable_compression: Option<bool>,

    /// Compression level (1-9, where 9 is maximum compression)
    #[default(Some(6))]
    pub compression_level: Option<u8>,

    /// Post cache specific settings
    #[default(Some(PostCacheConfig::default()))]
    pub posts: Option<PostCacheConfig>,
}

/// Configuration options for post-specific caching
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct PostCacheConfig {
    /// Enable post cache
    #[default(Some(true))]
    pub enabled: Option<bool>,

    /// Maximum number of posts to cache
    #[default(Some(50000000))]
    pub max_posts: Option<usize>,

    /// Enable write-ahead logging for better crash recovery
    #[default(Some(true))]
    pub wal: Option<bool>,

    /// Database page size in bytes (affects performance and size)
    #[default(Some(4))]
    pub page_size_kb: Option<usize>,

    /// Enable automatic compaction to reclaim space
    #[default(Some(true))]
    pub auto_compact: Option<bool>,

    /// Compaction threshold (compact when wasted space exceeds this percentage)
    #[default(Some(25))]
    pub compact_threshold: Option<u8>,
}

/// Configuration options for performance
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct PerformanceConfig {
    #[schemars(range(min = 1, max = 15))]
    /// Number of concurrent downloads
    #[default(Some(15))]
    pub concurrent_downloads: Option<usize>,

    /// Prefetch next batch of posts
    #[default(Some(true))]
    pub prefetch_enabled: Option<bool>,

    /// Prefetch batch size
    #[default(Some(10))]
    pub prefetch_batch_size: Option<usize>,

    /// Enable image preloading
    #[default(Some(true))]
    pub preload_images: Option<bool>,

    /// Max image preload size in MB
    #[default(Some(100))]
    pub max_preload_size_mb: Option<u64>,
}

/// Configuration options for the UI
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct UiConfig {
    /// Progress bar refresh rate (Hz)
    #[default(Some(20))]
    pub progress_refresh_rate: Option<u64>,

    /// Show detailed progress info
    #[default(Some(true))]
    pub detailed_progress: Option<bool>,

    /// Auto-clear completed progress bars
    #[default(Some(true))]
    pub auto_clear_progress: Option<bool>,

    /// Pagination size for post listings
    #[default(Some(20))]
    pub pagination_size: Option<usize>,

    /// Enable colored output
    #[default(Some(true))]
    pub colored_output: Option<bool>,

    /// The format to display download progress in
    #[default(Some(SizeFormat::default()))]
    pub progress_format: Option<SizeFormat>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
/// The format to log in
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
    #[default(Some(true))]
    pub enable: Option<bool>,

    /// The max level to log at
    #[default(Some("info".to_string()))]
    pub level: Option<String>,

    /// Use extra pretty logging
    #[default(Some(LoggingFormat::Pretty))]
    pub format: Option<LoggingFormat>,

    /// Enable ANSI escape codes for colors and stuff
    #[default(Some(true))]
    pub asni: Option<bool>,

    /// Display event targets in log messages
    #[default(Some(false))]
    pub event_targets: Option<bool>,

    /// Display line numbers in log messages
    #[default(Some(false))]
    pub line_numbers: Option<bool>,
}

/// Configuration options for displaying images
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct ImageDisplay {
    /// The max width of displayed images
    #[default(Some(800))]
    pub width: Option<u64>,

    /// The max height of displayed images
    #[default(Some(600))]
    pub height: Option<u64>,

    /// Whether to display the image when showing post info
    #[default(Some(true))]
    pub image_when_info: Option<bool>,

    /// Image quality for sixel conversion (1-100)
    #[default(Some(100))]
    pub sixel_quality: Option<u8>,

    /// Resize method (nearest, linear, cubic, gaussian, lanczos3)
    #[default(Some("lanczos3".to_string()))]
    pub resize_method: Option<String>,
}

/// Configuration options for searching posts/pools
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct SearchCfg {
    /// The minimum amount of posts on a tag for it to show up in tag selection
    #[default(Some(2))]
    pub min_posts_on_tag: Option<u64>,

    /// The minimum amount of posts on a pool for it to show up in pool selection
    #[default(Some(2))]
    pub min_posts_on_pool: Option<u64>,

    /// Whether or not to show inactive pools
    #[default(Some(true))]
    pub show_inactive_pools: Option<bool>,

    /// Whether or not to sort pools by how many posts they contain
    #[default(Some(false))]
    pub sort_pools_by_post_count: Option<bool>,

    /// Whether or not to sort tags by their post count
    #[default(Some(true))]
    pub sort_tags_by_post_count: Option<bool>,

    /// The minimum score a post should have to show up in search
    #[default(Some(0))]
    pub min_post_score: Option<i64>,

    /// The maximum score a post should have to show up in search
    #[default(Some(i64::MAX))]
    pub max_post_score: Option<i64>,

    /// Sort tags in reverse alphabetic order
    #[default(Some(false))]
    pub reverse_tags_order: Option<bool>,

    /// The number of threads to use when fetching post data
    #[default(Some(8))]
    pub fetch_threads: Option<usize>,
}

/// Configuration options for completion in menus
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct CompletionCfg {
    /// The similarity threshold to complete a tag
    #[default(Some(0.8))]
    pub tag_similarity_threshold: Option<f64>,

    /// The path to `tags.csv` that's used for tag searching/autocompletion
    #[default(Some("data/tags.csv".to_string()))]
    pub tags: Option<String>,

    /// The path to `tag_aliases.csv` that's used for tag alias computation
    #[default(Some("data/tag_aliases.csv".to_string()))]
    pub aliases: Option<String>,

    /// The path to `tag_implications.csv` that's used for tag implication computation
    #[default(Some("data/tag_implications.csv".to_string()))]
    pub implications: Option<String>,

    /// The path to `pools.csv` that's used for pool searching/autocompletion
    #[default(Some("data/pools.csv".to_string()))]
    pub pools: Option<String>,
}

/// Your login credentials
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct LoginCfg {
    #[default(Some(true))]
    /// Whether to login or not
    pub login: Option<bool>,

    #[default(Some(String::new()))]
    /// Your username
    pub username: Option<String>,

    #[default(Some(String::new()))]
    /// Your API key
    pub api_key: Option<String>,
}

/// Settings for automatically updating data snapshots
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct AutoUpdateCfg {
    /// Whether or not to auto-update tags
    #[default(Some(true))]
    pub tags: Option<bool>,

    /// Whether or not to auto-update pools
    #[default(Some(true))]
    pub pools: Option<bool>,
}

/// Settings for the downloads explorer
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct ExplorerCfg {
    /// Enable recursive directory scanning
    #[default(Some(true))]
    pub recursive: Option<bool>,

    /// Show scanning progress for directories with many files
    #[default(Some(true))]
    pub show_progress: Option<bool>,

    /// Minimum number of files before showing progress (0 = always show)
    #[default(Some(100))]
    pub progress_threshold: Option<usize>,

    /// Default sort order for explorer
    #[default(Some("date_newest".to_string()))]
    pub default_sort: Option<String>,

    /// Number of posts to display per page in explorer
    #[default(Some(20))]
    pub posts_per_page: Option<usize>,

    /// Cache scanned metadata in memory for faster subsequent access
    #[default(Some(true))]
    pub cache_metadata: Option<bool>,

    /// Automatically display image when viewing post details
    #[default(Some(true))]
    pub auto_display_image: Option<bool>,

    /// The amount of time to wait between slideshow images
    #[default(Some(5))]
    pub slideshow_delay: Option<u64>,
}

/// Settings for post downloading
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct DownloadCfg {
    /// The directory to download posts to
    #[default(Some("downloads".to_string()))]
    pub path: Option<String>,

    /// The directory to download pools to
    #[default(Some("downloads/pools".to_string()))]
    pub pools_path: Option<String>,

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
    #[default(Some(true))]
    pub save_metadata: Option<bool>,

    /// ## Filename Formatting
    ///
    /// The `output_format` setting controls how filenames are generated when saving posts. Forward slashes denote subfolders.
    ///
    /// ### Simple Placeholders
    ///
    /// These placeholders insert a single value:
    ///
    /// **Basic Post Information:**
    ///
    /// - `$id` → post ID
    /// - `$rating` → rating (e.g. `"safe"`, `"questionable"`, `"explicit"`)
    /// - `$rating_first` → first character of rating (`s`, `q`, `e`)
    /// - `$md5` → MD5 hash of file
    /// - `$ext` → file extension
    ///
    /// **Scores & Engagement:**
    ///
    /// - `$score` → total post score
    /// - `$score_up` → upvote score
    /// - `$score_down` → downvote score
    /// - `$fav_count` → number of favorites
    /// - `$comment_count` → number of comments
    ///
    /// **File Metadata:**
    ///
    /// - `$width` / `$height` → file dimensions in pixels
    /// - `$aspect_ratio` → aspect ratio (width/height)
    /// - `$orientation` → `"portrait"`, `"landscape"`, or `"square"`
    /// - `$resolution` → resolution category (`"SD"`, `"HD"`, `"FHD"`, `"QHD"`, `"4K"`, `"8K"`)
    /// - `$megapixels` → megapixel count (rounded to 1 decimal)
    /// - `$size` → file size in bytes
    /// - `$size_mb` → file size in megabytes (rounded to 2 decimals)
    /// - `$size_kb` → file size in kilobytes (rounded to 2 decimals)
    /// - `$file_type` → media type (`"image"`, `"video"`, `"flash"`, `"unknown"`)
    ///
    /// **Video-Specific:**
    ///
    /// - `$duration` → video duration in seconds (0 if not applicable)
    /// - `$duration_formatted` → video duration as `MM:SS` or `HH:MM:SS`
    ///
    /// **User Information:**
    ///
    /// - `$artist` → first listed artist (or `"unknown"`)
    /// - `$uploader` → uploader username
    /// - `$uploader_id` → uploader user ID
    /// - `$approver_id` → approver ID (or `"none"`)
    ///
    /// **Tag Counts:**
    ///
    /// - `$tag_count` → total number of tags
    /// - `$artist_count` → number of artist tags
    /// - `$tag_count_general` → number of general tags
    /// - `$tag_count_character` → number of character tags
    /// - `$tag_count_species` → number of species tags
    /// - `$tag_count_copyright` → number of copyright tags
    ///
    /// **Pool Information:**
    ///
    /// - `$pool_ids` → comma-separated list of pool IDs
    /// - `$pool_count` → number of pools the post is in
    ///
    /// **Relationships:**
    ///
    /// - `$has_children` → `"yes"` if post has children, `"no"` otherwise
    /// - `$parent_id` → parent post ID (or `"none"`)
    ///
    /// **Flags:**
    ///
    /// - `$is_pending` → `"yes"` if pending approval, `"no"` otherwise
    /// - `$is_flagged` → `"yes"` if flagged, `"no"` otherwise
    /// - `$is_deleted` → `"yes"` if deleted, `"no"` otherwise
    /// - `$has_notes` → `"yes"` if has notes, `"no"` otherwise
    ///
    /// ### Date/Time Placeholders
    ///
    /// **Post Creation Date:**
    ///
    /// - `$year`, `$month`, `$day` → creation date components
    /// - `$hour`, `$minute`, `$second` → creation time components
    /// - `$date` → shorthand for `$year-$month-$day`
    /// - `$time` → shorthand for `$hour-$minute-$second`
    /// - `$datetime` → shorthand for `$year-$month-$day $hour-$minute-$second`
    /// - `$timestamp` → Unix timestamp of upload
    ///
    /// **Post Update Date:**
    ///
    /// - `$year_updated`, `$month_updated`, `$day_updated` → last update date components
    /// - `$date_updated` → shorthand for `$year_updated-$month_updated-$day_updated`
    ///
    /// **Download Time:**
    ///
    /// - `$now_year`, `$now_month`, `$now_day` → download date components
    /// - `$now_hour`, `$now_minute`, `$now_second` → download time components
    /// - `$now_date` → shorthand for `$now_year-$now_month-$now_day`
    /// - `$now_time` → shorthand for `$now_hour-$now_minute-$now_second`
    /// - `$now_datetime` → shorthand for `$now_year-$now_month-$now_day $now_hour-$now_minute-$now_second`
    /// - `$now_timestamp` → Unix timestamp at download time
    ///
    /// ### Indexed Placeholders
    ///
    /// These placeholders allow you to extract multiple items from lists. They support several range syntaxes:
    ///
    /// **Syntax:**
    ///
    /// - `$key[N]` → first N items (e.g., `$tags[5]`)
    /// - `$key[L..R]` → items from index L to R (exclusive) (e.g., `$tags[2..5]`)
    /// - `$key[N..]` → all items from index N onwards (e.g., `$artists[1..]`)
    /// - `$key[..N]` → items from start to index N (exclusive) (e.g., `$sources[..3]`)
    ///
    /// **Available indexed placeholders:**
    ///
    /// - `$tags[...]` → general tags, joined by commas
    /// - `$artists[...]` → artist tags, joined by commas
    /// - `$characters[...]` → character tags, joined by commas
    /// - `$species[...]` → species tags, joined by commas
    /// - `$copyright[...]` → copyright tags, joined by commas
    /// - `$sources[...]` → source domains, joined by commas
    ///
    /// **Examples:**
    ///
    /// ```toml
    /// # First 3 tags
    /// output_format = "$tags[3] - $id.$ext"
    /// # → "anthro, digital_media, solo - 123456.png"
    ///
    /// # Tags 2 through 5
    /// output_format = "$tags[2..5] - $id.$ext"
    /// # → "fur, blue_eyes, sitting - 123456.png"
    ///
    /// # All artists except the first
    /// output_format = "$artists[1..]/$id.$ext"
    /// # → "collaborator1, collaborator2/123456.png"
    ///
    /// # First 2 sources
    /// output_format = "$sources[..2] - $id.$ext"
    /// # → "twitter.com, deviantart.com - 123456.png"
    ///
    /// # Organize by primary artist, include other artists in filename
    /// output_format = "$artists[..1]/$artists[1..] - $id.$ext"
    /// # → "primary_artist/collab1, collab2 - 123456.png"
    /// ```
    #[default(Some("$artists[3]/$rating/$tags[3] - $id - $date $time - $score.$ext".to_string()))]
    pub format: Option<String>,
}

/// Settings for the post gallery
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default")]
#[schemars(default)]
pub struct GalleryCfg {
    /// Enable the media gallery server
    #[default(Some(true))]
    pub enabled: Option<bool>,

    /// Enable metadata-based filtering (requires saved post metadata)
    #[default(Some(true))]
    pub enable_metadata_filtering: Option<bool>,

    /// Port to run the gallery server on
    #[default(Some(23794))]
    pub port: Option<u16>,

    /// Cache metadata in memory for faster filtering
    #[default(Some(true))]
    pub cache_metadata: Option<bool>,

    /// Automatically open browser when starting server
    #[default(Some(false))]
    pub auto_open_browser: Option<bool>,

    /// The number of threads to use for loading your downloads
    #[default(Some(8))]
    pub load_threads: Option<usize>,

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
    #[default(Some("catppuccin-frappe".to_string()))]
    pub theme: Option<String>,
}

/// E62RS configuration options
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, SmartDefault)]
#[schemars(bound = "T: JsonSchema + Default", default)]
pub struct E62Rs {
    /// Configuration file version (do not modify manually)
    #[default(Some(1))]
    pub version: Option<u32>,

    /// The amount of posts to show in a search
    #[default(Some(320))]
    pub results_limit: Option<u64>,

    /// The base URL of the API (defaults to <https://e621.net>)
    #[schemars(url)]
    #[default(Some("https://e621.net".to_string()))]
    pub base_url: Option<String>,

    /// Post viewing settings
    #[default(Some(ImageDisplay::default()))]
    pub display: Option<ImageDisplay>,

    /// HTTP client configuration
    #[default(Some(HttpConfig::default()))]
    pub http: Option<HttpConfig>,

    /// Cache configuration
    #[default(Some(CacheConfig::default()))]
    pub cache: Option<CacheConfig>,

    /// Performance settings
    #[default(Some(PerformanceConfig::default()))]
    pub performance: Option<PerformanceConfig>,

    /// UI settings
    #[default(Some(UiConfig::default()))]
    pub ui: Option<UiConfig>,

    /// Search settings
    #[default(Some(SearchCfg::default()))]
    pub search: Option<SearchCfg>,

    /// Login settings
    #[default(Some(LoginCfg::default()))]
    pub login: Option<LoginCfg>,

    /// Completion settings
    #[default(Some(CompletionCfg::default()))]
    pub completion: Option<CompletionCfg>,

    /// Autoupdate settings
    #[default(Some(AutoUpdateCfg::default()))]
    pub autoupdate: Option<AutoUpdateCfg>,

    /// Post download settings
    #[default(Some(DownloadCfg::default()))]
    pub download: Option<DownloadCfg>,

    /// Blacklisted tags to filter out from all operations
    #[default(Some(vec!["young".to_string(), "rape".to_string(), "feral".to_string(), "bestiality".to_string()]))]
    pub blacklist: Option<Vec<String>>,

    /// Downloads explorer settings
    #[default(Some(ExplorerCfg::default()))]
    pub explorer: Option<ExplorerCfg>,

    /// Media server settings
    #[default(Some(GalleryCfg::default()))]
    pub gallery: Option<GalleryCfg>,

    /// Logging settings
    #[default(Some(LoggingConfig::default()))]
    pub logging: Option<LoggingConfig>,
}

/// returns the path to the local config file if it exists
fn _find_local_config_file() -> Result<Option<PathBuf>> {
    let curr_dir = std::env::current_dir()
        .wrap_err("failed to get current working directory")
        .suggestion("ensure the current directory exists and is accessible")?;

    for ancestor in curr_dir.ancestors() {
        let config_path = ancestor.join("e62rs.toml");
        if config_path.exists() {
            return Ok(Some(config_path));
        }
    }

    Ok(None)
}

impl E62Rs {
    /// load config from default locations
    ///
    /// load prio: local > global > defaults
    pub fn load() -> Result<Self> {
        let global_config_path = Self::global_config_path()?;
        let defaults = Self::load_defaults()?;
        let mut builder = Self::create_builder(defaults.clone())?;

        builder = builder.add_source(
            config::File::with_name(global_config_path.to_str().unwrap()).required(false),
        );

        if let Some(local_config) = Self::find_local_config()? {
            builder = builder.add_source(
                config::File::with_name(local_config.to_str().unwrap()).required(false),
            );
        }

        builder = builder.add_source(config::Environment::with_prefix("E62RS"));

        let settings = builder.build().wrap_err("Failed to build configuration")?;
        let cfg: E62Rs = settings
            .try_deserialize::<E62Rs>()
            .wrap_err("Failed to deserialize configuration")?;

        match cfg.run_validation() {
            Ok(_) => {
                info!("Configuration validation successful");
            }
            Err(e) => {
                return Err(e);
            }
        }

        if !global_config_path.exists() {
            Self::create_default_config(&global_config_path, &defaults)?;
        }

        Ok(cfg)
    }

    /// get the global config file path
    fn global_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_eyre("Unable to determine system config directory")
            .suggestion("Ensure XDG_CONFIG_HOME or HOME environment variables are set")
            .suggestion("On Windows, APPDATA should be set")?;

        Ok(config_dir.join("e62rs.toml"))
    }

    /// load default config from embedded default config file
    fn load_defaults() -> Result<Self> {
        toml::from_str(include_str!("../../resources/e62rs.default.toml"))
            .wrap_err("Failed to parse embedded default configuration")
            .note("This is a bug - the embedded defaults are malformed")
    }

    /// create a config builder with defaults
    fn create_builder(defaults: E62Rs) -> Result<ConfigBuilder<config::builder::DefaultState>> {
        let builder = Config::builder();
        let config_source = config::Config::try_from(&defaults)
            .wrap_err("Failed to convert default E62Rs struct to config source")?;

        Ok(builder.add_source(config_source))
    }

    /// run validation and return a pretty error if it fails
    fn run_validation(&self) -> Result<()> {
        self.validate()
            .map_err(|errors| {
                let formatted = format_validation_errors(&errors);
                eyre!(formatted)
            })
            .wrap_err("config validation failed")
            .suggestion("Check your e62rs.toml for invalid values")
            .suggestion("Run with default config to see valid options")
    }

    /// find the local config file
    fn find_local_config() -> Result<Option<PathBuf>> {
        let curr_dir = std::env::current_dir()
            .wrap_err("Failed to get current working directory")
            .suggestion("Ensure the current directory exists and is accessible")?;

        for ancestor in curr_dir.ancestors() {
            let config_path = ancestor.join("e62rs.toml");
            if config_path.exists() {
                return Ok(Some(config_path));
            }
        }

        Ok(None)
    }

    /// create the default config file
    fn create_default_config(path: &Path, defaults: &E62Rs) -> Result<()> {
        let config_dir = path
            .parent()
            .ok_or_eyre("Unable to determine parent directory of config path")?;

        std::fs::create_dir_all(config_dir)
            .wrap_err("Failed to create config directory")
            .with_section(|| format!("{}", config_dir.display()).header("Directory:"))?;

        defaults
            .save_to_file(path)
            .wrap_err("Failed to write default configuration file")?;

        Ok(())
    }

    /// save config to a file
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let toml_str =
            toml::to_string_pretty(self).wrap_err("Failed to serialize config to TOML")?;

        std::fs::write(path, &toml_str)
            .wrap_err_with(|| format!("Failed to write config file: {}", path.display()))
            .with_section(|| path.display().to_string().header("File path"))
            .with_section(|| format!("{} bytes", toml_str.len()).header("Content size:"))?;

        Ok(())
    }

    /// save config to the global config location
    pub fn save(&self) -> Result<()> {
        let path = Self::global_config_path()?;
        self.save_to_file(path)
    }
}

/// get the current value of a given setting
#[macro_export]
macro_rules! getopt {
    () => {
        $crate::config::instance::config()
    };

    ($field:ident) => {{
        $crate::config::instance::get_or_default(
            |c| c.$field.clone(),
            $crate::config::options::E62Rs::default()
                .$field
                .expect(concat!("Default value missing for: ", stringify!($field))),
        )
    }};

    ($lvl1:ident . $field:ident) => {{
        $crate::config::instance::get_or_default(
            |c| c.$lvl1.as_ref().and_then(|sub| sub.$field.clone()),
            $crate::config::options::E62Rs::default()
                .$lvl1
                .and_then(|sub| sub.$field)
                .expect(concat!(
                    "Default value missing for: ",
                    stringify!($lvl1),
                    ".",
                    stringify!($field)
                )),
        )
    }};

    ($lvl1:ident . $lvl2:ident . $field:ident) => {{
        $crate::config::instance::get_or_default(
            |c| {
                c.$lvl1
                    .as_ref()
                    .and_then(|sub| sub.$lvl2.as_ref())
                    .and_then(|sub| sub.$field.clone())
            },
            $crate::config::options::E62Rs::default()
                .$lvl1
                .and_then(|sub| sub.$lvl2)
                .and_then(|sub| sub.$field)
                .expect(concat!(
                    "Default value missing for: ",
                    stringify!($lvl1),
                    ".",
                    stringify!($lvl2),
                    ".",
                    stringify!($field)
                )),
        )
    }};

    (raw $field:ident) => {{
        $crate::config::instance::config()
            .ok()
            .and_then(|c| c.$field.clone())
    }};

    (raw $lvl1:ident . $field:ident) => {{
        $crate::config::instance::config()
            .ok()
            .and_then(|c| c.$lvl1.as_ref().and_then(|sub| sub.$field.clone()))
    }};

    (raw $lvl1:ident . $lvl2:ident . $field:ident) => {{
        $crate::config::instance::config().ok().and_then(|c| {
            c.$lvl1
                .as_ref()
                .and_then(|sub| sub.$lvl2.as_ref())
                .and_then(|sub| sub.$field.clone())
        })
    }};
}

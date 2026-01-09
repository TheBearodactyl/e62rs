//! configuration validation stuff
use {
    crate::{config::options::*, validator, validator_nested},
    color_eyre::Result,
};

/// trait for validating config structs
pub trait Validate {
    /// validate the config
    fn validate(&self) -> Result<(), Vec<String>>;

    /// check if the config is valid
    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}

validator! { HttpConfig,
    pool_max_idle_per_host => |v: &usize| *v > 0,
        "must be greater than 0";
    pool_idle_timeout => |v: &u64| *v > 0,
        "must be greater than 0";
    timeout => |v: &u64| *v > 0,
        "must be greater than 0";
    connect_timeout => |v: &u64| *v > 0,
        "must be greater than 0";
    max_connections => |v: &usize| *v >= 1 && *v <= 15,
        "must be between 1 and 15";
    tcp_keepalive_secs => |v: &u64| *v > 0,
        "must be greater than 0";
    user_agent => |v: &String| !v.trim().is_empty(),
        "must not be empty";
    api => |v: &String| v.starts_with("http://") || v.starts_with("https://"),
        "must be a valid url and not link to e6ai";
}

validator! { PostCacheConfig,
    max_posts => |v: &usize| *v > 0,
        "must be greater than 0";
    page_size_kb => |v: &usize| *v > 0,
        "must be greater than 0";
    compact_threshold => |v: &u8| *v <= 100,
        "must be between 0 and 100";
}

validator_nested! { CacheConfig,
    fields: {
        cache_dir => |v: &String| !v.trim().is_empty(),
            "must not be empty";
        ttl_secs => |v: &u64| *v > 0,
            "must be greater than 0";
        tti_secs => |v: &u64| *v > 0,
            "must be greater than 0";
        max_size_mb => |v: &u64| *v > 0,
            "must be greater than 0";
        max_entries => |v: &usize| *v > 0,
            "must be greater than 0";
        cleanup_interval => |v: &u64| *v > 0,
            "must be greater than 0";
        compression_level => |v: &u8| *v >= 1 && *v <= 9,
            "must be between 1 and 9";
    }
    nested: {
        posts;
    }
}

validator! { PerformanceConfig,
    prefetch_batch_size => |v: &usize| *v > 0,
        "must be greater than 0";
    max_preload_size_mb => |v: &u64| *v > 0,
        "must be greater than 0";
}

/// valid modes for displaying in progress downloads
const VALID_PROGRESS_MESSAGE_MODES: &[&str] = &["id", "filename"];

validator! { UiConfig,
    pagination_size => |v: &usize| *v > 0 && *v <= 1000,
        "must be between 1 and 1000";
}

validator! { ProgressCfg,
    refresh_rate => |v: &u64| *v > 0 && *v <= 240,
        "must be between 1 and 240 Hz";
    message => |v: &String| VALID_PROGRESS_MESSAGE_MODES.contains(&v.to_lowercase().as_str()),
        "must be either 'id' or 'filename'";
}

/// valid log levels
const VALID_LOG_LEVELS: &[&str] = &["trace", "debug", "info", "warn", "error", "off"];

validator! { LoggingConfig,
    level => |v: &String| VALID_LOG_LEVELS.contains(&v.to_lowercase().as_str()),
        "must be one of: trace, debug, info, warn, error, off";
}

/// valid image resizing methods
const VALID_RESIZE_METHODS: &[&str] = &["nearest", "linear", "cubic", "gaussian", "lanczos3"];

validator! { ImageDisplay,
    width => |v: &u64| *v > 0,
        "must be greater than 0";
    height => |v: &u64| *v > 0,
        "must be greater than 0";
    sixel_quality => |v: &u8| *v >= 1 && *v <= 100,
        "must be between 1 and 100";
    resize_method => |v: &String| VALID_RESIZE_METHODS.contains(&v.to_lowercase().as_str()),
        "must be one of: nearest, linear, cubic, gaussian, lanczos3";
}

impl Validate for SearchCfg {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors: Vec<String> = Vec::new();

        if let Some(v) = self.results
            && v == 0
        {
            errors.push("results_limit: must be greater than 0".to_string());
        }

        if let Some(ref v) = self.blacklist
            && v.iter().any(|tag| tag.trim().is_empty())
        {
            errors.push("blacklist: tags must not be empty strings".to_string());
        }

        if let Some(v) = self.fetch_threads
            && v == 0
        {
            errors.push("fetch_threads: must be greater than 0".to_string());
        }

        if let (Some(min), Some(max)) = (self.min_post_score, self.max_post_score)
            && max < min
        {
            errors
                .push("max_post_score must be greater than or equal to min_post_score".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

validator! { CompletionCfg,
    tag_similarity_threshold => |v: &f64| *v >= 0.0 && *v <= 1.0,
        "must be between 0.0 and 1.0";
    tags => |v: &String| !v.trim().is_empty(),
        "must not be empty";
    aliases => |v: &String| !v.trim().is_empty(),
        "must not be empty";
    implications => |v: &String| !v.trim().is_empty(),
        "must not be empty";
    pools => |v: &String| !v.trim().is_empty(),
        "must not be empty";
}

impl Validate for LoginCfg {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors: Vec<String> = Vec::new();

        if let Some(true) = self.login {
            if self.username.as_ref().is_none_or(|s| s.trim().is_empty()) {
                errors.push("username: required when login is enabled".to_string());
            }

            if self.api_key.as_ref().is_none_or(|s| s.trim().is_empty()) {
                errors.push("api_key: required when login is enabled".to_string());
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Validate for AutoUpdateCfg {
    fn validate(&self) -> Result<(), Vec<String>> {
        Ok(())
    }
}

/// valid options for sorting posts in the explorer
const VALID_SORT_OPTIONS: &[&str] = &[
    "date_newest",
    "date_oldest",
    "score_highest",
    "score_lowest",
    "size_largest",
    "size_smallest",
    "name_asc",
    "name_desc",
];

validator! { ExplorerCfg,
    default_sort => |v: &String| VALID_SORT_OPTIONS.contains(&v.to_lowercase().as_str()),
        "must be one of: date_newest, date_oldest, score_highest, score_lowest, size_largest, size_smallest, name_asc, name_desc";
    posts_per_page => |v: &usize| *v > 0 && *v <= 1000,
        "must be between 1 and 1000";
    slideshow_delay => |v: &u64| *v > 0,
        "must be greater than 0";
}

validator! { DownloadCfg,
    path => |v: &String| !v.trim().is_empty(),
        "must not be empty";
    pools_path => |v: &String| !v.trim().is_empty(),
        "must not be empty";
    threads => |v: &usize| *v >= 1 && *v <= 15,
        "must be between 1 and 15";
    format => |v: &String| !v.trim().is_empty() && v.contains("$id"),
        "must not be empty and must contain $id placeholder";
}

/// valid themes for the gallery
const VALID_THEMES: &[&str] = &[
    "rose-pine",
    "rose-pine-moon",
    "rose-pine-dawn",
    "catppuccin-latte",
    "catppuccin-frappe",
    "catppuccin-macchiato",
    "catppuccin-mocha",
];

validator! { GalleryCfg,
    port => |v: &u16| *v > 0,
        "must be a valid port (1-65535)";
    load_threads => |v: &usize| *v > 0,
        "must be greater than 0";
    theme => |v: &String| VALID_THEMES.contains(&v.to_lowercase().as_str()),
        "must be one of: rose-pine, rose-pine-moon, rose-pine-dawn, catppuccin-latte, catppuccin-frappe, catppuccin-macchiato, catppuccin-mocha";
}

impl Validate for E62Rs {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors: Vec<String> = Vec::new();

        macro_rules! validate_nested {
            ($($field:ident),* $(,)?) => {
                $(
                    if let Some(ref nested) = self.$field {
                        if let Err(nested_errors) = nested.validate() {
                            for err in nested_errors {
                                errors.push(format!("{}.{}", stringify!($field), err));
                            }
                        }
                    }
                )*
            };
        }

        validate_nested!(
            display,
            http,
            cache,
            performance,
            ui,
            search,
            login,
            completion,
            autoupdate,
            download,
            explorer,
            gallery,
            logging,
        );

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// format validation errors for display
pub fn format_validation_errors(errors: &[String]) -> String {
    let mut output = String::from("Configuration validation failed:\n");
    for (i, err) in errors.iter().enumerate() {
        output.push_str(&format!("  {}. {}\n", i + 1, err));
    }
    output
}

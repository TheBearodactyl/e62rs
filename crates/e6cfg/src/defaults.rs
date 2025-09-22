use crate::{CacheConfig, Cfg, HttpConfig, ImageDisplay, PerformanceConfig, UiConfig};

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
            blacklist: Some(
                vec!["young", "rape", "feral", "bestiality"]
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>(),
            ),
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
            max_connections: Some(2),
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
            concurrent_downloads: Some(2),
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

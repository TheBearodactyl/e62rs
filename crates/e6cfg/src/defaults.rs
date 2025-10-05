use crate::{
    AutoUpdateCfg, CacheConfig, CompletionCfg, DownloadCfg, E62Rs, HttpConfig, ImageDisplay,
    LoginCfg, PerformanceConfig, SearchCfg, SizeFormat, UiConfig,
};

impl Default for E62Rs {
    fn default() -> Self {
        Self {
            progress_format: Some(SizeFormat::default()),
            post_count: Some(320),
            autoupdate: Some(AutoUpdateCfg::default()),
            base_url: Some("https://e621.net".to_string()),
            display: Some(ImageDisplay::default()),
            http: Some(HttpConfig::default()),
            cache: Some(CacheConfig::default()),
            performance: Some(PerformanceConfig::default()),
            ui: Some(UiConfig::default()),
            search: Some(SearchCfg::default()),
            completion: Some(CompletionCfg::default()),
            login: Some(LoginCfg::default()),
            download: Some(DownloadCfg::default()),
            blacklist: Some(
                vec!["young", "rape", "feral", "bestiality"]
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>(),
            ),
        }
    }
}

impl Default for DownloadCfg {
    fn default() -> Self {
        Self {
            download_dir: Some("downloads".to_string()),
            output_format: Some(
                "$artists[3]/$rating/$tags[3] - $id - $date $time - $score.$ext".to_string(),
            ),
            #[cfg(target_os = "windows")]
            save_metadata: Some(true),
            #[cfg(not(target_os = "windows"))]
            save_metadata: Some(false),
            save_download_data: Some(true),
        }
    }
}

impl Default for CompletionCfg {
    fn default() -> Self {
        Self {
            tag_similarity_threshold: Some(0.8),
            tags: Some("data/tags.csv".to_string()),
            pools: Some("data/pools.csv".to_string()),
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
            max_connections: Some(15),
            http2_prior_knowledge: Some(true),
            tcp_keepalive: Some(true),
            user_agent: Some(format!(
                "{}/v{} (by {} on e621)",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                "bearodactyl"
            )),
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
            concurrent_downloads: Some(15),
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
            image_when_info: Some(true),
            sixel_quality: Some(100),
            resize_method: Some("lanczos3".to_string()),
        }
    }
}

impl Default for SearchCfg {
    fn default() -> Self {
        Self {
            min_posts_on_tag: Some(2),
            min_posts_on_pool: Some(2),
            show_inactive_pools: Some(true),
            sort_pools_by_post_count: Some(false),
            sort_tags_by_post_count: Some(true),
            min_post_score: Some(0),
            max_post_score: Some(i64::MAX),
            reverse_tags_order: Some(false),
            fetch_threads: Some(8),
        }
    }
}

impl Default for AutoUpdateCfg {
    fn default() -> Self {
        Self {
            tags: Some(true),
            pools: Some(true),
        }
    }
}

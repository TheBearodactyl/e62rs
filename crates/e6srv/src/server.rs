use crate::cfg::ServerConfig;
use crate::media::MediaGallery;
use crate::routes::{AppState, index_handler, list_media_handler, stats_handler};
use crate::theme::RosePine;
use anyhow::Result;
use axum::{Router, routing::get};
use std::sync::Arc;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct MediaServer {
    config: ServerConfig,
    theme: RosePine,
}

impl MediaServer {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            theme: RosePine,
        }
    }

    pub fn with_theme(config: ServerConfig, theme: RosePine) -> Self {
        Self { config, theme }
    }

    pub async fn serve(self) -> Result<()> {
        let gallery = MediaGallery::new(
            self.config.media_directory.clone(),
            self.config.enable_metadata_filtering,
        );
        let state = Arc::new(AppState::new(gallery, self.clone().theme));

        let app = self.build_router(state);

        let listener = tokio::net::TcpListener::bind(self.config.bind_address).await?;

        log::info!("e6srv running at http://{}", self.config.bind_address);
        log::info!(
            "Serving media from: {}",
            self.config.media_directory.display()
        );
        log::info!(
            "Metadata filtering: {}",
            if self.config.enable_metadata_filtering {
                "enabled"
            } else {
                "disabled"
            }
        );

        axum::serve(listener, app).await?;

        Ok(())
    }

    fn build_router(&self, state: Arc<AppState>) -> Router {
        Router::new()
            .route("/", get(index_handler))
            .route("/api/media", get(list_media_handler))
            .route("/api/stats", get(stats_handler))
            .nest_service("/files", ServeDir::new(&self.config.media_directory))
            .with_state(state)
    }
}

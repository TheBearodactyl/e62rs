use {
    crate::serve::{
        cfg::ServerConfig,
        media::gallery::MediaGallery,
        routes::{AppState, index_handler, list_media_handler, stats_handler},
    },
    axum::{Router, routing::get},
    color_eyre::eyre::Result,
    std::sync::Arc,
    tower_http::services::ServeDir,
    tracing::info,
};

#[derive(Clone)]
pub struct MediaServer {
    config: ServerConfig,
}

impl MediaServer {
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    pub async fn serve(self) -> Result<()> {
        let gallery = MediaGallery::new(
            self.config.media_directory.clone(),
            self.config.enable_metadata_filtering,
        );
        let state = Arc::new(AppState::new(gallery));

        let app = self.build_router(state);

        let listener = tokio::net::TcpListener::bind(self.config.bind_address).await?;

        info!("e6srv running at http://{}", self.config.bind_address);
        info!(
            "Serving media from: {}",
            self.config.media_directory.display()
        );
        info!(
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
            .nest_service("/static", ServeDir::new("public"))
            .with_state(state)
    }
}

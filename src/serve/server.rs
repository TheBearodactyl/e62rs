//! the actual media server
use {
    crate::serve::{
        cfg::ServerConfig,
        media::gallery::MediaGallery,
        routes::{
            AppState, css_handler, index_handler, js_handler, list_media_handler, stats_handler,
        },
    },
    color_eyre::eyre::Result,
    rocket::{Config, figment::Figment, fs::FileServer, routes},
    std::sync::Arc,
    tracing::info,
};

#[derive(Clone)]
/// the media server
pub struct MediaServer {
    /// the server config
    config: ServerConfig,
}

impl MediaServer {
    /// initialize the media server
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    /// serve the media server
    pub async fn serve(self) -> Result<()> {
        let gallery = MediaGallery::new(
            self.config.media_directory.clone(),
            self.config.enable_metadata_filtering,
        );
        let state = Arc::new(AppState::new(gallery));

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

        let figment = Figment::from(Config::default())
            .merge(("address", self.config.bind_address.ip()))
            .merge(("port", self.config.bind_address.port()));

        let mut rocket = rocket::custom(figment)
            .mount(
                "/",
                routes![
                    index_handler,
                    list_media_handler,
                    stats_handler,
                    css_handler,
                    js_handler
                ],
            )
            .mount("/files", FileServer::from(&self.config.media_directory));

        rocket = rocket.manage(state);
        rocket.launch().await?;

        Ok(())
    }
}

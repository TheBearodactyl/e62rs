//! routes for the gallery
use {
    crate::{
        getopt,
        serve::{
            media::{
                filter::MediaFilter, gallery::MediaGallery, item::MediaItem, stats::FilterStats,
            },
            theme::registry::ThemeRegistry,
        },
    },
    rocket::{
        State, get,
        http::Status,
        response::content::{RawCss, RawHtml, RawJavaScript},
        serde::json::Json,
    },
    std::sync::Arc,
    tokio::sync::RwLock,
};

/// the HTML code of the media gallery
const HTML_TEMPLATE: &str = include_str!("../../resources/gallery/index.html");
/// the CSS for the html template
const CSS: &str = include_str!("../../resources/gallery/styles.css");
/// the JS for the html template
const JS: &str = include_str!("../../resources/gallery/script.js");

/// the state of the app
pub struct AppState {
    /// the gallery
    pub gallery: Arc<RwLock<MediaGallery>>,
}

impl AppState {
    /// initialize a new AppState
    pub fn new(gallery: MediaGallery) -> Self {
        Self {
            gallery: Arc::new(RwLock::new(gallery)),
        }
    }
}

#[get("/")]
/// the main gallery page
pub async fn index_handler(_state: &State<Arc<AppState>>) -> RawHtml<String> {
    RawHtml(HTML_TEMPLATE.to_string())
}

#[get("/styles.css")]
/// serve the dynamically themed CSS
pub async fn css_handler() -> RawCss<String> {
    let configured_theme = getopt!(gallery.theme);
    let registry = ThemeRegistry::new();
    let css_vars = registry.get_theme_css_vars(&configured_theme).unwrap();
    let css_code = CSS.replace("/* {{THEME_CSS_VARS}} */", &css_vars);
    RawCss(css_code)
}

#[get("/script.js")]
/// serve the JS
pub async fn js_handler() -> RawJavaScript<String> {
    RawJavaScript(JS.to_string())
}

#[get("/api/media?<filter..>")]
/// handler for listing media
pub async fn list_media_handler(
    state: &State<Arc<AppState>>,
    filter: Option<MediaFilter>,
) -> Result<Json<Vec<MediaItem>>, Status> {
    let mut gallery = state.gallery.write().await;
    let filter = filter.unwrap_or_default();

    let items = gallery
        .get_filtered_items(&filter)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(items))
}

#[get("/api/stats")]
/// handler for getting current filter stats
pub async fn stats_handler(state: &State<Arc<AppState>>) -> Result<Json<FilterStats>, Status> {
    let gallery = state.gallery.read().await;
    let stats = gallery.get_filter_stats();
    Ok(Json(stats))
}

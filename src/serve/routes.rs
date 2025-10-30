use {
    crate::{
        config::options::E62Rs,
        serve::{
            media::{filter::MediaFilter, gallery::MediaGallery},
            theme::registry::ThemeRegistry,
        },
    },
    axum::{
        extract::{Query, State},
        response::{Html, IntoResponse, Response},
    },
    reqwest::StatusCode,
    std::sync::Arc,
    tokio::sync::RwLock,
};

pub struct AppState {
    pub gallery: Arc<RwLock<MediaGallery>>,
}

impl AppState {
    pub fn new(gallery: MediaGallery) -> Self {
        Self {
            gallery: Arc::new(RwLock::new(gallery)),
        }
    }
}

pub async fn index_handler(State(_): State<Arc<AppState>>) -> Html<String> {
    let configured_theme = E62Rs::get_unsafe().gallery.theme;

    let registry = ThemeRegistry::new();
    let css_vars = registry.get_theme_css_vars(&configured_theme).unwrap();
    let html = HTML_TEMPLATE.replace("{{THEME_CSS_VARS}}", &css_vars);
    Html(html)
}

pub async fn list_media_handler(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<MediaFilter>,
) -> Result<Response, StatusCode> {
    let mut gallery = state.gallery.write().await;

    let items = gallery
        .get_filtered_items(&filter)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/json")],
        serde_json::to_string(&items).unwrap(),
    )
        .into_response())
}

pub async fn stats_handler(State(state): State<Arc<AppState>>) -> Result<Response, StatusCode> {
    let gallery = state.gallery.read().await;
    let stats = gallery.get_filter_stats();

    Ok((
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/json")],
        serde_json::to_string(&stats).unwrap(),
    )
        .into_response())
}

const HTML_TEMPLATE: &str = include_str!("../../index.html");

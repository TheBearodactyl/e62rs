use {
    crate::{
        data::{pools::PoolDatabase, tags::TagDatabase},
        models::{PoolEntry, TagEntry},
    },
    color_eyre::eyre::{Context, Result},
    rocket::{get, serde::json::Json},
    std::sync::Arc,
};

fn load_data() -> Result<(Arc<TagDatabase>, Arc<PoolDatabase>)> {
    let tag_db = Arc::new(
        TagDatabase::load()
            .context("Failed to load tag database. Please ensure data/tags.csv exists")?,
    );

    let pool_db = Arc::new(
        PoolDatabase::load()
            .context("Failed to load pool database. Please ensure data/pools.csv exists")?,
    );

    Ok((tag_db, pool_db))
}

#[get("/api/v1/tags/exists/<tag>")]
pub async fn tag_exists(tag: &str) -> std::io::Result<Json<bool>> {
    let (tag_db, _) = load_data().expect("Failed to load databases");

    Ok(Json(tag_db.exists(tag)))
}

#[get("/api/v1/tags/list")]
pub async fn list_tags() -> std::io::Result<Json<Vec<TagEntry>>> {
    let (tag_db, _) = load_data().expect("Failed to load databases");

    Ok(Json(tag_db.list()))
}

#[get("/api/v1/pools/list")]
pub async fn list_pools() -> std::io::Result<Json<Vec<PoolEntry>>> {
    let (_, pool_db) = load_data().expect("Failed to load databases");

    Ok(Json(pool_db.list()))
}

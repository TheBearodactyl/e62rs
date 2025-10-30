pub mod handlers;

use {
    color_eyre::eyre::Result,
    rocket::{launch, routes},
    tracing::info,
};

pub async fn run_api() -> Result<()> {
    let _rocket = rocket::build()
        .mount(
            "/",
            routes![
                crate::serve::api::handlers::data::list_tags,
                crate::serve::api::handlers::data::tag_exists,
                crate::serve::api::handlers::data::list_pools
            ],
        )
        .launch()
        .await?;

    info!("API started successfully");

    Ok(())
}

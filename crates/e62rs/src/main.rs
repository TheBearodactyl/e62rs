use anyhow::{Context, Result};
use e6cfg::Cfg;
use e6core::{
    client::E6Client,
    data::{pools::PoolDatabase, tags::TagDatabase},
};
use e6ui::ui::{E6Ui, menus::MainMenu};
use env_logger::{Builder, Env};
use log::error;
use log::info;
use std::{process, sync::Arc};

#[tokio::main]
async fn main() -> Result<()> {
    Builder::from_env(Env::default().default_filter_or("info")).init();

    if let Err(e) = run().await {
        error!("Application error: {:#}", e);
        process::exit(1);
    }

    info!("Application finished successfully");
    Ok(())
}

async fn run() -> Result<()> {
    let client = Arc::new(E6Client::default());

    client.update_tags().await?;
    client.update_pools().await?;

    let tag_db = Arc::new(
        TagDatabase::load()
            .context("Failed to load tag database. Please ensure data/tags.csv exists")?,
    );

    let pool_db = Arc::new(
        PoolDatabase::load()
            .context("Failed to load pool database. Please ensure data/pools.csv exists")?,
    );

    info!(
        "Starting {} v0.2.0 using {}",
        env!("CARGO_PKG_NAME"),
        Cfg::get().unwrap_or_default().base_url.unwrap()
    );

    let selection = MainMenu::select("What would you like to do?").prompt()?;
    let ui = E6Ui::new(client, tag_db, pool_db);

    loop {
        match selection {
            MainMenu::SearchPosts => ui.search_posts().await?,
            MainMenu::SearchPools => ui.search_pools().await?,
            MainMenu::SearchPoolsAdv => ui.search_pools_adv().await?,
            MainMenu::ViewLatest => ui.display_latest_posts().await?,
            MainMenu::ViewBlacklist => ui.show_blacklist_info()?,
            MainMenu::ManageBlacklist => ui.manage_blacklist().await?,
            MainMenu::Exit => break,
        }
    }

    Ok(())
}

use {
    anyhow::{Context, Result},
    e6cfg::E62Rs,
    e6core::{
        client::E6Client,
        data::{pools::PoolDatabase, tags::TagDatabase},
    },
    e6ui::ui::{E6Ui, menus::MainMenu},
    env_logger::{Builder, Env},
    log::{error, info},
    std::{process, sync::Arc},
};

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
    let cfg = E62Rs::get().unwrap_or_default();
    let autoup = cfg.autoupdate.unwrap_or_default();
    let client = Arc::new(E6Client::default());

    if autoup.tags.unwrap_or_default() {
        client.update_tags().await?;
    }

    if autoup.pools.unwrap_or_default() {
        client.update_pools().await?;
    }

    let tag_db = Arc::new(
        TagDatabase::load()
            .context("Failed to load tag database. Please ensure data/tags.csv exists")?,
    );

    let pool_db = Arc::new(
        PoolDatabase::load()
            .context("Failed to load pool database. Please ensure data/pools.csv exists")?,
    );

    info!(
        "Starting {} v{} using {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        cfg.base_url.unwrap_or_default()
    );

    let selection = MainMenu::select("What would you like to do?").prompt()?;
    let ui = E6Ui::new(client, tag_db, pool_db);

    loop {
        match selection {
            MainMenu::SearchPosts => ui.search_posts().await?,
            MainMenu::SearchPools => ui.search_pools().await?,
            MainMenu::SearchPoolsAdv => ui.search_pools_adv().await?,
            MainMenu::ViewLatest => ui.display_latest_posts().await?,
            MainMenu::ManageBlacklist => ui.manage_blacklist().await?,
            MainMenu::ReorganizeDownloads => ui.reorganize_downloads().await?,
            MainMenu::EditConfig => ui.edit_config_file().await?,
            MainMenu::Exit => break,
        }
    }

    Ok(())
}

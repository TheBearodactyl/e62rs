use {
    anyhow::{Context, Result},
    clap::Parser,
    e6cfg::E62Rs,
    e6core::{
        client::E6Client,
        data::{pools::PoolDatabase, tags::TagDatabase},
    },
    e6ui::ui::{
        E6Ui,
        menus::{MainMenu, PoolSearchModeMenu},
    },
    std::{process, sync::Arc},
    tracing::*,
    tracing_subscriber::FmtSubscriber,
};

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    init: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .pretty()
        .without_time()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    if let Err(e) = run().await {
        error!("Application error: {:#}", e);
        process::exit(1);
    }

    info!("Application finished successfully");

    Ok(())
}

async fn run() -> Result<()> {
    let argv = Cli::parse();
    let cfg = E62Rs::get()?;
    let autoup = cfg.clone().autoupdate;
    let client = Arc::new(E6Client::default());

    if argv.init {
        cfg.save_to_file("./e62rs.toml")?;
    }

    if autoup.tags {
        client.update_tags().await?;
    }

    if autoup.pools {
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
        cfg.base_url
    );

    let selection = MainMenu::select("What would you like to do?").prompt()?;
    let ui = E6Ui::new(client, tag_db, pool_db);

    loop {
        match selection {
            MainMenu::SearchPosts => {
                ui.search_posts().await?;
                continue;
            }
            MainMenu::SearchPools => {
                let search_mode =
                    PoolSearchModeMenu::select("Which search mode would you like to use?")
                        .prompt()?;

                match search_mode {
                    PoolSearchModeMenu::Simple => ui.search_pools().await?,
                    PoolSearchModeMenu::Advanced => ui.search_pools_adv().await?,
                }

                continue;
            }
            MainMenu::ViewLatest => {
                ui.display_latest_posts().await?;
                continue;
            }
            MainMenu::ManageBlacklist => {
                ui.manage_blacklist().await?;
                continue;
            }
            MainMenu::OpenDownloadsInBrowser => {
                ui.serve_downloads().await?;
                continue;
            }
            MainMenu::ReorganizeDownloads => {
                ui.reorganize_downloads().await?;
                continue;
            }
            MainMenu::EditConfig => {
                ui.edit_config_file().await?;
                continue;
            }
            MainMenu::ExploreDownloads => {
                ui.explore_downloads().await?;
                continue;
            }
            MainMenu::Exit => break,
        }
    }

    Ok(())
}

use {
    crate::{
        cli::cli,
        client::E6Client,
        config::options::{E62Rs, LoggingFormat},
        data::{pools::PoolDatabase, tags::TagDatabase},
        serve::api::run_api,
        ui::{
            E6Ui, ROSE_PINE,
            menus::{MainMenu, PoolSearchModeMenu},
        },
        utils::string_to_log_level,
    },
    color_eyre::eyre::{Context, Result},
    std::sync::Arc,
    tracing::info,
    tracing_subscriber::FmtSubscriber,
};

pub async fn run() -> Result<()> {
    cli().await?;
    let cfg = E62Rs::get()?;
    let autoup = cfg.clone().autoupdate;
    let client = Arc::new(E6Client::default());

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

    let ui = E6Ui::new(client, tag_db, pool_db);
    let selection = MainMenu::select("What would you like to do?")
        .theme(&ROSE_PINE)
        .run()?;

    loop {
        match selection {
            MainMenu::SearchPosts => {
                ui.search_posts().await?;
            }
            MainMenu::SearchPools => {
                let search_mode =
                    PoolSearchModeMenu::select("Which search mode would you like to use?")
                        .theme(&ROSE_PINE)
                        .run()?;

                match search_mode {
                    PoolSearchModeMenu::Simple => ui.search_pools().await?,
                    PoolSearchModeMenu::Advanced => ui.search_pools_adv().await?,
                }
            }
            MainMenu::ViewLatest => {
                ui.display_latest_posts().await?;
            }
            MainMenu::ManageBlacklist => {
                ui.manage_blacklist().await?;
            }
            MainMenu::OpenDownloadsInBrowser => {
                ui.serve_downloads().await?;
            }
            MainMenu::ReorganizeDownloads => {
                ui.reorganize_downloads().await?;
            }
            MainMenu::EditConfig => {
                ui.edit_config_file().await?;
            }
            MainMenu::ExploreDownloads => {
                ui.explore_downloads().await?;
            }
            MainMenu::Exit => break,
        }
    }

    Ok(())
}

pub fn setup_logging() -> Result<()> {
    let cfg = E62Rs::get()?;
    let logging_config = cfg.logging;
    let max_level = string_to_log_level(&logging_config.level);

    match logging_config.log_format {
        LoggingFormat::Pretty => {
            let subscriber = FmtSubscriber::builder()
                .pretty()
                .with_max_level(max_level)
                .with_ansi(logging_config.asni)
                .with_line_number(logging_config.line_numbers)
                .with_target(logging_config.event_targets);

            if logging_config.enable {
                tracing::subscriber::set_global_default(subscriber.finish())?;
                tracing::info!("Logging setup successfully");
            }
        }
        LoggingFormat::Compact => {
            let subscriber = FmtSubscriber::builder()
                .compact()
                .with_max_level(max_level)
                .with_ansi(logging_config.asni)
                .with_line_number(logging_config.line_numbers)
                .with_target(logging_config.event_targets);

            if logging_config.enable {
                tracing::subscriber::set_global_default(subscriber.finish())?;
                tracing::info!("Logging setup successfully");
            }
        }
    }

    Ok(())
}

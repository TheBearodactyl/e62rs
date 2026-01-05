//! the core app
use {
    super::{handlers::Handlers, interrupt::InterruptHandler, logging},
    crate::{
        app::cli::Cli,
        client::E6Client,
        config::instance::reload_config,
        data::{pools::PoolDb, tags::TagDb},
        getopt, opt_and,
        ui::E6Ui,
    },
    color_eyre::eyre::Context,
    std::sync::Arc,
    tracing::info,
};

/// the e62rs app
pub struct E6App {
    /// the logic handlers
    handlers: Handlers,
}

impl E6App {
    /// init e62rs
    pub async fn init() -> color_eyre::Result<Self> {
        color_eyre::install()?;
        Cli::run().await?;
        Self::clear_screen();

        if getopt!(logging.enable) {
            logging::setup()?;
        }

        let interrupt = Self::setup_interrupt_handler()?;
        let ui = Self::setup_ui().await?;

        reload_config()?;

        Ok(Self {
            handlers: Handlers::new(ui, interrupt),
        })
    }

    /// run the main loop
    pub async fn run(&self) -> color_eyre::Result<()> {
        self.handlers.run_main_loop().await
    }

    /// clear the screen
    fn clear_screen() {
        print!("\x1B[2J\x1B[3J\x1B[H");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }

    /// setup the interruption handler
    fn setup_interrupt_handler() -> color_eyre::Result<InterruptHandler> {
        let handler = InterruptHandler::new();
        let handler_clone = handler.clone();

        ctrlc::set_handler(move || {
            handler_clone.trigger();
        })
        .context("failed to set Ctrl+C handler")?;

        Ok(handler)
    }

    /// setup the UI
    async fn setup_ui() -> color_eyre::Result<E6Ui> {
        let client = Arc::new(E6Client::default());

        opt_and!(autoupdate.tags, client.update_tags().await?);
        opt_and!(autoupdate.pools, client.update_pools().await?);

        let tag_db = Arc::new(TagDb::load().context("failed to load tag db")?);
        let pool_db = Arc::new(PoolDb::load().context("failed to load pool db")?);

        info!(
            "Starting {} v{} using {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            getopt!(base_url)
        );

        Ok(E6Ui::new(client, tag_db, pool_db))
    }
}

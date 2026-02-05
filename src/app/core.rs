//! the core app
use {
    super::{handlers::Handlers, interrupt::InterruptHandler, logging},
    crate::{
        app::cli::Cli,
        client::E6Client,
        config::instance::reload_config,
        data::{pools::PoolDb, tags::TagDb},
        error::Result,
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
    /// initialize e62rs
    ///
    /// - 1. installs the miette error handler hook
    /// - 2. sets up the custom inquire render config
    /// - 3. validates the base api url
    /// - 4. handles any cli arguments if any
    /// - 5. clears the screen
    /// - 6. sets up logging
    /// - 8. sets up the custom interruption handler if enabled
    /// - 9. sets up the ui
    /// - 10. loads the config file
    ///
    /// # Errors
    ///
    /// returns an error if color_eyre fails to install [`color_eyre::install`]  
    /// returns an error if the cli fails to run  
    /// returns an error if it fails to setup logging  
    /// returns an error if it fails to setup the interrupt handler  
    /// returns an error if it fails to setup the UI  
    /// returns an error if it fails to load the configuration file  
    pub async fn init() -> Result<Self> {
        miette::set_hook(Box::new(|_| {
            Box::new(
                miette::MietteHandlerOpts::new()
                    .terminal_links(true)
                    .unicode(true)
                    .context_lines(3)
                    .tab_width(4)
                    .build(),
            )
        }))?;
        Self::setup_inquire_render_config();

        if crate::utils::ꟿ(crate::getopt!(http.api)) {
            std::process::exit(1);
        }

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
    ///
    /// # Errors
    ///
    /// returns an error if the main loop fails
    pub async fn run(&self) -> Result<()> {
        self.handlers.run_main_loop().await
    }

    /// sets up inquire with the rose pine theme
    fn setup_inquire_render_config() {
        inquire::set_global_render_config(crate::ui::themes::get_render_config());
    }

    /// clear the screen
    fn clear_screen() {
        print!("\x1B[2J\x1B[3J\x1B[H");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }

    /// setup the interruption handler
    fn setup_interrupt_handler() -> Result<InterruptHandler> {
        let handler = InterruptHandler::new();
        let handler_clone = handler.clone();

        if getopt!(ui.ctrlc_handler) {
            ctrlc::set_handler(move || {
                handler_clone.trigger();
            })
            .context("failed to set Ctrl+C handler")?;
        }

        Ok(handler)
    }

    /// setup the UI
    async fn setup_ui() -> Result<E6Ui> {
        let client = Arc::new(E6Client::default());

        opt_and!(autoupdate.tags, client.update_tags().await?);
        opt_and!(autoupdate.pools, client.update_pools().await?);

        let tag_db = Arc::new(TagDb::load().context("failed to load tag db")?);
        let pool_db = Arc::new(PoolDb::load().context("failed to load pool db")?);

        info!(
            "Starting {} v{} using {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            getopt!(http.api)
        );

        Ok(E6Ui::new(client, tag_db, pool_db))
    }
}

//! main menu stuff
use {
    super::Handlers,
    crate::{
        config::instance::reload_config,
        ui::{menus::MainMenu, themes::ROSE_PINE},
    },
};

impl Handlers {
    /// run the main loop
    pub async fn run_main_loop(&self) -> color_eyre::Result<()> {
        'main: loop {
            let selection = match MainMenu::select("What would you like to do?")
                .theme(&ROSE_PINE)
                .run()
            {
                Ok(sel) => sel,
                Err(_) if self.was_interrupted() => continue 'main,
                Err(e) => return Err(e.into()),
            };

            match selection {
                MainMenu::ManageBlacklist => self.ui.manage_blacklist().await?,
                MainMenu::EditConfig => self.ui.edit_config_file().await?,
                MainMenu::ViewLatest => self.ui.display_latest_posts().await?,
                MainMenu::OpenInBrowser => self.ui.serve_downloads().await?,
                MainMenu::Reorganize => self.ui.reorganize_downloads().await?,
                MainMenu::ExploreDownloads => self.ui.explore_downloads().await?,
                MainMenu::UpdateDownloads => self.ui.redownload_by_artists().await?,
                MainMenu::Search => self.handle_search().await?,
                MainMenu::ReloadConfig => reload_config()?,
                MainMenu::Exit => break 'main,
            }
        }

        Ok(())
    }
}

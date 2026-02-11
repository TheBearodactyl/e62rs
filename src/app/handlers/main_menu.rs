//! main menu stuff
use {
    super::Handlers,
    crate::{
        config::instance::reload_config,
        error::Result,
        ui::menus::{
            ConfigMenu, MainMenu, blacklist::BlacklistMenu, explore::ExploreMenu,
            reorganize::RegorganizeMenu, view::ViewMenu,
        },
    },
    miette::IntoDiagnostic,
};

impl Handlers {
    /// run the main loop
    ///
    /// [`MainMenu::ManageBlacklist`] runs the blacklist manager
    /// [`MainMenu::EditConfig`] lets the user edit their config file
    /// [`MainMenu::ViewLatest`] displays the latest uploads on e621
    /// [`MainMenu::OpenInBrowser`] opens the downloads gallery in the users browser
    /// [`MainMenu::Reorganize`] runs the downloads reorganizer
    /// [`MainMenu::ExploreDownloads`] runs the downloads explorer
    /// [`MainMenu::UpdateDownloads`] runs the downloads updater
    /// [`MainMenu::Search`] runs the search menu (see [`crate::app::handlers::search`])
    /// [`MainMenu::ReloadConfig`] reloads and reapplies the config file
    /// [`MainMenu::Exit`] exits e62rs
    ///
    /// # Errors
    ///
    /// returns an error if it fails to get the user selection in the main menu
    /// returns an error if it fails to run the logic associated with the user selection
    pub async fn run_main_loop(&self) -> Result<()> {
        'main: loop {
            let selection = match MainMenu::select("What would you like to do?").ask() {
                Ok(sel) => sel,
                Err(_) if self.was_interrupted() => continue 'main,
                Err(e) => return Err(e.into()),
            };

            match selection {
                MainMenu::ManageBlacklist => self.ui.manage_blacklist().await.into_diagnostic()?,
                MainMenu::ViewLatest => self.ui.display_latest_posts().await.into_diagnostic()?,
                MainMenu::OpenInBrowser => self.ui.serve_downloads().await.into_diagnostic()?,
                MainMenu::Reorganize => self.ui.reorganize_downloads().await.into_diagnostic()?,
                MainMenu::ExploreDownloads => {
                    self.ui.explore_downloads().await.into_diagnostic()?
                }
                MainMenu::UpdateDownloads => {
                    self.ui.redownload_by_artists().await.into_diagnostic()?
                }
                MainMenu::Search => self.handle_search().await?,
                MainMenu::ManageConfig => {
                    match ConfigMenu::select("What would you like to do?").ask() {
                        Ok(sel) => match sel {
                            ConfigMenu::Edit => {
                                self.ui.edit_config_file().await.into_diagnostic()?
                            }
                            ConfigMenu::Reload => reload_config()?,
                            ConfigMenu::Back => continue 'main,
                        },

                        Err(_) if self.was_interrupted() => continue 'main,
                        Err(e) => return Err(e.into()),
                    }
                }
                MainMenu::Exit => break 'main,
            }
        }

        Ok(())
    }
}

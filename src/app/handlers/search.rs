//! search handling stuff
//!
//! See [`Handlers::handle_search`] and [`Handlers::handle_pool_search`]
use {
    super::Handlers,
    crate::ui::menus::{PoolSearchModeMenu, SearchMenu, search::SearchMenu as _},
    miette::IntoDiagnostic,
};

impl Handlers {
    /// handle search logic
    ///
    /// opens a menu so that the user can choose to either
    /// search for posts or for pools.
    ///
    /// [`SearchMenu::Posts`] searches posts by their tags
    /// [`SearchMenu::Pools`] searches pools (see [`Handlers::handle_pool_search`])
    ///
    /// # Errors
    ///
    /// returns an error if it fails to get the user selection
    /// returns an error if it fails to run the logic associated with the user selection
    pub async fn handle_search(&self) -> miette::Result<()> {
        let selection = match SearchMenu::select("What would you like to search for?").ask() {
            Ok(sel) => sel,
            Err(_) if self.was_interrupted() => return Ok(()),
            Err(e) => return Err(e),
        };

        match selection {
            SearchMenu::Posts => self.ui.search_posts().await.into_diagnostic()?,
            SearchMenu::Pools => self.handle_pool_search().await?,
            SearchMenu::Back => {}
        }

        Ok(())
    }

    /// handle pool search
    ///
    /// opens up a menu so that the user can
    /// choose a search mode for searching pools
    ///
    /// [`PoolSearchModeMenu::Simple`] just shows a menu to search by the pool id
    /// [`PoolSearchModeMenu::Advanced`] allows the user to apply advanced filters
    ///
    /// # Errors
    ///
    /// returns an error if it fails to get the user selection
    /// returns an error if it fails to run the logic associated with the user selection
    pub async fn handle_pool_search(&self) -> miette::Result<()> {
        let pool_mode =
            match PoolSearchModeMenu::select("Which search mode would you like to use?").ask() {
                Ok(sel) => sel,
                Err(_) if self.was_interrupted() => return Ok(()),
                Err(e) => return Err(e),
            };

        match pool_mode {
            PoolSearchModeMenu::Simple => self.ui.search_pools().await.into_diagnostic()?,
            PoolSearchModeMenu::Advanced => self.ui.search_pools_adv().await.into_diagnostic()?,
        }

        Ok(())
    }
}

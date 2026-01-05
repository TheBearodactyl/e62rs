//! search ui stuff
use {
    super::Handlers,
    crate::ui::{
        menus::{PoolSearchModeMenu, SearchMenu},
        themes::ROSE_PINE,
    },
};

impl Handlers {
    /// handle search logic
    pub(crate) async fn handle_search(&self) -> color_eyre::Result<()> {
        let selection = match SearchMenu::select("What would you like to search for?")
            .theme(&ROSE_PINE)
            .run()
        {
            Ok(sel) => sel,
            Err(_) if self.was_interrupted() => return Ok(()),
            Err(e) => return Err(e.into()),
        };

        match selection {
            SearchMenu::Posts => self.ui.search_posts().await?,
            SearchMenu::Pools => self.handle_pool_search().await?,
            SearchMenu::Back => {}
        }

        Ok(())
    }

    /// handle pool search
    async fn handle_pool_search(&self) -> color_eyre::Result<()> {
        let pool_mode = match PoolSearchModeMenu::select("Which search mode would you like to use?")
            .theme(&ROSE_PINE)
            .run()
        {
            Ok(sel) => sel,
            Err(_) if self.was_interrupted() => return Ok(()),
            Err(e) => return Err(e.into()),
        };

        match pool_mode {
            PoolSearchModeMenu::Simple => self.ui.search_pools().await?,
            PoolSearchModeMenu::Advanced => self.ui.search_pools_adv().await?,
        }

        Ok(())
    }
}

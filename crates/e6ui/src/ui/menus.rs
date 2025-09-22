use inquire_derive::Selectable;

#[derive(Selectable, Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum InteractionMenu {
    /// Open the post in your browser
    OpenInBrowser,
    /// Download the post
    Download,
    /// View the post image in terminal (requires a SIXEL compatible terminal)
    View,
    /// Go back to search
    Back,
}

#[derive(Selectable, Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum PoolInteractionMenu {
    /// View posts from this pool
    ViewPosts,
    /// Download all posts from this pool
    DownloadPool,
    /// Open pool page in browser
    OpenInBrowser,
    /// Go back to pool search
    Back,
}

#[derive(Selectable, Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum BatchAction {
    /// Download all selected posts
    DownloadAll,
    /// Open all selected posts in browser
    OpenAllInBrowser,
    /// Download and open all selected posts
    DownloadAndOpenAll,
    /// View all post images in terminal (requires a SIXEL compatible terminal)
    ViewAll,
    /// Go back to search
    Back,
}

#[derive(Selectable, Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum AdvPoolSearch {
    /// Search by name
    ByName,
    /// Search by description
    ByDesc,
    /// Search by creator
    ByCreator,
    /// Browse latest pools
    BrowseLatest,
    /// Go back to main menu
    Back,
}

#[derive(Selectable, Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum BlacklistManager {
    /// Show current blacklist
    ShowCurrent,
    /// Add tag to blacklist
    AddTag,
    /// Remove tag from blacklist
    RemoveTag,
    /// Clear entire blacklist
    Clear,
    /// Import tags from search
    ImportFromSearch,
    /// Go back to main menu
    Back,
}

#[derive(Selectable, Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum MainMenu {
    /// Search posts
    SearchPosts,
    /// Search pools
    SearchPools,
    /// Search pools (advanced filters)
    SearchPoolsAdv,
    /// View the latest posts
    ViewLatest,
    /// View your blacklist
    ViewBlacklist,
    /// Manage your blacklist
    ManageBlacklist,
    /// Exit e62rs
    Exit,
}

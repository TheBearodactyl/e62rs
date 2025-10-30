use inquire_derive::Selectable;

pub mod blacklist;
pub mod download;
pub mod explorer;
pub mod reorganize;
pub mod search;
pub mod view;

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
    /// View the latest posts
    ViewLatest,
    /// View downloaded files
    ExploreDownloads,
    /// Open your downloads with your browser
    OpenDownloadsInBrowser,
    /// Manage your blacklist
    ManageBlacklist,
    /// Reorganize already downloaded files
    ReorganizeDownloads,
    /// Edit your config file
    EditConfig,
    /// Exit e62rs
    Exit,
}

#[derive(Selectable, Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum PoolSearchModeMenu {
    /// Simple search
    Simple,

    /// Advanced search
    Advanced,
}

#[derive(Selectable, Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum ExplorerMenu {
    /// Browse downloaded posts
    BrowsePosts,
    /// Search posts by tags, ID, uploader, or description
    SearchPosts,
    /// Filter by rating
    FilterByRating,
    /// Sort posts
    SortBy,
    /// View statistics
    ViewStatistics,
    /// Clear all filters
    ClearFilters,
    /// Watch a slideshow
    Slideshow,
    /// Go back to main menu
    Back,
}

#[derive(Selectable, Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum ExplorerSortBy {
    /// Sort by date (newest first)
    DateNewest,
    /// Sort by date (oldest first)
    DateOldest,
    /// Sort by score (highest first)
    ScoreHighest,
    /// Sort by score (lowest first)
    ScoreLowest,
    /// Sort by favorites (highest first)
    FavoritesHighest,
    /// Sort by ID (ascending)
    IdAscending,
    /// Sort by ID (descending)
    IdDescending,
}

#[derive(Selectable, Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum ExplorerFilterBy {
    /// All ratings
    AllRatings,
    /// Safe only
    Safe,
    /// Questionable only
    Questionable,
    /// Explicit only
    Explicit,
}

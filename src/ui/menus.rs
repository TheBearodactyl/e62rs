pub mod blacklist;
pub mod download;
pub mod explorer;
pub mod reorganize;
pub mod search;
pub mod view;

use {
    crate::ui::{ROSE_PINE, RosePineTheme},
    color_eyre::eyre::Result,
    demand::{DemandOption, Select, Theme},
    enum_display::EnumDisplay,
    std::fmt::Display,
};

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, EnumDisplay)]
pub enum InteractionMenu {
    #[display("Open the post in your browser")]
    OpenInBrowser,
    #[display("Download the post")]
    Download,
    #[display("View the post in your terminal (requires a SIXEL compatible terminal)")]
    View,
    #[display("Go back to search")]
    Back,
}

impl InteractionMenu {
    pub fn select(prompt: &str) -> Select<'_, Self> {
        let theme = Theme::rose_pine().clone();
        Select::new(prompt)
            .filterable(false)
            .option(DemandOption::new(Self::OpenInBrowser).label("Open the post in your browser"))
            .option(DemandOption::new(Self::Download).label("Download the post"))
            .option(
                DemandOption::new(Self::View).label(
                    "View the post image in terminal (requires a SIXEL compatible terminal)",
                ),
            )
            .option(DemandOption::new(Self::Back).label("Go back to search"))
            .theme(&ROSE_PINE)
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, EnumDisplay)]
pub enum PoolInteractionMenu {
    #[display("View posts from this pool")]
    ViewPosts,
    #[display("Download all posts from this pool")]
    DownloadPool,
    #[display("Open pool page in browser")]
    OpenInBrowser,
    #[display("Go back to pool search")]
    Back,
}

impl PoolInteractionMenu {
    pub fn select(prompt: &str) -> Select<'_, Self> {
        Select::new(prompt)
            .theme(&ROSE_PINE)
            .filterable(false)
            .option(DemandOption::new(Self::ViewPosts).label("View posts from this pool"))
            .option(
                DemandOption::new(Self::DownloadPool).label("Download all posts from this pool"),
            )
            .option(DemandOption::new(Self::OpenInBrowser).label("Open pool page in browser"))
            .option(DemandOption::new(Self::Back).label("Go back to pool search"))
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, EnumDisplay)]
pub enum BatchAction {
    DownloadAll,
    OpenAllInBrowser,
    DownloadAndOpenAll,
    ViewAll,
    Back,
}

impl BatchAction {
    pub fn select(prompt: &str) -> Select<'_, Self> {
        Select::new(prompt)
            .theme(&ROSE_PINE)
            .filterable(false)
            .option(DemandOption::new(Self::DownloadAll).label("Download all selected posts"))
            .option(
                DemandOption::new(Self::OpenAllInBrowser)
                    .label("Open all selected posts in browser"),
            )
            .option(
                DemandOption::new(Self::DownloadAndOpenAll)
                    .label("Download and open all selected posts"),
            )
            .option(
                DemandOption::new(Self::ViewAll).label(
                    "View all post images in terminal (requires a SIXEL compatible terminal)",
                ),
            )
            .option(DemandOption::new(Self::Back).label("Go back to search"))
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, EnumDisplay)]
pub enum AdvPoolSearch {
    ByName,
    ByDesc,
    ByCreator,
    BrowseLatest,
    Back,
}

impl AdvPoolSearch {
    pub fn select(prompt: &str) -> Select<'_, Self> {
        Select::new(prompt)
            .theme(&ROSE_PINE)
            .filterable(false)
            .option(DemandOption::new(Self::ByName).label("Search by name"))
            .option(DemandOption::new(Self::ByDesc).label("Search by description"))
            .option(DemandOption::new(Self::ByCreator).label("Search by creator"))
            .option(DemandOption::new(Self::BrowseLatest).label("Browse latest pools"))
            .option(DemandOption::new(Self::Back).label("Go back to main menu"))
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, EnumDisplay)]
pub enum BlacklistManager {
    ShowCurrent,
    AddTag,
    RemoveTag,
    Clear,
    ImportFromSearch,
    Back,
}

impl BlacklistManager {
    pub fn select(prompt: &str) -> Select<'_, Self> {
        Select::new(prompt)
            .theme(&ROSE_PINE)
            .filterable(false)
            .option(DemandOption::new(Self::ShowCurrent).label("Show current blacklist"))
            .option(DemandOption::new(Self::AddTag).label("Add tag to blacklist"))
            .option(DemandOption::new(Self::RemoveTag).label("Remove tag from blacklist"))
            .option(DemandOption::new(Self::Clear).label("Clear entire blacklist"))
            .option(DemandOption::new(Self::ImportFromSearch).label("Import tags from search"))
            .option(DemandOption::new(Self::Back).label("Go back to main menu"))
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, EnumDisplay)]
pub enum MainMenu {
    SearchPosts,
    SearchPools,
    ViewLatest,
    ExploreDownloads,
    OpenDownloadsInBrowser,
    ManageBlacklist,
    ReorganizeDownloads,
    EditConfig,
    Exit,
}

impl MainMenu {
    pub fn select(prompt: &str) -> Select<'_, Self> {
        Select::new(prompt)
            .theme(&ROSE_PINE)
            .filterable(true)
            .option(DemandOption::new(Self::SearchPosts).label("Search posts"))
            .option(DemandOption::new(Self::SearchPools).label("Search pools"))
            .option(DemandOption::new(Self::ViewLatest).label("View the latest posts"))
            .option(DemandOption::new(Self::ExploreDownloads).label("View downloaded files"))
            .option(
                DemandOption::new(Self::OpenDownloadsInBrowser)
                    .label("Open your downloads with your browser"),
            )
            .option(DemandOption::new(Self::ManageBlacklist).label("Manage your blacklist"))
            .option(
                DemandOption::new(Self::ReorganizeDownloads)
                    .label("Reorganize already downloaded files"),
            )
            .option(DemandOption::new(Self::EditConfig).label("Edit your config file"))
            .option(DemandOption::new(Self::Exit).label("Exit e62rs"))
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, EnumDisplay)]
pub enum PoolSearchModeMenu {
    Simple,
    Advanced,
}

impl PoolSearchModeMenu {
    pub fn select(prompt: &str) -> Select<'_, Self> {
        Select::new(prompt)
            .theme(&ROSE_PINE)
            .filterable(false)
            .option(DemandOption::new(Self::Simple).label("Simple search"))
            .option(DemandOption::new(Self::Advanced).label("Advanced search"))
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, EnumDisplay)]
pub enum ExplorerMenu {
    BrowsePosts,
    SearchPosts,
    FilterByRating,
    SortBy,
    ViewStatistics,
    ClearFilters,
    Slideshow,
    Back,
}

impl ExplorerMenu {
    pub fn select(prompt: &str) -> Select<'_, Self> {
        Select::new(prompt)
            .theme(&ROSE_PINE)
            .filterable(true)
            .option(DemandOption::new(Self::BrowsePosts).label("Browse downloaded posts"))
            .option(
                DemandOption::new(Self::SearchPosts)
                    .label("Search posts by tags, ID, uploader, or description"),
            )
            .option(DemandOption::new(Self::FilterByRating).label("Filter by rating"))
            .option(DemandOption::new(Self::SortBy).label("Sort posts"))
            .option(DemandOption::new(Self::ViewStatistics).label("View statistics"))
            .option(DemandOption::new(Self::ClearFilters).label("Clear all filters"))
            .option(DemandOption::new(Self::Slideshow).label("Watch a slideshow"))
            .option(DemandOption::new(Self::Back).label("Go back to main menu"))
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, EnumDisplay)]
pub enum ExplorerSortBy {
    DateNewest,
    DateOldest,
    ScoreHighest,
    ScoreLowest,
    FavoritesHighest,
    IdAscending,
    IdDescending,
}

impl ExplorerSortBy {
    pub fn select(prompt: &str) -> Select<'_, Self> {
        Select::new(prompt)
            .theme(&ROSE_PINE)
            .filterable(false)
            .option(DemandOption::new(Self::DateNewest).label("Sort by date (newest first)"))
            .option(DemandOption::new(Self::DateOldest).label("Sort by date (oldest first)"))
            .option(DemandOption::new(Self::ScoreHighest).label("Sort by score (highest first)"))
            .option(DemandOption::new(Self::ScoreLowest).label("Sort by score (lowest first)"))
            .option(
                DemandOption::new(Self::FavoritesHighest)
                    .label("Sort by favorites (highest first)"),
            )
            .option(DemandOption::new(Self::IdAscending).label("Sort by ID (ascending)"))
            .option(DemandOption::new(Self::IdDescending).label("Sort by ID (descending)"))
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, EnumDisplay)]
pub enum ExplorerFilterBy {
    AllRatings,
    Safe,
    Questionable,
    Explicit,
}

impl ExplorerFilterBy {
    pub fn select(prompt: &str) -> Select<'_, Self> {
        Select::new(prompt)
            .theme(&ROSE_PINE)
            .filterable(false)
            .option(DemandOption::new(Self::AllRatings).label("All ratings"))
            .option(DemandOption::new(Self::Safe).label("Safe only"))
            .option(DemandOption::new(Self::Questionable).label("Questionable only"))
            .option(DemandOption::new(Self::Explicit).label("Explicit only"))
    }
}

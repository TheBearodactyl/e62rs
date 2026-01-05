//! ui menus
use crate::{getopt, ui::themes::ROSE_PINE};

pub mod blacklist;
pub mod download;
pub mod explore;
pub mod reorganize;
pub mod search;
pub mod view;

/// make a menu from an enum
macro_rules! menu {
    (
        $(#[$enum_meta:meta])*
        $vis:vis $enum_name:ident { filterable: $filterable:expr,
            $(
                $(#[$variant_meta:meta])*
                $variant:ident => {
                    label: $label:expr,
                    desc: $desc:expr
                }
            ),* $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis enum $enum_name {
            $(
                $(#[$variant_meta])*
                $variant,
            )*
        }

        impl ::std::fmt::Display for $enum_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    $(
                        Self::$variant => write!(f, "{}", $desc),
                    )*
                }
            }
        }

        impl $enum_name {
            /// display a menu and return the selected option
            #[allow(dead_code)]
            pub fn select(prompt: &str) -> ::demand::Select<'_, Self> {
                ::demand::Select::new(prompt)
                    .filterable($filterable)
                    .theme(&ROSE_PINE)
                    .options(vec![
                        $(
                            ::demand::DemandOption::new(Self::$variant)
                                .label($label)
                                .description($desc),
                        )*
                    ])
            }

            /// get the label of the given variant
            #[allow(dead_code)]
            pub const fn label(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant => $label,
                    )*
                }
            }
        }
    };
}

menu! {
    /// Post interaction menu
    pub InteractionMenu {  filterable: false,
        /// Open the post in a browser
        OpenInBrowser => {
            label: "Open in browser",
            desc: "Open the post in your browser at the e621 website"
        },
        /// Download the post
        Download => {
            label: "Download",
            desc: &format!("Download the post to your downloads folder ({})", getopt!(download.path))
        },
        /// View the post in your terminal
        View => {
            label: "View in terminal",
            desc: "Display the post image in your terminal via SIXEL"
        },
        /// Go back
        Back => {
            label: "Go back to search",
            desc: "Go back to the search menu"
        }
    }
}

menu! {
    /// Pool interaction menu
    pub PoolInteractionMenu { filterable: false,
        /// View posts from this pool
        ViewPosts => {
            label: "View posts from this pool",
            desc: "Browse and interact with all posts contained in this pool"
        },
        /// Download all posts from this pool
        Download => {
            label: "Download all posts from this pool",
            desc: "Download all posts from this pool to your downloads folder"
        },
        /// Download pool to dedicated pools folder
        DownloadToPoolsFolder => {
            label: "Download to pools dir",
            desc: "Download the entire pool to your dedicated pools folder with metadata"
        },
        /// Open the pool in e621
        OpenInBrowser => {
            label: "Open pool page in browser",
            desc: "Open the pool page in your browser at the e621 website"
        },
        /// Go back
        Back => {
            label: "Go back to pool search",
            desc: "Return to the pool search menu"
        }
    }
}

menu! {
    /// Batch post interaction
    pub BatchAction {
        filterable: false,

        /// Download all selected
        DownloadAll => {
            label: "Download all selected posts",
            desc: "Download all currently selected posts to your downloads folder"
        },
        /// Open all selected in browser
        Browser => {
            label: "Open all selected posts in browser",
            desc: "Open all selected posts in your default web browser"
        },
        /// Download and open all selected posts
        DlAndOpen => {
            label: "Download + open all selected posts",
            desc: "Download all selected posts and then open them in your browser"
        },
        /// View all posts in terminal
        ViewAll => {
            label: "View all post images in terminal",
            desc: "Display all selected post images in your terminal via SIXEL"
        },
        /// Go back
        Back => {
            label: "Go back to search",
            desc: "Return to the search results menu"
        }
    }
}

menu! {
    /// Advanced pool search
    pub AdvPoolSearch {
        filterable: false,

        /// Search pools by name
        ByName => {
            label: "Search by name",
            desc: "Search for pools by their name or title"
        },
        /// Search pools by description
        ByDesc => {
            label: "Search by description",
            desc: "Search for pools by keywords in their description"
        },
        /// Search pools by creator
        ByCreator => {
            label: "Search by creator",
            desc: "Search for pools by the username of their creator"
        },
        /// Browse the latest pools
        BrowseLatest => {
            label: "Browse latest pools",
            desc: "Browse the most recently created or updated pools"
        },
        /// Go back
        Back => {
            label: "Go back to main menu",
            desc: "Return to the main menu"
        }
    }
}

menu! {
    /// Blacklist manager
    pub BlacklistManager {
        filterable: false,

        /// Show current blacklist
        ShowCurrent => {
            label: "Show current blacklist",
            desc: "Display all tags currently in your blacklist"
        },
        /// Add a tag to the blacklist
        AddTag => {
            label: "Add tag to blacklist",
            desc: "Add a new tag to your blacklist to filter unwanted content"
        },
        /// Remove a tag from the blacklist
        RemoveTag => {
            label: "Remove tag from blacklist",
            desc: "Remove a tag from your blacklist"
        },
        /// Clear the blacklist
        Clear => {
            label: "Clear entire blacklist",
            desc: "Remove all tags from your blacklist"
        },
        /// Import tags into the blacklist from a search
        ImportFromSearch => {
            label: "Import tags from search",
            desc: "Import multiple tags from a search query into your blacklist"
        },
        /// Go back
        Back => {
            label: "Go back to main menu",
            desc: "Return to the main menu"
        }
    }
}

menu! {
    /// The main menu
    pub MainMenu {
        filterable: true,

        /// Search posts/pools
        Search => {
            label: "Search",
            desc: "Search for posts or pools by tags and filters"
        },
        /// View the latest posts
        ViewLatest => {
            label: "View the latest posts",
            desc: "Browse the most recently uploaded posts on e621"
        },
        /// Explore downloads
        ExploreDownloads => {
            label: "View downloaded files",
            desc: "Browse, search, and manage your downloaded posts"
        },
        /// Open your downloads in your browser
        OpenInBrowser => {
            label: "View downloads in your browser",
            desc: "Open your local downloads folder in your default web browser"
        },
        /// Manage your blacklist
        ManageBlacklist => {
            label: "Manage your blacklist",
            desc: "Add, remove, or view tags in your blacklist"
        },
        /// Reorganize downloads
        Reorganize => {
            label: "Reorganize downloaded files",
            desc: "Reorganize and sort your downloaded files by various criteria"
        },
        /// Edit your configuration file
        EditConfig => {
            label: "Edit your config file",
            desc: "Open your configuration file in your default text editor"
        },
        /// Update currently downloaded files based on already downloaded artists
        UpdateDownloads => {
            label: "Update downloaded artists",
            desc: "Check for and download new posts from artists you've already downloaded"
        },
        /// Reload the configuration file from disk
        ReloadConfig => {
            label: "Reload config",
            desc: "Reload the config file and apply any changes made since last load"
        },
        /// Exit
        Exit => {
            label: "Exit e62rs",
            desc: "Exit the app"
        }
    }
}

menu! {
    /// Search type
    pub SearchMenu {
        filterable: false,

        /// Search posts
        Posts => {
            label: "Posts",
            desc: "Search for individual posts by tags and filters"
        },
        /// Search pools
        Pools => {
            label: "Pools",
            desc: "Search for collections of related posts organized into pools"
        },
        /// Go back to the main menu
        Back => {
            label: "Go back",
            desc: "Go back to the main menu"
        }
    }
}

menu! {
    /// Pool search mode
    pub PoolSearchModeMenu {
        filterable: false,

        /// Simple search
        Simple => {
            label: "Simple search",
            desc: "Quick and easy pool search with basic filters"
        },
        /// Advanced search
        Advanced => {
            label: "Advanced search",
            desc: "Advanced pool search with detailed filtering options"
        }
    }
}

menu! {
    /// Downloads explorer menu
    pub ExplorerMenu {
        filterable: true,

        /// Browse posts
        Browse => {
            label: "Browse downloaded posts",
            desc: "View and interact with your downloaded posts"
        },
        /// Search for posts
        Search => {
            label: "Search by tags, ID, uploader, or desc",
            desc: "Search your downloaded posts using various criteria"
        },
        /// Filter posts by rating
        FilterByRating => {
            label: "Filter by rating",
            desc: "Filter posts by their content rating (safe, questionable, explicit)"
        },
        /// Sort posts
        SortBy => {
            label: "Sort posts",
            desc: "Sort your downloaded posts by date, score, favorites, or ID"
        },
        /// View statistics
        ViewStatistics => {
            label: "View statistics",
            desc: "View detailed statistics about your downloaded collection"
        },
        /// Clear all filters
        ClearFilters => {
            label: "Clear all filters",
            desc: "Remove all active filters and sorting options"
        },
        /// Watch selected posts in a slideshow
        Slideshow => {
            label: "Watch a slideshow",
            desc: "View your posts in an automated slideshow"
        },
        /// Go back
        Back => {
            label: "Go back to main menu",
            desc: "Return to the main menu"
        }
    }
}

menu! {
    /// Explorer sorting options
    pub ExplorerSortBy {
        filterable: false,

        /// Sort by date (newest)
        DateNewest => {
            label: "Sort by date (newest first)",
            desc: "Sort posts with the most recent uploads first"
        },
        /// Sort by date (oldest)
        DateOldest => {
            label: "Sort by date (oldest first)",
            desc: "Sort posts with the oldest uploads first"
        },
        /// Sort by score (highest)
        ScoreHighest => {
            label: "Sort by score (highest first)",
            desc: "Sort posts by their score with highest rated first"
        },
        /// Sort by score (lowest)
        ScoreLowest => {
            label: "Sort by score (lowest first)",
            desc: "Sort posts by their score with lowest rated first"
        },
        /// Sort by favorites (highest)
        FavoritesHighest => {
            label: "Sort by favs (highest first)",
            desc: "Sort posts by favorite count with most favorited first"
        },
        /// Sort by favorites (lowest)
        FavoritesLowest => {
            label: "Sort by favs (lowest first)",
            desc: "Sort posts by favorite count with least favorited first"
        },
        /// Sort by id (ascending)
        IDAscending => {
            label: "Sort by ID (ascending)",
            desc: "Sort posts by their ID in ascending order (oldest to newest)"
        },
        /// Sort by id (descending)
        IDDescending => {
            label: "Sort by ID (descending)",
            desc: "Sort posts by their ID in descending order (newest to oldest)"
        }
    }
}

menu! {
    /// the rating to filter for in the explorer
    pub ExplorerFilterBy { filterable: true,
        /// all ratings
        AllRatings => {
            label: "All ratings",
            desc: "Display posts of all ratings"
        },

        /// safe posts
        Safe => {
            label: "Safe posts",
            desc: "Display posts with the 'safe' rating"
        },

        /// questionable posts
        Questionable => {
            label: "Questionable posts",
            desc: "Display posts with the `questionable` rating"
        },

        /// explicit posts
        Explicit => {
            label: "Explicit posts",
            desc: "Display posts with the 'explicit' rating"
        },
    }
}

//! ui menus
pub mod blacklist;
pub mod download;
pub mod explore;
pub mod reorganize;
pub mod search;
pub mod view;

#[derive(Default)]
/// stats for translation progress on a language
pub struct TranslationStats {
    /// the number of labels translated for a language
    pub labels_translated: usize,
    /// the number of descriptions translated for a language
    pub descriptions_translated: usize,
    /// the total number of variants
    pub total_variants: usize,
}

crate::menu! {
    /// Configuration menu
    pub ConfigMenu {
        filterable: true,

        /// Edit the current configuration file
        Edit => {
            label: {
                english => "Edit",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Edit your config file",
                japanese => "",
                spanish => ""
            },
            online: false
        },

        /// Reload the config file
        Reload => {
            label: {
                english => "Reload",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Reload your config file",
                japanese => "",
                spanish => ""
            },
            online: false
        },

        /// Go back
        Back => {
            label: {
                english => "Go back to main menu",
                japanese => "",
                spanish => "Volver al menú principal"
            },
            desc: {
                english => "Return to the main menu",
                japanese => "",
                spanish => "Volver al menú principal"
            },
            online: false
        }
    }
}

crate::menu! {
    /// Post interaction menu
    pub InteractionMenu {  filterable: true,
        /// Open the post in a browser
        OpenInBrowser => {
            label: {
                english => "Open in browser",
                japanese => "",
                spanish => "Abrir en el navegador"
            },
            desc: {
                english => "Open the post in your browser at the e621 website",
                japanese => "e621のウェでブサイトで",
                spanish => "Abre la publicación en tu navegador en e621 welfare q"
            },
            online: true
        },
        /// Download the post
        Download => {
            label: {
                english => "Download",
                japanese => "",
                spanish => "Descargar"
            },
            desc: {
                english => "Download the post to your downloads folder",
                japanese => "",
                spanish => "Descarga la publicación en tu carpeta de descargas"
            },
            online: true
        },
        /// Make a QR code of the post
        MakeQr => {
            label: {
                english => "Make QR code",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Make a QR code that you can scan to go to the post at e621",
                japanese => "",
                spanish => ""
            },
            online: true
        },
        /// View the post in your terminal
        View => {
            label: {
                english => "View in terminal",
                japanese => "",
                spanish => "Ver en terminal"
            },
            desc: {
                english => "Display the post image in your terminal via SIXEL",
                japanese => "",
                spanish => "Muestra la imagen de la publicación en tu terminal mediante SIXEL"
            },
            online: true
        },
        /// Go back
        Back => {
            label: {
                english => "Go back to search",
                japanese => "",
                spanish => "Volver a la búsqueda"
            },
            desc: {
                english => "Go back to the search menu",
                japanese => "",
                spanish => "Volver al menú de búsqueda"
            },
            online: false
        }
    }
}

crate::menu! {
    /// Pool interaction menu
    pub PoolInteractionMenu { filterable: true,
        /// View posts from this pool
        ViewPosts => {
            label: {
                english => "View posts from this pool",
                japanese => "",
                spanish => "Ver publicaciones de este grupo"
            },
            desc: {
                english => "Browse and interact with all posts contained in this pool",
                japanese => "",
                spanish => "Explora e interactúa con todas las publicaciones contenidas en este grupo"
            },
            online: true
        },
        /// Download all posts from this pool
        Download => {
            label: {
                english => "Download all posts from this pool",
                japanese => "",
                spanish => "Descargar todas las publicaciones de este grupo"
            },
            desc: {
                english => "Download all posts from this pool to your downloads folder",
                japanese => "",
                spanish => "Descarga todas las publicaciones de este grupo a tu carpeta de descargas"
            },
            online: true
        },
        /// Download pool to dedicated pools folder
        DownloadToPoolsFolder => {
            label: {
                english => "Download to pools dir",
                japanese => "",
                spanish => "Descargar al directorio de grupos"
            },
            desc: {
                english => "Download the entire pool to your dedicated pools folder with metadata",
                japanese => "",
                spanish => "Descargar todo el grupo a tu directorio de grupos dedicado con metadatos"
            },
            online: true
        },
        /// Create BBF file from pool
        CreateBBF => {
            label: {
                english => "Create BBF file from pool",
                japanese => "",
                spanish => "Crear archivo BBF del grupo"
            },
            desc: {
                english => "Download and package pool into Bound Book Format (.bbf) file",
                japanese => "",
                spanish => "Descargar y empaquetar grupo en formato Bound Book (.bbf)"
            },
            online: true
        },
        /// Open the pool in e621
        OpenInBrowser => {
            label: {
                english => "Open pool page in browser",
                japanese => "",
                spanish => "Abrir la página del grupo en el navegador"
            },
            desc: {
                english => "Open the pool page in your browser at the e621 website",
                japanese => "",
                spanish => "Abre la página del grupo en tu navegador en el sitio web e621"
            },
            online: true
        },
        /// Go back
        Back => {
            label: {
                english => "Go back to pool search",
                japanese => "",
                spanish => "Volver a la búsqueda por grupos"
            },
            desc: {
                english => "Return to the pool search menu",
                japanese => "",
                spanish => "Volver al menú de búsqueda de grupos"
            },
            online: false
        }
    }
}

crate::menu! {
    /// Batch post interaction
    pub BatchAction {
        filterable: true,

        /// Download all selected
        DownloadAll => {
            label: {
                english => "Download all selected posts",
                japanese => "",
                spanish => "Descargar todos las publicaciones seleccionadas"
            },
            desc: {
                english => "Download all currently selected posts to your downloads folder",
                japanese => "",
                spanish => "Descarga todas las publicaciones seleccionadas actualmente a tu carpeta de descargas"
            },
            online: true
        },
        /// Open all selected in browser
        Browser => {
            label: {
                english => "Open all selected posts in browser",
                japanese => "",
                spanish => "Abrir todas las publicaciones seleccionadas en el navegador"
            },
            desc: {
                english => "Open all selected posts in your default web browser",
                japanese => "",
                spanish => "Abre todas las publicaciones seleccionadas en tu navegador web predeterminado"
            },
            online: true
        },
        /// Download and open all selected posts
        DlAndOpen => {
            label: {
                english => "Download + open all selected posts",
                japanese => "",
                spanish => "Descargar y abrir todas las publicaciones seleccionadas"
            },
            desc: {
                english => "Download all selected posts and then open them in your browser",
                japanese => "",
                spanish => "Descarga todas las publicaciones seleccionadas y luego ábrelas en tu navegador"
            },
            online: true
        },
        /// View all posts in terminal
        ViewAll => {
            label: {
                english => "View all post images in terminal",
                japanese => "",
                spanish => "Ver todas las imágenes en terminal"
            },
            desc: {
                english => "Display all selected post images in your terminal via SIXEL",
                japanese => "",
                spanish => "Muestra todos las imágenes de publicaciones seleccionadas en tu terminal mediante SIXEL"
            },
            online: true
        },
        /// Go back
        Back => {
            label: {
                english => "Go back to search",
                japanese => "",
                spanish => "Volver a la búsqueda"
            },
            desc: {
                english => "Return to the search results menu",
                japanese => "",
                spanish => "Volver al menú de resultados de búsqueda"
            },
            online: false
        }
    }
}

crate::menu! {
    /// Advanced pool search
    pub AdvPoolSearch {
        filterable: true,

        /// Search pools by name
        ByName => {
            label: {
                english => "Search by name",
                japanese => "",
                spanish => "Buscar por nombre"
            },
            desc: {
                english => "Search for pools by their name or title",
                japanese => "",
                spanish => "Busca grupos por su nombre o título"
            },
            online: true
        },
        /// Search pools by description
        ByDesc => {
            label: {
                english => "Search by description",
                japanese => "",
                spanish => "Buscar por descripción"
            },
            desc: {
                english => "Search for pools by keywords in their description",
                japanese => "",
                spanish => "Busca grupos por palabras clave en su descripción"
            },
            online: true
        },
        /// Search pools by creator
        ByCreator => {
            label: {
                english => "Search by creator",
                japanese => "",
                spanish => "Busca por creador"
            },
            desc: {
                english => "Search for pools by the username of their creator",
                japanese => "",
                spanish => "Busca grupos por el nombre de usuario de su creador"
            },
            online: true
        },
        /// Browse the latest pools
        BrowseLatest => {
            label: {
                english => "Browse latest pools",
                japanese => "",
                spanish => "Explorar grupos recientes"
            },
            desc: {
                english => "Browse the most recently created or updated pools",
                japanese => "",
                spanish => "Explora los grupos creados o actualizados más recientemente"
            },
            online: true
        },
        /// Go back
        Back => {
            label: {
                english => "Go back to main menu",
                japanese => "",
                spanish => "Volver al menú principal"
            },
            desc: {
                english => "Return to the main menu",
                japanese => "",
                spanish => "Volver al menú principal"
            },
            online: false
        }
    }
}

crate::menu! {
    /// Blacklist manager
    pub BlacklistManager {
        filterable: true,

        /// Show current blacklist
        ShowCurrent => {
            label: {
                english => "Show current blacklist",
                japanese => "",
                spanish => "Mostrar lista negra actual"
            },
            desc: {
                english => "Display all tags currently in your blacklist",
                japanese => "",
                spanish => "Muestra todas las etiquetas que se encuentran actualmente en tu lista negra"
            },
            online: false
        },
        /// Add a tag to the blacklist
        AddTag => {
            label: {
                english => "Add tag to blacklist",
                japanese => "",
                spanish => "Agregar etiqueta a la lista negra"
            },
            desc: {
                english => "Add a new tag to your blacklist to filter unwanted content",
                japanese => "",
                spanish => "Agrega una nueva e"
            },
            online: false
        },
        /// Remove a tag from the blacklist
        RemoveTag => {
            label: {
                english => "Remove tag from blacklist",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Remove a tag from your blacklist",
                japanese => "",
                spanish => ""
            },
            online: false
        },
        /// Clear the blacklist
        Clear => {
            label: {
                english => "Clear entire blacklist",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Remove all tags from your blacklist",
                japanese => "",
                spanish => ""
            },
            online: false
        },
        /// Import tags into the blacklist from a search
        ImportFromSearch => {
            label: {
                english => "Import tags from search",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Import multiple tags from a search query into your blacklist",
                japanese => "",
                spanish => ""
            },
            online: true
        },
        /// Go back
        Back => {
            label: {
                english => "Go back to main menu",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Return to the main menu",
                japanese => "",
                spanish => ""
            },
            online: false
        }
    }
}

crate::menu! {
    /// The main menu
    pub MainMenu {
        filterable: true,

        /// Search posts/pools
        Search => {
            label: {
                english => "Search",
                japanese => "",
                spanish => "Buscar"
            },
            desc: {
                english => "Search for posts or pools by tags and filters",
                japanese => "",
                spanish => "Busca publicaciones o encuestas mediante etiquetas y filtros"
            },
            online: true
        },
        /// View the latest posts
        ViewLatest => {
            label: {
                english => "View the latest posts",
                japanese => "",
                spanish => "Ver las últimas publicaciones"
            },
            desc: {
                english => "Browse the most recently uploaded posts on e621",
                japanese => "",
                spanish => "Explora las publicaciones subidas más recientemente en e621"
            },
            online: true
        },
        /// Explore downloads
        ExploreDownloads => {
            label: {
                english => "View downloaded posts",
                japanese => "",
                spanish => "Ver publicaciones descargadas"
            },
            desc: {
                english => "Browse, search, and manage your downloaded posts",
                japanese => "",
                spanish => "Explora, busca y gestiona tus publicaciones descargadas"
            },
            online: false
        },
        /// Open your downloads in your browser
        OpenInBrowser => {
            label: {
                english => "View downloads in your browser",
                japanese => "",
                spanish => "Ver las descargas en tu navegador"
            },
            desc: {
                english => "Open your local downloads folder in your default web browser",
                japanese => "",
                spanish => "Abre tus descargas como una galería en tu navegador predeterminado"
            },
            online: false
        },
        /// Manage your blacklist
        ManageBlacklist => {
            label: {
                english => "Manage your blacklist",
                japanese => "",
                spanish => "Gestiona tu lista negra"
            },
            desc: {
                english => "Add, remove, or view tags in your blacklist",
                japanese => "",
                spanish => "Agregue, elimine o vea las etiquetas en su lista negra"
            },
            online: false
        },
        /// Reorganize downloads
        Reorganize => {
            label: {
                english => "Reorganize downloads",
                japanese => "",
                spanish => "Reorganizar descargas"
            },
            desc: {
                english => "Reorganize and sort your downloaded files by various criteria",
                japanese => "",
                spanish => "Reorganiza y clasifica tus archivos descargados según diversos criterios"
            },
            online: false
        },
        /// Update currently downloaded files based on already downloaded artists
        UpdateDownloads => {
            label: {
                english => "Update downloaded artists",
                japanese => "",
                spanish => "Actualizar artistas descargadas"
            },
            desc: {
                english => "Check for and download new posts from artists you've already downloaded",
                japanese => "",
                spanish => "Busca y descarga nuevas publicaciones de los artistas cuyas obras ya has descargado"
            },
            online: true
        },
        /// Manage the configuration file
        ManageConfig => {
            label: {
                english => "Manage config",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Manage your current configuration",
                japanese => "",
                spanish => ""
            },
            online: false
        },
        /// Exit
        Exit => {
            label: {
                english => "Exit e62rs",
                japanese => "",
                spanish => "Salir de e62rs"
            },
            desc: {
                english => "Exit the app",
                japanese => "",
                spanish => "Salir de la aplicación"
            },
            online: false
        }
    }
}

crate::menu! {
    /// Search type
    pub SearchMenu {
        filterable: true,

        /// Search posts
        Posts => {
            label: {
                english => "Posts",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Search for individual posts by tags and filters",
                japanese => "",
                spanish => ""
            },
            online: true
        },
        /// Search pools
        Pools => {
            label: {
                english => "Pools",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Search for collections of related posts organized into pools",
                japanese => "",
                spanish => ""
            },
            online: true
        },
        /// Go back to the main menu
        Back => {
            label: {
                english => "Go back",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Go back to the main menu",
                japanese => "",
                spanish => ""
            },
            online: false
        }
    }
}

crate::menu! {
    /// Pool search mode
    pub PoolSearchModeMenu {
        filterable: true,

        /// Simple search
        Simple => {
            label: {
                english => "Simple search",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Quick and easy pool search with basic filters",
                japanese => "",
                spanish => ""
            },
            online: false
        },
        /// Advanced search
        Advanced => {
            label: {
                english => "Advanced search",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Advanced pool search with detailed filtering options",
                japanese => "",
                spanish => ""
            },
            online: false
        }
    }
}

crate::menu! {
    /// Downloads explorer menu
    pub ExplorerMenu {
        filterable: true,

        /// Browse posts
        Browse => {
            label: {
                english => "Browse downloaded posts",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "View and interact with your downloaded posts",
                japanese => "",
                spanish => ""
            },
            online: false
        },
        /// Search for posts
        Search => {
            label: {
                english => "Search by tags, ID, uploader, or desc",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Search your downloaded posts using various criteria",
                japanese => "",
                spanish => ""
            },
            online: false
        },
        /// Filter posts by rating
        FilterByRating => {
            label: {
                english => "Filter by rating",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Filter posts by their content rating (safe, questionable, explicit)",
                japanese => "",
                spanish => ""
            },
            online: false
        },
        /// Sort posts
        SortBy => {
            label: {
                english => "Sort posts",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Sort your downloaded posts by date, score, favorites, or ID",
                japanese => "",
                spanish => ""
            },
            online: false
        },
        /// View statistics
        ViewStatistics => {
            label: {
                english => "View statistics",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "View detailed statistics about your downloaded collection",
                japanese => "",
                spanish => ""
            },
            online: false
        },
        /// Clear all filters
        ClearFilters => {
            label: {
                english => "Clear all filters",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Remove all active filters and sorting options",
                japanese => "",
                spanish => ""
            },
            online: false
        },
        /// Watch selected posts in a slideshow
        Slideshow => {
            label: {
                english => "Watch a slideshow",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "View your posts in an automated slideshow",
                japanese => "",
                spanish => ""
            },
            online: false
        },
        /// Go back
        Back => {
            label: {
                english => "Go back to main menu",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Return to the main menu",
                japanese => "",
                spanish => ""
            },
            online: false
        }
    }
}

crate::menu! {
    /// Explorer sorting options
    pub ExplorerSortBy {
        filterable: true,

        /// Sort by date (newest)
        DateNewest => {
            label: {
                english => "Sort by date (newest first)",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Sort posts with the most recent uploads first",
                japanese => "",
                spanish => ""
            },
            online: false
        },
        /// Sort by date (oldest)
        DateOldest => {
            label: {
                english => "Sort by date (oldest first)",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Sort posts with the oldest uploads first",
                japanese => "",
                spanish => ""
            },
            online: false
        },
        /// Sort by score (highest)
        ScoreHighest => {
            label: {
                english => "Sort by score (highest first)",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Sort posts by their score with highest rated first",
                japanese => "",
                spanish => ""
            },
            online: false
        },
        /// Sort by score (lowest)
        ScoreLowest => {
            label: {
                english => "Sort by score (lowest first)",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Sort posts by their score with lowest rated first",
                japanese => "",
                spanish => ""
            },
            online: false
        },
        /// Sort by favorites (highest)
        FavoritesHighest => {
            label: {
                english => "Sort by favs (highest first)",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Sort posts by favorite count with most favorited first",
                japanese => "",
                spanish => ""
            },
            online: false
        },
        /// Sort by favorites (lowest)
        FavoritesLowest => {
            label: {
                english => "Sort by favs (lowest first)",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Sort posts by favorite count with least favorited first",
                japanese => "",
                spanish => ""
            },
            online: false
        },
        /// Sort by id (ascending)
        IDAscending => {
            label: {
                english => "Sort by ID (ascending)",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Sort posts by their ID in ascending order (oldest to newest)",
                japanese => "",
                spanish => ""
            },
            online: false
        },
        /// Sort by id (descending)
        IDDescending => {
            label: {
                english => "Sort by ID (descending)",
                japanese => "",
                spanish => "Ordenar por ID (descendente)"
            },
            desc: {
                english => "Sort posts by their ID in descending order (newest to oldest)",
                japanese => "",
                spanish => ""
            },
            online: false
        }
    }
}

crate::menu! {
    /// the rating to filter for in the explorer
    pub ExplorerFilterBy { filterable: true,
        /// all ratings
        AllRatings => {
            label: {
                english => "All ratings",
                japanese => "",
                spanish => "Todas las calificaciones"
            },
            desc: {
                english => "Display posts of all ratings",
                japanese => "",
                spanish => ""
            },
            online: false
        },

        /// safe posts
        Safe => {
            label: {
                english => "Safe posts",
                japanese => "",
                spanish => "Publicaciones seguras"
            },
            desc: {
                english => "Display posts with the 'safe' rating",
                japanese => "",
                spanish => ""
            },
            online: false
        },

        /// questionable posts
        Questionable => {
            label: {
                english => "Questionable posts",
                japanese => "Publicaciones cuestionables",
                spanish => ""
            },
            desc: {
                english => "Display posts with the `questionable` rating",
                japanese => "",
                spanish => ""
            },
            online: false
        },

        /// explicit posts
        Explicit => {
            label: {
                english => "Explicit posts",
                japanese => "",
                spanish => ""
            },
            desc: {
                english => "Display posts with the 'explicit' rating",
                japanese => "",
                spanish => "Publicaciones explicítas"
            },
            online: false
        },
    }
}

/// calculates and prints the total
/// localization progress across all menus
pub fn calculate_localization_progress() {
    let mut translated_labels_en = 0;
    let mut translated_labels_es = 0;
    let mut translated_labels_ja = 0;
    let mut translated_descs_es = 0;
    let mut translated_descs_en = 0;
    let mut translated_descs_ja = 0;

    let menu_stats = vec![
        InteractionMenu::translation_stats(),
        PoolInteractionMenu::translation_stats(),
        BatchAction::translation_stats(),
        AdvPoolSearch::translation_stats(),
        BlacklistManager::translation_stats(),
        MainMenu::translation_stats(),
        SearchMenu::translation_stats(),
        PoolSearchModeMenu::translation_stats(),
        ExplorerMenu::translation_stats(),
        ExplorerSortBy::translation_stats(),
        ExplorerFilterBy::translation_stats(),
    ];

    for stat in menu_stats {
        if let Some(en) = stat.get("english") {
            translated_descs_en += en.descriptions_translated;
            translated_labels_en += en.labels_translated;
        }

        if let Some(es) = stat.get("spanish") {
            translated_labels_es += es.labels_translated;
            translated_descs_es += es.descriptions_translated;
        }

        if let Some(ja) = stat.get("japanese") {
            translated_labels_ja += ja.labels_translated;
            translated_descs_ja += ja.descriptions_translated;
        }
    }

    let print_stat =
        |lang: &str, labels: usize, descs: usize, total_labels: usize, total_descs: usize| {
            let label_pct = (labels as f64 / total_labels as f64) * 100.0;
            let desc_pct = (descs as f64 / total_descs as f64) * 100.0;

            println!(
                "- [{}] **{}**",
                if label_pct == 100.0 && desc_pct == 100.0 {
                    "x"
                } else if label_pct == 100.0 || desc_pct == 100.0 {
                    "-"
                } else {
                    " "
                },
                lang.to_uppercase()
            );

            println!(
                "- [{}] Labels: {}/{} ({:.2}%)",
                if label_pct == 100.0 { "x" } else { " " },
                labels,
                total_labels,
                label_pct
            );
            println!(
                "- [{}] Descriptions: {}/{} ({:.2}%)",
                if desc_pct == 100.0 { "x" } else { " " },
                descs,
                total_descs,
                desc_pct
            );
            println!();
        };

    print_stat(
        "English",
        translated_labels_en,
        translated_descs_en,
        translated_labels_en,
        translated_descs_en,
    );

    print_stat(
        "Spanish",
        translated_labels_es,
        translated_descs_es,
        translated_labels_en,
        translated_descs_en,
    );
    print_stat(
        "Japanese",
        translated_labels_ja,
        translated_descs_ja,
        translated_labels_en,
        translated_descs_en,
    );
}

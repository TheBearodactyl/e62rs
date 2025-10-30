use {
    crate::{
        config::options::{E62Rs, ExplorerCfg},
        display::image::display_image_from_path_as_sixel,
        models::E6Post,
        ui::{
            E6Ui,
            menus::{ExplorerMenu, ExplorerSortBy, view::print_post_to_terminal},
            progress::ProgressManager,
        },
    },
    color_eyre::eyre::{Context, Result, bail},
    inquire::{Confirm, Select, Text},
    jwalk::WalkDir,
    std::{
        collections::HashMap,
        io::Read,
        path::{Path, PathBuf},
        sync::{Arc, Mutex},
        thread::sleep,
        time::Duration,
    },
    tracing::*,
};

lazy_static::lazy_static! {
    static ref METADATA_CACHE: Arc<Mutex<HashMap<PathBuf, E6Post>>> = Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Debug, Clone)]
pub struct LocalPost {
    pub post: E6Post,
    pub file_path: PathBuf,
}

impl LocalPost {
    pub fn from_file(file_path: PathBuf) -> Result<Self> {
        let post = Self::read_metadata(&file_path)?;
        Ok(Self { post, file_path })
    }

    pub fn view(&self) -> Result<()> {
        display_image_from_path_as_sixel(self.file_path.as_path())?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn read_metadata(file_path: &Path) -> Result<E6Post> {
        use std::fs::OpenOptions;

        let ads_path = format!("{}:metadata", file_path.display());
        let mut file = OpenOptions::new()
            .read(true)
            .open(&ads_path)
            .with_context(|| format!("Failed to open ADS metadata for {}", file_path.display()))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .with_context(|| format!("Failed to read ADS metadata for {}", file_path.display()))?;

        serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse metadata for {}", file_path.display()))
    }

    #[cfg(not(target_os = "windows"))]
    fn read_metadata(file_path: &Path) -> Result<E6Post> {
        let json_path = file_path.with_extension(format!(
            "{}.json",
            file_path.extension().and_then(|e| e.to_str()).unwrap_or("")
        ));

        if !json_path.exists() {
            anyhow::bail!("Metadata file not found: {}", json_path.display());
        }

        let contents = fs::read_to_string(&json_path)
            .with_context(|| format!("Failed to read metadata file {}", json_path.display()))?;

        serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse metadata for {}", file_path.display()))
    }
}

pub struct ExplorerState {
    pub posts: Vec<LocalPost>,
    pub filtered_posts: Vec<LocalPost>,
    pub current_sort: ExplorerSortBy,
    pub search_query: Option<String>,
    pub rating_filter: Option<String>,
}

impl ExplorerState {
    pub fn new(posts: Vec<LocalPost>) -> Self {
        let filtered_posts = posts.clone();
        Self {
            posts,
            filtered_posts,
            current_sort: ExplorerSortBy::DateNewest,
            search_query: None,
            rating_filter: None,
        }
    }

    pub fn sort(&mut self, sort_by: ExplorerSortBy) {
        self.current_sort = sort_by;
        match sort_by {
            ExplorerSortBy::DateNewest => {
                self.filtered_posts
                    .sort_by(|a, b| b.post.created_at.cmp(&a.post.created_at));
            }
            ExplorerSortBy::DateOldest => {
                self.filtered_posts
                    .sort_by(|a, b| a.post.created_at.cmp(&b.post.created_at));
            }
            ExplorerSortBy::ScoreHighest => {
                self.filtered_posts
                    .sort_by(|a, b| b.post.score.total.cmp(&a.post.score.total));
            }
            ExplorerSortBy::ScoreLowest => {
                self.filtered_posts
                    .sort_by(|a, b| a.post.score.total.cmp(&b.post.score.total));
            }
            ExplorerSortBy::FavoritesHighest => {
                self.filtered_posts
                    .sort_by(|a, b| b.post.fav_count.cmp(&a.post.fav_count));
            }
            ExplorerSortBy::IdAscending => {
                self.filtered_posts
                    .sort_by(|a, b| a.post.id.cmp(&b.post.id));
            }
            ExplorerSortBy::IdDescending => {
                self.filtered_posts
                    .sort_by(|a, b| b.post.id.cmp(&a.post.id));
            }
        }
    }

    pub fn filter_by_rating(&mut self, rating: Option<String>) {
        self.rating_filter = rating.clone();
        self.apply_filters();
    }

    pub fn search(&mut self, query: Option<String>) {
        self.search_query = query;
        self.apply_filters();
    }

    fn apply_filters(&mut self) {
        self.filtered_posts = self
            .posts
            .iter()
            .filter(|local_post| {
                if let Some(ref rating) = self.rating_filter
                    && &local_post.post.rating != rating
                {
                    return false;
                }

                if let Some(ref query) = self.search_query {
                    let query_lower = query.to_lowercase();
                    let matches_id = local_post.post.id.to_string().contains(&query_lower);
                    let matches_tags = local_post
                        .post
                        .tags
                        .general
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
                        || local_post
                            .post
                            .tags
                            .artist
                            .iter()
                            .any(|tag| tag.to_lowercase().contains(&query_lower))
                        || local_post
                            .post
                            .tags
                            .character
                            .iter()
                            .any(|tag| tag.to_lowercase().contains(&query_lower))
                        || local_post
                            .post
                            .tags
                            .species
                            .iter()
                            .any(|tag| tag.to_lowercase().contains(&query_lower));
                    let matches_description = local_post
                        .post
                        .description
                        .to_lowercase()
                        .contains(&query_lower);
                    let matches_uploader = local_post
                        .post
                        .uploader_name
                        .to_lowercase()
                        .contains(&query_lower);

                    if !matches_id && !matches_tags && !matches_description && !matches_uploader {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        self.sort(self.current_sort);
    }

    pub fn get_statistics(&self) -> ExplorerStatistics {
        let total_posts = self.posts.len();
        let filtered_posts = self.filtered_posts.len();

        let rating_counts = self.posts.iter().fold(
            (0, 0, 0, 0),
            |(safe, questionable, explicit, unknown), local_post| match local_post
                .post
                .rating
                .as_str()
            {
                "s" => (safe + 1, questionable, explicit, unknown),
                "q" => (safe, questionable + 1, explicit, unknown),
                "e" => (safe, questionable, explicit + 1, unknown),
                _ => (safe, questionable, explicit, unknown + 1),
            },
        );

        let avg_score = if total_posts > 0 {
            self.posts.iter().map(|lp| lp.post.score.total).sum::<i64>() as f64 / total_posts as f64
        } else {
            0.0
        };

        let total_favorites = self.posts.iter().map(|lp| lp.post.fav_count).sum();

        ExplorerStatistics {
            total_posts,
            filtered_posts,
            safe: rating_counts.0,
            questionable: rating_counts.1,
            explicit: rating_counts.2,
            unknown: rating_counts.3,
            avg_score,
            total_favorites,
        }
    }
}

#[derive(Debug)]
pub struct ExplorerStatistics {
    pub total_posts: usize,
    pub filtered_posts: usize,
    pub safe: usize,
    pub questionable: usize,
    pub explicit: usize,
    pub unknown: usize,
    pub avg_score: f64,
    pub total_favorites: i64,
}

impl E6Ui {
    pub async fn explore_downloads(&self) -> Result<()> {
        println!("\n=== Downloads Explorer ===\n");

        let cfg = E62Rs::get()?;
        let dl_cfg = cfg.download;
        let explorer_cfg = cfg.explorer;

        let download_dir = dl_cfg.download_dir;

        let directory = Path::new(&download_dir);
        if !directory.exists() {
            bail!("Download directory does not exist: {}", directory.display());
        }

        let local_posts = self
            .scan_downloads_directory(directory, &explorer_cfg)
            .await?;

        if local_posts.is_empty() {
            println!("No posts with metadata found in {}", directory.display());
            return Ok(());
        }

        println!("Found {} posts with metadata\n", local_posts.len());

        let mut state = ExplorerState::new(local_posts);

        let default_sort = match explorer_cfg.default_sort.as_str() {
            "date_newest" => ExplorerSortBy::DateNewest,
            "date_oldest" => ExplorerSortBy::DateOldest,
            "score_highest" => ExplorerSortBy::ScoreHighest,
            "score_lowest" => ExplorerSortBy::ScoreLowest,
            "favorites_highest" => ExplorerSortBy::FavoritesHighest,
            "id_ascending" => ExplorerSortBy::IdAscending,
            "id_descending" => ExplorerSortBy::IdDescending,
            _ => ExplorerSortBy::DateNewest,
        };
        state.sort(default_sort);

        loop {
            let action = ExplorerMenu::select(&format!(
                "Downloads Explorer ({} posts shown)",
                state.filtered_posts.len()
            ))
            .prompt()?;

            match action {
                ExplorerMenu::BrowsePosts => {
                    if state.filtered_posts.is_empty() {
                        println!("No posts match the current filters.");
                        continue;
                    }
                    self.browse_local_posts(&state.filtered_posts, &explorer_cfg)
                        .await?;
                }
                ExplorerMenu::SearchPosts => {
                    let query =
                        Text::new("Enter search query (tags, ID, uploader, or description):")
                            .prompt_skippable()?;
                    state.search(query);
                    println!("Found {} matching posts", state.filtered_posts.len());
                }
                ExplorerMenu::FilterByRating => {
                    self.filter_by_rating(&mut state)?;
                }
                ExplorerMenu::SortBy => {
                    self.sort_posts(&mut state)?;
                }
                ExplorerMenu::ViewStatistics => {
                    self.display_statistics(&state);
                }
                ExplorerMenu::ClearFilters => {
                    state.search(None);
                    state.filter_by_rating(None);
                    println!(
                        "Filters cleared. Showing all {} posts",
                        state.filtered_posts.len()
                    );
                }
                ExplorerMenu::Slideshow => self.slideshow(&state.filtered_posts).await?,
                ExplorerMenu::Back => break,
            }

            if !Confirm::new("Continue exploring?")
                .with_default(true)
                .prompt()?
            {
                break;
            }
        }

        Ok(())
    }

    async fn scan_downloads_directory(
        &self,
        directory: &Path,
        explorer_cfg: &ExplorerCfg,
    ) -> Result<Vec<LocalPost>> {
        let mut local_posts = Vec::new();
        let mut skipped_count = 0;
        let recursive = explorer_cfg.recursive_scan;
        let show_progress = explorer_cfg.show_scan_progress;
        let progress_threshold = explorer_cfg.progress_threshold;
        let cache_enabled = explorer_cfg.cache_metadata;

        let walker = if recursive {
            WalkDir::new(directory).follow_links(false)
        } else {
            WalkDir::new(directory).max_depth(1).follow_links(false)
        };

        let total_files: usize = walker
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .count();

        let show_progress_bar = show_progress && total_files >= progress_threshold;
        let progress_manager = Arc::new(ProgressManager::new());

        let pb = if show_progress_bar {
            Some(
                progress_manager
                    .create_bar(
                        "explorer_scan",
                        total_files as u64,
                        "Scanning files for metadata",
                    )
                    .await?,
            )
        } else {
            None
        };

        let cache = if cache_enabled {
            METADATA_CACHE.lock().ok()
        } else {
            None
        };

        let walker = if recursive {
            WalkDir::new(directory).follow_links(false)
        } else {
            WalkDir::new(directory).max_depth(1).follow_links(false)
        };

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if let Some(ref pb) = pb {
                pb.inc(1);
                let pos = pb.position();
                if pos.is_multiple_of(25) || pos == total_files as u64 {
                    pb.set_message(format!(
                        "Scanning files for metadata ({}/{})",
                        pos, total_files
                    ));
                }
            }

            let has_metadata = {
                #[cfg(target_os = "windows")]
                {
                    use std::fs::OpenOptions;
                    let ads_path = format!("{}:metadata", path.display());
                    OpenOptions::new().read(true).open(&ads_path).is_ok()
                }

                #[cfg(not(target_os = "windows"))]
                {
                    let json_path = path.with_extension(format!(
                        "{}.json",
                        path.extension().and_then(|e| e.to_str()).unwrap_or("")
                    ));
                    json_path.exists()
                }
            };

            if has_metadata {
                let post = if let Some(ref cache) = cache {
                    cache.get(&path).cloned()
                } else {
                    None
                };

                let local_post = if let Some(post) = post {
                    LocalPost {
                        post,
                        file_path: path.to_path_buf(),
                    }
                } else {
                    match LocalPost::from_file(path.to_path_buf()) {
                        Ok(local_post) => {
                            if let Some(ref _cache) = cache
                                && let Ok(mut cache_map) = METADATA_CACHE.lock()
                            {
                                cache_map.insert(path.to_path_buf(), local_post.post.clone());
                            }
                            local_post
                        }
                        Err(e) => {
                            warn!("Failed to load metadata for {}: {}", path.display(), e);
                            skipped_count += 1;
                            continue;
                        }
                    }
                };

                local_posts.push(local_post);
            }
        }

        if let Some(pb) = pb {
            pb.finish_with_message(format!(
                "Scan complete: found {} posts with metadata",
                local_posts.len()
            ));
            progress_manager.remove_bar("explorer_scan").await;
        }

        if skipped_count > 0 {
            println!(
                "Warning: Skipped {} files due to metadata read errors",
                skipped_count
            );
        }

        Ok(local_posts)
    }

    async fn slideshow(&self, posts: &[LocalPost]) -> Result<()> {
        let sleep_time = E62Rs::get()?.explorer.slideshow_wait_seconds;

        for post in posts {
            post.view().ok();
            sleep(Duration::from_secs(sleep_time));
        }

        Ok(())
    }

    async fn browse_local_posts(
        &self,
        posts: &[LocalPost],
        explorer_cfg: &ExplorerCfg,
    ) -> Result<()> {
        let posts_per_page = explorer_cfg.posts_per_page;
        let mut current_page = 0;
        let total_pages = posts.len().div_ceil(posts_per_page);

        loop {
            let start = current_page * posts_per_page;
            let end = (start + posts_per_page).min(posts.len());
            let page_posts = &posts[start..end];

            let mut options: Vec<String> = page_posts
                .iter()
                .map(|local_post| {
                    let post = &local_post.post;
                    format!(
                        "ID: {} | Score: {} | Rating: {} | Favs: {} | {}",
                        post.id,
                        post.score.total,
                        post.rating,
                        post.fav_count,
                        post.tags.artist.first().unwrap_or(&"unknown".to_string())
                    )
                })
                .collect();

            if total_pages > 1 {
                options.push(format!("--- Page {}/{} ---", current_page + 1, total_pages));
                if current_page > 0 {
                    options.push("◄ Previous Page".to_string());
                }
                if current_page < total_pages - 1 {
                    options.push("Next Page ►".to_string());
                }
            }
            options.push("◄ Back to Explorer Menu".to_string());

            let selection = Select::new("Select a post to view:", options)
                .with_help_message("Use arrow keys to navigate, Enter to select, Esc to cancel")
                .prompt_skippable()?;

            if let Some(selected) = selection {
                if selected.starts_with("Next Page") {
                    current_page = (current_page + 1).min(total_pages - 1);
                    continue;
                } else if selected.starts_with("◄ Previous Page") {
                    current_page = current_page.saturating_sub(1);
                    continue;
                } else if selected.starts_with("◄ Back") || selected.starts_with("---") {
                    break;
                }

                let index = page_posts.iter().position(|lp| {
                    format!(
                        "ID: {} | Score: {} | Rating: {} | Favs: {} | {}",
                        lp.post.id,
                        lp.post.score.total,
                        lp.post.rating,
                        lp.post.fav_count,
                        lp.post
                            .tags
                            .artist
                            .first()
                            .unwrap_or(&"unknown".to_string())
                    ) == selected
                });

                if let Some(idx) = index {
                    self.view_local_post(&page_posts[idx], explorer_cfg).await?;
                }
            } else {
                break;
            }
        }

        Ok(())
    }

    async fn view_local_post(
        &self,
        local_post: &LocalPost,
        explorer_cfg: &ExplorerCfg,
    ) -> Result<()> {
        self.display_post(&local_post.post);

        if explorer_cfg.auto_display_image
            && let Err(e) = display_image_from_path_as_sixel(&local_post.file_path)
        {
            warn!("Failed to auto-display image: {}", e);
        }

        loop {
            let action = Select::new(
                "What would you like to do?",
                vec![
                    "View image in terminal",
                    "Open in browser",
                    "Open file location",
                    "Show full metadata",
                    "Back to list",
                ],
            )
            .prompt()?;

            match action {
                "View image in terminal" => {
                    match display_image_from_path_as_sixel(&local_post.file_path) {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Failed to display local image: {}", e);
                            if let Some(ref _url) = local_post.post.file.url {
                                println!("Trying to fetch from URL instead...");
                                print_post_to_terminal(local_post.post.clone())
                                    .await
                                    .context("Failed to view image from URL")?;
                            }
                        }
                    }
                }
                "Open in browser" => {
                    self.open_in_browser(&local_post.post)?;
                }
                "Open file location" => {
                    let parent = local_post.file_path.parent().unwrap_or(Path::new("."));
                    open::that(parent).context("Failed to open file location")?;
                    println!("Opened: {}", parent.display());
                }
                "Show full metadata" => {
                    println!("\n{}", "=".repeat(70));
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&local_post.post)
                            .unwrap_or_else(|_| "Failed to serialize metadata".to_string())
                    );
                    println!("{}", "=".repeat(70));
                }
                "Back to list" => break,
                _ => {}
            }

            if !Confirm::new("Continue viewing this post?")
                .with_default(false)
                .prompt()?
            {
                break;
            }
        }

        Ok(())
    }

    fn filter_by_rating(&self, state: &mut ExplorerState) -> Result<()> {
        let options = vec!["All ratings", "Safe", "Questionable", "Explicit"];
        let selection = Select::new("Filter by rating:", options).prompt()?;

        match selection {
            "All ratings" => state.filter_by_rating(None),
            "Safe" => state.filter_by_rating(Some("s".to_string())),
            "Questionable" => state.filter_by_rating(Some("q".to_string())),
            "Explicit" => state.filter_by_rating(Some("e".to_string())),
            _ => {}
        }

        println!("Showing {} posts", state.filtered_posts.len());
        Ok(())
    }

    fn sort_posts(&self, state: &mut ExplorerState) -> Result<()> {
        let sort_by = ExplorerSortBy::select("Sort posts by:").prompt()?;
        state.sort(sort_by);
        println!("Posts sorted");
        Ok(())
    }

    fn display_statistics(&self, state: &ExplorerState) {
        let stats = state.get_statistics();

        println!("\n{}", "=".repeat(50));
        println!("Downloads Statistics");
        println!("{}", "=".repeat(50));
        println!("Total posts: {}", stats.total_posts);
        println!("Filtered posts: {}", stats.filtered_posts);
        println!("\nBy Rating:");
        println!("  Safe: {}", stats.safe);
        println!("  Questionable: {}", stats.questionable);
        println!("  Explicit: {}", stats.explicit);
        if stats.unknown > 0 {
            println!("  Unknown: {}", stats.unknown);
        }
        println!("\nStatistics:");
        println!("  Average score: {:.2}", stats.avg_score);
        println!("  Total favorites: {}", stats.total_favorites);

        if let Some(ref query) = state.search_query {
            println!("\nCurrent search: \"{}\"", query);
        }
        if let Some(ref rating) = state.rating_filter {
            println!(
                "Current rating filter: {}",
                match rating.as_str() {
                    "s" => "Safe",
                    "q" => "Questionable",
                    "e" => "Explicit",
                    _ => "Unknown",
                }
            );
        }
        println!("Current sort: {:?}", state.current_sort);
        println!("{}", "=".repeat(50));
    }
}

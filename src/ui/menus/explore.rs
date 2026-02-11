//! downloads explorer stuff
//!
//! provides a menu for browsing downloaded posts
use {
    crate::{
        bail,
        error::Result,
        getopt,
        models::E6Post,
        ui::{
            E6Ui,
            menus::{
                ExplorerMenu, ExplorerSortBy,
                view::{ViewMenu, print_dl_to_terminal, print_post_to_terminal},
            },
            progress::ProgressManager,
        },
    },
    color_eyre::eyre::Context,
    crossterm::event::{Event, KeyCode, KeyEventKind},
    demand::Confirm,
    futures::lock::Mutex,
    hashbrown::HashMap,
    inquire::Select,
    jwalk::WalkDir,
    owo_colors::OwoColorize,
    qrcode::QrCode,
    std::{
        fs::OpenOptions,
        io::Read,
        path::{Path, PathBuf},
        sync::Arc,
        time::Duration,
    },
    tracing::warn,
};

lazy_static::lazy_static! {
    /// the metadata cache for the explorer
    ///
    /// stores parsed post metadata in mem to avoid repeatedly reading from disk. shared across all
    /// explorer ops.
    static ref METADATA_CACHE: Arc<Mutex<HashMap<PathBuf, E6Post>>> = Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Debug, Clone)]
/// a local post
///
/// represents a downloaded post with its associated file and metadata
pub struct LocalPost {
    /// the post metadata
    ///
    /// contains all e6 metadata for this post
    pub post: E6Post,

    /// the path to the post
    ///
    /// the location of the downloaded file on the filesystem
    pub file_path: PathBuf,
}

impl LocalPost {
    /// read a file into a [`LocalPost`]
    ///
    /// loads a file from disk and parses its associated metadata into a [`LocalPost`]. metadata is
    /// read from alternate data streams (Windows) or JSON sidecar files (Unix)
    #[bearive::argdoc]
    #[error = "the metadata can't be read"]
    #[error = "the metadata is malformed"]
    pub fn from_file(
        /// the path to the downloaded file
        file_path: PathBuf,
    ) -> Result<Self> {
        let post = Self::read_metadata(&file_path)?;
        Ok(Self { post, file_path })
    }

    /// view a post
    ///
    /// displays the post image/animation in the terminal via sixel
    #[bearive::argdoc]
    #[error = "the file cannot be displayed"]
    pub fn view(&self) -> Result<()> {
        crate::ui::menus::view::print_dl_to_terminal(&self.file_path)
    }

    #[cfg(target_os = "windows")]
    /// read post metadata (windows)
    ///
    /// reads metadata from an NTFS alternate data stream named `metadata`. the metadata is stored
    /// as JSON in the ADS
    #[bearive::argdoc]
    #[error = "the ads can't be opened"]
    #[error = "the ads can't be parsed"]
    pub fn read_metadata(
        /// the path to the file containing the alternate data stream
        file_path: &Path,
    ) -> Result<E6Post> {
        let ads_path = format!("{}:metadata", file_path.display());
        let mut contents = String::new();
        let mut file = OpenOptions::new()
            .read(true)
            .open(&ads_path)
            .with_context(|| format!("Failed to open ADS metadata for {}", file_path.display()))?;

        file.read_to_string(&mut contents)
            .with_context(|| format!("Failed to read ADS metadata for {}", file_path.display()))?;

        serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse metadata for {}", file_path.display()))
            .map_err(|e| e.into())
    }

    #[cfg(not(target_os = "windows"))]
    /// read post metadata (non-windows)
    pub fn read_metadata(file_path: &Path) -> Result<E6Post> {
        let json_path = file_path.with_extension(format!(
            "{}.json",
            file_path.extension().and_then(|e| e.to_str()).unwrap_or("")
        ));

        if !json_path.exists() {
            bail!("Metadata file not found: {}", json_path.display());
        }

        let contents = std::fs::read_to_string(&json_path)
            .with_context(|| format!("Failed to read metadata file {}", json_path.display()))?;

        serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse metadata for {}", file_path.display()))
            .map_err(|e| e.into())
    }
}

/// state of the explorer
///
/// maintains the curr state of the explorer including all loaded posts, filters, queries, and sort
/// order
pub struct ExplorerState {
    /// loaded posts
    ///
    /// all posts that're loaded from the downloads dir
    pub posts: Vec<LocalPost>,

    /// posts that match the current filters
    ///
    /// all posts that pass all active filters and queries
    pub filtered_posts: Vec<LocalPost>,

    /// the current sort mode
    ///
    /// how the filtered posts are sorted
    pub current_sort: ExplorerSortBy,

    /// the current search query (if any)
    ///
    /// text that posts must match in tags, id, desc, file ext, or uploader
    pub search_query: Option<String>,

    /// the current content rating filter (if any)
    ///
    /// if set, only posts with this rating (`s`/`q`/`e`) will be shown
    pub rating_filter: Option<String>,
}

impl ExplorerState {
    /// initialize the explorer state
    ///
    /// makes a new explorer state with the given posts. initially, all posts are included in the
    /// filtered list with default date-newest sorting
    #[bearive::argdoc]
    pub fn new(
        /// all posts loaded from the downloads directory
        posts: Vec<LocalPost>,
    ) -> Self {
        let filtered_posts = posts.clone();
        Self {
            posts,
            filtered_posts,
            current_sort: ExplorerSortBy::DateNewest,
            search_query: None,
            rating_filter: None,
        }
    }

    /// sort the current loaded downloads
    ///
    /// applies the specified sort order to the filtered posts. does not affect the full posts list
    #[bearive::argdoc]
    pub fn sort(
        &mut self,
        /// how to sort the filtered posts
        sort_by: ExplorerSortBy,
    ) {
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
                    .sort_by_key(|b| std::cmp::Reverse(b.post.score.total));
            }
            ExplorerSortBy::ScoreLowest => {
                self.filtered_posts.sort_by_key(|a| a.post.score.total);
            }
            ExplorerSortBy::FavoritesHighest => {
                self.filtered_posts
                    .sort_by_key(|b| std::cmp::Reverse(b.post.fav_count));
            }
            ExplorerSortBy::FavoritesLowest => {
                self.filtered_posts.sort_by_key(|a| a.post.fav_count);
            }
            ExplorerSortBy::IDAscending => {
                self.filtered_posts.sort_by_key(|a| a.post.id);
            }
            ExplorerSortBy::IDDescending => {
                self.filtered_posts
                    .sort_by_key(|b| std::cmp::Reverse(b.post.id));
            }
        }
    }

    /// filter posts by content rating
    ///
    /// filters the posts to only show those matching the specified rating. using `None` removes
    /// the rating filter
    #[bearive::argdoc]
    pub fn filter_by_rating(
        &mut self,
        /// the rating to filter for (`s`/`q`/`e`) or None to show all
        rating: Option<String>,
    ) {
        self.rating_filter = rating.clone();
        self.apply_filters();
    }

    /// search for posts given a query
    ///
    /// filters posts to only show those matching the search query in their id, tags, desc,
    /// uploader name, or file ext. using `None` removes the search filter
    #[bearive::argdoc]
    pub fn search(
        &mut self,
        /// the search text, or `None` to clear search
        query: Option<String>,
    ) {
        self.search_query = query;
        self.apply_filters();
    }

    /// apply the selected filters
    ///
    /// rebuilds the filtered posts list by applying all active filters (rating + search).
    /// automatically re-sorts after filtering
    pub fn apply_filters(&mut self) {
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
                    let matches_extension = local_post
                        .file_path
                        .extension()
                        .map(|a| a.to_str().unwrap_or(""))
                        .unwrap_or("")
                        .contains(&query_lower);

                    if !matches_id
                        && !matches_tags
                        && !matches_description
                        && !matches_uploader
                        && !matches_extension
                    {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        self.sort(self.current_sort);
    }

    /// get statistics based on the current filtered posts
    ///
    /// calculates stats about both the full post collection and the curr filtered subset. includes
    /// counts by rating, avg score, and total faves
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

/// statistics for the explorer
///
/// contains aggregate stats about the post collection including:
/// - counts
/// - rating distrobution
/// - score metrics
#[derive(Debug)]
pub struct ExplorerStatistics {
    /// total available posts
    ///
    /// the total number of posts loaded from disk
    pub total_posts: usize,

    /// total filtered posts
    ///
    /// the number of posts matching curr filters
    pub filtered_posts: usize,

    /// total safe posts
    ///
    /// count of posts with safe rating
    pub safe: usize,

    /// total questionable posts
    ///
    /// count of posts with questionable rating
    pub questionable: usize,

    /// total explicit posts
    ///
    /// count of posts with explicit rating
    pub explicit: usize,

    /// total unknown posts
    ///
    /// count of posts with unrecognized rating
    pub unknown: usize,

    /// the average score of loaded posts
    ///
    /// mean total scores across all loaded posts
    pub avg_score: f64,

    /// the total favorites across all loaded posts
    ///
    /// sum of favorite counts from all posts
    pub total_favorites: i64,
}

/// functions for the explorer menu
///
/// provides the ui for the downloads browser (TUI | For the web gallery, see [`crate::serve`])
#[bearive::argdoc]
pub trait ExploreMenu {
    /// show downloads explorer
    ///
    /// shows the main explorer interface with opts for browsing, searching, filtering, sorting,
    /// and viewing stats. loops until the user chooses to exit
    ///
    /// # Errors
    ///
    /// returns an error if:
    /// - the downloads dir doesn't exist
    /// - posts can't be scanned
    /// - user interaction fails
    fn explore_downloads(&self) -> impl Future<Output = Result<()>>;

    /// scan the downloads directory for posts
    ///
    /// # Returns
    ///
    /// a list of local posts found in the dl dir
    ///
    /// # Errors
    ///
    /// returns an error if:
    /// - dir traversal fails
    fn scan_downloads_directory(
        &self,
        /// the directory to scan
        directory: &Path,
    ) -> impl Future<Output = Result<Vec<LocalPost>>>;

    /// show a slideshow of filtered posts
    ///
    /// displays posts sequentially in the term with configurable timing. supports pausing,
    /// navigation, and exit controls via the kb
    ///
    /// # Keyboard controls
    ///
    /// - Space: pause/resume
    /// - Left/H: prev post
    /// - Right/L: next post
    /// - Q/Esc: exit slideshow
    ///
    /// # Errors
    ///
    /// returns an error if:
    /// - image display fails
    /// - keyboard input fails
    fn slideshow(
        &self,
        /// the posts to include in the slideshow
        posts: &[LocalPost],
    ) -> impl Future<Output = Result<()>>;

    /// browse local downloads
    ///
    /// shows a paginated, searchable list of posts for selection and viewing. supports navigation
    /// between pages and returns to the explorer menu
    ///
    /// # Errors
    ///
    /// returns an error if:
    /// - user interaction fails
    fn browse_local_posts(
        &self,
        /// the posts to browse
        posts: &[LocalPost],
    ) -> impl Future<Output = Result<()>>;

    /// print a local post
    ///
    /// displays details info about a post and provides opts to view its image, open it in browser,
    /// show it in explorer, or display full metadata
    ///
    /// # Errors
    ///
    /// returns an error if:
    /// - image display fails
    /// - browser opening fails
    /// - file ops fail
    fn view_local_post(
        &self,
        /// the post to view
        local_post: &LocalPost,
    ) -> impl Future<Output = Result<()>>;

    /// filter posts by content rating
    ///
    /// prompts the user to select a rating filter (`all`/`safe`/`questionable`/`explicit`) and
    /// applies it to the explorer state
    ///
    /// # Errors
    ///
    /// returns an error if:
    /// - user inteeraction fails
    fn filter_by_rating(
        &self,
        /// the explorer state to update
        state: &mut ExplorerState,
    ) -> Result<()>;

    /// sort posts
    ///
    /// prompts the user to select a sort mode and applies it to the explorer state
    ///
    /// # Errors
    ///
    /// returns an error if:
    /// - user interaction fails
    fn sort_posts(
        &self,
        /// the explorer state to update
        state: &mut ExplorerState,
    ) -> Result<()>;

    /// display explorer stats
    ///
    /// prints stats about the post collection including:
    /// - counts by rating
    /// - average score
    /// - total faves
    /// - active filters
    fn display_statistics(
        &self,
        /// the explorer state to analyze
        state: &ExplorerState,
    );
}

impl ExploreMenu for E6Ui {
    /// show downloads explorer
    async fn explore_downloads(&self) -> Result<()> {
        println!("\n{0} Downloads Explorer {0}\n", "===".green());

        let download_dir: String = getopt!(download.path);
        let directory = Path::new(&download_dir);

        if !directory.exists() {
            bail!("Download directory does not exist: {}", directory.display());
        }

        let local_posts = self.scan_downloads_directory(directory).await?;

        if local_posts.is_empty() {
            println!("No posts with metadata found in {}", directory.display());
            return Ok(());
        }

        println!("Found {} posts with metadata\n", local_posts.len());

        let mut state = ExplorerState::new(local_posts);

        let default_sort_str: String = getopt!(explorer.default_sort);
        let default_sort = match default_sort_str.as_str() {
            "date_newest" => ExplorerSortBy::DateNewest,
            "date_oldest" => ExplorerSortBy::DateOldest,
            "score_highest" => ExplorerSortBy::ScoreHighest,
            "score_lowest" => ExplorerSortBy::ScoreLowest,
            "favorites_highest" => ExplorerSortBy::FavoritesHighest,
            "id_ascending" => ExplorerSortBy::IDAscending,
            "id_descending" => ExplorerSortBy::IDDescending,
            _ => ExplorerSortBy::DateNewest,
        };
        state.sort(default_sort);

        loop {
            let action = ExplorerMenu::select(&format!(
                "Downloads Explorer ({} posts shown)",
                state.filtered_posts.len()
            ))
            .ask()?;

            let should_break = match action {
                ExplorerMenu::Browse => {
                    if state.filtered_posts.is_empty() {
                        println!("No posts match the current filters.");
                    } else {
                        self.browse_local_posts(&state.filtered_posts).await?;
                    }
                    false
                }
                ExplorerMenu::Search => {
                    let query = inquire::Text::new(
                        "Enter search query (tags, file extension, ID, uploader, or description):",
                    )
                    .prompt()?;
                    state.search(Some(query));
                    println!("Found {} matching posts", state.filtered_posts.len());
                    false
                }
                ExplorerMenu::FilterByRating => {
                    self.filter_by_rating(&mut state)?;
                    false
                }
                ExplorerMenu::SortBy => {
                    self.sort_posts(&mut state)?;
                    false
                }
                ExplorerMenu::ViewStatistics => {
                    self.display_statistics(&state);
                    false
                }
                ExplorerMenu::ClearFilters => {
                    state.search(None);
                    state.filter_by_rating(None);
                    println!(
                        "Filters cleared. Showing all {} posts",
                        state.filtered_posts.len()
                    );
                    false
                }
                ExplorerMenu::Slideshow => {
                    self.slideshow(&state.filtered_posts).await?;
                    false
                }
                ExplorerMenu::Back => true,
            };

            if should_break {
                break;
            }

            if !Confirm::new("Continue exploring?").run()? {
                break;
            }
        }

        Ok(())
    }

    /// scan the downloads directory for posts
    async fn scan_downloads_directory(&self, directory: &Path) -> Result<Vec<LocalPost>> {
        let mut local_posts = Vec::new();
        let mut skipped_count = 0;

        let recursive: bool = getopt!(explorer.recursive);
        let show_progress: bool = getopt!(explorer.show_progress);
        let progress_threshold: usize = getopt!(explorer.progress_threshold);
        let cache_enabled: bool = getopt!(explorer.cache_metadata);

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
                    .create_count_bar(
                        "explorer_scan",
                        total_files as u64,
                        "Scanning files for metadata",
                    )
                    .await?,
            )
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
                let cached_post = if cache_enabled {
                    let cache = METADATA_CACHE.lock().await;
                    cache.get(&path).cloned()
                } else {
                    None
                };

                let local_post = if let Some(post) = cached_post {
                    LocalPost {
                        post,
                        file_path: path.to_path_buf(),
                    }
                } else {
                    match LocalPost::from_file(path.to_path_buf()) {
                        Ok(local_post) => {
                            if cache_enabled {
                                let mut cache_map = METADATA_CACHE.lock().await;
                                cache_map.insert(path.to_path_buf(), local_post.post.clone());
                            }

                            local_post
                        }

                        Err(e) => {
                            warn!("failed to load metadata for {}: {}", path.display(), e);
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

    /// show a slideshow of filtered posts
    async fn slideshow(&self, posts: &[LocalPost]) -> Result<()> {
        let mut paused = false;
        let sleep_seconds: u64 = getopt!(explorer.slideshow_delay);
        let slide_duration = Duration::from_secs(sleep_seconds);
        let tick_rate = Duration::from_millis(100);
        let mut i = 0;

        while i < posts.len() {
            print!("\x1B[2J\x1B[3J\x1B[H");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();

            println!("Space to pause | Left/H - Go back | Right/L - Go forward");

            posts[i].view().ok();

            let mut time_elapsed = Duration::ZERO;

            loop {
                if crossterm::event::poll(tick_rate)?
                    && let Event::Key(key) = crossterm::event::read()?
                    && key.kind == KeyEventKind::Press
                {
                    match key.code {
                        KeyCode::Char(' ') => paused = !paused,
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),

                        KeyCode::Right | KeyCode::Char('l') => {
                            i += 1;
                            break;
                        }

                        KeyCode::Left | KeyCode::Char('h') => {
                            i = i.saturating_sub(1);
                            break;
                        }

                        _ => {}
                    }

                    if paused {
                        continue;
                    }

                    time_elapsed += tick_rate;
                    if time_elapsed >= slide_duration {
                        i += 1;
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// browse local downloads
    async fn browse_local_posts(&self, posts: &[LocalPost]) -> Result<()> {
        let posts_per_page: usize = getopt!(explorer.posts_per_page);
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

            let selection = Some(
                Select::new("Select a post to view:", options)
                    .with_help_message("Use arrow keys to navigate, Enter to select, Esc to cancel")
                    .prompt()?,
            );

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
                    self.view_local_post(&page_posts[idx]).await?;
                }
            } else {
                break;
            }
        }

        Ok(())
    }

    /// print a local post
    async fn view_local_post(&self, local_post: &LocalPost) -> Result<()> {
        self.display_post(&local_post.post);

        let auto_display: bool = getopt!(explorer.auto_display_image);
        if auto_display && let Err(e) = print_dl_to_terminal(&local_post.file_path) {
            warn!("Failed to auto-display image: {}", e);
        }

        let opts = [
            "Make QR",
            "View image in terminal",
            "Open in browser",
            "Open file location",
            "Show full metadata",
            "Back to list",
        ];

        loop {
            let action = Select::new("What would you like to do?", opts.to_vec()).prompt()?;

            match action {
                "View image in terminal" => match print_dl_to_terminal(&local_post.file_path) {
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
                },
                "Open in browser" => {
                    self.open_in_browser(&local_post.post)?;
                }
                "Open file location" => {
                    let parent = local_post.file_path.parent().unwrap_or(Path::new("."));
                    open::that(parent).context("Failed to open file location")?;
                    println!("Opened: {}", parent.display());
                }
                "Make QR" => {
                    let url = format!("https://e621.net/posts/{}", local_post.post.id);
                    let qr = QrCode::with_version(
                        url.into_bytes(),
                        qrcode::Version::Normal(4),
                        qrcode::EcLevel::L,
                    )?;
                    let str = qr
                        .render::<char>()
                        .quiet_zone(true)
                        .module_dimensions(2, 1)
                        .build()
                        .replace("#", "█");

                    println!("{}", str);
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
                "Back to list" => {
                    print!("\x1B[2J\x1B[3J\x1B[H");
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                    break;
                }
                _ => {}
            }

            if !Confirm::new("Continue viewing this post?").run()? {
                break;
            }
        }

        Ok(())
    }

    /// filter posts by content rating
    fn filter_by_rating(&self, state: &mut ExplorerState) -> Result<()> {
        let options = ["All ratings", "Safe", "Questionable", "Explicit"];
        let selection = Select::new("Filter by rating:", options.to_vec()).prompt()?;

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

    /// sort posts
    fn sort_posts(&self, state: &mut ExplorerState) -> Result<()> {
        let sort_by = ExplorerSortBy::select("Sort posts by:").ask()?;
        state.sort(sort_by);
        println!("Posts sorted");
        Ok(())
    }

    /// display explorer stats
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

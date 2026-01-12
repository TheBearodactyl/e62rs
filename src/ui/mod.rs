//! tui stuff for e62rs
use {
    crate::{
        client::E6Client,
        config::{blacklist::get_blacklist, options::E62Rs},
        data::{pools::PoolDb, tags::TagDb},
        getopt,
        models::{E6Pool, E6Post},
        serve::{cfg::ServerConfig, server::MediaServer},
        ui::{
            autocomplete::TagAutocompleter,
            menus::{
                BatchAction, InteractionMenu, PoolInteractionMenu,
                download::{PostDownloader, sanitize_pool_name},
                explore::ExploreMenu,
                view::{ViewMenu, print_post_to_terminal, print_posts_to_terminal},
            },
            progress::ProgressManager,
        },
    },
    color_eyre::eyre::{Context, Result, bail},
    hashbrown::{HashMap, HashSet},
    indicatif::{ProgressBar, ProgressStyle},
    inquire::{Confirm, MultiSelect, Text},
    owo_colors::OwoColorize,
    qrcode::QrCode,
    serde::{Deserialize, Serialize},
    std::{path::PathBuf, str::FromStr, sync::Arc, time::Duration},
    tokio::{fs, sync::Semaphore},
    tracing::{debug, info, warn},
};

pub mod autocomplete;
pub mod menus;
pub mod progress;
pub mod themes;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
/// a post download
pub struct Download {
    /// the post data
    pub post_data: E6Post,
    /// the path to download the posts to
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
/// all posts being downloaded
pub struct DownloadData {
    /// the posts being downloaded
    pub posts: Vec<Download>,
}

#[derive(Default, Clone)]
/// the ui for e62rs
pub struct E6Ui {
    /// the e6 api client
    pub client: Arc<E6Client>,
    /// the post downloader
    pub downloader: Arc<PostDownloader>,
    /// the tags db
    pub tag_db: Arc<TagDb>,
    /// the pools db
    pub pool_db: Arc<PoolDb>,
}

impl E6Ui {
    /// make a new e6ui
    ///
    /// # Arguments
    ///
    /// * `client` - an e621 api client (see [`crate::client::E6Client`])
    /// * `tag_db` - a loaded tag database
    /// * `pool_db` - a loaded pool database
    pub fn new(client: Arc<E6Client>, tag_db: Arc<TagDb>, pool_db: Arc<PoolDb>) -> Self {
        let downloader = Arc::new(PostDownloader::with_download_dir_and_format(
            getopt!(download.path),
            Some(getopt!(download.format)),
        ));

        if std::fs::exists(getopt!(download.path)).is_err() {
            std::fs::create_dir_all(getopt!(download.path))
                .expect("Failed to create output directory");
        }

        Self {
            client,
            downloader,
            tag_db,
            pool_db,
        }
    }

    /// get tags to be searched via user input (has autocompletion)
    pub fn collect_tags(&self) -> Result<(Vec<String>, Vec<String>, Vec<String>)> {
        if getopt!(ui.tag_guide) {
            println!("\n{}", "Tag Input Instructions:".bold().cyan());
            println!("  {} Separate tags with spaces", "•".bright_blue());
            println!(
                "  {} Prefix with {} to exclude a tag (e.g., {})",
                "•".bright_blue(),
                "-".red().bold(),
                "-gore".red()
            );
            println!(
                "  {} Prefix with {} for OR logic (e.g., {} {} means cat OR dog)",
                "•".bright_blue(),
                "~".yellow().bold(),
                "~cat".yellow(),
                "~dog".yellow()
            );
            println!(
                "  {} Press {} to autocomplete",
                "•".bright_blue(),
                "Tab".green().bold()
            );
            println!(
                "  {} Aliases shown as {}\n",
                "•".bright_blue(),
                "'alias -> canonical'".italic().bright_black()
            );
        }

        let autocompleter = TagAutocompleter::new(self.tag_db.clone());
        let tags_input = inquire::Text::new("Enter tags:")
            .with_help_message(
                "Space-separated tags. Use - to exclude, ~ for OR. Tab to autocomplete",
            )
            .with_placeholder("e.g., cat dog -gore")
            .with_autocomplete(autocompleter)
            .prompt()
            .context("failed to get tags input")?;

        let mut include_tags = Vec::new();
        let mut exclude_tags = Vec::new();
        let mut or_tags = Vec::new();

        for tag in tags_input.split_whitespace() {
            let tag = tag.trim();

            if tag.is_empty() {
                continue;
            }

            if let Some(stripped) = tag.strip_prefix('-') {
                let canonical = self.tag_db.get_canon_name(stripped);
                if !exclude_tags.contains(&canonical) {
                    exclude_tags.push(canonical);
                }
            } else if let Some(stripped) = tag.strip_prefix('~') {
                let canonical = self.tag_db.get_canon_name(stripped);
                if !or_tags.contains(&canonical) {
                    or_tags.push(canonical);
                }
            } else {
                let stripped = tag.strip_prefix('+').unwrap_or(tag);
                let canonical = self.tag_db.get_canon_name(stripped);
                if !include_tags.contains(&canonical) {
                    include_tags.push(canonical);
                }
            }
        }

        println!();
        if !include_tags.is_empty() {
            println!(
                "{} Include tags: {}",
                "✓".green().bold(),
                include_tags.join(" ").bright_green()
            );
        }
        if !exclude_tags.is_empty() {
            println!(
                "{} Exclude tags: {}",
                "✓".red().bold(),
                format!("-{}", exclude_tags.join(" -")).red()
            );
        }
        if !or_tags.is_empty() {
            println!(
                "{} OR tags: {}",
                "✓".yellow().bold(),
                format!("~{}", or_tags.join(" ~")).yellow()
            );
        }

        if include_tags.is_empty() && exclude_tags.is_empty() && or_tags.is_empty() {
            println!("{}", "No tags entered.".bright_black().italic());
        }

        Ok((include_tags, or_tags, exclude_tags))
    }

    /// shows a list of posts and allows the user to select from them
    ///
    /// # Arguments
    ///
    /// * `posts` - the posts to select from
    pub fn select_multiple_posts<'a>(&self, posts: &'a [E6Post]) -> Result<Vec<&'a E6Post>> {
        let options: Vec<String> = posts
            .iter()
            .map(|post| {
                format!(
                    "ID: {} | Score: {} | Rating: {}",
                    post.id, post.score.total, post.rating
                )
            })
            .collect();

        let selections =
            MultiSelect::new("Select posts (Space to select, Enter to confirm):", options)
                .with_help_message(
                    "Use arrow keys to navigate, Space to select/deselect, Enter to confirm",
                )
                .prompt()
                .context("Failed to get post selections")?;

        let selected_posts: Vec<&E6Post> = selections
            .iter()
            .filter_map(|selection| {
                selection
                    .split_whitespace()
                    .nth(1)
                    .and_then(|id_str| id_str.parse::<i64>().ok())
                    .and_then(|id| posts.iter().find(|p| p.id == id))
            })
            .collect();

        Ok(selected_posts)
    }

    /// shows the interaction menu for a post
    ///
    /// # Arguments
    ///
    /// * `post` - the post to interact with
    pub async fn interaction_menu(&self, post: E6Post) -> Result<InteractionMenu> {
        let choice = InteractionMenu::select("What would you like to do?")
            .prompt()
            .context("Failed to get interaction choice")?;

        match choice {
            InteractionMenu::OpenInBrowser => {
                self.open_in_browser(&post)?;
            }
            InteractionMenu::Download => {
                self.downloader
                    .clone()
                    .download_post(post.clone(), post.id as usize)
                    .await?;
            }
            InteractionMenu::Back => {
                return Ok(InteractionMenu::Back);
            }
            InteractionMenu::View => {
                print_post_to_terminal(post)
                    .await
                    .context("Failed to view image")?;
            }

            InteractionMenu::MakeQr => {
                let url = format!("https://e621.net/posts/{}", post.id);
                let code = QrCode::new(url.into_bytes())?;
                let str = code
                    .render::<char>()
                    .quiet_zone(true)
                    .module_dimensions(2, 1)
                    .build();

                println!("{}", str.replace("#", "█"));
            }
        }

        Ok(choice)
    }

    /// opens the current configuration in the default editor
    pub async fn edit_config_file(&self) -> Result<()> {
        let curr_cfg = toml::to_string_pretty(&E62Rs::load()?)?;

        println!("Opening config file in your default editor...");

        let temp_file = std::env::temp_dir().join("e62rs_config.toml");
        fs::write(&temp_file, &curr_cfg).await?;

        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
        std::process::Command::new(editor)
            .arg(&temp_file)
            .status()?;

        let new_cfg_text = fs::read_to_string(&temp_file).await?;

        if let Ok(new_cfg) = toml::from_str::<E62Rs>(new_cfg_text.as_str()) {
            fs::write("e62rs.toml", toml::to_string_pretty(&new_cfg)?).await?;
            fs::remove_file(&temp_file).await?;
        } else {
            eprintln!("Error validating new config text");
            std::process::exit(1);
        }

        crate::config::instance::reload_config()?;

        Ok(())
    }

    /// display a menu for interacting with multiple posts at once
    ///
    /// # Arguments
    ///
    /// * `posts` - the posts to interact with
    pub async fn batch_interaction_menu(&self, posts: Vec<E6Post>) -> Result<BatchAction> {
        let choice = BatchAction::select(&format!(
            "What would you like to do with {} selected posts?",
            posts.len()
        ))
        .prompt()
        .context("Failed to get batch action choice")?;

        match choice {
            BatchAction::DownloadAll => {
                self.downloader.clone().download_posts(posts).await?;
            }
            BatchAction::Browser => {
                self.open_posts_in_browser(&posts)?;
            }
            BatchAction::DlAndOpen => {
                let posts_clone = posts.clone();
                self.downloader.clone().download_posts(posts).await?;
                self.open_posts_in_browser(&posts_clone)?;
            }
            BatchAction::Back => {
                return Ok(BatchAction::Back);
            }
            BatchAction::ViewAll => {
                let posts_clone = posts.clone();
                print_posts_to_terminal(posts_clone).await?;
            }
        }

        Ok(choice)
    }

    /// display a menu for interacting with posts
    ///
    /// # Arguments
    ///
    /// * `pool` - the pool to interact with
    pub async fn pool_interaction_menu(&self, pool: E6Pool) -> Result<PoolInteractionMenu> {
        let choice = PoolInteractionMenu::select("What would you like to do with this pool?")
            .prompt()
            .context("Failed to get pool interaction choice")?;

        match choice {
            PoolInteractionMenu::ViewPosts => {
                let posts = self.client.get_pool_posts(pool.id).await?;
                if posts.posts.is_empty() {
                    println!("No posts found in this pool.");
                } else {
                    self.display_posts(&posts.posts);

                    let interact =
                        inquire::Confirm::new("Would you like to interact with these posts?")
                            .prompt()?;

                    if interact {
                        let _selected_posts = self.select_multiple_posts(&posts.posts)?;
                        let selected_posts = self.select_multiple_posts(&posts.posts)?;
                        if !selected_posts.is_empty() {
                            let mut fetched_posts = Vec::new();
                            for post in &selected_posts {
                                let fetched = self.client.get_post_by_id(post.id).await?;
                                fetched_posts.push(fetched.post);
                            }
                            self.batch_interaction_menu(fetched_posts).await?;
                        }
                    }
                }
            }
            PoolInteractionMenu::Download => {
                let posts = self.client.get_pool_posts(pool.id).await?;
                if posts.posts.is_empty() {
                    println!("No posts found in this pool.");
                } else {
                    println!(
                        "Downloading {} posts from pool '{}'...",
                        posts.posts.len(),
                        pool.name
                    );
                    self.downloader.clone().download_posts(posts.posts).await?;
                }
            }
            PoolInteractionMenu::DownloadToPoolsFolder => {
                self.download_pool_to_pools_folder(&pool).await?;
            }
            PoolInteractionMenu::OpenInBrowser => {
                let url = format!("https://e621.net/pools/{}", pool.id);
                open::that(&url).context("Failed to open pool in browser")?;
                println!("Opened pool in browser: {}", url);
            }
            PoolInteractionMenu::Back => {
                return Ok(PoolInteractionMenu::Back);
            }
        }

        Ok(choice)
    }

    /// download pool posts to `<pools_dir>/<pool_name>/`
    ///
    /// # Arguments
    ///
    /// * `pool` - the pool to download
    pub async fn download_pool_to_pools_folder(&self, pool: &E6Pool) -> Result<()> {
        if pool.post_ids.is_empty() {
            println!("This pool has no posts to download.");
            return Ok(());
        }

        let download_dir: PathBuf = getopt!(download.pools_path).into();
        if !download_dir.exists() {
            match fs::create_dir_all(&download_dir).await {
                Ok(_) => info!("Automatically created pools download dir"),
                Err(e) => {
                    tracing::error!(
                        "Could not create pools download dir at path '{}': {}",
                        download_dir.display(),
                        e
                    );

                    bail!(
                        "Could not create pools download dir at path '{}': {}",
                        download_dir.display(),
                        e
                    );
                }
            }
        }

        let pool_dir = download_dir
            .join("pools")
            .join(sanitize_pool_name(&pool.name));

        println!(
            "Downloading {} posts to: {}",
            pool.post_ids.len(),
            pool_dir.display()
        );
        println!("Files will be named sequentially: 001.ext, 002.ext, ...");

        let posts = self.fetch_pool_posts(pool).await?;

        if posts.is_empty() {
            println!("Failed to fetch any posts from this pool.");
            return Ok(());
        }

        let downloader = Arc::new(PostDownloader::for_pool(&download_dir, &pool.name));

        downloader.download_pool_posts(posts).await?;

        println!(
            "✓ Pool '{}' downloaded to {}",
            pool.name,
            pool_dir.display()
        );

        Ok(())
    }

    /// fetch all posts from a pool
    ///
    /// # Arguments
    ///
    /// * `pool` - the pool to fetch posts from
    pub async fn fetch_pool_posts(&self, pool: &E6Pool) -> Result<Vec<E6Post>> {
        let total = pool.post_ids.len();
        if total == 0 {
            return Ok(Vec::new());
        }

        let cached_results = self
            .client
            .post_cache
            .get_batch(&pool.post_ids)
            .await
            .context("Failed to check post cache")?;

        let mut final_posts: Vec<Option<E6Post>> = cached_results;
        let uncached_indices: Vec<usize> = final_posts
            .iter()
            .enumerate()
            .filter_map(|(idx, post)| if post.is_none() { Some(idx) } else { None })
            .collect();

        let cached_count = total - uncached_indices.len();
        if cached_count > 0 {
            println!(
                "Found {}/{} posts in cache, fetching {} from API...",
                cached_count,
                total,
                uncached_indices.len()
            );
        }

        if uncached_indices.is_empty() {
            return Ok(final_posts.into_iter().flatten().collect());
        }

        let pb = ProgressBar::new(uncached_indices.len() as u64);
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.bright_cyan} [{elapsed_precise}] [{bar:40.bright_cyan/blue}] \
                 {pos}/{len} Fetching pool posts...",
            )
            .context("Failed to create progress bar template")?
            .progress_chars("━╸─"),
        );
        pb.enable_steady_tick(Duration::from_millis(100));

        let concurrent_limit = getopt!(search.fetch_threads).max(1);
        let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrent_limit));

        let tasks: Vec<_> = uncached_indices
            .iter()
            .map(|&idx| {
                let post_id = pool.post_ids[idx];
                let client = self.client.clone();
                let semaphore = Arc::clone(&semaphore);
                let pb = pb.clone();

                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.ok()?;
                    let result = client.get_post_by_id(post_id).await.ok();
                    pb.inc(1);
                    result.map(|r| (idx, r.post))
                })
            })
            .collect();

        let results = futures_util::future::join_all(tasks).await;
        pb.finish_with_message("✓ Fetched pool posts");

        let mut posts_to_cache: Vec<E6Post> = Vec::new();

        for result in results {
            if let Ok(Some((idx, post))) = result {
                posts_to_cache.push(post.clone());
                final_posts[idx] = Some(post);
            }
        }

        if !posts_to_cache.is_empty() {
            if let Err(e) = self.client.post_cache.insert_batch(&posts_to_cache).await {
                warn!("Failed to cache fetched posts: {}", e);
            } else {
                debug!("Cached {} newly fetched posts", posts_to_cache.len());
            }
        }

        let posts: Vec<E6Post> = final_posts.into_iter().flatten().collect();

        let fetched_count = posts.len();
        if fetched_count < total {
            warn!(
                "Could only fetch {}/{} posts from pool",
                fetched_count, total
            );
        }

        Ok(posts)
    }

    /// serve all downloaded files
    pub async fn serve_downloads(&self) -> Result<()> {
        let downloads_dir = getopt!(download.path);
        let enable_metadata = getopt!(gallery.metadata_filtering);
        let cache_metadata = getopt!(gallery.cache_metadata);

        let srv_cfg = ServerConfig::builder()
            .media_directory(PathBuf::from_str(&downloads_dir)?)
            .bind_address(format!("127.0.0.1:{}", getopt!(gallery.port)).parse()?)
            .max_file_size(100 * 1024 * 1024)
            .enable_metadata_filtering(enable_metadata)
            .cache_metadata(cache_metadata)
            .build()
            .expect("Failed to build server config");

        let srv = MediaServer::new(srv_cfg);

        if getopt!(gallery.auto_open) {
            let url = format!("http://localhost:{}", getopt!(gallery.port));
            let _ = open::that(&url);
            println!("Opening browser at {}", url);
        }

        srv.serve().await
    }

    /// check if a post contains any blacklisted tags
    ///
    /// # Arguments
    ///
    /// * `post` - the post to check
    /// * `blacklist` - the current blacklist
    pub fn post_contains_blacklisted_tags(post: &E6Post, blacklist: &HashSet<String>) -> bool {
        if blacklist.is_empty() {
            return false;
        }

        let all_tags = post
            .tags
            .general
            .iter()
            .chain(post.tags.artist.iter())
            .chain(post.tags.character.iter())
            .chain(post.tags.species.iter())
            .chain(post.tags.copyright.iter())
            .chain(post.tags.meta.iter())
            .chain(post.tags.lore.iter());

        for tag in all_tags {
            if blacklist.contains(&tag.to_lowercase()) {
                return true;
            }
        }

        false
    }

    /// update downloads
    pub async fn redownload_by_artists(&self) -> Result<()> {
        println!("\n=== Update Downloads by Artists ===\n");
        println!(
            "This will scan your downloads, find all artists, and download NEW posts from them."
        );

        let download_path = getopt!(download.path);
        let download_dir = std::path::Path::new(&download_path);

        if !download_dir.exists() {
            bail!(
                "Download directory does not exist: {}",
                download_dir.display()
            );
        }

        println!("Scanning downloaded posts for artist names and post IDs...\n");

        let local_posts = self.scan_downloads_directory(download_dir).await?;

        if local_posts.is_empty() {
            println!("No posts with metadata found in {}", download_dir.display());
            return Ok(());
        }

        let mut artist_post_counts: HashMap<String, usize> = HashMap::new();
        let mut downloaded_post_ids: HashSet<i64> = HashSet::with_capacity(local_posts.len());
        let special_tags: HashSet<&str> = HashSet::from([
            "conditional_dnp",
            "conditional-dnp",
            "sound_warning",
            "sound-warning",
            "epilepsy_warning",
            "epilepsy-warning",
            "animated",
            "comic",
            "unknown_artist",
            "unknown-artist",
            "anonymous_artist",
            "anonymous-artist",
        ]);

        for local_post in &local_posts {
            downloaded_post_ids.insert(local_post.post.id);

            for artist in &local_post.post.tags.artist {
                let artist_lower = artist.to_lowercase();

                if !special_tags.contains(artist_lower.as_str()) {
                    *artist_post_counts.entry(artist.clone()).or_insert(0) += 1;
                }
            }
        }

        if artist_post_counts.is_empty() {
            println!("No artist tags found in downloaded posts.");
            return Ok(());
        }

        println!(
            "{} Found {} downloaded posts from {} unique artists",
            "✓".green().bold(),
            downloaded_post_ids.len(),
            artist_post_counts.len()
        );

        let mut sorted_artists: Vec<_> = artist_post_counts.iter().collect();
        sorted_artists.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));

        let artist_options: Vec<String> = sorted_artists
            .iter()
            .map(|(artist, count)| {
                format!(
                    "{} ({} downloaded post{})",
                    artist,
                    count,
                    if **count == 1 { "" } else { "s" }
                )
            })
            .collect();

        let selected_artists = MultiSelect::new(
            "Select artists to check for new posts (Space to toggle, Enter to confirm):",
            artist_options.clone(),
        )
        .with_help_message(
            "All artists are selected by default. Deselect any you don't want to update.",
        )
        .with_default(&(0..artist_options.len()).collect::<Vec<_>>())
        .prompt()
        .context("Failed to get artist selection")?;

        if selected_artists.is_empty() {
            println!("No artists selected. Operation cancelled.");
            return Ok(());
        }

        println!(
            "\n{} Selected {} artist{} to check for updates:",
            "→".bright_cyan(),
            selected_artists.len(),
            if selected_artists.len() == 1 { "" } else { "s" }
        );

        for (i, artist) in selected_artists.iter().enumerate() {
            let count = artist_post_counts.get(artist).unwrap_or(&0);
            if i < 20 {
                println!(
                    "  • {} ({} already downloaded)",
                    artist.bright_white(),
                    count,
                );
            } else if i == 20 {
                println!("  ... and {} more", selected_artists.len() - 20);
                break;
            }
        }

        let blacklist: HashSet<String> = get_blacklist()
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.to_lowercase())
            .collect();

        if !blacklist.is_empty() {
            println!(
                "\n{} Blacklist active: {} tag{} will be filtered",
                "⚠".yellow().bold(),
                blacklist.len(),
                if blacklist.len() == 1 { "" } else { "s" }
            );
        }

        println!();
        let confirm = Confirm::new(&format!(
            "Check for and download new posts from these {} artist{}?",
            selected_artists.len(),
            if selected_artists.len() == 1 { "" } else { "s" }
        ))
        .prompt()?;

        if !confirm {
            println!("Operation cancelled.");
            return Ok(());
        }

        let limit_per_artist =
            Text::new("Maximum NEW posts per artist to download (leave empty for all):")
                .with_placeholder("e.g., 50")
                .prompt_skippable()?;

        let limit: Option<u64> = if let Some(input) = limit_per_artist {
            if input.is_empty() {
                None
            } else {
                Some(input.parse()?)
            }
        } else {
            None
        };

        println!("\n{} Checking for new posts...\n", "→".bright_cyan());

        let concurrent_artists = getopt!(search.fetch_threads).clamp(1, 4);
        let semaphore = Arc::new(Semaphore::new(concurrent_artists));
        let client = self.client.clone();
        let downloader = self.downloader.clone();
        let downloaded_ids = Arc::new(downloaded_post_ids);
        let blacklist = Arc::new(blacklist);
        let progress_manager = Arc::new(ProgressManager::new());
        let total_pb = progress_manager
            .create_count_bar(
                "artists",
                selected_artists.len() as u64,
                "Processing artists",
            )
            .await?;

        type ArtistResult = Result<(u64, u64, u64), String>; // (new, skipped, blacklisted)
        let mut handles = Vec::with_capacity(selected_artists.len());

        for artist in selected_artists {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let client = client.clone();
            let downloader = downloader.clone();
            let downloaded_ids = downloaded_ids.clone();
            let blacklist = blacklist.clone();
            let total_pb = total_pb.clone();

            let handle = tokio::spawn(async move {
                let result = Self::download_new_artist_posts(
                    &client,
                    &downloader,
                    &artist,
                    limit,
                    &downloaded_ids,
                    &blacklist,
                )
                .await;

                total_pb.inc(1);
                drop(permit);

                match result {
                    Ok((new, skipped, blacklisted)) => (artist, Ok((new, skipped, blacklisted))),
                    Err(e) => (artist, Err(e.to_string())),
                }
            });

            handles.push(handle);
        }

        let results = futures_util::future::join_all(handles).await;
        total_pb.finish_with_message("✓ Processing complete");

        let mut total_new_posts = 0u64;
        let mut total_already_downloaded = 0u64;
        let mut total_blacklisted = 0u64;
        let mut total_errors = 0u64;
        let mut artist_results: Vec<(String, ArtistResult)> = Vec::new();

        for result in results {
            match result {
                Ok((artist, Ok((new_count, skipped_count, blacklisted_count)))) => {
                    total_new_posts += new_count;
                    total_already_downloaded += skipped_count;
                    total_blacklisted += blacklisted_count;
                    artist_results
                        .push((artist, Ok((new_count, skipped_count, blacklisted_count))));
                }
                Ok((artist, Err(error_msg))) => {
                    total_errors += 1;
                    artist_results.push((artist.clone(), Err(error_msg.clone())));
                    warn!("Failed to check posts from {}: {}", artist, error_msg);
                }
                Err(e) => {
                    total_errors += 1;
                    warn!("Task join error: {}", e);
                }
            }
        }

        println!("\n{}", "=".repeat(70));
        println!("Update Summary:");
        println!("{}", "=".repeat(70));
        println!("Artists checked: {}", artist_results.len());
        println!(
            "{} NEW posts downloaded: {}",
            "✓".green().bold(),
            total_new_posts.to_string().bright_green()
        );
        println!(
            "{} Posts already downloaded: {}",
            "→".bright_black(),
            total_already_downloaded
        );

        if total_blacklisted > 0 {
            println!(
                "{} Posts filtered by blacklist: {}",
                "⚠".yellow().bold(),
                total_blacklisted.to_string().yellow()
            );
        }

        if total_errors > 0 {
            println!("{} Errors encountered: {}", "✗".red().bold(), total_errors);
            println!("\n{} Failed artists:", "✗".red().bold());
            for (artist, result) in &artist_results {
                if let Err(error) = result {
                    println!("  • {}: {}", artist.red(), error);
                }
            }
        }

        let mut with_new_posts: Vec<_> = artist_results
            .iter()
            .filter_map(|(artist, result)| {
                if let Ok((new_count, skipped, blacklisted)) = result {
                    if *new_count > 0 {
                        Some((artist, new_count, skipped, blacklisted))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        if !with_new_posts.is_empty() {
            with_new_posts.sort_by(|a, b| b.1.cmp(a.1));

            println!("\n{} Artists with new posts:", "✓".green().bold());
            for (i, (artist, new_count, skipped, blacklisted)) in with_new_posts.iter().enumerate()
            {
                if i < 15 {
                    let mut extra = format!("{} already had", skipped);
                    if **blacklisted > 0 {
                        extra.push_str(&format!(", {} filtered", blacklisted));
                    }
                    println!(
                        "  • {}: {} new post{} ({})",
                        artist.green(),
                        new_count.to_string().bright_green().bold(),
                        if **new_count == 1 { "" } else { "s" },
                        extra
                    );
                } else if i == 15 {
                    println!("  ... and {} more", with_new_posts.len() - 15);
                    break;
                }
            }
        }

        let no_new_posts: Vec<_> = artist_results
            .iter()
            .filter_map(|(artist, result)| {
                if let Ok((new_count, skipped, _)) = result {
                    if *new_count == 0 {
                        Some((artist, skipped))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        if !no_new_posts.is_empty() {
            println!("\n{} Artists with no new posts:", "→".bright_black());
            for (i, (artist, skipped)) in no_new_posts.iter().enumerate() {
                if i < 10 {
                    println!(
                        "  • {}: {} already downloaded",
                        artist.bright_black(),
                        skipped
                    );
                } else if i == 10 {
                    println!("  ... and {} more", no_new_posts.len() - 10);
                    break;
                }
            }
        }

        println!("{}", "=".repeat(70));

        if total_new_posts > 0 {
            println!(
                "\n{} Successfully downloaded {} new post{} from {} artist{}!",
                "✓".green().bold(),
                total_new_posts.to_string().bright_green().bold(),
                if total_new_posts == 1 { "" } else { "s" },
                with_new_posts.len(),
                if with_new_posts.len() == 1 { "" } else { "s" }
            );
        } else {
            println!(
                "\n{} All downloads are up to date! No new posts found.",
                "✓".green().bold()
            );
        }

        Ok(())
    }

    /// download new artist posts based on already downloaded posts
    ///
    /// # Arguments
    ///
    /// * `client` - an e621 api client
    /// * `downloader` - a post downloader
    /// * `artist` - the artist to download
    /// * `limit` - the max amount of new posts to download from the artist
    /// * `downloaded_post_ids` - a list of already downloaded ids from the artist
    /// * `blacklist` - the current loaded blacklist
    pub async fn download_new_artist_posts(
        client: &Arc<E6Client>,
        downloader: &Arc<PostDownloader>,
        artist: &str,
        limit: Option<u64>,
        downloaded_post_ids: &HashSet<i64>,
        blacklist: &HashSet<String>,
    ) -> Result<(u64, u64, u64)> {
        let search_tags = vec![format!("~{}", artist), format!("~{}_(artist)", artist)];
        let mut new_posts: Vec<E6Post> = Vec::new();
        let mut skipped_count = 0u64;
        let mut blacklisted_count = 0u64;
        let mut before_id: Option<i64> = None;
        let max_fetch = limit.unwrap_or(u64::MAX);

        const BATCH_SIZE: u64 = 320;
        const MAX_CONSECUTIVE_EMPTY: u32 = 2;

        let mut consecutive_empty = 0u32;
        let rate_limit_delay = Duration::from_millis(250);
        let mut last_request = std::time::Instant::now();

        loop {
            let elapsed = last_request.elapsed();
            if elapsed < rate_limit_delay {
                tokio::time::sleep(rate_limit_delay - elapsed).await;
            }

            let results = client
                .search_posts(&search_tags, Some(BATCH_SIZE), before_id)
                .await?;

            last_request = std::time::Instant::now();

            if results.posts.is_empty() {
                break;
            }

            let batch_size = results.posts.len();
            let mut found_new_in_batch = false;
            let mut min_id_in_batch: Option<i64> = None;

            for post in results.posts {
                if min_id_in_batch.is_none() || post.id < min_id_in_batch.unwrap() {
                    min_id_in_batch = Some(post.id);
                }

                if downloaded_post_ids.contains(&post.id) {
                    skipped_count += 1;
                    continue;
                }

                if Self::post_contains_blacklisted_tags(&post, blacklist) {
                    blacklisted_count += 1;
                    continue;
                }

                new_posts.push(post);
                found_new_in_batch = true;

                if new_posts.len() >= max_fetch as usize {
                    break;
                }
            }

            if let Some(min_id) = min_id_in_batch {
                before_id = Some(min_id);
            }

            if !found_new_in_batch {
                consecutive_empty += 1;
                if consecutive_empty >= MAX_CONSECUTIVE_EMPTY {
                    break;
                }
            } else {
                consecutive_empty = 0;
            }

            if new_posts.len() >= max_fetch as usize || batch_size < BATCH_SIZE as usize {
                break;
            }
        }

        if let Some(lim) = limit {
            new_posts.truncate(lim as usize);
        }

        let new_count = new_posts.len() as u64;

        if !new_posts.is_empty() {
            downloader.clone().download_posts(new_posts).await?;
        }

        Ok((new_count, skipped_count, blacklisted_count))
    }
}

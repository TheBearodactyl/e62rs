use std::sync::Arc;

use crate::config::get_config;
use crate::models::{E6Pool, PoolEntry};
use crate::pool_db::PoolDatabase;
use crate::tag_db::TagDatabase;
use crate::ui::download::PostDownloader;
use crate::ui::view::{fetch_and_display_image_as_sixel, fetch_and_display_images_as_sixel};
use crate::{client::E6Client, formatting::format_text, models::E6Post};
use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use inquire::{Confirm, MultiSelect, Select, Text};

pub mod download;
pub mod view;

#[derive(Default, Clone)]
pub struct E6Ui {
    client: Arc<E6Client>,
    downloader: Arc<PostDownloader>,
    tag_db: Arc<TagDatabase>,
    pool_db: Arc<PoolDatabase>,
}

#[derive(inquiry::Choice, Clone, Copy, PartialEq, PartialOrd, Debug)]
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

#[derive(inquiry::Choice, Clone, Copy, PartialEq, PartialOrd, Debug)]
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

#[derive(inquiry::Choice, Clone, Copy, PartialEq, PartialOrd, Debug)]
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

impl E6Ui {
    pub fn new(
        client: Arc<E6Client>,
        tag_db: Arc<TagDatabase>,
        pool_db: Arc<PoolDatabase>,
    ) -> Self {
        let settings = get_config().expect("Failed to get settings");
        let downloader = Arc::new(PostDownloader::with_download_dir_and_format(
            settings.download_dir.clone().unwrap(),
            settings.output_format.clone(),
        ));

        if std::fs::exists(
            settings
                .download_dir
                .clone()
                .unwrap_or("output".to_string()),
        )
        .is_err()
        {
            std::fs::create_dir_all(
                settings
                    .download_dir
                    .clone()
                    .unwrap_or("output".to_string()),
            )
            .expect("Failed to create output directory");
        }

        Self {
            client,
            downloader,
            tag_db,
            pool_db,
        }
    }

    pub async fn search_pools(&self) -> Result<()> {
        loop {
            match self.perform_pool_search().await {
                Ok(should_continue) => {
                    if !should_continue {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Pool search error: {}", e);
                    if !self.ask_retry()? {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    async fn perform_pool_search(&self) -> Result<bool> {
        let query = self.get_pool_search_query()?;
        let limit = self.get_pool_limit()? as usize;

        let pools = if query.is_empty() {
            let local_pools: Vec<PoolEntry> =
                unsafe { self.pool_db.iter_pools().take(limit).cloned().collect() };

            local_pools
                .iter()
                .map(|entry| self.pool_entry_to_e6pool(entry))
                .collect()
        } else {
            let local_matches = self.pool_db.search(&query, limit);
            if !local_matches.is_empty() {
                local_matches
                    .iter()
                    .filter_map(|name| self.pool_db.get_by_name(name))
                    .map(|entry| self.pool_entry_to_e6pool(&entry))
                    .collect()
            } else {
                let results = self
                    .client
                    .search_pools(query, Some(limit as u64))
                    .await
                    .context("Failed to search pools")?;
                results.pools
            }
        };

        if pools.is_empty() {
            println!("No pools found matching your search criteria.");
            return self.ask_retry();
        }

        let selected_pool = self.select_pool(&pools)?;

        if let Some(pool) = selected_pool {
            self.display_pool(pool);
            match self.pool_interaction_menu(pool.clone()).await? {
                PoolInteractionMenu::Back => Ok(true),
                _ => Ok(self.ask_retry()?),
            }
        } else {
            Ok(false)
        }
    }

    pub async fn search_pools_advanced(&self) -> Result<()> {
        loop {
            let search_type = Select::new(
                "How would you like to search for pools?",
                vec![
                    "By name",
                    "By description",
                    "By creator",
                    "Browse latest pools",
                    "Back to main menu",
                ],
            )
            .prompt()?;

            match search_type {
                "By name" => {
                    if let Err(e) = self.perform_pool_search().await {
                        eprintln!("Pool name search error: {}", e);
                    }
                }
                "By description" => {
                    if let Err(e) = self.perform_pool_description_search().await {
                        eprintln!("Pool description search error: {}", e);
                    }
                }
                "By creator" => {
                    if let Err(e) = self.perform_pool_creator_search().await {
                        eprintln!("Pool creator search error: {}", e);
                    }
                }
                "Browse latest pools" => {
                    if let Err(e) = self.browse_latest_pools().await {
                        eprintln!("Error browsing pools: {}", e);
                    }
                }
                "Back to main menu" => break,
                _ => {}
            }

            if !self.ask_retry()? {
                break;
            }
        }
        Ok(())
    }

    async fn perform_pool_description_search(&self) -> Result<()> {
        let query = Text::new("Enter pool description search:")
            .prompt()
            .context("Failed to get pool description query")?;

        let limit = self.get_pool_limit()?;
        let results = self
            .client
            .search_pools_by_description(query, Some(limit))
            .await
            .context("Failed to search pools by description")?;

        if results.pools.is_empty() {
            println!("No pools found matching your description search.");
            return Ok(());
        }

        let selected_pool = self.select_pool(&results.pools)?;
        if let Some(pool) = selected_pool {
            let fetched_pool = self.client.get_pool_by_id(pool.id).await?;
            self.display_pool(&fetched_pool.pool);
            self.pool_interaction_menu(fetched_pool.pool).await?;
        }

        Ok(())
    }

    async fn perform_pool_creator_search(&self) -> Result<()> {
        let creator = Text::new("Enter creator name:")
            .prompt()
            .context("Failed to get creator name")?;

        let limit = self.get_pool_limit()?;
        let results = self
            .client
            .search_pools_by_creator(creator, Some(limit))
            .await
            .context("Failed to search pools by creator")?;

        if results.pools.is_empty() {
            println!("No pools found by that creator.");
            return Ok(());
        }

        let selected_pool = self.select_pool(&results.pools)?;
        if let Some(pool) = selected_pool {
            let fetched_pool = self.client.get_pool_by_id(pool.id).await?;
            self.display_pool(&fetched_pool.pool);
            self.pool_interaction_menu(fetched_pool.pool).await?;
        }

        Ok(())
    }

    async fn browse_latest_pools(&self) -> Result<()> {
        let limit = self.get_pool_limit()?;
        let results = self
            .client
            .get_pools(Some(limit))
            .await
            .context("Failed to fetch latest pools")?;

        if results.pools.is_empty() {
            println!("No pools found.");
            return Ok(());
        }

        let selected_pool = self.select_pool(&results.pools)?;
        if let Some(pool) = selected_pool {
            let fetched_pool = self.client.get_pool_by_id(pool.id).await?;
            self.display_pool(&fetched_pool.pool);
            self.pool_interaction_menu(fetched_pool.pool).await?;
        }

        Ok(())
    }

    fn get_pool_search_query(&self) -> Result<String> {
        let query = Text::new("Enter pool search query (leave empty for latest pools):")
            .with_autocomplete(move |input: &str| {
                let suggestions = self.pool_db.autocomplete(input, 5);
                Ok(suggestions)
            })
            .prompt()
            .context("Failed to get pool search query")?;

        Ok(query.trim().to_string())
    }

    fn get_pool_limit(&self) -> Result<u64> {
        let settings = get_config()?;
        let default_limit = settings.post_count.unwrap_or(32);

        let prompt = inquire::CustomType::<u64>::new("How many pools to return?")
            .with_default(default_limit)
            .with_error_message("Please enter a valid number")
            .prompt_skippable()
            .context("Failed to get pool limit")?;

        Ok(prompt.unwrap_or(default_limit).min(100))
    }

    fn select_pool<'a>(&self, pools: &'a [E6Pool]) -> Result<Option<&'a E6Pool>> {
        let options: Vec<String> = pools
            .iter()
            .map(|pool| {
                format!(
                    "ID: {} | {} | {} posts | {}",
                    pool.id,
                    self.truncate_string(&pool.name, 40),
                    pool.post_ids.len(),
                    pool.category
                )
            })
            .collect();

        let selection = Select::new("Select a pool to view:", options)
            .with_help_message("Use arrow keys to navigate, Enter to select, Esc to cancel")
            .prompt_skippable()
            .context("Failed to get pool selection")?;

        Ok(selection.and_then(|s| {
            let index = pools.iter().position(|p| {
                format!(
                    "ID: {} | {} | {} posts | {}",
                    p.id,
                    self.truncate_string(&p.name, 40),
                    p.post_ids.len(),
                    p.category
                ) == s
            });
            index.map(|i| &pools[i])
        }))
    }

    fn pool_entry_to_e6pool(&self, entry: &crate::models::PoolEntry) -> E6Pool {
        E6Pool {
            id: entry.id,
            name: entry.name.clone(),
            created_at: entry.created_at.clone(),
            updated_at: entry.updated_at.clone(),
            creator_id: entry.creator_id,
            creator_name: self.pool_db.get_creator_name(entry.creator_id),
            description: entry.description.clone(),
            is_active: entry.is_active,
            category: entry.category.clone(),
            post_ids: entry.post_ids.clone(),
            post_count: entry.post_ids.len() as i64,
        }
    }

    fn display_pool(&self, pool: &E6Pool) {
        println!("\n{}", "=".repeat(70));
        println!("Pool: {}", pool.name);
        println!("ID: {}", pool.id);
        println!("Posts: {}", pool.post_count);
        println!("Creator: {}", pool.creator_name);
        println!("Category: {}", pool.category);
        println!("Active: {}", if pool.is_active { "Yes" } else { "No" });

        if !pool.description.is_empty() {
            println!("Description: {}", format_text(&pool.description));
        }

        println!("{}", "=".repeat(70));
    }

    async fn pool_interaction_menu(&self, pool: E6Pool) -> Result<PoolInteractionMenu> {
        let choice = PoolInteractionMenu::choice("What would you like to do with this pool?")
            .context("Failed to get pool interaction choice")?;

        match choice {
            PoolInteractionMenu::ViewPosts => {
                let posts = self.client.get_pool_posts(pool.id).await?;
                if posts.posts.is_empty() {
                    println!("No posts found in this pool.");
                } else {
                    self.display_posts(&posts.posts);

                    let interact = Confirm::new("Would you like to interact with these posts?")
                        .with_default(false)
                        .prompt()?;

                    if interact {
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
            PoolInteractionMenu::DownloadPool => {
                let posts = self.client.get_pool_posts(pool.id).await?;
                if posts.posts.is_empty() {
                    println!("No posts found in this pool.");
                } else {
                    println!(
                        "Downloading {} posts from pool '{}'...",
                        posts.posts.len(),
                        pool.name
                    );
                    self.download_posts(posts.posts).await?;
                }
            }
            PoolInteractionMenu::OpenInBrowser => {
                let url = format!("https://e621.net/pools/{}", pool.id);
                open::that(&url).context("Failed to open pool in browser")?;
                println!("Opened pool in browser: {}", url);
            }
            PoolInteractionMenu::Back => {}
        }

        Ok(choice)
    }

    pub async fn search_posts(&self) -> Result<()> {
        loop {
            match self.perform_search().await {
                Ok(should_continue) => {
                    if !should_continue {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Search error: {}", e);
                    if !self.ask_retry()? {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    async fn perform_search(&self) -> Result<bool> {
        let tags = self.collect_tags()?;
        let limit = self.get_post_limit()?;

        let results = self
            .client
            .search_posts(tags, Some(limit))
            .await
            .context("Failed to search posts")?;

        if results.posts.is_empty() {
            println!("No posts found matching your search criteria.");
            return self.ask_retry();
        }

        let use_multi_select = Confirm::new("Select multiple posts?")
            .with_default(false)
            .prompt()
            .context("Failed to get multi-select preference")?;

        if use_multi_select {
            let selected_posts = self.select_multiple_posts(&results.posts)?;

            if !selected_posts.is_empty() {
                println!("\nSelected {} posts", selected_posts.len());

                let mut fetched_posts = Vec::new();
                for post in &selected_posts {
                    let fetched = self.client.get_post_by_id(post.id).await?;
                    fetched_posts.push(fetched.post);
                }

                match self.batch_interaction_menu(fetched_posts).await? {
                    BatchAction::Back => Ok(true),
                    _ => Ok(self.ask_retry()?),
                }
            } else {
                Ok(self.ask_retry()?)
            }
        } else {
            let selected_post = self.select_post(&results.posts)?;

            if let Some(post) = selected_post {
                let fetched_post = self.client.get_post_by_id(post.id).await?;
                self.display_post(&fetched_post.post);

                match self.interaction_menu(fetched_post.post).await? {
                    InteractionMenu::Back => Ok(true),
                    _ => Ok(self.ask_retry()?),
                }
            } else {
                Ok(false)
            }
        }
    }

    fn collect_tags(&self) -> Result<Vec<String>> {
        let mut tags = Vec::new();

        loop {
            let prompt = if tags.is_empty() {
                "Would you like to add a tag to the search?"
            } else {
                println!("Current tags: {}", tags.join(" "));
                "Would you like to add another tag?"
            };

            let add_tag = Confirm::new(prompt)
                .with_default(false)
                .prompt()
                .context("Failed to get user confirmation")?;

            if !add_tag {
                break;
            }

            let tag = Text::new("Enter tag:")
                .with_autocomplete(move |input: &str| {
                    let suggestions = self.tag_db.autocomplete(input, 5);
                    Ok(suggestions)
                })
                .prompt()
                .context("Failed to get tag input")?;

            if !tag.trim().is_empty() {
                let tag = tag.trim().to_string();
                if self.tag_db.exists(&tag) {
                    tags.push(tag);
                } else {
                    let suggestions = self.tag_db.search(&tag, 5);
                    if !suggestions.is_empty() {
                        let msg = format!("Tag '{}' not found. Did you mean:", tag);
                        let selected = Select::new(&msg, suggestions)
                            .with_help_message("Select a tag or press ESC to cancel")
                            .prompt_skippable()
                            .context("Failed to get tag selection")?;

                        if let Some(selected_tag) = selected {
                            tags.push(selected_tag);
                        }
                    } else {
                        println!("Tag '{}' not found and no similar tags found.", tag);
                    }
                }
            }
        }

        Ok(tags)
    }

    fn get_post_limit(&self) -> Result<u64> {
        let settings = get_config()?;

        if settings.post_count.unwrap_or(32).eq(&32) {
            let prompt = inquire::CustomType::<u64>::new("How many posts to return?")
                .with_default(32)
                .with_error_message("Please enter a valid number")
                .prompt_skippable()
                .context("Failed to get post limit")?;

            Ok(prompt.unwrap_or(32).min(320))
        } else {
            Ok(settings.post_count.unwrap_or(32))
        }
    }

    fn select_post<'a>(&self, posts: &'a [E6Post]) -> Result<Option<&'a E6Post>> {
        let options: Vec<String> = posts
            .iter()
            .map(|post| {
                format!(
                    "ID: {} | Score: {} | Rating: {}",
                    post.id, post.score.total, post.rating
                )
            })
            .collect();

        let selection = Select::new("Select a post to view:", options)
            .with_help_message("Use arrow keys to navigate, Enter to select, Esc to cancel")
            .prompt_skippable()
            .context("Failed to get post selection")?;

        Ok(selection.and_then(|s| {
            let index = posts.iter().position(|p| {
                format!(
                    "ID: {} | Score: {} | Rating: {}",
                    p.id, p.score.total, p.rating
                ) == s
            });
            index.map(|i| &posts[i])
        }))
    }

    fn select_multiple_posts<'a>(&self, posts: &'a [E6Post]) -> Result<Vec<&'a E6Post>> {
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
            .filter_map(|s| {
                posts
                    .iter()
                    .position(|p| {
                        format!(
                            "ID: {} | Score: {} | Rating: {}",
                            p.id, p.score.total, p.rating
                        ) == *s
                    })
                    .map(|i| &posts[i])
            })
            .collect();

        Ok(selected_posts)
    }

    fn display_post(&self, post: &E6Post) {
        println!("\n{}", "=".repeat(50));
        println!("Post ID: {}", post.id);
        println!("Rating: {}", post.rating);
        println!(
            "Score: ↑{} ↓{} = {}",
            post.score.up, post.score.down, post.score.total
        );
        println!("Tags: {}", post.tags.general[0..=3].join(", "));
        println!("Favorites: {}", post.fav_count);
        println!("Uploaded by: {}", post.uploader_name);

        if !post.tags.artist.is_empty() {
            println!("Artists: {}", post.tags.artist.join(", "));
        }

        if !post.description.is_empty() {
            println!("Description: {}", format_text(&post.description));
        }

        println!("{}", "=".repeat(50));
    }

    async fn interaction_menu(&self, post: E6Post) -> Result<InteractionMenu> {
        let choice = InteractionMenu::choice("What would you like to do?")
            .context("Failed to get interaction choice")?;

        match choice {
            InteractionMenu::OpenInBrowser => {
                self.open_in_browser(&post)?;
            }
            InteractionMenu::Download => {
                self.downloader.download_post(post).await?;
            }
            InteractionMenu::Back => {}
            InteractionMenu::View => {
                let post_url = post.file.url.unwrap();
                fetch_and_display_image_as_sixel(&post_url)
                    .await
                    .context("Failed to view image")?;
            }
        }

        Ok(choice)
    }

    async fn batch_interaction_menu(&self, posts: Vec<E6Post>) -> Result<BatchAction> {
        let choice = BatchAction::choice(&format!(
            "What would you like to do with {} selected posts?",
            posts.len()
        ))
        .context("Failed to get batch action choice")?;

        match choice {
            BatchAction::DownloadAll => {
                self.download_posts(posts).await?;
            }
            BatchAction::OpenAllInBrowser => {
                self.open_posts_in_browser(&posts)?;
            }
            BatchAction::DownloadAndOpenAll => {
                let posts_clone = posts.clone();
                self.download_posts(posts).await?;
                self.open_posts_in_browser(&posts_clone)?;
            }
            BatchAction::Back => {}
            BatchAction::ViewAll => {
                let posts_clone = posts.clone();
                fetch_and_display_images_as_sixel(
                    posts_clone
                        .iter()
                        .map(|post| post.file.url.clone().unwrap())
                        .collect::<Vec<String>>()
                        .iter()
                        .map(|st| st.as_str())
                        .collect::<Vec<&str>>(),
                )
                .await?;
            }
        }

        Ok(choice)
    }

    fn open_in_browser(&self, post: &E6Post) -> Result<()> {
        let url = format!("https://e621.net/posts/{}", post.id);
        open::that(&url).context("Failed to open post in browser")?;
        println!("Opened post in browser: {}", url);
        Ok(())
    }

    fn open_posts_in_browser(&self, posts: &[E6Post]) -> Result<()> {
        println!("Opening {} posts in browser...", posts.len());
        for post in posts {
            let url = format!("https://e621.net/posts/{}", post.id);
            open::that(&url).context("Failed to open post in browser")?;
            println!("Opened post {} in browser", post.id);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        println!("✓ Opened all {} posts in browser", posts.len());
        Ok(())
    }

    async fn download_posts(&self, posts: Vec<E6Post>) -> Result<()> {
        println!("Downloading {} posts...", posts.len());
        let total = posts.len();
        let multi_prog = MultiProgress::new();
        let sty = ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-");

        let total_downloaded_pb = multi_prog.add(ProgressBar::new(total as u64));
        total_downloaded_pb.set_style(sty.clone());
        total_downloaded_pb.set_message("Total");

        self.downloader
            .clone()
            .download_posts_concurrent(posts)
            .await?;

        println!("\n✓ Batch download complete");
        Ok(())
    }

    fn ask_retry(&self) -> Result<bool> {
        Confirm::new("Would you like to perform another search?")
            .with_default(true)
            .prompt()
            .context("Failed to get retry confirmation")
    }

    pub async fn display_latest_posts(&self) -> Result<()> {
        let results = self
            .client
            .get_latest_posts()
            .await
            .context("Failed to fetch latest posts")?;

        if results.posts.is_empty() {
            println!("No latest posts found.");
            return Ok(());
        }

        println!("\n📋 Latest Posts:");
        self.display_posts(&results.posts);
        Ok(())
    }

    fn display_posts(&self, posts: &[E6Post]) {
        let posts_per_row = 3;
        let column_width = 28;

        for chunk in posts.chunks(posts_per_row) {
            self.display_posts_row(chunk, column_width);
            println!();
        }
    }

    fn display_posts_row(&self, posts: &[E6Post], column_width: usize) {
        self.print_row_separator(posts.len(), column_width, "┌", "┬", "┐", "─");
        self.print_posts_field(posts, column_width, |post| format!("ID: {}", post.id));
        self.print_row_separator(posts.len(), column_width, "├", "┼", "┤", "─");
        self.print_posts_field(posts, column_width, |post| {
            format!("Rating: {} | Score: {}", post.rating, post.score.total)
        });
        self.print_posts_field(posts, column_width, |post| {
            let uploader = self.truncate_string(&post.uploader_name, 15);
            format!("❤️ {} | By: {}", post.fav_count, uploader)
        });
        self.print_posts_field(posts, column_width, |post| {
            if !post.tags.artist.is_empty() {
                let artists = post.tags.artist.join(", ");
                format!("🎨 {}", self.truncate_string(&artists, column_width - 4))
            } else {
                "🎨 Unknown artist".to_string()
            }
        });

        self.print_posts_field(posts, column_width, |post| {
            let tags = post
                .tags
                .general
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            format!("🏷️ {}", self.truncate_string(&tags, column_width - 4))
        });

        self.print_row_separator(posts.len(), column_width, "└", "┴", "┘", "─");
    }

    fn print_posts_field<F>(&self, posts: &[E6Post], column_width: usize, field_fn: F)
    where
        F: Fn(&E6Post) -> String,
    {
        print!("│");
        for (i, post) in posts.iter().enumerate() {
            let content = field_fn(post);
            let truncated = self.truncate_string(&content, column_width - 2);
            print!(" {:width$} ", truncated, width = column_width - 2);
            print!("│");
            if i < posts.len() - 1 {}
        }
        println!();
    }

    fn print_row_separator(
        &self,
        count: usize,
        column_width: usize,
        left: &str,
        mid: &str,
        right: &str,
        fill: &str,
    ) {
        print!("{}", left);
        for i in 0..count {
            print!("{}", fill.repeat(column_width));
            if i < count - 1 {
                print!("{}", mid);
            }
        }
        print!("{}", right);
        println!();
    }

    fn truncate_string(&self, s: &str, max_width: usize) -> String {
        if s.len() <= max_width {
            format!("{:width$}", s, width = max_width)
        } else {
            format!("{}...", &s[..max_width.saturating_sub(3)])
        }
    }

    #[allow(unused)]
    pub async fn browse_latest_posts(&'static self) -> Result<()> {
        loop {
            match self.perform_latest_posts_browse().await {
                Ok(should_continue) => {
                    if !should_continue {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error browsing latest posts: {}", e);
                    if !self.ask_retry()? {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    #[allow(unused)]
    async fn perform_latest_posts_browse(&'static self) -> Result<bool> {
        let results = self
            .client
            .get_latest_posts()
            .await
            .context("Failed to fetch latest posts")?;

        if results.posts.is_empty() {
            println!("No latest posts found.");
            return self.ask_retry();
        }

        println!("\n📋 Latest Posts:");
        self.display_posts(&results.posts);

        let use_multi_select = Confirm::new("Select posts to interact with?")
            .with_default(true)
            .prompt()
            .context("Failed to get interaction preference")?;

        if use_multi_select {
            let selected_posts = self.select_multiple_posts(&results.posts)?;

            if !selected_posts.is_empty() {
                println!("\nSelected {} posts", selected_posts.len());

                let mut fetched_posts = Vec::new();
                for post in &selected_posts {
                    let fetched = self.client.get_post_by_id(post.id).await?;
                    fetched_posts.push(fetched.post);
                }

                match self.batch_interaction_menu(fetched_posts).await? {
                    BatchAction::Back => Ok(true),
                    _ => Ok(self.ask_retry()?),
                }
            } else {
                Ok(self.ask_retry()?)
            }
        } else {
            Ok(self.ask_retry()?)
        }
    }
}

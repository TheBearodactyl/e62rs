//! search ui stuff
use {
    crate::{
        bail,
        display::dtext::parser::format_text,
        error::{Report, Result},
        getopt,
        models::*,
        ui::{
            E6Ui,
            autocomplete::PoolAutocompleter,
            menus::{AdvPoolSearch, view::ViewMenu},
        },
    },
    bearask::{AskOption, Confirm, ErrorMessage, Select, TextInput, Validation},
    color_eyre::eyre::Context,
    indicatif::{ProgressBar, ProgressStyle},
    std::{collections::HashSet, sync::Arc, time::Duration},
    tracing::{debug, warn},
};

/// functions for searching posts and pools
pub trait SearchMenu {
    /// search for pools in an advanced way
    fn search_pools_adv(&self) -> impl Future<Output = Result<()>>;

    /// search pools by their description
    fn perform_pool_description_search(&self) -> impl Future<Output = Result<()>>;

    /// search pools by creator
    fn perform_pool_creator_search(&self) -> impl Future<Output = Result<()>>;

    /// browse the latest pools
    fn browse_latest_pools(&self) -> impl Future<Output = Result<()>>;

    /// handle the results of a search
    fn handle_pool_results(&self, pools: Vec<E6Pool>) -> impl Future<Output = Result<()>>;

    /// get the search query for finding pools
    fn get_pool_search_query(&self) -> Result<String>;

    /// get the max number of pools to display
    fn get_pool_limit(&self) -> Result<u64>;

    /// select a pool from a list
    fn select_pool<'a>(&self, pools: &'a [E6Pool]) -> Result<Option<&'a E6Pool>>;

    /// convert a PoolEntry to an E6Pool
    fn pool_entry_to_e6pool(&self, entry: &PoolEntry) -> E6Pool;

    /// search posts
    fn search_posts(&self) -> impl Future<Output = Result<()>>;

    /// search pools
    fn search_pools(&self) -> impl Future<Output = Result<()>>;

    /// perform a pool search
    fn perform_pool_search(&self) -> impl Future<Output = Result<bool>>;

    /// perform a post search
    fn perform_search(&self) -> impl Future<Output = Result<bool>>;

    /// perform a paginated post search
    fn fetch_posts_paginated(
        &self,
        all_tags: Vec<String>,
        total_limit: u64,
    ) -> impl Future<Output = Result<Vec<E6Post>>>;

    /// make a search progress bar
    fn create_search_progress_bar(&self, total: u64) -> Result<ProgressBar>;

    /// handle a post interaction
    fn handle_post_interaction(&self, posts: Vec<E6Post>) -> impl Future<Output = Result<bool>>;

    /// fetch posts selected in list of results
    fn fetch_selected_posts(
        &self,
        selected_posts: Vec<&E6Post>,
    ) -> impl Future<Output = Result<Vec<E6Post>>>;

    /// make a progress bar for fetching
    fn create_fetch_progress_bar(&self, total: usize) -> Result<ProgressBar>;

    /// get the limit of posts to return
    fn get_post_limit(&self) -> Result<u64>;

    /// ask whether to continue
    fn ask_continue(&self, message: &str) -> Result<bool>;

    /// select a post from a list of posts
    fn select_post<'a>(&self, posts: &'a [E6Post]) -> Result<Option<&'a E6Post>>;

    /// display a pools info
    fn display_pool(&self, pool: &E6Pool);
}

impl SearchMenu for E6Ui {
    /// search for pools in an advanced way
    async fn search_pools_adv(&self) -> Result<()> {
        loop {
            let search_type = miette::Context::context(
                AdvPoolSearch::select("How would you like to search for pools").ask(),
                "Failed to get search type selection",
            )?;

            let should_break = match search_type.value {
                AdvPoolSearch::ByName => {
                    if let Err(e) = self.perform_pool_search().await {
                        warn!("Pool name search error: {:#}", e);
                        eprintln!("Pool name search error: {}", e);
                    }
                    false
                }
                AdvPoolSearch::ByDesc => {
                    if let Err(e) = self.perform_pool_description_search().await {
                        warn!("Pool description search error: {:#}", e);
                        eprintln!("Pool description search error: {}", e);
                    }
                    false
                }
                AdvPoolSearch::ByCreator => {
                    if let Err(e) = self.perform_pool_creator_search().await {
                        warn!("Pool creator search error: {:#}", e);
                        eprintln!("Pool creator search error: {}", e);
                    }
                    false
                }
                AdvPoolSearch::BrowseLatest => {
                    if let Err(e) = self.browse_latest_pools().await {
                        warn!("Error browsing pools: {:#}", e);
                        eprintln!("Error browsing pools: {}", e);
                    }
                    false
                }
                AdvPoolSearch::Back => true,
            };

            if should_break {
                break;
            }

            if !self.ask_continue("Would you like to perform another search?")? {
                break;
            }
        }

        Ok(())
    }

    /// search pools by their description
    async fn perform_pool_description_search(&self) -> Result<()> {
        let query = miette::Context::context(
            TextInput::new("Enter pool description search:").ask(),
            "Failed to get description search input",
        )?;

        let query = query.trim();
        if query.is_empty() {
            println!("Search query cannot be empty.");
            return Ok(());
        }

        let limit = self.get_pool_limit()?;
        let results = self
            .client
            .search_pools_by_description(query, Some(limit))
            .await
            .context("Failed to search pools by description")?;

        self.handle_pool_results(results.pools).await
    }

    /// search pools by creator
    async fn perform_pool_creator_search(&self) -> Result<()> {
        let creator = miette::Context::context(
            TextInput::new("Enter creator name:").ask(),
            "Failed to get creator name input",
        )?;

        let creator = creator.trim();
        if creator.is_empty() {
            println!("Creator name cannot be empty.");
            return Ok(());
        }

        let limit = self.get_pool_limit()?;
        let results = self
            .client
            .search_pools_by_creator(creator, Some(limit))
            .await
            .context("Failed to search pools by creator")?;

        self.handle_pool_results(results.pools).await
    }

    /// browse the latest pools
    async fn browse_latest_pools(&self) -> Result<()> {
        let limit = self.get_pool_limit()?;
        let results = self
            .client
            .get_pools(Some(limit))
            .await
            .context("Failed to fetch latest pools")?;

        self.handle_pool_results(results.pools).await
    }

    /// handle the results of a search
    async fn handle_pool_results(&self, pools: Vec<E6Pool>) -> Result<()> {
        if pools.is_empty() {
            println!("No pools found matching your search criteria.");
            return Ok(());
        }

        let selected_pool = self.select_pool(&pools)?;
        if let Some(pool) = selected_pool {
            let fetched_pool = self
                .client
                .get_pool_by_id(pool.id)
                .await
                .context("Failed to fetch pool details")?;

            self.display_pool(&fetched_pool.pool);
            self.pool_interaction_menu(fetched_pool.pool).await?;
        }

        Ok(())
    }

    /// get the search query for finding pools
    fn get_pool_search_query(&self) -> Result<String> {
        let autocompleter = PoolAutocompleter::new(self.pool_db.clone());
        let query = miette::Context::context(
            TextInput::new("Enter pool search query (leave empty for latest pools):")
                .with_autocomplete(autocompleter)
                .ask(),
            "Failed to get pool search query",
        )?;

        Ok(query.trim().to_string())
    }

    /// get the max number of pools to display
    fn get_pool_limit(&self) -> Result<u64> {
        let default_limit = getopt!(search.results).min(getopt!(search.results));

        let input = miette::Context::context(
            TextInput::new("How many pools to return?")
                .with_validation(|input: &str| {
                    let err_msg = "Please enter a number between 1 and 100";

                    if input.trim().is_empty() {
                        return Ok(Validation::Valid);
                    }

                    match input.parse::<u64>() {
                        Ok(n) if n > 0 && n <= 100 => Ok(Validation::Valid),
                        Ok(_) => Ok(Validation::Invalid(ErrorMessage::Custom(
                            err_msg.to_string(),
                        ))),
                        Err(_) => Ok(Validation::Invalid(ErrorMessage::Custom(
                            "Please enter a valid number".to_string(),
                        ))),
                    }
                })
                .with_placeholder(default_limit.to_string())
                .ask(),
            "Failed to get pool limit input",
        )?;

        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(default_limit);
        }

        let limit = trimmed
            .parse::<u64>()
            .context("Failed to parse pool limit")?;

        Ok(limit.clamp(1, 100))
    }

    /// select a pool from a list
    fn select_pool<'a>(&self, pools: &'a [E6Pool]) -> Result<Option<&'a E6Pool>> {
        if pools.is_empty() {
            return Ok(None);
        }

        let options = pools
            .iter()
            .map(|pool| AskOption::with_name(pool.name.clone(), pool))
            .collect();

        let selection = match miette::Context::context(
            Select::new("Select a pool to view:")
                .with_options(options)
                .ask(),
            "Failed to get pool selection",
        ) {
            Ok(pool) => Some(pool.value),
            Err(e) => return Err(e.into()),
        };

        Ok(selection)
    }

    /// convert a PoolEntry to an E6Pool
    fn pool_entry_to_e6pool(&self, entry: &PoolEntry) -> E6Pool {
        E6Pool {
            id: entry.id,
            name: entry.name.clone(),
            created_at: entry.created_at.clone(),
            updated_at: entry.updated_at.clone(),
            creator_id: entry.creator_id,
            creator_name: entry.creator_id.to_string(),
            description: entry.description.clone(),
            is_active: entry.is_active,
            category: entry.category.clone(),
            post_ids: entry.post_ids.clone(),
            post_count: entry.post_ids.len() as i64,
        }
    }

    /// search posts
    async fn search_posts(&self) -> Result<()> {
        loop {
            match self.perform_search().await {
                Ok(should_continue) => {
                    if !should_continue {
                        break;
                    }
                }
                Err(e) => {
                    warn!("Search error: {:#}", e);
                    eprintln!("Search error: {}", e);
                    if !self.ask_continue("An error occurred. Would you like to try again?")? {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    /// search pools
    async fn search_pools(&self) -> Result<()> {
        loop {
            match self.perform_pool_search().await {
                Ok(should_continue) => {
                    if !should_continue {
                        break;
                    }
                }
                Err(e) => {
                    warn!("Pool search error: {:#}", e);
                    eprintln!("Pool search error: {}", e);
                    if !self.ask_continue("An error occurred. Would you like to try again?")? {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    /// perform a pool search
    async fn perform_pool_search(&self) -> Result<bool> {
        let query = self.get_pool_search_query()?;
        let limit = self.get_pool_limit()? as usize;

        let pools = if query.is_empty() {
            let local_pools: Vec<PoolEntry> =
                self.pool_db.iter_pools().take(limit).cloned().collect();

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
                    .map(|entry| self.pool_entry_to_e6pool(entry))
                    .collect()
            } else {
                let results = self
                    .client
                    .search_pools(query.as_str(), Some(limit as u64))
                    .await
                    .context("Failed to search pools via API")?;
                results.pools
            }
        };

        if pools.is_empty() {
            println!("No pools found matching your search criteria.");
            return self.ask_continue("Would you like to perform another search?");
        }

        let selected_pool = self.select_pool(&pools)?;

        if let Some(pool) = selected_pool {
            self.display_pool(pool);
            self.pool_interaction_menu(pool.clone()).await?;
        }

        Ok(true)
    }

    /// perform a post search
    async fn perform_search(&self) -> Result<bool> {
        let (include_tags, or_tags, exclude_tags) = self.collect_tags()?;
        let total_limit = self.get_post_limit()?;
        if include_tags.is_empty() && or_tags.is_empty() && exclude_tags.is_empty() {
            println!("Please specify at least one search tag.");
            return Ok(true);
        }

        let mut all_tags =
            Vec::with_capacity(include_tags.len() + or_tags.len() + exclude_tags.len());

        all_tags.extend(include_tags);
        all_tags.extend(exclude_tags.into_iter().map(|tag| format!("-{}", tag)));
        all_tags.extend(or_tags.into_iter().map(|tag| format!("~{}", tag)));

        debug!("Searching with tags: {:?}", all_tags);

        let posts = self.fetch_posts_paginated(all_tags, total_limit).await?;

        if posts.is_empty() {
            println!("No posts found matching your search criteria.");
            return self.ask_continue("Would you like to perform another search?");
        }

        self.handle_post_interaction(posts).await
    }

    /// perform a paginated post search
    async fn fetch_posts_paginated(
        &self,
        all_tags: Vec<String>,
        total_limit: u64,
    ) -> Result<Vec<E6Post>> {
        let mut all_fetched_posts: Vec<E6Post> = Vec::new();
        let mut before_id: Option<i64> = None;
        let mut seen = HashSet::new();
        let mut consecutive_empty_batches = 0;

        println!("Fetching up to {} posts...", total_limit);
        let pb = self.create_search_progress_bar(total_limit)?;

        while (all_fetched_posts.len() as u64) < total_limit && consecutive_empty_batches < 3 {
            let remaining = total_limit.saturating_sub(all_fetched_posts.len() as u64);
            let current_limit = (remaining * 2).min(getopt!(search.results)).max(20);

            pb.set_message(format!(
                "fetching batch {} ({} total so far)",
                (all_fetched_posts.len() / getopt!(search.results) as usize) + 1,
                all_fetched_posts.len()
            ));

            let results = self
                .client
                .search_posts(&all_tags.clone(), Some(current_limit), before_id)
                .await
                .context("Failed to search posts")?;

            let batch_size_before_filtering = results.posts.len();

            if results.posts.is_empty() {
                consecutive_empty_batches += 1;
                pb.println("no more posts available from api.");
                if consecutive_empty_batches >= 3 {
                    pb.println("reached maximum consecutive empty batches, stopping.");
                    break;
                }
                continue;
            }

            consecutive_empty_batches = 0;

            let mut new_posts = Vec::with_capacity(results.posts.len());
            let mut min_id: Option<i64> = None;

            for post in results.posts {
                if seen.insert(post.id) {
                    if min_id.is_none() || post.id < min_id.unwrap_or(3) {
                        min_id = Some(post.id);
                    }
                    new_posts.push(post);
                }
            }

            let fetched_count = new_posts.len();

            if let Some(id) = min_id {
                before_id = Some(id);
            }

            all_fetched_posts.extend(new_posts);
            pb.inc(fetched_count as u64);

            let filtered = batch_size_before_filtering.saturating_sub(fetched_count);
            if filtered > 0 {
                pb.println(format!(
                    "filtered out {} blacklisted/duplicate posts from this batch",
                    filtered
                ));
            }

            if fetched_count < 320 {
                debug!("small batch received, approaching end of results");
                pb.println("approaching end of available results.");
            }

            if all_fetched_posts.len() >= total_limit as usize {
                break;
            }
        }

        all_fetched_posts.truncate(total_limit as usize);

        pb.set_length(all_fetched_posts.len() as u64);
        pb.finish_with_message(format!("✓ Fetched {} posts.", all_fetched_posts.len()));

        Ok(all_fetched_posts)
    }

    /// make a search progress bar
    fn create_search_progress_bar(&self, total: u64) -> Result<ProgressBar> {
        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.bright_cyan} [{elapsed_precise}] [{bar:40.bright_cyan/blue}] \
                 {pos}/{len} ({percent}%) {msg}",
            )
            .context("Failed to create progress bar template")?
            .progress_chars("█▓░"),
        );
        pb.enable_steady_tick(Duration::from_millis(100));
        Ok(pb)
    }

    /// handle a post interaction
    async fn handle_post_interaction(&self, posts: Vec<E6Post>) -> Result<bool> {
        let use_multi_select = miette::Context::context(
            Confirm::new("Select multiple posts?").ask(),
            "Failed to get multi-select confirmation",
        )?;

        if use_multi_select {
            let selected_posts = self.select_multiple_posts(&posts)?;

            if !selected_posts.is_empty() {
                println!("\nSelected {} posts", selected_posts.len());

                let fetched_posts = self.fetch_selected_posts(selected_posts).await?;

                if !fetched_posts.is_empty() {
                    self.batch_interaction_menu(fetched_posts).await?;
                } else {
                    warn!("Failed to fetch any of the selected posts");
                    eprintln!("Failed to fetch any posts");
                }
            }

            self.ask_continue("Would you like to perform another search?")
        } else {
            let selected_post = self.select_post(&posts)?;

            if let Some(post) = selected_post {
                let fetched_post = self
                    .client
                    .get_post_by_id(post.id)
                    .await
                    .context("Failed to fetch post details")?;

                self.display_post(&fetched_post.post);
                self.interaction_menu(fetched_post.post).await?;
            }

            self.ask_continue("Would you like to perform another search?")
        }
    }

    /// fetch posts selected in list of results
    async fn fetch_selected_posts(&self, selected_posts: Vec<&E6Post>) -> Result<Vec<E6Post>> {
        let concurrent_limit = getopt!(search.fetch_threads).max(1);
        let post_ids: Vec<i64> = selected_posts.iter().map(|post| post.id).collect();
        let total_count = post_ids.len();

        if total_count == 0 {
            return Ok(Vec::new());
        }

        let pb = self.create_fetch_progress_bar(total_count)?;

        let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrent_limit));
        let pb_arc = Arc::new(pb);

        let tasks: Vec<_> = post_ids
            .into_iter()
            .map(|post_id| {
                let client = self.client.clone();
                let semaphore = Arc::clone(&semaphore);
                let pb = Arc::clone(&pb_arc);

                tokio::task::spawn(async move {
                    let _permit = semaphore
                        .acquire()
                        .await
                        .map_err(|e| {
                            warn!("Failed to acquire semaphore: {}", e);
                            e
                        })
                        .ok()?;

                    let result = match client.get_post_by_id(post_id).await {
                        Ok(fetched) => {
                            pb.inc(1);
                            Some(fetched.post)
                        }
                        Err(e) => {
                            pb.inc(1);
                            let err_msg = format!("Failed to fetch post {}: {}", post_id, e);
                            warn!("{}", err_msg);
                            pb.println(err_msg);
                            None
                        }
                    };

                    let pos = pb.position();
                    let len = pb.length().unwrap_or(0);
                    if pos % 10 == 0 || pos == len {
                        pb.set_message(format!("Fetching posts... ({}/{})", pos, len));
                    }

                    result
                })
            })
            .collect();

        let mut all_fetched_posts = Vec::with_capacity(total_count);

        for task in tasks {
            match task.await {
                Ok(Some(post)) => all_fetched_posts.push(post),
                Ok(None) => {}
                Err(e) => {
                    warn!("Task failed: {}", e);
                    pb_arc.println(format!("Task failed: {}", e));
                }
            }
        }

        pb_arc.finish_with_message(format!(
            "✓ Successfully fetched {}/{} posts",
            all_fetched_posts.len(),
            total_count
        ));

        if all_fetched_posts.is_empty() {
            bail!("Failed to fetch any posts");
        }

        Ok(all_fetched_posts)
    }

    /// make a progress bar for fetching
    fn create_fetch_progress_bar(&self, total: usize) -> Result<ProgressBar> {
        let pb = ProgressBar::new(total as u64);
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.bright_cyan} [{elapsed_precise}] [{wide_bar:.bright_cyan/blue}] \
                 {pos}/{len} ({percent}%) {msg}",
            )
            .context("Failed to create fetch progress bar template")?
            .with_key(
                "eta",
                |state: &indicatif::ProgressState, w: &mut dyn std::fmt::Write| {
                    write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap_or(())
                },
            )
            .progress_chars("━╸─"),
        );
        pb.set_message("Fetching posts...");
        pb.enable_steady_tick(Duration::from_millis(100));
        Ok(pb)
    }

    /// get the limit of posts to return
    fn get_post_limit(&self) -> Result<u64> {
        let default_limit = getopt!(search.results);

        if default_limit == 32 {
            let input = miette::Context::context(
                TextInput::new("How many posts to return?")
                    .with_validation(|input: &str| {
                        if input.trim().is_empty() {
                            return Ok(Validation::Valid);
                        }

                        match input.parse::<u64>() {
                            Ok(n) if n > 0 => Ok(Validation::Valid),
                            Ok(_) => Ok(Validation::Invalid(ErrorMessage::Custom(
                                "Please enter a positive number".to_string(),
                            ))),
                            Err(_) => Ok(Validation::Invalid(ErrorMessage::Custom(
                                "Please enter a valid number".to_string(),
                            ))),
                        }
                    })
                    .with_placeholder("32")
                    .ask(),
                "Failed to get post limit input",
            )?;

            let trimmed = input.trim();
            if trimmed.is_empty() {
                return Ok(32);
            }

            trimmed
                .parse::<u64>()
                .context("Failed to parse post limit")
                .map_err(Report::new)
        } else {
            Ok(default_limit)
        }
    }

    /// ask whether to continue
    fn ask_continue(&self, message: &str) -> Result<bool> {
        miette::Context::context(
            Confirm::new(message).ask(),
            "Failed to get user confirmation",
        )
        .map_err(Report::new)
    }

    /// select a post from a list of posts
    fn select_post<'a>(&self, posts: &'a [E6Post]) -> Result<Option<&'a E6Post>> {
        if posts.is_empty() {
            return Ok(None);
        }

        let options: Vec<_> = posts
            .iter()
            .map(|post| {
                AskOption::with_name(
                    format!(
                        "ID: {} | Score: {} | Rating: {}",
                        post.id, post.score.total, post.rating
                    ),
                    post,
                )
            })
            .collect();

        let selection = match miette::Context::context(
            Select::new("Select a post to view:")
                .with_options(options)
                .ask(),
            "Failed to get post selection",
        ) {
            Ok(p) => Some(p.value),
            Err(e) => return Err(e.into()),
        };

        Ok(selection)
    }

    /// display a pools info
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
}

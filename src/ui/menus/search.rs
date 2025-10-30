use {
    crate::{
        config::options::E62Rs,
        display::dtext::format_text,
        models::*,
        ui::{E6Ui, menus::AdvPoolSearch},
    },
    color_eyre::eyre::{Context, Result},
    indicatif::{ProgressBar, ProgressStyle},
    inquire::{Confirm, Select, Text},
    std::{collections::HashSet, fmt::Display, sync::Arc, time::Duration},
};

impl E6Ui {
    pub async fn search_pools_adv(&self) -> Result<()> {
        loop {
            let search_type =
                AdvPoolSearch::select("How would you like to search for pools").prompt()?;

            match search_type {
                AdvPoolSearch::ByName => {
                    if let Err(e) = self.perform_pool_search().await {
                        eprintln!("Pool name search error: {}", e);
                    }
                }
                AdvPoolSearch::ByDesc => {
                    if let Err(e) = self.perform_pool_description_search().await {
                        eprintln!("Pool description search error: {}", e);
                    }
                }
                AdvPoolSearch::ByCreator => {
                    if let Err(e) = self.perform_pool_creator_search().await {
                        eprintln!("Pool creator search error: {}", e);
                    }
                }
                AdvPoolSearch::BrowseLatest => {
                    if let Err(e) = self.browse_latest_pools().await {
                        eprintln!("Error browsing pools: {}", e);
                    }
                }
                AdvPoolSearch::Back => break,
            }

            if !Confirm::new("Would you like to perform another search?")
                .with_default(true)
                .prompt()
                .context("Failed to get retry confirmation")?
            {
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

    pub fn get_pool_search_query(&self) -> Result<String> {
        let query = Text::new("Enter pool search query (leave empty for latest pools):")
            .with_autocomplete(move |input: &str| {
                let suggestions = self.pool_db.autocomplete(input, 5);
                Ok(suggestions)
            })
            .prompt()
            .context("Failed to get pool search query")?;

        Ok(query.trim().to_string())
    }

    pub fn get_pool_limit(&self) -> Result<u64> {
        let settings = E62Rs::get()?;
        let default_limit = settings.post_count;

        let prompt = inquire::CustomType::<u64>::new("How many pools to return?")
            .with_default(default_limit)
            .with_error_message("Please enter a valid number")
            .prompt_skippable()
            .context("Failed to get pool limit")?;

        Ok(prompt.unwrap_or(default_limit).min(100))
    }

    pub fn select_pool<'a>(&self, pools: &'a [E6Pool]) -> Result<Option<&'a E6Pool>> {
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

    pub fn pool_entry_to_e6pool(&self, entry: &PoolEntry) -> E6Pool {
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
            self.pool_interaction_menu(pool.clone()).await?;

            Ok(true)
        } else {
            Ok(true)
        }
    }

    pub async fn perform_search(&self) -> Result<bool> {
        let (include_tags, exclude_tags) = self.collect_tags()?;
        let total_limit = self.get_post_limit()?;

        let mut all_tags = include_tags;
        for exclude_tag in exclude_tags {
            all_tags.push(format!("-{}", exclude_tag));
        }

        let mut all_fetched_posts: Vec<E6Post> = Vec::new();
        let limit_per_page = 320;
        let mut before_id: Option<i64> = None;
        let mut seen = HashSet::new();
        let mut consecutive_empty_batches = 0;
        let max_empty_batches = 3;

        println!("Fetching up to {} posts...", total_limit);
        let pb = ProgressBar::new(total_limit);
        pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}",
        )?
        .progress_chars("█▓░"),
    );
        pb.enable_steady_tick(Duration::from_millis(100));

        while (all_fetched_posts.len() as u64) < total_limit
            && consecutive_empty_batches < max_empty_batches
        {
            let remaining = total_limit - all_fetched_posts.len() as u64;
            let current_limit = (remaining * 2).min(limit_per_page).max(20);

            pb.set_message(format!(
                "Fetching batch {} ({} total so far)",
                (all_fetched_posts.len() / 320) + 1,
                all_fetched_posts.len()
            ));

            let results = self
                .client
                .search_posts(all_tags.clone(), Some(current_limit), before_id)
                .await
                .context("Failed to search posts")?;

            let batch_size_before_filtering = results.posts.len();

            if results.posts.is_empty() {
                consecutive_empty_batches += 1;
                pb.println("No more posts available from API.");
                if consecutive_empty_batches >= max_empty_batches {
                    pb.println("Reached maximum consecutive empty batches, stopping.");
                    break;
                }
                continue;
            } else {
                consecutive_empty_batches = 0;
            }

            let mut new_posts = Vec::with_capacity(results.posts.len());
            for p in results.posts {
                if seen.insert(p.id) {
                    new_posts.push(p);
                }
            }

            let fetched_count = new_posts.len();
            if let Some(min_id) = new_posts.iter().map(|p| p.id).min() {
                before_id = Some(min_id);
            }

            all_fetched_posts.extend(new_posts);
            pb.inc(fetched_count as u64);

            if batch_size_before_filtering > fetched_count {
                pb.println(format!(
                    "Filtered out {} blacklisted posts from this batch",
                    batch_size_before_filtering - fetched_count
                ));
            }

            if fetched_count < current_limit as usize / 2 {
                pb.println("Approaching end of available results.");
            }
        }

        all_fetched_posts.truncate(total_limit as usize);
        pb.set_length(all_fetched_posts.len() as u64);
        pb.finish_with_message(format!("✓ Fetched {} posts.", all_fetched_posts.len()));

        if all_fetched_posts.is_empty() {
            println!("No posts found matching your search criteria.");
            return self.ask_retry();
        }

        self.handle_post_interaction(all_fetched_posts).await
    }

    async fn handle_post_interaction(&self, posts: Vec<E6Post>) -> Result<bool> {
        let use_multi_select = Confirm::new("Select multiple posts?")
            .with_default(false)
            .prompt()
            .context("Failed to get multi-select preference")?;

        if use_multi_select {
            let selected_posts = self.select_multiple_posts(&posts)?;

            if !selected_posts.is_empty() {
                println!("\nSelected {} posts", selected_posts.len());

                let fetched_posts = self.fetch_selected_posts(selected_posts).await?;

                if !fetched_posts.is_empty() {
                    self.batch_interaction_menu(fetched_posts).await?;
                } else {
                    eprintln!("Failed to fetch any posts");
                }
            }

            self.ask_retry()
        } else {
            let selected_post = self.select_post(&posts)?;

            if let Some(post) = selected_post {
                let fetched_post = self.client.get_post_by_id(post.id).await?;
                self.display_post(&fetched_post.post);
                self.interaction_menu(fetched_post.post).await?;

                self.ask_retry()
            } else {
                Ok(true)
            }
        }
    }

    async fn fetch_selected_posts(&self, selected_posts: Vec<&E6Post>) -> Result<Vec<E6Post>> {
        let search_cfg = E62Rs::get()?.search;
        let concurrent_limit = search_cfg.fetch_threads;
        let post_ids: Vec<i64> = selected_posts.iter().map(|post| post.id).collect();
        let total_count = post_ids.len();

        let pb = ProgressBar::new(total_count as u64);
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({percent}%) {msg}",
            )?
            .with_key(
                "eta",
                |state: &indicatif::ProgressState, w: &mut dyn std::fmt::Write| {
                    write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap_or(())
                },
            )
            .progress_chars("█▓░"),
        );
        pb.set_message("Fetching posts...");
        pb.enable_steady_tick(Duration::from_millis(100));

        let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrent_limit));
        let pb_arc = Arc::new(pb);

        let tasks = post_ids.into_iter().map(|post_id| {
            let client = self.client.clone();
            let semaphore = Arc::clone(&semaphore);
            let pb = Arc::clone(&pb_arc);

            tokio::task::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();

                let result = match client.get_post_by_id(post_id).await {
                    Ok(fetched) => {
                        pb.inc(1);
                        Some(fetched.post)
                    }
                    Err(e) => {
                        pb.inc(1);
                        pb.println(format!("Failed to fetch post {}: {}", post_id, e));
                        None
                    }
                };

                let pos = pb.position();
                let len = pb.length().unwrap_or(0);
                if pos.is_multiple_of(10) || pos == len {
                    pb.set_message(format!("Fetching posts... ({}/{})", pos, len));
                }

                result
            })
        });

        let mut all_fetched_posts = Vec::new();
        for task in tasks {
            match task.await {
                Ok(Some(post)) => all_fetched_posts.push(post),
                Ok(None) => {}
                Err(e) => {
                    pb_arc.println(format!("Task failed: {}", e));
                }
            }
        }

        pb_arc.finish_with_message(format!(
            "✓ Successfully fetched {}/{} posts",
            all_fetched_posts.len(),
            total_count
        ));

        Ok(all_fetched_posts)
    }

    fn get_post_limit(&self) -> Result<u64> {
        let settings = E62Rs::get()?;

        if settings.post_count.eq(&32) {
            let prompt = inquire::CustomType::<u64>::new("How many posts to return?")
                .with_default(32)
                .with_error_message("Please enter a valid number")
                .prompt_skippable()
                .context("Failed to get post limit")?;

            Ok(prompt.unwrap_or(32))
        } else {
            Ok(settings.post_count)
        }
    }

    fn ask_retry(&self) -> Result<bool> {
        Confirm::new("Would you like to perform another search?")
            .with_default(true)
            .prompt()
            .context("Failed to get retry confirmation")
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

    pub fn display_pool(&self, pool: &E6Pool) {
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

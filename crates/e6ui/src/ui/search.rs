use anyhow::{Context, Result};
use e6cfg::Cfg;
use e6core::{
    formatting::format_text,
    models::{E6Pool, E6Post, PoolEntry},
};
use inquire::{Confirm, Select, Text};

use crate::ui::{
    E6Ui,
    menus::{AdvPoolSearch, BatchAction, InteractionMenu, PoolInteractionMenu},
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
        let settings = Cfg::get().unwrap_or_default();
        let default_limit = settings.post_count.unwrap_or(32);

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

    pub async fn perform_search(&self) -> Result<bool> {
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

    fn get_post_limit(&self) -> Result<u64> {
        let settings = Cfg::get().unwrap_or_default();

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
}

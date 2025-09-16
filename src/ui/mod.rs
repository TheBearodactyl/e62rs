use crate::config::get_config;
use crate::ui::download::PostDownloader;
use crate::{client::E6Client, formatting::format_text, models::E6Post};
use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use inquire::{Confirm, MultiSelect, Select, Text};

mod download;
mod search;

#[derive(Default)]
pub struct E6Ui {
    client: E6Client,
    downloader: PostDownloader,
}

#[derive(inquiry::Choice, Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum InteractionMenu {
    /// Open the post in your browser
    OpenInBrowser,
    /// Download the post
    Download,
    /// Go back to search
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
    /// Go back to search
    Back,
}

impl E6Ui {
    pub fn new(client: E6Client) -> Self {
        let settings = get_config().expect("Failed to get settings");
        let downloader = PostDownloader::with_download_dir(settings.download_dir);

        Self { client, downloader }
    }

    pub async fn search(&self) -> Result<()> {
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
                .prompt()
                .context("Failed to get tag input")?;

            if !tag.trim().is_empty() {
                tags.push(tag.trim().to_string());
            }
        }

        Ok(tags)
    }

    fn get_post_limit(&self) -> Result<u64> {
        let settings = get_config()?;

        if settings.post_count.eq(&32) {
            let prompt = inquire::CustomType::<u64>::new("How many posts to return?")
                .with_default(32)
                .with_error_message("Please enter a valid number")
                .prompt_skippable()
                .context("Failed to get post limit")?;

            Ok(prompt.unwrap_or(32).min(320))
        } else {
            Ok(settings.post_count)
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

        for post in posts.into_iter() {
            total_downloaded_pb.inc(1);
            match self.downloader.download_post(post).await {
                Ok(_) => println!("✓ Post downloaded successfully"),
                Err(e) => eprintln!("✗ Failed to download post: {}", e),
            }
        }

        println!("\n✓ Batch download complete");
        Ok(())
    }

    fn ask_retry(&self) -> Result<bool> {
        Confirm::new("Would you like to perform another search?")
            .with_default(true)
            .prompt()
            .context("Failed to get retry confirmation")
    }
}

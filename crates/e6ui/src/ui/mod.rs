use std::sync::Arc;

use e6cfg::Cfg;
use e6core::{
    client::E6Client,
    data::{pools::PoolDatabase, tags::TagDatabase},
    image::fetch_and_display_images_as_sixel,
    models::E6Post,
};

use anyhow::{Context, Result};
use inquire::{Confirm, MultiSelect, Text};

use crate::ui::{
    download::PostDownloader,
    menus::{BatchAction, InteractionMenu},
    view::*,
};

pub mod blacklist;
pub mod download;
pub mod menus;
pub mod search;
pub mod view;

#[derive(Default, Clone)]
pub struct E6Ui {
    client: Arc<E6Client>,
    downloader: Arc<PostDownloader>,
    tag_db: Arc<TagDatabase>,
    pool_db: Arc<PoolDatabase>,
}

impl E6Ui {
    pub fn new(
        client: Arc<E6Client>,
        tag_db: Arc<TagDatabase>,
        pool_db: Arc<PoolDatabase>,
    ) -> Self {
        let settings = Cfg::get().unwrap_or_default();
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

    pub async fn batch_interaction_menu(&self, posts: Vec<E6Post>) -> Result<BatchAction> {
        let choice = BatchAction::select(&format!(
            "What would you like to do with {} selected posts?",
            posts.len()
        ))
        .prompt()
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
                    &posts_clone
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

    pub fn collect_tags(&self) -> Result<Vec<String>> {
        let mut tags = Vec::new();

        loop {
            let prompt = if tags.is_empty() {
                "Would you like to add tags to the search?"
            } else {
                println!("Current tags: {}", tags.join(" "));
                "Would you like to add more tags?"
            };

            let add_tags = Confirm::new(prompt)
                .with_default(false)
                .prompt()
                .context("Failed to get user confirmation")?;

            if !add_tags {
                break;
            }

            let all_tags: Vec<String> = unsafe { self.tag_db.iter_tags() }
                .map(|entry| entry.name.to_string())
                .collect();

            if all_tags.is_empty() {
                println!("No tags available in the database.");
                break;
            }

            let selected_tags = MultiSelect::new(
                "Select tags (use spacebar to select, enter to confirm):",
                all_tags,
            )
            .with_help_message("Select multiple tags using spacebar, press enter when done")
            .with_page_size(20)
            .prompt_skippable()
            .context("Failed to get tag selection")?;

            if let Some(selected) = selected_tags {
                for tag in selected {
                    if !tags.contains(&tag) {
                        tags.push(tag);
                    } else {
                        println!("Tag '{}' is already selected.", tag);
                    }
                }
            }

            if tags.is_empty() {
                let try_again = Confirm::new("No tags were selected. Would you like to try again?")
                    .with_default(true)
                    .prompt()
                    .context("Failed to get retry confirmation")?;

                if !try_again {
                    break;
                }
            }
        }

        Ok(tags)
    }

    async fn interaction_menu(&self, post: E6Post) -> Result<InteractionMenu> {
        let choice = InteractionMenu::select("What would you like to do?")
            .prompt()
            .context("Failed to get interaction choice")?;

        match choice {
            InteractionMenu::OpenInBrowser => {
                self.open_in_browser(&post)?;
            }
            InteractionMenu::Download => {
                self.download_post(post).await?;
            }
            InteractionMenu::Back => {}
            InteractionMenu::View => {
                print_post_to_terminal(post)
                    .await
                    .context("Failed to view image")?;
            }
        }

        Ok(choice)
    }
}

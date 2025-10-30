use {
    crate::{
        client::E6Client,
        config::options::E62Rs,
        data::{pools::PoolDatabase, tags::TagDatabase},
        display::image::fetch_and_display_images_as_sixel,
        models::*,
        serve::{cfg::ServerConfig, server::MediaServer},
        ui::menus::{
            BatchAction, InteractionMenu, PoolInteractionMenu, download::PostDownloader,
            view::print_post_to_terminal,
        },
    },
    color_eyre::eyre::{Context, Result},
    inquire::{Confirm, Editor, MultiSelect, Select},
    serde::{Deserialize, Serialize},
    std::{path::PathBuf, str::FromStr, sync::Arc},
    tokio::fs,
};

pub mod menus;
pub mod progress;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Download {
    pub post_data: E6Post,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DownloadData {
    pub posts: Vec<Download>,
}

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
        let settings = E62Rs::get_unsafe();
        let dl_cfg = settings.download;
        let downloader = Arc::new(PostDownloader::with_download_dir_and_format(
            dl_cfg.download_dir.clone(),
            Some(dl_cfg.output_format.clone()),
        ));

        if std::fs::exists(dl_cfg.download_dir.clone()).is_err() {
            std::fs::create_dir_all(dl_cfg.download_dir.clone())
                .expect("Failed to create output directory");
        }

        Self {
            client,
            downloader,
            tag_db,
            pool_db,
        }
    }

    pub fn collect_tags(&self) -> Result<(Vec<String>, Vec<String>)> {
        let mut include_tags = Vec::new();
        let mut exclude_tags = Vec::new();

        loop {
            let prompt = if include_tags.is_empty() && exclude_tags.is_empty() {
                "Would you like to add tags to the search?"
            } else {
                if !include_tags.is_empty() {
                    println!("Include tags: {}", include_tags.join(" "));
                }
                if !exclude_tags.is_empty() {
                    println!("Exclude tags: -{}", exclude_tags.join(" -"));
                }

                "Would you like to add more tags?"
            };

            let add_tags = Confirm::new(prompt)
                .with_default(false)
                .prompt()
                .context("Failed to get user confirmation")?;

            if !add_tags {
                break;
            }

            let tag_type = Select::new(
                "What type of tags would you like to add?",
                vec!["Include tags", "Exclude tags"],
            )
            .prompt()
            .context("Failed to get tag type selection")?;

            let all_tags: Vec<String> = unsafe { self.tag_db.iter_tags() }
                .map(|entry| entry.name.to_string())
                .collect();

            if all_tags.is_empty() {
                println!("No tags available in the database.");
                break;
            }

            let selected_tags = MultiSelect::new(
                if tag_type == "Include tags" {
                    "Select tags to include (use spacebar to select, enter to confirm):"
                } else {
                    "Select tags to exclude (use spacebar to select, enter to confirm):"
                },
                all_tags,
            )
            .with_help_message("Select multiple tags using spacebar, press enter when done")
            .with_page_size(20)
            .prompt_skippable()
            .context("Failed to get tag selection")?;

            if let Some(selected) = selected_tags {
                let target_list = if tag_type == "Include tags" {
                    &mut include_tags
                } else {
                    &mut exclude_tags
                };

                for tag in selected {
                    if !target_list.contains(&tag) {
                        target_list.push(tag);
                    } else {
                        println!(
                            "Tag '{}' is already selected for {}.",
                            tag,
                            tag_type.to_lowercase()
                        );
                    }
                }
            }

            if include_tags.is_empty() && exclude_tags.is_empty() {
                let try_again = Confirm::new("No tags were selected. Would you like to try again?")
                    .with_default(true)
                    .prompt()
                    .context("Failed to get retry confirmation")?;

                if !try_again {
                    break;
                }
            }
        }

        Ok((include_tags, exclude_tags))
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
            BatchAction::Back => {
                return Ok(BatchAction::Back);
            }
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
            PoolInteractionMenu::Back => {
                return Ok(PoolInteractionMenu::Back);
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
            InteractionMenu::Back => {
                return Ok(InteractionMenu::Back);
            }
            InteractionMenu::View => {
                print_post_to_terminal(post)
                    .await
                    .context("Failed to view image")?;
            }
        }

        Ok(choice)
    }

    pub async fn edit_config_file(&self) -> Result<()> {
        let curr_cfg = toml::to_string_pretty(&E62Rs::get()?)?;
        let new_cfg_text = Editor::new("Edit your config file")
            .with_file_extension(".toml")
            .with_predefined_text(&curr_cfg)
            .prompt()?;

        if let Ok(new_cfg) = toml::from_str::<E62Rs>(new_cfg_text.as_str()) {
            fs::write("e62rs.toml", toml::to_string_pretty(&new_cfg)?).await?;
        } else {
            eprintln!("Error validating new config text");
            std::process::exit(1);
        }

        Ok(())
    }

    pub async fn serve_downloads(&self) -> Result<()> {
        let cfg = E62Rs::get()?;
        let downloads_dir = cfg.download.download_dir;

        let gallery_cfg = cfg.gallery;
        let port = gallery_cfg.port;
        let enable_metadata = gallery_cfg.enable_metadata_filtering;
        let cache_metadata = gallery_cfg.cache_metadata;

        let srv_cfg = ServerConfig::builder()
            .media_directory(PathBuf::from_str(&downloads_dir)?)
            .bind_address(format!("127.0.0.1:{}", port).parse()?)
            .max_file_size(100 * 1024 * 1024)
            .enable_metadata_filtering(enable_metadata)
            .cache_metadata(cache_metadata)
            .build()
            .expect("Failed to build server config");

        let srv = MediaServer::new(srv_cfg);

        if gallery_cfg.auto_open_browser {
            let url = format!("http://localhost:{}", port);
            let _ = open::that(&url);
            println!("Opening browser at {}", url);
        }

        srv.serve().await
    }
}

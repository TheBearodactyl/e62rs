use {
    crate::{
        client::E6Client,
        config::options::E62Rs,
        data::{pools::PoolDatabase, tags::TagDatabase},
        display::image::fetch_and_display_images_as_sixel,
        models::*,
        serve::{cfg::ServerConfig, server::MediaServer},
        ui::{
            autocomplete::TagAutocompleter,
            menus::{
                BatchAction, InteractionMenu, PoolInteractionMenu, download::PostDownloader,
                view::print_post_to_terminal,
            },
        },
    },
    color_eyre::eyre::{Context, Result},
    inquire::{Confirm, Editor, MultiSelect, Select, Text},
    owo_colors::OwoColorize,
    serde::{Deserialize, Serialize},
    std::{path::PathBuf, str::FromStr, sync::Arc},
    tokio::fs,
};

pub mod autocomplete;
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

    pub fn collect_tags(&self) -> Result<(Vec<String>, Vec<String>, Vec<String>)> {
        let autocompleter = TagAutocompleter::new(self.tag_db.clone());

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

        let tags_input = Text::new("Enter tags:")
            .with_autocomplete(autocompleter)
            .with_help_message(
                "Space-separated tags. Use - to exclude, ~ for OR. Tab to autocomplete.",
            )
            .prompt()
            .context("Failed to get tags input")?;

        let mut include_tags = Vec::new();
        let mut exclude_tags = Vec::new();
        let mut or_tags = Vec::new();

        for tag in tags_input.split_whitespace() {
            let tag = tag.trim();

            if tag.is_empty() {
                continue;
            }

            if let Some(stripped) = tag.strip_prefix('-') {
                let canonical = self.tag_db.get_canonical_name(stripped);
                if !exclude_tags.contains(&canonical) {
                    exclude_tags.push(canonical);
                }
            } else if let Some(stripped) = tag.strip_prefix('~') {
                let canonical = self.tag_db.get_canonical_name(stripped);
                if !or_tags.contains(&canonical) {
                    or_tags.push(canonical);
                }
            } else {
                let stripped = tag.strip_prefix('+').unwrap_or(tag);
                let canonical = self.tag_db.get_canonical_name(stripped);
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

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
    demand::{Confirm, DemandOption, Input, MultiSelect, Select, Theme},
    owo_colors::OwoColorize,
    serde::{Deserialize, Serialize},
    std::{
        path::PathBuf,
        str::FromStr,
        sync::{Arc, LazyLock},
    },
    termcolor::{Color, ColorSpec},
    tokio::fs,
};

pub mod autocomplete;
pub mod menus;
pub mod progress;

trait RosePineTheme {
    fn rose_pine() -> Theme;
}

fn make_color(color: Color) -> ColorSpec {
    let mut spec = ColorSpec::new();
    spec.set_fg(Some(color));
    spec
}

impl RosePineTheme for Theme {
    fn rose_pine() -> Self {
        let base = Color::Rgb(25, 23, 36);
        let text = Color::Rgb(224, 222, 244);
        let subtle = Color::Rgb(144, 140, 170);
        let muted = Color::Rgb(110, 106, 134);
        let love = Color::Rgb(235, 111, 146);
        let rose = Color::Rgb(235, 188, 186);
        let foam = Color::Rgb(156, 207, 216);
        let iris = Color::Rgb(196, 167, 231);

        let mut title = make_color(iris);
        title.set_bold(true);

        let mut focused_button = make_color(base);
        focused_button.set_bg(Some(rose));

        let mut blurred_button = make_color(text);
        blurred_button.set_bg(Some(base));

        let mut cursor_style = ColorSpec::new();
        cursor_style
            .set_fg(Some(Color::White))
            .set_bg(Some(Color::Black));

        Self {
            title,
            error_indicator: make_color(love),
            description: make_color(subtle),
            cursor: make_color(rose),
            cursor_str: String::from("❯"),

            selected_prefix: String::from(" [•]"),
            selected_prefix_fg: make_color(foam),
            selected_option: make_color(foam),
            unselected_prefix: String::from(" [ ]"),
            unselected_prefix_fg: make_color(muted),
            unselected_option: make_color(text),

            input_cursor: make_color(rose),
            input_placeholder: make_color(muted),
            input_prompt: make_color(rose),

            help_key: make_color(subtle),
            help_desc: make_color(muted),
            help_sep: make_color(subtle),

            focused_button,
            blurred_button,

            cursor_style,
            force_style: true,
            ..Default::default()
        }
    }
}

pub static ROSE_PINE: LazyLock<Theme> = LazyLock::new(Theme::rose_pine);

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

        let autocompleter = TagAutocompleter::new(self.tag_db.clone());
        let tags_input = Input::new("Enter tags:")
            .description("Space-separated tags. Use - to exclude, ~ for OR. Tab to autocomplete.")
            .placeholder("e.g., cat dog -gore")
            .theme(&ROSE_PINE)
            .autocomplete(autocompleter)
            .run()
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
        .theme(&ROSE_PINE)
        .run()
        .context("Failed to get batch action choice")?;

        match choice {
            BatchAction::DownloadAll => {
                self.downloader.clone().download_posts(posts).await?;
            }
            BatchAction::OpenAllInBrowser => {
                self.open_posts_in_browser(&posts)?;
            }
            BatchAction::DownloadAndOpenAll => {
                let posts_clone = posts.clone();
                self.downloader.clone().download_posts(posts).await?;
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
            .theme(&Theme::rose_pine())
            .run()
            .context("Failed to get pool interaction choice")?;

        match choice {
            PoolInteractionMenu::ViewPosts => {
                let posts = self.client.get_pool_posts(pool.id).await?;
                if posts.posts.is_empty() {
                    println!("No posts found in this pool.");
                } else {
                    self.display_posts(&posts.posts);

                    let interact = Confirm::new("Would you like to interact with these posts?")
                        .theme(&Theme::rose_pine())
                        .affirmative("Yes")
                        .negative("No")
                        .run()?;

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
                    self.downloader.clone().download_posts(posts.posts).await?;
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
        let options: Vec<DemandOption<usize>> = posts
            .iter()
            .enumerate()
            .map(|(idx, post)| {
                DemandOption::new(idx).label(
                    format!(
                        "ID: {} | Score: {} | Rating: {}",
                        post.id, post.score.total, post.rating
                    )
                    .as_str(),
                )
            })
            .collect();

        let selections = MultiSelect::new("Select posts (Space to select, Enter to confirm):")
            .description("Use arrow keys to navigate, Space to select/deselect, Enter to confirm")
            .filterable(true)
            .options(options)
            .theme(&Theme::rose_pine())
            .run()
            .context("Failed to get post selections")?;

        let selected_posts: Vec<&E6Post> = selections.iter().map(|&idx| &posts[idx]).collect();

        Ok(selected_posts)
    }

    async fn interaction_menu(&self, post: E6Post) -> Result<InteractionMenu> {
        let choice = InteractionMenu::select("What would you like to do?")
            .theme(&Theme::rose_pine())
            .run()
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
        }

        Ok(choice)
    }

    pub async fn edit_config_file(&self) -> Result<()> {
        let curr_cfg = toml::to_string_pretty(&E62Rs::get()?)?;

        println!("Opening config file in your default editor...");

        let temp_file = std::env::temp_dir().join("e62rs_config.toml");
        fs::write(&temp_file, &curr_cfg).await?;

        #[cfg(unix)]
        {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
            std::process::Command::new(editor)
                .arg(&temp_file)
                .status()?;
        }

        #[cfg(windows)]
        {
            std::process::Command::new("notepad")
                .arg(&temp_file)
                .status()?;
        }

        let new_cfg_text = fs::read_to_string(&temp_file).await?;

        if let Ok(new_cfg) = toml::from_str::<E62Rs>(new_cfg_text.as_str()) {
            fs::write("e62rs.toml", toml::to_string_pretty(&new_cfg)?).await?;
            fs::remove_file(&temp_file).await?;
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

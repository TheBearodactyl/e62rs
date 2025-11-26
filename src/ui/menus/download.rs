use {
    crate::{
        config::options::E62Rs,
        models::E6Post,
        ui::{E6Ui, ROSE_PINE, progress::ProgressManager},
    },
    color_eyre::eyre::{Context, ContextCompat, Result, bail},
    demand::{Confirm, DemandOption, Input, MultiSelect},
    futures::StreamExt,
    indicatif::ProgressBar,
    owo_colors::OwoColorize,
    reqwest::Client,
    std::{
        borrow::Cow,
        collections::{HashMap, HashSet},
        fs::OpenOptions,
        io::Write,
        path::{Path, PathBuf},
        sync::Arc,
    },
    tokio::{fs::File, io::AsyncWriteExt},
    tracing::{info, warn},
    url::Url,
};

#[derive(Default, Clone)]
pub struct PostDownloader {
    client: Client,
    download_dir: Option<PathBuf>,
    output_format: Option<String>,
    progress_manager: Arc<ProgressManager>,
}

fn sanitize_path<S: AsRef<str>>(input: S) -> PathBuf {
    let s = input.as_ref();
    let mut sanitized = String::with_capacity(s.len());

    #[cfg(target_os = "windows")]
    {
        for ch in s.chars() {
            sanitized.push(match ch {
                '<' => '＜',
                '>' => '＞',
                ':' => '：',
                '"' => '＂',
                '|' => '｜',
                '?' => '？',
                '*' => '＊',
                _ => ch,
            });
        }
    }

    PathBuf::from(sanitized)
}

#[derive(Debug, Clone, Copy)]
enum IndexRange {
    Exact(usize),
    Range(usize, usize),
    RangeFrom(usize),
    RangeTo(usize),
}

impl IndexRange {
    fn parse(s: &str) -> Option<Self> {
        if let Some((left, right)) = s.split_once("..") {
            match (left, right) {
                ("", "") => None,
                ("", r) => r.parse().ok().map(IndexRange::RangeTo),
                (l, "") => l.parse().ok().map(IndexRange::RangeFrom),
                (l, r) => {
                    let start = l.parse().ok()?;
                    let end = r.parse().ok()?;
                    Some(IndexRange::Range(start, end))
                }
            }
        } else {
            s.parse().ok().map(IndexRange::Exact)
        }
    }

    fn apply<'a, T>(&self, items: &'a [T]) -> &'a [T] {
        match *self {
            IndexRange::Exact(n) => &items[..n.min(items.len())],
            IndexRange::Range(start, end) => {
                let start = start.min(items.len());
                let end = end.min(items.len());
                &items[start..end]
            }
            IndexRange::RangeFrom(start) => {
                let start = start.min(items.len());
                &items[start..]
            }
            IndexRange::RangeTo(end) => {
                let end = end.min(items.len());
                &items[..end]
            }
        }
    }
}

struct FormatContext<'a> {
    post: &'a E6Post,
    now: chrono::DateTime<chrono::Local>,
}

impl<'a> FormatContext<'a> {
    fn new(post: &'a E6Post) -> Self {
        Self {
            post,
            now: chrono::Local::now(),
        }
    }

    fn get_simple(&self, key: &str) -> Option<Cow<'a, str>> {
        match key {
            "id" => Some(Cow::Owned(self.post.id.to_string())),
            "md5" => Some(Cow::Owned(self.post.file.md5.to_string())),
            "ext" => Some(Cow::Owned(self.post.file.ext.to_string())),
            "width" => Some(Cow::Owned(self.post.file.width.to_string())),
            "height" => Some(Cow::Owned(self.post.file.height.to_string())),
            "size" => Some(Cow::Owned(self.post.file.size.to_string())),
            "size_mb" => Some(Cow::Owned(format!(
                "{:.2}",
                self.post.file.size as f64 / (1024.0 * 1024.0)
            ))),
            "size_kb" => Some(Cow::Owned(format!(
                "{:.2}",
                self.post.file.size as f64 / 1024.0
            ))),

            "rating" => Some(Cow::Borrowed(match self.post.rating.as_str() {
                "e" => "explicit",
                "q" => "questionable",
                "s" => "safe",
                _ => "unknown",
            })),
            "rating_first" => Some(Cow::Owned(
                self.post
                    .rating
                    .chars()
                    .next()
                    .unwrap_or('u')
                    .to_lowercase()
                    .to_string(),
            )),

            "score" => Some(Cow::Owned(self.post.score.total.to_string())),
            "score_up" => Some(Cow::Owned(self.post.score.up.to_string())),
            "score_down" => Some(Cow::Owned(self.post.score.down.to_string())),
            "fav_count" => Some(Cow::Owned(self.post.fav_count.to_string())),
            "comment_count" => Some(Cow::Owned(self.post.comment_count.to_string())),

            "aspect_ratio" => {
                let ratio = if self.post.file.height > 0 {
                    self.post.file.width as f64 / self.post.file.height as f64
                } else {
                    0.0
                };
                Some(Cow::Owned(format!("{:.2}", ratio)))
            }
            "orientation" => Some(Cow::Borrowed(
                if self.post.file.width > self.post.file.height {
                    "landscape"
                } else if self.post.file.width < self.post.file.height {
                    "portrait"
                } else {
                    "square"
                },
            )),
            "resolution" => Some(Cow::Borrowed(
                match (self.post.file.width, self.post.file.height) {
                    (w, h) if w >= 7680 || h >= 4320 => "8K",
                    (w, h) if w >= 3840 || h >= 2160 => "4K",
                    (w, h) if w >= 2560 || h >= 1440 => "QHD",
                    (w, h) if w >= 1920 || h >= 1080 => "FHD",
                    (w, h) if w >= 1280 || h >= 720 => "HD",
                    _ => "SD",
                },
            )),
            "megapixels" => Some(Cow::Owned(format!(
                "{:.1}",
                (self.post.file.width * self.post.file.height) as f64 / 1_000_000.0
            ))),

            "artist" => Some(Cow::Borrowed(
                self.post
                    .tags
                    .artist
                    .first()
                    .map(|s| s.as_str())
                    .unwrap_or("unknown"),
            )),
            "artist_count" => Some(Cow::Owned(self.post.tags.artist.len().to_string())),

            "tag_count" => {
                let total = self.post.tags.general.len()
                    + self.post.tags.artist.len()
                    + self.post.tags.character.len()
                    + self.post.tags.species.len()
                    + self.post.tags.copyright.len()
                    + self.post.tags.meta.len()
                    + self.post.tags.lore.len();
                Some(Cow::Owned(total.to_string()))
            }
            "tag_count_general" => Some(Cow::Owned(self.post.tags.general.len().to_string())),
            "tag_count_character" => Some(Cow::Owned(self.post.tags.character.len().to_string())),
            "tag_count_species" => Some(Cow::Owned(self.post.tags.species.len().to_string())),
            "tag_count_copyright" => Some(Cow::Owned(self.post.tags.copyright.len().to_string())),

            "pool_count" => Some(Cow::Owned(self.post.pools.len().to_string())),
            "pool_ids" => Some(Cow::Owned(
                self.post
                    .pools
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            )),

            "uploader" => Some(Cow::Borrowed(&self.post.uploader_name)),
            "uploader_id" => Some(Cow::Owned(self.post.uploader_id.to_string())),
            "approver_id" => Some(Cow::Owned(
                self.post
                    .approver_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "none".to_string()),
            )),

            "has_children" => Some(Cow::Borrowed(if self.post.relationships.has_children {
                "yes"
            } else {
                "no"
            })),
            "parent_id" => Some(Cow::Owned(
                self.post
                    .relationships
                    .parent_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "none".to_string()),
            )),

            "is_pending" => Some(Cow::Borrowed(if self.post.flags.pending {
                "yes"
            } else {
                "no"
            })),
            "is_flagged" => Some(Cow::Borrowed(if self.post.flags.flagged {
                "yes"
            } else {
                "no"
            })),
            "is_deleted" => Some(Cow::Borrowed(if self.post.flags.deleted {
                "yes"
            } else {
                "no"
            })),
            "has_notes" => Some(Cow::Borrowed(if self.post.has_notes {
                "yes"
            } else {
                "no"
            })),

            "duration" => Some(Cow::Owned(
                self.post
                    .duration
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "0".to_string()),
            )),
            "duration_formatted" => {
                if let Some(duration) = self.post.duration {
                    let total_seconds = duration as i64;
                    let hours = total_seconds / 3600;
                    let minutes = (total_seconds % 3600) / 60;
                    let seconds = total_seconds % 60;
                    Some(Cow::Owned(if hours > 0 {
                        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
                    } else {
                        format!("{:02}:{:02}", minutes, seconds)
                    }))
                } else {
                    Some(Cow::Borrowed("N/A"))
                }
            }

            "file_type" => Some(Cow::Borrowed(match self.post.file.ext.as_str() {
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => "image",
                "mp4" | "webm" | "mov" | "avi" | "mkv" => "video",
                "swf" => "flash",
                _ => "unknown",
            })),

            key if key.starts_with("year")
                || key.starts_with("month")
                || key.starts_with("day")
                || key.starts_with("hour")
                || key.starts_with("minute")
                || key.starts_with("second")
                || key.starts_with("date")
                || key.starts_with("time")
                || key == "timestamp"
                || key == "datetime" =>
            {
                self.get_date_field(key)
            }

            key if key.starts_with("now_") => self.get_now_field(key),

            _ => None,
        }
    }

    fn get_indexed(&self, key: &str, range: IndexRange) -> Option<Cow<'a, str>> {
        let result = match key {
            "tags" => {
                let items = range.apply(&self.post.tags.general);
                items.to_vec().join(", ")
            }
            "artists" => {
                let items = range.apply(&self.post.tags.artist);
                items.to_vec().join(", ")
            }
            "characters" => {
                let items = range.apply(&self.post.tags.character);
                items.to_vec().join(", ")
            }
            "species" => {
                let items = range.apply(&self.post.tags.species);
                items.to_vec().join(", ")
            }
            "copyright" => {
                let items = range.apply(&self.post.tags.copyright);
                items.to_vec().join(", ")
            }
            "sources" => {
                let items = range.apply(&self.post.sources);
                items
                    .iter()
                    .map(|source| {
                        Url::parse(source)
                            .ok()
                            .and_then(|u| u.domain().map(String::from))
                            .unwrap_or_else(|| "unknown".to_string())
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            }
            _ => return None,
        };

        Some(Cow::Owned(result))
    }

    fn get_date_field(&self, key: &str) -> Option<Cow<'a, str>> {
        let date_str = if key.contains("updated") {
            &self.post.updated_at
        } else {
            &self.post.created_at
        };

        let parsed_date = chrono::DateTime::parse_from_rfc3339(date_str).ok()?;

        Some(Cow::Owned(match key {
            k if k.contains("year") => parsed_date.format("%Y").to_string(),
            k if k.contains("month") => parsed_date.format("%m").to_string(),
            k if k.contains("day") => parsed_date.format("%d").to_string(),
            k if k.contains("hour") => parsed_date.format("%H").to_string(),
            k if k.contains("minute") => parsed_date.format("%M").to_string(),
            k if k.contains("second") => parsed_date.format("%S").to_string(),
            "date" => parsed_date.format("%Y-%m-%d").to_string(),
            "date_updated" => parsed_date.format("%Y-%m-%d").to_string(),
            "time" => parsed_date.format("%H-%M-%S").to_string(),
            "datetime" => parsed_date.format("%Y-%m-%d %H-%M-%S").to_string(),
            "timestamp" => parsed_date.timestamp().to_string(),
            _ => return None,
        }))
    }

    fn get_now_field(&self, key: &str) -> Option<Cow<'a, str>> {
        Some(Cow::Owned(match key {
            "now_year" => self.now.format("%Y").to_string(),
            "now_month" => self.now.format("%m").to_string(),
            "now_day" => self.now.format("%d").to_string(),
            "now_hour" => self.now.format("%H").to_string(),
            "now_minute" => self.now.format("%M").to_string(),
            "now_second" => self.now.format("%S").to_string(),
            "now_date" => self.now.format("%Y-%m-%d").to_string(),
            "now_time" => self.now.format("%H-%M-%S").to_string(),
            "now_datetime" => self.now.format("%Y-%m-%d %H-%M-%S").to_string(),
            "now_timestamp" => self.now.timestamp().to_string(),
            _ => return None,
        }))
    }
}

impl PostDownloader {
    pub fn with_download_dir_and_format<T>(download_dir: T, output_format: Option<String>) -> Self
    where
        T: AsRef<Path>,
        PathBuf: From<T>,
    {
        Self {
            client: Client::new(),
            download_dir: Some(download_dir.into()),
            output_format,
            progress_manager: Arc::new(ProgressManager::new()),
        }
    }

    pub async fn download_posts(self: Arc<Self>, posts: Vec<E6Post>) -> Result<()> {
        let cfg = E62Rs::get()?;
        let concurrent_limit = cfg.performance.concurrent_downloads;

        let total_pb = self
            .progress_manager
            .create_count_bar("total", posts.len() as u64, "Total Downloads")
            .await
            .unwrap();

        let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrent_limit));

        let tasks: Vec<_> = posts
            .into_iter()
            .enumerate()
            .map(|(i, post)| {
                let downloader = Arc::clone(&self);
                let semaphore = Arc::clone(&semaphore);
                let total_pb = total_pb.clone();

                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    let result = downloader.download_post(post, i).await;
                    total_pb.inc(1);
                    result
                })
            })
            .collect();

        let results = futures_util::future::join_all(tasks).await;

        total_pb.finish_with_message("✓ All downloads completed");

        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => warn!("Download {} failed: {}", i, e),
                Err(e) => warn!("Task {} failed: {}", i, e),
            }
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn write_to_ads<P: AsRef<Path>>(file_path: P, stream_name: &str, data: &str) -> Result<()> {
        let file_path = file_path.as_ref();
        let ads_path = format!("{}:{}", file_path.display(), stream_name);

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&ads_path)?;

        file.write_all(data.as_bytes())?;
        Ok(())
    }

    pub async fn download_post(&self, post: E6Post, index: usize) -> Result<()> {
        let url = post
            .file
            .url
            .clone()
            .context("Post has no downloadable file URL")?;

        let filename = self.format_filename(&post)?;
        let filepath = self.get_filepath(&filename)?;

        let pb_key = format!("download_{}", index);
        let pb = self
            .progress_manager
            .create_download_bar(
                &pb_key,
                0,
                &format!(
                    "Downloading {}",
                    crate::utils::shorten_path(filename.as_str(), 30)
                ),
            )
            .await
            .unwrap();

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("Failed to download from '{}'", url))?;

        let total_size = response.content_length().unwrap_or(0);
        pb.set_length(total_size);

        let response = response
            .error_for_status()
            .with_context(|| format!("Server returned error for '{}'", url))?;

        self.save_to_file(response, &filepath, pb.clone(), &post)
            .await?;

        pb.finish_with_message(format!("Downloaded {}", filename));
        self.progress_manager.remove_bar(&pb_key).await;

        Ok(())
    }

    async fn save_to_file(
        &self,
        response: reqwest::Response,
        filepath: &Path,
        pb: ProgressBar,
        post: &E6Post,
    ) -> Result<()> {
        let cfg = E62Rs::get()?;
        let dl_cfg = cfg.download;
        let mut file = File::create(filepath)
            .await
            .with_context(|| format!("Failed to create file '{}'", filepath.display()))?;

        #[cfg(target_os = "windows")]
        {
            if dl_cfg.save_metadata {
                crate::utils::write_to_ads(
                    filepath,
                    "metadata",
                    serde_json::to_string_pretty(post)
                        .expect("Failed to serialize")
                        .as_str(),
                )?;
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            if dl_cfg.save_metadata.unwrap_or_default() {
                crate::utils::write_to_json(filepath, serde_json::to_string_pretty(post)?)?;
            }
        }

        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;
        let mut last_update = 0u64;
        const UPDATE_THRESHOLD: u64 = 8192;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Error reading chunk from response")?;
            downloaded += chunk.len() as u64;

            file.write_all(&chunk)
                .await
                .with_context(|| format!("Error writing to file '{}'", filepath.display()))?;

            if downloaded - last_update >= UPDATE_THRESHOLD {
                pb.set_position(downloaded);
                last_update = downloaded;
            }
        }

        file.flush().await.context("Failed to flush file to disk")?;

        Ok(())
    }

    pub fn format_filename(&self, post: &E6Post) -> Result<String> {
        let out_fmt = self.output_format.as_deref().unwrap_or("{id}.{ext}");

        let indexed_re =
            regex::Regex::new(r"\{([a-z_]+)\[([0-9]+(?:\.\.[0-9]*)?|\.\.(?:[0-9]+)?)\]\}").unwrap();

        let simple_re = regex::Regex::new(r"\{([a-z_]+)\}").unwrap();

        let mut formatted = String::from(out_fmt);

        formatted = indexed_re
            .replace_all(&formatted, |caps: &regex::Captures| {
                let key = &caps[1];
                let range_str = &caps[2];
                let ctx = FormatContext::new(post);

                IndexRange::parse(range_str)
                    .and_then(|range| ctx.get_indexed(key, range))
                    .map(|v| v.into_owned())
                    .unwrap_or_else(|| caps[0].to_string())
            })
            .into_owned();

        formatted = simple_re
            .replace_all(&formatted, |caps: &regex::Captures| {
                let key = &caps[1];
                let ctx = FormatContext::new(post);

                ctx.get_simple(key)
                    .map(|v| v.into_owned())
                    .unwrap_or_else(|| caps[0].to_string())
            })
            .into_owned();

        Ok(formatted)
    }

    pub fn get_filepath(&self, filename: &str) -> Result<PathBuf> {
        let filename = sanitize_path(filename);

        let path = if let Some(ref dir) = self.download_dir {
            dir.join(filename)
        } else {
            filename
        };

        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        if path.exists() {
            bail!("File '{}' already exists", path.display());
        }

        Ok(path)
    }
}

impl E6Ui {
    pub async fn redownload_by_artists(&self) -> Result<()> {
        println!("\n=== Update Downloads by Artists ===\n");
        println!(
            "This will scan your downloads, find all artists, and download NEW posts from them."
        );

        let cfg = E62Rs::get()?;
        let dl_cfg = cfg.download;
        let explorer_cfg = cfg.explorer;
        let download_dir = std::path::Path::new(&dl_cfg.download_dir);

        if !download_dir.exists() {
            bail!(
                "Download directory does not exist: {}",
                download_dir.display()
            );
        }

        println!("Scanning downloaded posts for artist names and post IDs...\n");
        let progress_manager = Arc::new(ProgressManager::new());

        let local_posts = self
            .scan_downloads_directory(download_dir, &explorer_cfg)
            .await?;

        if local_posts.is_empty() {
            println!("No posts with metadata found in {}", download_dir.display());
            return Ok(());
        }

        let mut artist_post_counts: HashMap<String, usize> = HashMap::new();
        let mut downloaded_post_ids: HashSet<i64> = HashSet::new();

        let special_tags = HashSet::from([
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

        let artist_options: Vec<DemandOption<String>> = sorted_artists
            .iter()
            .map(|(artist, count)| {
                DemandOption::new((*artist).clone())
                    .label(&format!(
                        "{} ({} downloaded post{})",
                        artist,
                        count,
                        if **count == 1 { "" } else { "s" }
                    ))
                    .selected(true)
            })
            .collect();

        let selected_artists = MultiSelect::new(
            "Select artists to check for new posts (Space to toggle, Enter to confirm):",
        )
        .description("All artists are selected by default. Deselect any you don't want to update.")
        .options(artist_options)
        .theme(&ROSE_PINE)
        .filterable(true)
        .run()
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

        println!();
        let confirm = Confirm::new(format!(
            "Check for and download new posts from these {} artist{}?",
            selected_artists.len(),
            if selected_artists.len() == 1 { "" } else { "s" }
        ))
        .affirmative("Yes")
        .negative("No")
        .theme(&ROSE_PINE)
        .run()?;

        if !confirm {
            println!("Operation cancelled.");
            return Ok(());
        }

        let limit_per_artist =
            demand::Input::new("Maximum NEW posts per artist to download (leave empty for all):")
                .theme(&ROSE_PINE)
                .placeholder("e.g., 50")
                .validation(|input| {
                    if input.is_empty() {
                        return Ok(());
                    }
                    if input.parse::<u64>().is_ok() {
                        Ok(())
                    } else {
                        Err("Please enter a valid number or leave empty")
                    }
                })
                .run()?;

        let limit: Option<u64> = if limit_per_artist.is_empty() {
            None
        } else {
            Some(limit_per_artist.parse()?)
        };

        println!("\n{} Checking for new posts...\n", "→".bright_cyan());

        let total_pb = progress_manager
            .create_count_bar(
                "artists",
                selected_artists.len() as u64,
                "Processing artists",
            )
            .await?;

        let mut total_new_posts = 0u64;
        let mut total_already_downloaded = 0u64;
        let mut total_errors = 0u64;
        type U64x2 = Result<(u64, u64), String>;
        let mut artist_results: Vec<(String, U64x2)> = Vec::new();

        for artist in selected_artists {
            total_pb.set_message(format!("Processing artist: {}", artist));

            match self
                .download_new_artist_posts(&artist, limit, &downloaded_post_ids)
                .await
            {
                Ok((new_count, skipped_count)) => {
                    total_new_posts += new_count;
                    total_already_downloaded += skipped_count;
                    artist_results.push((artist.clone(), Ok((new_count, skipped_count))));
                }
                Err(e) => {
                    total_errors += 1;
                    let error_msg = e.to_string();
                    artist_results.push((artist.clone(), Err(error_msg.clone())));
                    warn!("Failed to check posts from {}: {}", artist, error_msg);
                }
            }

            total_pb.inc(1);
        }

        total_pb.finish_with_message(format!(
            "✓ Processed {} artist{}",
            artist_results.len(),
            if artist_results.len() == 1 { "" } else { "s" }
        ));

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
                if let Ok((new_count, skipped)) = result {
                    if *new_count > 0 {
                        Some((artist, new_count, skipped))
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
            for (i, (artist, new_count, skipped)) in with_new_posts.iter().enumerate() {
                if i < 15 {
                    println!(
                        "  • {}: {} new post{} ({} already had)",
                        artist.green(),
                        new_count.to_string().bright_green().bold(),
                        if **new_count == 1 { "" } else { "s" },
                        skipped
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
                if let Ok((new_count, skipped)) = result {
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

    async fn download_new_artist_posts(
        &self,
        artist: &str,
        limit: Option<u64>,
        downloaded_post_ids: &HashSet<i64>,
    ) -> Result<(u64, u64)> {
        let search_tags = vec![format!("~{}", artist), format!("~{}_(artist)", artist)];

        let mut new_posts = Vec::new();
        let mut skipped_count = 0u64;
        let mut before_id: Option<i64> = None;
        let max_fetch = limit.unwrap_or(u64::MAX);

        let mut consecutive_empty = 0;
        const MAX_CONSECUTIVE_EMPTY: i32 = 3;

        loop {
            let results = self
                .client
                .search_posts(search_tags.clone(), Some(320), before_id)
                .await?;

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
                } else {
                    new_posts.push(post.clone());
                    found_new_in_batch = true;

                    if new_posts.len() >= max_fetch as usize {
                        break;
                    }
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

            if new_posts.len() >= max_fetch as usize || batch_size < 320 {
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        if let Some(lim) = limit {
            new_posts.truncate(lim as usize);
        }

        let new_count = new_posts.len() as u64;

        if !new_posts.is_empty() {
            self.downloader.clone().download_posts(new_posts).await?;
        }

        Ok((new_count, skipped_count))
    }
}

use {
    crate::{
        config::options::E62Rs,
        models::E6Post,
        ui::{E6Ui, progress::ProgressManager},
    },
    color_eyre::eyre::{Context, ContextCompat, Result, bail},
    futures::StreamExt,
    indicatif::ProgressBar,
    reqwest::Client,
    std::{
        borrow::Cow,
        fs::OpenOptions,
        io::Write,
        path::{Path, PathBuf},
        sync::Arc,
    },
    tokio::{fs::File, io::AsyncWriteExt},
    tracing::warn,
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
    pub async fn download_posts(&self, posts: Vec<E6Post>) -> Result<()> {
        println!("Downloading {} posts...", posts.len());

        self.downloader.clone().download_posts(posts).await?;

        println!("\n✓ Batch download complete");
        Ok(())
    }

    pub async fn download_post(&self, post: E6Post) -> Result<()> {
        self.download_posts(vec![post]).await
    }
}

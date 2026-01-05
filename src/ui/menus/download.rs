//! post downloading stuff
use {
    crate::{
        config::format::FormatTemplate, getopt, models::E6Post, ui::progress::ProgressManager,
        utils,
    },
    color_eyre::eyre::{Context, ContextCompat, Result, bail},
    futures::StreamExt,
    hashbrown::HashMap,
    indicatif::ProgressBar,
    reqwest::Client,
    std::{
        path::{Path, PathBuf},
        sync::Arc,
    },
    tokio::{fs::File, io::AsyncWriteExt},
    tracing::warn,
    url::Url,
};

/// a post downloader
#[derive(Default, Clone)]
pub struct PostDownloader {
    /// the http client
    client: Client,
    /// the path to download to
    download_dir: Option<PathBuf>,
    /// the format template to use for output
    output_format: Option<String>,
    /// the progress bar manager
    progress_manager: Arc<ProgressManager>,
}

/// sanitize a value for use in filenames (before template substitution)
fn sanitize_value<S: AsRef<str>>(input: S) -> String {
    let s = input.as_ref();
    let mut sanitized = String::with_capacity(s.len());

    for ch in s.chars() {
        let chstr = ch.to_string();
        sanitized.push_str(match ch {
            '/' | '\\' => " on ",
            #[cfg(target_os = "windows")]
            '<' => "＜",
            #[cfg(target_os = "windows")]
            '>' => "＞",
            #[cfg(target_os = "windows")]
            ':' => "：",
            #[cfg(target_os = "windows")]
            '"' => "＂",
            #[cfg(target_os = "windows")]
            '|' => "｜",
            #[cfg(target_os = "windows")]
            '?' => "？",
            #[cfg(target_os = "windows")]
            '*' => "＊",
            _ => &chstr,
        });
    }

    sanitized
}

/// sanitize a path for fs compatibility (only OS-specific issues, not `/`)
fn sanitize_path<S: AsRef<str>>(input: S) -> PathBuf {
    let s = input.as_ref();

    #[cfg(target_os = "windows")]
    {
        let mut sanitized = String::with_capacity(s.len());
        for ch in s.chars() {
            let chstr = ch.to_string();
            sanitized.push_str(match ch {
                '<' => "＜",
                '>' => "＞",
                ':' => "：",
                '"' => "＂",
                '|' => "｜",
                '?' => "？",
                '*' => "＊",
                _ => &chstr,
            });
        }
        PathBuf::from(sanitized)
    }

    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from(s)
    }
}

impl PostDownloader {
    /// make a new post downloader with a given download dir and output format
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

    /// download multiple posts
    pub async fn download_posts(self: Arc<Self>, posts: Vec<E6Post>) -> Result<()> {
        let concurrent_limit = getopt!(download.threads);

        let total_pb = self
            .progress_manager
            .create_count_bar("total", posts.len() as u64, "Total Downloads")
            .await?;

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
                Ok(Err(e)) => warn!("download {} failed: {}", i, e),
                Err(e) => warn!("task {} failed: {}", i, e),
            }
        }

        Ok(())
    }

    /// download an individual post
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
            .mk_dl_bar(
                &pb_key,
                0,
                &format!(
                    "Downloading {}",
                    crate::utils::shorten_path(filename.as_str(), 30)
                ),
            )
            .await?;

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context(format!("Failed to download from '{}'", url))?;

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

    /// save a post to a file
    async fn save_to_file(
        &self,
        response: reqwest::Response,
        filepath: &Path,
        pb: ProgressBar,
        post: &E6Post,
    ) -> Result<()> {
        let mut file = File::create(filepath)
            .await
            .context(format!("Failed to create file '{}'", filepath.display()))?;

        if getopt!(download.save_metadata) {
            let metadata =
                serde_json::to_string_pretty(post).context("Failed to serialize post metadata")?;

            #[cfg(target_os = "windows")]
            {
                utils::write_to_ads(filepath, "metadata", &metadata)?;
            }

            #[cfg(not(target_os = "windows"))]
            {
                utils::write_to_json(filepath, &metadata)?;
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

    /// format the filename based on the post metadata
    pub fn format_filename(&self, post: &E6Post) -> Result<String> {
        let out_fmt = self.output_format.as_deref().unwrap_or("$id.$ext");
        let template = FormatTemplate::parse(out_fmt).context("failed to parse output format")?;
        let (simple_context, array_context) = build_context_from_post(post);
        let formatted = template
            .render_with_arrays(&simple_context, &array_context)
            .context("Failed to render filename template")?;

        Ok(formatted)
    }

    /// get the path to a file
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
                .context(format!("Failed to create directory: {}", parent.display()))?;
        }

        if path.exists() {
            bail!("File '{}' already exists", path.display());
        }

        Ok(path)
    }

    /// make a new downloader for a pool, saving to `<download_dir>/<pool_name>`
    pub fn for_pool<T, S>(base_download_dir: T, pool_name: S) -> Self
    where
        T: AsRef<Path>,
        S: AsRef<str>,
    {
        let sanitized_name = sanitize_pool_name(pool_name.as_ref());
        let pool_dir = base_download_dir.as_ref().join(sanitized_name);

        Self {
            client: Client::new(),
            download_dir: Some(pool_dir),
            output_format: None,
            progress_manager: Arc::new(ProgressManager::new()),
        }
    }

    /// download pool posts with sequential naming based on pool order
    pub async fn download_pool_posts(self: Arc<Self>, posts: Vec<E6Post>) -> Result<()> {
        let concurrent_limit = getopt!(download.threads);
        let total = posts.len();
        let pad_width = total.to_string().len().max(3);

        let total_pb = self
            .progress_manager
            .create_count_bar("total", total as u64, "Total Downloads")
            .await?;

        let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrent_limit));

        let tasks: Vec<_> = posts
            .into_iter()
            .enumerate()
            .map(|(i, post)| {
                let downloader = Arc::clone(&self);
                let semaphore = Arc::clone(&semaphore);
                let total_pb = total_pb.clone();
                let sequence_num = i + 1;

                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    let result = downloader
                        .download_pool_post(post, sequence_num, pad_width)
                        .await;
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
                Ok(Err(e)) => warn!("download {} failed: {}", i + 1, e),
                Err(e) => warn!("task {} failed: {}", i + 1, e),
            }
        }

        Ok(())
    }

    /// download a single pool post with sequential naming
    async fn download_pool_post(
        &self,
        post: E6Post,
        sequence_num: usize,
        pad_width: usize,
    ) -> Result<()> {
        let url = post
            .file
            .url
            .clone()
            .context("Post has no downloadable file URL")?;

        let filename = format!(
            "{:0width$}.{}",
            sequence_num,
            post.file.ext,
            width = pad_width
        );
        let filepath = self.get_filepath(&filename)?;

        let pb_key = format!("download_{}", sequence_num);
        let pb = self
            .progress_manager
            .mk_dl_bar(&pb_key, 0, &format!("Downloading {}", filename))
            .await?;

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context(format!("Failed to download from '{}'", url))?;

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
}

/// build template context based on post metadata
fn build_context_from_post(
    post: &E6Post,
) -> (HashMap<String, String>, HashMap<String, Vec<String>>) {
    let mut simple = HashMap::new();
    let mut arrays = HashMap::new();
    let now = chrono::Local::now();

    let mut insert = |key: &str, value: String| {
        simple.insert(key.to_string(), sanitize_value(&value));
    };

    insert("id", post.id.to_string());
    insert("md5", post.file.md5.to_string());
    insert("ext", post.file.ext.to_string());
    insert("width", post.file.width.to_string());
    insert("height", post.file.height.to_string());
    insert("size", post.file.size.to_string());
    insert(
        "size_mb",
        format!("{:.2}", post.file.size as f64 / (1024.0 * 1024.0)),
    );
    insert("size_kb", format!("{:.2}", post.file.size as f64 / 1024.0));

    let rating_full = match post.rating.as_str() {
        "e" => "explicit",
        "q" => "questionable",
        "s" => "safe",
        _ => "unknown",
    };
    insert("rating", rating_full.to_string());
    insert(
        "rating_first",
        post.rating
            .chars()
            .next()
            .unwrap_or('u')
            .to_lowercase()
            .to_string(),
    );

    insert("score", post.score.total.to_string());
    insert("score_up", post.score.up.to_string());
    insert("score_down", post.score.down.to_string());
    insert("fav_count", post.fav_count.to_string());
    insert("comment_count", post.comment_count.to_string());

    let ratio = if post.file.height > 0 {
        post.file.width as f64 / post.file.height as f64
    } else {
        0.0
    };
    insert("aspect_ratio", format!("{:.2}", ratio));

    let orientation = if post.file.width > post.file.height {
        "landscape"
    } else if post.file.width < post.file.height {
        "portrait"
    } else {
        "square"
    };
    insert("orientation", orientation.to_string());

    let resolution = match (post.file.width, post.file.height) {
        (w, h) if w >= 7680 || h >= 4320 => "8K",
        (w, h) if w >= 3840 || h >= 2160 => "4K",
        (w, h) if w >= 2560 || h >= 1440 => "QHD",
        (w, h) if w >= 1920 || h >= 1080 => "FHD",
        (w, h) if w >= 1280 || h >= 720 => "HD",
        _ => "SD",
    };
    insert("resolution", resolution.to_string());
    insert(
        "megapixels",
        format!(
            "{:.1}",
            (post.file.width * post.file.height) as f64 / 1_000_000.0
        ),
    );

    insert(
        "artist",
        post.tags
            .artist
            .first()
            .cloned()
            .unwrap_or_else(|| "unknown".to_string()),
    );
    insert("artist_count", post.tags.artist.len().to_string());

    let total_tags = post.tags.general.len()
        + post.tags.artist.len()
        + post.tags.character.len()
        + post.tags.species.len()
        + post.tags.copyright.len()
        + post.tags.meta.len()
        + post.tags.lore.len();
    insert("tag_count", total_tags.to_string());
    insert("tag_count_general", post.tags.general.len().to_string());
    insert("tag_count_character", post.tags.character.len().to_string());
    insert("tag_count_species", post.tags.species.len().to_string());
    insert("tag_count_copyright", post.tags.copyright.len().to_string());

    insert("pool_count", post.pools.len().to_string());
    insert(
        "pool_ids",
        post.pools
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(","),
    );

    insert("uploader", post.uploader_name.clone());
    insert("uploader_id", post.uploader_id.to_string());
    insert(
        "approver_id",
        post.approver_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "none".to_string()),
    );

    insert(
        "has_children",
        if post.relationships.has_children {
            "yes"
        } else {
            "no"
        }
        .to_string(),
    );
    insert(
        "parent_id",
        post.relationships
            .parent_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "none".to_string()),
    );

    insert(
        "is_pending",
        if post.flags.pending { "yes" } else { "no" }.to_string(),
    );
    insert(
        "is_flagged",
        if post.flags.flagged { "yes" } else { "no" }.to_string(),
    );
    insert(
        "is_deleted",
        if post.flags.deleted { "yes" } else { "no" }.to_string(),
    );
    insert(
        "has_notes",
        if post.has_notes { "yes" } else { "no" }.to_string(),
    );

    if let Some(duration) = post.duration {
        insert("duration", duration.to_string());
        let total_seconds = duration as i64;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        let formatted = if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        };
        insert("duration_formatted", formatted);
    } else {
        insert("duration", "0".to_string());
        insert("duration_formatted", "N/A".to_string());
    }

    let file_type = match post.file.ext.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => "image",
        "mp4" | "webm" | "mov" | "avi" | "mkv" => "video",
        "swf" => "flash",
        _ => "unknown",
    };
    insert("file_type", file_type.to_string());

    if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&post.created_at) {
        insert("year", created.format("%Y").to_string());
        insert("month", created.format("%m").to_string());
        insert("day", created.format("%d").to_string());
        insert("hour", created.format("%H").to_string());
        insert("minute", created.format("%M").to_string());
        insert("second", created.format("%S").to_string());
        insert("date", created.format("%Y-%m-%d").to_string());
        insert("time", created.format("%H-%M-%S").to_string());
        insert("datetime", created.format("%Y-%m-%d %H-%M-%S").to_string());
        insert("timestamp", created.timestamp().to_string());
    }

    if let Ok(updated) = chrono::DateTime::parse_from_rfc3339(&post.updated_at) {
        insert("year_updated", updated.format("%Y").to_string());
        insert("month_updated", updated.format("%m").to_string());
        insert("day_updated", updated.format("%d").to_string());
        insert("date_updated", updated.format("%Y-%m-%d").to_string());
    }

    insert("now_year", now.format("%Y").to_string());
    insert("now_month", now.format("%m").to_string());
    insert("now_day", now.format("%d").to_string());
    insert("now_hour", now.format("%H").to_string());
    insert("now_minute", now.format("%M").to_string());
    insert("now_second", now.format("%S").to_string());
    insert("now_date", now.format("%Y-%m-%d").to_string());
    insert("now_time", now.format("%H-%M-%S").to_string());
    insert("now_datetime", now.format("%Y-%m-%d %H-%M-%S").to_string());
    insert("now_timestamp", now.timestamp().to_string());

    arrays.insert(
        "tags".to_string(),
        post.tags.general.iter().map(sanitize_value).collect(),
    );

    if post.tags.artist.is_empty() {
        arrays.insert("artists".to_string(), vec!["NA".to_string()]);
    } else {
        arrays.insert(
            "artists".to_string(),
            post.tags.artist.iter().map(sanitize_value).collect(),
        );
    }

    arrays.insert(
        "characters".to_string(),
        post.tags.character.iter().map(sanitize_value).collect(),
    );
    arrays.insert(
        "species".to_string(),
        post.tags.species.iter().map(sanitize_value).collect(),
    );
    arrays.insert(
        "copyright".to_string(),
        post.tags.copyright.iter().map(sanitize_value).collect(),
    );

    let sources: Vec<String> = post
        .sources
        .iter()
        .filter_map(|source| {
            Url::parse(source)
                .ok()
                .and_then(|u| u.domain().map(sanitize_value))
        })
        .collect();
    arrays.insert("sources".to_string(), sources);

    (simple, arrays)
}

/// sanitize a pool name for use as a directory name
pub fn sanitize_pool_name<S: AsRef<str>>(name: S) -> String {
    let s = name.as_ref();
    let mut sanitized = String::with_capacity(s.len());

    for ch in s.chars() {
        let replacement = match ch {
            '<' | '>' | ':' | '"' | '|' | '?' | '*' => '_',
            '/' | '\\' => '_',
            c if c.is_control() => continue,
            ' ' => '_',
            _ => ch,
        };
        sanitized.push(replacement);
    }

    let mut result = String::with_capacity(sanitized.len());
    let mut last_was_underscore = false;
    for ch in sanitized.chars() {
        if ch == '_' {
            if !last_was_underscore {
                result.push(ch);
            }
            last_was_underscore = true;
        } else {
            result.push(ch);
            last_was_underscore = false;
        }
    }

    result.trim_matches('_').to_string()
}

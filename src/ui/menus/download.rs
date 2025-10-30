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
            .create_bar("total", posts.len() as u64, "Total Downloads")
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

        total_pb.finish_with_message("All downloads completed");

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
            .create_bar(
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
        let out_fmt = self.output_format.as_deref().unwrap_or("$id.$ext");
        let artist = post
            .tags
            .artist
            .first()
            .map(|s| s.as_str())
            .unwrap_or("unknown");

        let tags_re = regex::Regex::new(r"\$tags\[(\d+)\]").unwrap();
        let artists_re = regex::Regex::new(r"\$artists\[(\d+)\]").unwrap();
        let characters_re = regex::Regex::new(r"\$characters\[(\d+)\]").unwrap();
        let species_re = regex::Regex::new(r"\$species\[(\d+)\]").unwrap();
        let copyright_re = regex::Regex::new(r"\$copyright\[(\d+)\]").unwrap();
        let sources_re = regex::Regex::new(r"\$sources\[(\d+)\]").unwrap();

        let mut formatted = out_fmt.to_string();

        for cap in tags_re.captures_iter(out_fmt) {
            if let Some(num_match) = cap.get(1)
                && let Ok(num_tags) = num_match.as_str().parse::<usize>()
            {
                let tags = post
                    .tags
                    .general
                    .iter()
                    .take(num_tags)
                    .cloned()
                    .collect::<Vec<String>>()
                    .join(", ");
                formatted = formatted.replace(&cap[0], &tags);
            }
        }

        for cap in artists_re.captures_iter(out_fmt) {
            if let Some(num_match) = cap.get(1)
                && let Ok(num_artists) = num_match.as_str().parse::<usize>()
            {
                let artists = post
                    .tags
                    .artist
                    .iter()
                    .take(num_artists)
                    .cloned()
                    .collect::<Vec<String>>()
                    .join(", ");
                formatted = formatted.replace(&cap[0], &artists);
            }
        }

        for cap in characters_re.captures_iter(out_fmt) {
            if let Some(num_match) = cap.get(1)
                && let Ok(num_chars) = num_match.as_str().parse::<usize>()
            {
                let characters = post
                    .tags
                    .character
                    .iter()
                    .take(num_chars)
                    .cloned()
                    .collect::<Vec<String>>()
                    .join(", ");
                formatted = formatted.replace(&cap[0], &characters);
            }
        }

        for cap in species_re.captures_iter(out_fmt) {
            if let Some(num_match) = cap.get(1)
                && let Ok(num_species) = num_match.as_str().parse::<usize>()
            {
                let species = post
                    .tags
                    .species
                    .iter()
                    .take(num_species)
                    .cloned()
                    .collect::<Vec<String>>()
                    .join(", ");
                formatted = formatted.replace(&cap[0], &species);
            }
        }

        for cap in copyright_re.captures_iter(out_fmt) {
            if let Some(num_match) = cap.get(1)
                && let Ok(num_copyright) = num_match.as_str().parse::<usize>()
            {
                let copyright = post
                    .tags
                    .copyright
                    .iter()
                    .take(num_copyright)
                    .cloned()
                    .collect::<Vec<String>>()
                    .join(", ");
                formatted = formatted.replace(&cap[0], &copyright);
            }
        }

        for cap in sources_re.captures_iter(out_fmt) {
            if let Some(num_match) = cap.get(1)
                && let Ok(num_sources) = num_match.as_str().parse::<usize>()
            {
                let sources = post
                    .sources
                    .iter()
                    .take(num_sources)
                    .map(|source| {
                        Url::parse(source)
                            .ok()
                            .and_then(|u| u.domain().map(String::from))
                            .unwrap_or_else(|| "unknown".to_string())
                    })
                    .collect::<Vec<String>>()
                    .join(", ");
                formatted = formatted.replace(&cap[0], &sources);
            }
        }

        let aspect_ratio = if post.file.height > 0 {
            post.file.width as f64 / post.file.height as f64
        } else {
            0.0
        };

        let orientation = if post.file.width > post.file.height {
            "landscape"
        } else if post.file.width < post.file.height {
            "portrait"
        } else {
            "square"
        };

        let megapixels = (post.file.width * post.file.height) as f64 / 1_000_000.0;

        let resolution = match (post.file.width, post.file.height) {
            (w, h) if w >= 7680 || h >= 4320 => "8K",
            (w, h) if w >= 3840 || h >= 2160 => "4K",
            (w, h) if w >= 2560 || h >= 1440 => "QHD",
            (w, h) if w >= 1920 || h >= 1080 => "FHD",
            (w, h) if w >= 1280 || h >= 720 => "HD",
            _ => "SD",
        };

        let size_mb = post.file.size as f64 / (1024.0 * 1024.0);
        let size_kb = post.file.size as f64 / 1024.0;

        let file_type = match post.file.ext.as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => "image",
            "mp4" | "webm" | "mov" | "avi" | "mkv" => "video",
            "swf" => "flash",
            _ => "unknown",
        };

        let duration_formatted = if let Some(duration) = post.duration {
            let total_seconds = duration as i64;
            let hours = total_seconds / 3600;
            let minutes = (total_seconds % 3600) / 60;
            let seconds = total_seconds % 60;

            if hours > 0 {
                format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
            } else {
                format!("{:02}:{:02}", minutes, seconds)
            }
        } else {
            "N/A".to_string()
        };

        let tag_count = post.tags.general.len()
            + post.tags.artist.len()
            + post.tags.character.len()
            + post.tags.species.len()
            + post.tags.copyright.len()
            + post.tags.meta.len()
            + post.tags.lore.len();

        let now = chrono::Local::now();
        let rating_first = post
            .rating
            .chars()
            .next()
            .unwrap_or('u')
            .to_lowercase()
            .to_string();

        let (year, month, day, hour, minute, second, timestamp) =
            if let Ok(created_date) = chrono::DateTime::parse_from_rfc3339(&post.created_at) {
                (
                    created_date.format("%Y").to_string(),
                    created_date.format("%m").to_string(),
                    created_date.format("%d").to_string(),
                    created_date.format("%H").to_string(),
                    created_date.format("%M").to_string(),
                    created_date.format("%S").to_string(),
                    created_date.timestamp().to_string(),
                )
            } else {
                (
                    now.format("%Y").to_string(),
                    now.format("%m").to_string(),
                    now.format("%d").to_string(),
                    now.format("%H").to_string(),
                    now.format("%M").to_string(),
                    now.format("%S").to_string(),
                    now.timestamp().to_string(),
                )
            };

        let (year_updated, month_updated, day_updated) =
            if let Ok(updated_date) = chrono::DateTime::parse_from_rfc3339(&post.updated_at) {
                (
                    updated_date.format("%Y").to_string(),
                    updated_date.format("%m").to_string(),
                    updated_date.format("%d").to_string(),
                )
            } else {
                (year.clone(), month.clone(), day.clone())
            };

        let rating_full = match post.rating.as_str() {
            "e" => "explicit",
            "q" => "questionable",
            "s" => "safe",
            _ => "unknown",
        };

        let pool_ids = post
            .pools
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let approver_id = post
            .approver_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "none".to_string());

        let formatted = formatted
            .replace("$id", &post.id.to_string())
            .replace("$rating", rating_full)
            .replace("$rating_first", &rating_first)
            .replace("$score", &post.score.total.to_string())
            .replace("$score_up", &post.score.up.to_string())
            .replace("$score_down", &post.score.down.to_string())
            .replace("$fav_count", &post.fav_count.to_string())
            .replace("$comment_count", &post.comment_count.to_string())
            .replace("$md5", &post.file.md5)
            .replace("$ext", &post.file.ext)
            .replace("$width", &post.file.width.to_string())
            .replace("$height", &post.file.height.to_string())
            .replace("$aspect_ratio", &format!("{:.2}", aspect_ratio))
            .replace("$orientation", orientation)
            .replace("$resolution", resolution)
            .replace("$megapixels", &format!("{:.1}", megapixels))
            .replace("$size", &post.file.size.to_string())
            .replace("$size_mb", &format!("{:.2}", size_mb))
            .replace("$size_kb", &format!("{:.2}", size_kb))
            .replace("$artist", artist)
            .replace("$artist_count", &post.tags.artist.len().to_string())
            .replace("$tag_count", &tag_count.to_string())
            .replace("$tag_count_general", &post.tags.general.len().to_string())
            .replace(
                "$tag_count_character",
                &post.tags.character.len().to_string(),
            )
            .replace("$tag_count_species", &post.tags.species.len().to_string())
            .replace(
                "$tag_count_copyright",
                &post.tags.copyright.len().to_string(),
            )
            .replace("$pool_ids", &pool_ids)
            .replace("$pool_count", &post.pools.len().to_string())
            .replace("$uploader", &post.uploader_name)
            .replace("$uploader_id", &post.uploader_id.to_string())
            .replace("$approver_id", &approver_id)
            .replace(
                "$has_children",
                if post.relationships.has_children {
                    "yes"
                } else {
                    "no"
                },
            )
            .replace(
                "$parent_id",
                &post
                    .relationships
                    .parent_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "none".to_string()),
            )
            .replace("$year", &year)
            .replace("$month", &month)
            .replace("$day", &day)
            .replace("$hour", &hour)
            .replace("$minute", &minute)
            .replace("$second", &second)
            .replace("$date", &format!("{}-{}-{}", year, month, day))
            .replace("$time", &format!("{}-{}-{}", hour, minute, second))
            .replace(
                "$datetime",
                &format!("{}-{}-{} {}-{}-{}", year, month, day, hour, minute, second),
            )
            .replace("$timestamp", &timestamp)
            .replace("$year_updated", &year_updated)
            .replace("$month_updated", &month_updated)
            .replace("$day_updated", &day_updated)
            .replace(
                "$date_updated",
                &format!("{}-{}-{}", year_updated, month_updated, day_updated),
            )
            .replace("$now_year", &now.format("%Y").to_string())
            .replace("$now_month", &now.format("%m").to_string())
            .replace("$now_day", &now.format("%d").to_string())
            .replace("$now_hour", &now.format("%H").to_string())
            .replace("$now_minute", &now.format("%M").to_string())
            .replace("$now_second", &now.format("%S").to_string())
            .replace("$now_date", &now.format("%Y-%m-%d").to_string())
            .replace("$now_time", &now.format("%H-%M-%S").to_string())
            .replace(
                "$now_datetime",
                &now.format("%Y-%m-%d %H-%M-%S").to_string(),
            )
            .replace("$now_timestamp", &now.timestamp().to_string())
            .replace("$is_pending", if post.flags.pending { "yes" } else { "no" })
            .replace("$is_flagged", if post.flags.flagged { "yes" } else { "no" })
            .replace("$is_deleted", if post.flags.deleted { "yes" } else { "no" })
            .replace("$has_notes", if post.has_notes { "yes" } else { "no" })
            .replace(
                "$duration",
                &post
                    .duration
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "0".to_string()),
            )
            .replace("$duration_formatted", &duration_formatted)
            .replace("$file_type", file_type);

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

use crate::{progress::ProgressManager, ui::E6Ui};
use anyhow::{Context, Result};
use e6cfg::Cfg;
use e6core::models::E6Post;
use futures_util::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{fs::File, io::AsyncWriteExt};

#[derive(Default, Clone)]
pub struct PostDownloader {
    client: reqwest::Client,
    download_dir: Option<PathBuf>,
    output_format: Option<String>,
    progress_manager: Arc<ProgressManager>,
}

impl PostDownloader {
    pub fn with_download_dir_and_format<T>(download_dir: T, output_format: Option<String>) -> Self
    where
        T: AsRef<Path>,
        PathBuf: From<T>,
    {
        Self {
            client: reqwest::Client::new(),
            download_dir: Some(download_dir.into()),
            output_format,
            progress_manager: Arc::new(ProgressManager::new()),
        }
    }

    pub async fn download_posts(self: Arc<Self>, posts: Vec<E6Post>) -> Result<()> {
        let cfg = Cfg::get().unwrap_or_default();
        let concurrent_limit = cfg
            .performance
            .as_ref()
            .and_then(|p| p.concurrent_downloads)
            .unwrap_or(8);

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
                Ok(Err(e)) => log::warn!("Download {} failed: {}", i, e),
                Err(e) => log::warn!("Task {} failed: {}", i, e),
            }
        }

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
            .create_bar(&pb_key, 0, &format!("Downloading {}", filename))
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

        self.save_to_file(response, &filepath, pb.clone()).await?;

        pb.finish_with_message(format!("Downloaded {}", filename));
        self.progress_manager.remove_bar(&pb_key).await;

        Ok(())
    }

    async fn save_to_file(
        &self,
        response: reqwest::Response,
        filepath: &Path,
        pb: ProgressBar,
    ) -> Result<()> {
        let mut file = File::create(filepath)
            .await
            .with_context(|| format!("Failed to create file '{}'", filepath.display()))?;

        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Error reading chunk from response")?;
            downloaded += chunk.len() as u64;

            file.write_all(&chunk)
                .await
                .with_context(|| format!("Error writing to file '{}'", filepath.display()))?;

            pb.set_position(downloaded);
        }

        file.flush().await.context("Failed to flush file to disk")?;

        Ok(())
    }

    pub fn format_filename(&self, post: &E6Post) -> Result<String> {
        let format = self.output_format.as_deref().unwrap_or("$id.$ext");
        let artist = post
            .tags
            .artist
            .first()
            .map(|s| s.as_str())
            .unwrap_or("unknown");

        let tags_re = regex::Regex::new(r"\$tags\[(\d+)\]").unwrap();
        let artists_re = regex::Regex::new(r"\$artists\[(\d+)\]").unwrap();
        let mut formatted = format.to_string();

        for cap in tags_re.captures_iter(format) {
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

        for cap in artists_re.captures_iter(format) {
            if let Some(num_match) = cap.get(1)
                && let Ok(num_artists) = num_match.as_str().parse::<usize>()
            {
                let tags = post
                    .tags
                    .artist
                    .iter()
                    .take(num_artists)
                    .cloned()
                    .collect::<Vec<String>>()
                    .join(", ");

                formatted = formatted.replace(&cap[0], &tags);
            }
        }

        let now = chrono::Local::now();
        let rating_first = post
            .rating
            .chars()
            .next()
            .unwrap_or('u')
            .to_lowercase()
            .to_string();

        let (year, month, day, hour, minute, second) =
            if let Ok(created_date) = chrono::DateTime::parse_from_rfc3339(&post.created_at) {
                (
                    created_date.format("%Y").to_string(),
                    created_date.format("%m").to_string(),
                    created_date.format("%d").to_string(),
                    created_date.format("%H").to_string(),
                    created_date.format("%M").to_string(),
                    created_date.format("%S").to_string(),
                )
            } else {
                (
                    now.format("%Y").to_string(),
                    now.format("%m").to_string(),
                    now.format("%d").to_string(),
                    now.format("%H").to_string(),
                    now.format("%M").to_string(),
                    now.format("%S").to_string(),
                )
            };

        let rating_full = match post.rating.as_str() {
            "e" => "explicit",
            "q" => "questionable",
            "s" => "safe",
            _ => "unknown",
        };

        let formatted = formatted
            .replace("$id", &post.id.to_string())
            .replace("$rating", rating_full)
            .replace("$rating_first", &rating_first)
            .replace("$score", &post.score.total.to_string())
            .replace("$fav_count", &post.fav_count.to_string())
            .replace("$comment_count", &post.comment_count.to_string())
            .replace("$md5", &post.file.md5)
            .replace("$ext", &post.file.ext)
            .replace("$width", &post.file.width.to_string())
            .replace("$height", &post.file.height.to_string())
            .replace("$size", &post.file.size.to_string())
            .replace("$artist", artist)
            .replace("$uploader", &post.uploader_name)
            .replace("$uploader_id", &post.uploader_id.to_string())
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
            );

        Ok(formatted)
    }

    pub fn get_filepath(&self, filename: &str) -> Result<PathBuf> {
        let path = if let Some(ref dir) = self.download_dir {
            dir.join(filename)
        } else {
            PathBuf::from(filename)
        };

        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        if path.exists() {
            anyhow::bail!("File '{}' already exists", path.display());
        }

        Ok(path)
    }
}

impl E6Ui {
    pub async fn download_posts(&self, posts: Vec<E6Post>) -> Result<()> {
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

        self.downloader.clone().download_posts(posts).await?;

        println!("\n✓ Batch download complete");
        Ok(())
    }

    pub async fn download_post(&self, post: E6Post) -> Result<()> {
        self.download_posts(vec![post]).await
    }
}

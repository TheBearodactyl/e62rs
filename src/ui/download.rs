use crate::models::E6Post;
use anyhow::{Context, Result};
use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use tokio::{fs::File, io::AsyncWriteExt};

#[derive(Default)]
pub struct PostDownloader {
    client: reqwest::Client,
    download_dir: Option<PathBuf>,
    output_format: Option<String>,
}

impl PostDownloader {
    #[allow(unused)]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            download_dir: None,
            output_format: None,
        }
    }

    pub fn with_download_dir_and_format<T>(download_dir: T, output_format: Option<String>) -> Self
    where
        T: AsRef<Path>,
        PathBuf: From<T>,
    {
        Self {
            client: reqwest::Client::new(),
            download_dir: Some(download_dir.into()),
            output_format,
        }
    }

    pub async fn download_post(&self, post: E6Post) -> Result<()> {
        let url = post
            .file
            .url
            .clone()
            .context("Post has no downloadable file URL")?;

        let filename = self.format_filename(&post)?;
        let filepath = self.get_filepath(&filename)?;

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("Failed to download from '{}'", url))?;

        let total_size = response.content_length();

        let response = response
            .error_for_status()
            .with_context(|| format!("Server returned error for '{}'", url))?;

        self.save_to_file(response, &filepath, total_size).await?;
        Ok(())
    }

    fn format_filename(&self, post: &E6Post) -> Result<String> {
        let format = self.output_format.as_deref().unwrap_or("$id.$ext");
        let artist = post
            .tags
            .artist
            .first()
            .map(|s| s.as_str())
            .unwrap_or("unknown");

        let re = regex::Regex::new(r"\$tags\[(\d+)\]").unwrap();
        let mut formatted = format.to_string();

        for cap in re.captures_iter(format) {
            if let Some(num_match) = cap.get(1) {
                if let Ok(num_tags) = num_match.as_str().parse::<usize>() {
                    let tags = post
                        .tags
                        .general
                        .iter()
                        .take(num_tags)
                        .cloned()
                        .collect::<Vec<String>>()
                        .join(",");

                    formatted = formatted.replace(&cap[0], &tags);
                }
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

        let formatted = formatted
            .replace("$id", &post.id.to_string())
            .replace("$rating", &post.rating)
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

    fn get_filepath(&self, filename: &str) -> Result<PathBuf> {
        let path = if let Some(ref dir) = self.download_dir {
            dir.join(filename)
        } else {
            PathBuf::from(filename)
        };

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }
        }

        if path.exists() {
            anyhow::bail!("File '{}' already exists", path.display());
        }

        Ok(path)
    }

    async fn save_to_file(
        &self,
        response: reqwest::Response,
        filepath: &Path,
        total_size: Option<u64>,
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

            if let Some(total) = total_size {
                let progress = (downloaded as f64 / total as f64 * 100.0) as u32;
                print!("\rProgress: {}%", progress);
                use std::io::{self, Write};
                io::stdout().flush().unwrap();
            }
        }

        if total_size.is_some() {
            println!();
        }

        file.flush().await.context("Failed to flush file to disk")?;

        Ok(())
    }
}

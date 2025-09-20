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

        let formatted = format
            .replace("$rating", &post.rating)
            .replace("$artist", artist)
            .replace("$id", &post.id.to_string())
            .replace("$score", &post.score.total.to_string())
            .replace("$score_up", &post.score.up.to_string())
            .replace("$score_down", &post.score.down.to_string())
            .replace("$md5", &post.file.md5)
            .replace("$filesize", &post.file.size.to_string())
            .replace("$ext", &post.file.ext);

        Ok(formatted)
    }

    fn extract_filename(&self, url: &str, post: &E6Post) -> Result<String> {
        url.split('/')
            .next_back()
            .map(String::from)
            .or_else(|| Some(format!("{}.{}", post.id, post.file.ext)))
            .context("Failed to extract filename from URL")
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

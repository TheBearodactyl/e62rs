use crate::models::E6Post;
use anyhow::{Context, Result};
use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use tokio::{fs::File, io::AsyncWriteExt};

#[derive(Default)]
pub struct PostDownloader {
    client: reqwest::Client,
    download_dir: Option<PathBuf>,
}

impl PostDownloader {
    #[allow(unused)]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            download_dir: None,
        }
    }

    pub fn with_download_dir<T>(download_dir: T) -> Self
    where
        T: AsRef<Path>, std::path::PathBuf: std::convert::From<T>
    {
        Self {
            client: reqwest::Client::new(),
            download_dir: Some(download_dir.into()),
        }
    }

    pub async fn download_post(&self, post: E6Post) -> Result<()> {
        let url = post
            .file
            .url
            .clone()
            .context("Post has no downloadable file URL")?;

        let filename = self.extract_filename(&url, &post)?;
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

use std::{io::Read, path::Path, sync::Arc};

use crate::{
    client::{DEFAULT_LIMIT, E6Client},
    models::{E6PostResponse, E6PostsResponse},
};
use anyhow::{Context, Result, bail};
use chrono::{Datelike, Local};
use e6cfg::Cfg;
use flate2::read::GzDecoder;
use futures::future::join_all;
use log::{debug, info, warn};
use sha2::{Digest, Sha256};
use tokio::{fs, io::AsyncWriteExt};

impl E6Client {
    pub async fn get_latest_posts(&self) -> Result<E6PostsResponse> {
        let url = format!("{}/posts.json", self.base_url);
        let bytes = self.get_cached_or_fetch(&url).await?;

        let mut posts: E6PostsResponse =
            serde_json::from_slice(&bytes).context("Failed to deserialize posts response")?;

        posts = posts.filter_blacklisted(&[]);

        debug!("Successfully fetched {} posts", posts.posts.len());
        Ok(posts)
    }

    pub async fn search_posts(
        &self,
        tags: Vec<String>,
        limit: Option<u64>,
    ) -> Result<E6PostsResponse> {
        let url = format!("{}/posts.json", self.base_url);
        let limit = limit.unwrap_or(DEFAULT_LIMIT);

        let query_url = format!(
            "{}?tags={}&limit={}",
            url,
            urlencoding::encode(&tags.join(" ")),
            limit
        );

        let bytes = self.get_cached_or_fetch(&query_url).await?;

        let mut posts: E6PostsResponse =
            serde_json::from_slice(&bytes).context("Failed to deserialize search response")?;

        posts = posts.filter_blacklisted(&tags);

        debug!(
            "Successfully searched and found {} posts",
            posts.posts.len()
        );
        Ok(posts)
    }

    pub async fn get_post_by_id(&self, id: i64) -> Result<E6PostResponse> {
        let url = format!("{}/posts/{}.json", self.base_url, id);
        let bytes = self.get_cached_or_fetch(&url).await?;

        let post: E6PostResponse = serde_json::from_slice(&bytes)
            .with_context(|| format!("Failed to deserialize post {}", id))?;

        Ok(post)
    }

    pub async fn get_posts_by_ids(&self, ids: Vec<i64>) -> Result<Vec<E6PostResponse>> {
        let config = Cfg::get().unwrap_or_default();
        let concurrent_limit = config
            .performance
            .as_ref()
            .and_then(|p| p.concurrent_downloads)
            .unwrap_or(8);

        let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrent_limit));
        let futures: Vec<_> = ids
            .into_iter()
            .map(|id| {
                let client = self.clone();
                let semaphore = semaphore.clone();

                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    client.get_post_by_id(id).await
                })
            })
            .collect();

        let results = join_all(futures).await;
        let mut posts = Vec::new();

        for result in results {
            match result {
                Ok(Ok(post)) => {
                    if !post.post.is_blacklisted() {
                        posts.push(post)
                    }
                }
                Ok(Err(e)) => warn!("Failed to fetch post: {}", e),
                Err(e) => warn!("Task failed: {}", e),
            }
        }

        Ok(posts)
    }

    pub async fn update_tags(&self) -> Result<()> {
        let local_file: &str = "data/tags.csv";
        let local_hash_file: &str = "data/tags.csv.hash";

        let now = Local::now();
        let url = format!(
            "https://e621.net/db_export/tags-{:04}-{:02}-{:02}.csv.gz",
            now.year(),
            now.month(),
            now.day()
        );

        info!("{}", url);

        let response = self.client.get(&url).send().await?;
        if !response.status().clone().is_success() {
            bail!("Failed to download tags: {}", response.status());
        }

        let remote_bytes = response.bytes().await?;
        let mut hasher = Sha256::new();

        hasher.update(&remote_bytes);

        let remote_hash = hasher.finalize();
        let remote_hash_hex = hex::encode(remote_hash);

        let update_needed = if Path::new(local_hash_file).exists() {
            let local_hash_hex = fs::read_to_string(local_hash_file).await?;
            local_hash_hex.trim() != remote_hash_hex
        } else {
            true
        };

        if update_needed {
            info!("Updating local tags snapshot...");

            let mut gz = GzDecoder::new(&remote_bytes[..]);
            let mut decompressed_data = Vec::new();
            gz.read_to_end(&mut decompressed_data)?;

            fs::create_dir_all("data").await?;
            let mut file = fs::File::create(local_file).await?;
            file.write_all(&decompressed_data).await?;

            let mut hash_file = fs::File::create(local_hash_file).await?;
            hash_file.write_all(remote_hash_hex.as_bytes()).await?;

            info!("Updated local tags snapshot at {}", local_file);
        } else {
            info!("Local snapshot of `tags.csv` is up to date, continuing");
        }

        Ok(())
    }
}

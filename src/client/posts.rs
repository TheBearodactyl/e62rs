use {
    crate::{
        client::E6Client,
        config::options::E62Rs,
        models::{E6PostResponse, E6PostsResponse},
    },
    chrono::{Datelike, Local},
    color_eyre::eyre::{Context, Result, bail},
    flate2::read::GzDecoder,
    futures::future::join_all,
    sha2::{Digest, Sha256},
    std::{io::Read, path::Path, sync::Arc},
    tokio::{fs, io::AsyncWriteExt},
    tracing::{debug, info, warn},
};

impl E6Client {
    pub async fn get_latest_posts(&self) -> Result<E6PostsResponse> {
        let url = format!("{}/posts.json", self.get_base_url());
        let bytes = self.get_cached_or_fetch(&url).await?;
        let mut posts: E6PostsResponse =
            serde_json::from_slice(&bytes).context("Failed to deserialize posts response")?;

        if !posts.posts.is_empty()
            && let Err(e) = self.post_cache.insert_batch(&posts.posts).await
        {
            warn!("Failed to cache posts: {}", e);
        }

        let cfg = E62Rs::get()?;
        let apply_blacklist = !cfg.blacklist.is_empty();

        if apply_blacklist {
            posts = posts.filter_blacklisted(&[]);
        }

        debug!("Successfully fetched {} posts", posts.posts.len());
        Ok(posts)
    }

    pub async fn search_posts(
        &self,
        tags: Vec<String>,
        limit: Option<u64>,
        page_before_id: Option<i64>,
    ) -> Result<E6PostsResponse> {
        let url = format!("{}/posts.json", self.base_url);
        let limit = limit.unwrap_or(20);

        let mut query_url = format!(
            "{}?tags={}&limit={}",
            url,
            urlencoding::encode(&tags.join(" ")),
            limit
        );

        if let Some(before_id) = page_before_id {
            query_url.push_str(&format!("&page=b{}", before_id));
        }

        debug!("Fetching posts from: {}", query_url);
        let bytes = self.get_cached_or_fetch(&query_url).await?;

        let mut posts: E6PostsResponse =
            serde_json::from_slice(&bytes).context("Failed to deserialize search response")?;

        let count_before_filtering = posts.posts.len();

        if !posts.posts.is_empty()
            && let Err(e) = self.post_cache.insert_batch(&posts.posts).await
        {
            warn!("Failed to cache posts: {}", e);
        }

        posts = posts.filter_blacklisted(&tags);

        if posts.posts.len() < count_before_filtering {
            info!(
                "Filtered out {} blacklisted posts ({} remaining)",
                count_before_filtering - posts.posts.len(),
                posts.posts.len()
            );
        }

        debug!(
            "Successfully searched and found {} posts",
            posts.posts.len()
        );
        Ok(posts)
    }

    pub async fn get_post_by_id(&self, id: i64) -> Result<E6PostResponse> {
        if let Ok(Some(cached_post)) = self.post_cache.get(id).await {
            debug!("Post {} retrieved from persistent cache", id);
            return Ok(E6PostResponse { post: cached_post });
        }

        let url = format!("{}/posts/{}.json", self.base_url, id);
        let bytes = self.get_cached_or_fetch(&url).await?;

        let post: E6PostResponse = serde_json::from_slice(&bytes)
            .with_context(|| format!("Failed to deserialize post {}", id))?;

        if let Err(e) = self.post_cache.insert(&post.post).await {
            warn!("Failed to cache post {}: {}", id, e);
        }

        Ok(post)
    }

    pub async fn get_posts_by_ids(&self, ids: Vec<i64>) -> Result<Vec<E6PostResponse>> {
        let cached_results = self.post_cache.get_batch(&ids).await?;
        let mut posts = Vec::new();
        let mut missing_ids = Vec::new();

        for (i, cached_post) in cached_results.into_iter().enumerate() {
            match cached_post {
                Some(post) => {
                    if !post.is_blacklisted() {
                        posts.push(E6PostResponse { post });
                    }
                }
                None => {
                    missing_ids.push(ids[i]);
                }
            }
        }

        info!(
            "Retrieved {}/{} posts from cache, fetching {} from network",
            posts.len(),
            ids.len(),
            missing_ids.len()
        );

        if !missing_ids.is_empty() {
            let config = E62Rs::get()?;
            let concurrent_limit = config.performance.concurrent_downloads;

            let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrent_limit));
            let futures: Vec<_> = missing_ids
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
        }

        Ok(posts)
    }

    pub async fn update_tags(&self) -> Result<()> {
        let cfg = E62Rs::get()?;
        let local_file_cfg = cfg.completion.tags;
        let local_file = local_file_cfg.as_str();
        let local_hash_file: &str = &format!("{}.hash", local_file);

        let now = Local::now();
        let url = format!(
            "https://e621.net/db_export/tags-{:04}-{:02}-{:02}.csv.gz",
            now.year(),
            now.month(),
            now.day()
        );

        info!("{}", url);

        let response = self.update_client.get(&url).send().await?;
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

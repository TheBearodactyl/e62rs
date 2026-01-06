//! client extensions for post operations on the e6 api
use {
    crate::{
        client::E6Client,
        getopt,
        models::{E6PostResponse, E6PostsResponse},
    },
    chrono::{Datelike, Days, Local},
    color_eyre::eyre::{Context, Result, bail},
    flate2::read::GzDecoder,
    sha2::{Digest, Sha256},
    std::{io::Read, path::Path},
    tokio::{fs, sync::Semaphore},
    tracing::{debug, info, instrument, warn},
};

impl E6Client {
    /// try to get the latest posts
    pub async fn get_latest_posts(&self) -> Result<E6PostsResponse> {
        let url = format!("{}/posts.json", self.base_url);
        let bytes = self.get_cached_or_fetch(&url).await?;
        let mut posts: E6PostsResponse =
            serde_json::from_slice(&bytes).context("failed to deser")?;

        if !posts.posts.is_empty() {
            let cache = self.post_cache.clone();
            let posts_clone = posts.posts.clone();

            tokio::spawn(async move {
                if let Err(e) = cache.insert_batch(&posts_clone).await {
                    warn!(error = %e, "failed to cache posts");
                }
            });
        }

        if !getopt!(search.blacklist).is_empty() {
            posts = posts.filter_blacklisted(&[]);
        }

        debug!(count = posts.posts.len(), "fetched latest posts");
        Ok(posts)
    }

    #[instrument(skip(self, tags))]
    /// search posts with the given tags (paginated)
    pub async fn search_posts(
        &self,
        tags: &[String],
        limit: Option<u64>,
        page_before_id: Option<i64>,
    ) -> Result<E6PostsResponse> {
        let limit = limit.unwrap_or(20).min(320);
        let mut url = format!(
            "{}/posts.json?tags={}&limit={}",
            self.base_url,
            urlencoding::encode(&tags.join(" ")),
            limit
        );

        if let Some(before_id) = page_before_id {
            url.push_str(&format!("&page=b{}", before_id));
        }

        debug!(url, "Searching posts");

        let bytes = self.get_cached_or_fetch(&url).await?;
        let mut posts: E6PostsResponse =
            serde_json::from_slice(&bytes).context("Failed to deserialize search response")?;
        let count_before = posts.posts.len();

        if !posts.posts.is_empty() {
            let cache = self.post_cache.clone();
            let posts_clone = posts.posts.clone();
            tokio::spawn(async move {
                if let Err(e) = cache.insert_batch(&posts_clone).await {
                    warn!(error = %e, "Failed to cache posts");
                }
            });
        }

        posts = posts.filter_blacklisted(tags);

        if posts.posts.len() < count_before {
            info!(
                filtered = count_before - posts.posts.len(),
                remaining = posts.posts.len(),
                "Filtered blacklisted posts"
            );
        }

        Ok(posts)
    }

    #[instrument(skip(self))]
    /// get a post by its id
    pub async fn get_post_by_id(&self, id: i64) -> Result<E6PostResponse> {
        if let Ok(Some(cached_post)) = self.post_cache.get(id).await {
            debug!(id, "Post retrieved from cache");
            return Ok(E6PostResponse { post: cached_post });
        }

        let url = format!("{}/posts/{}.json", self.base_url, id);
        let bytes = self.get_cached_or_fetch(&url).await?;
        let post: E6PostResponse =
            serde_json::from_slice(&bytes).context(format!("Failed to deserialize post {}", id))?;
        let cache = self.post_cache.clone();
        let post_clone = post.post.clone();

        tokio::spawn(async move {
            if let Err(e) = cache.insert(&post_clone).await {
                warn!(id, error = %e, "Failed to cache post");
            }
        });

        Ok(post)
    }

    /// get posts by their ids
    #[instrument(skip(self, ids), fields(count = ids.len()))]
    pub async fn get_posts_by_ids(&self, ids: &[i64]) -> Result<Vec<E6PostResponse>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let cached_results = self.post_cache.get_batch(ids).await?;
        let mut posts = Vec::with_capacity(ids.len());
        let mut missing_ids = Vec::new();

        for (i, cached_post) in cached_results.into_iter().enumerate() {
            match cached_post {
                Some(post) if !post.is_blacklisted() => {
                    posts.push(E6PostResponse { post });
                }
                Some(_) => {}
                None => missing_ids.push(ids[i]),
            }
        }

        info!(
            cached = posts.len(),
            missing = missing_ids.len(),
            "Cache lookup complete"
        );

        if missing_ids.is_empty() {
            return Ok(posts);
        }

        let concurrent_limit = getopt!(download.threads);
        let semaphore = std::sync::Arc::new(Semaphore::new(concurrent_limit));
        let fetch_futures: Vec<_> = missing_ids
            .into_iter()
            .map(|id| {
                let client = self.clone();
                let permit = semaphore.clone();

                async move {
                    let _permit = permit.acquire().await.ok()?;
                    client.get_post_by_id(id).await.ok()
                }
            })
            .collect();

        let results = futures::future::join_all(fetch_futures).await;

        for result in results.into_iter().flatten() {
            if !result.post.is_blacklisted() {
                posts.push(result);
            }
        }

        Ok(posts)
    }

    #[instrument(skip(self), name = "update_tags")]
    /// update the local tag databases
    pub async fn update_tags(&self) -> Result<()> {
        let now = Local::now()
            .checked_sub_days(Days::new(1))
            .unwrap_or(Local::now());
        let date_str = format!("{:04}-{:02}-{:02}", now.year(), now.month(), now.day());
        let files = [
            ("tags", getopt!(completion.tags)),
            ("tag_aliases", getopt!(completion.aliases)),
            ("tag_implications", getopt!(completion.implications)),
        ];

        for (ty, local_file) in files {
            let hash_file = format!("{}.hash", local_file);
            let url = format!("https://e621.net/db_export/{}-{}.csv.gz", ty, date_str);

            info!(ty, "checking for updates");

            let response = match self.client.get(&url).send().await {
                Ok(r) => r,
                Err(e) => {
                    warn!("couldn't download the update: {}", e);
                    return Ok(());
                }
            };

            if !response.status().is_success() {
                bail!("failed to download {}: http {}", ty, response.status());
            }

            let remote_bytes = response.bytes().await?;
            let remote_hash_hex = {
                let mut hasher = Sha256::new();
                hasher.update(&remote_bytes);
                hex::encode(hasher.finalize())
            };

            let update_needed = match fs::read_to_string(&hash_file).await {
                Ok(local_hash) => local_hash.trim() != remote_hash_hex,
                Err(_) => true,
            };

            if !update_needed {
                info!(ty, "✓ Already up to date");
                continue;
            }

            info!(ty, "Updating local snapshot");

            let mut gz = GzDecoder::new(&remote_bytes[..]);
            let mut decompressed = Vec::new();
            gz.read_to_end(&mut decompressed)
                .context(format!("failed to decompress {}", ty))?;

            if let Some(parent) = Path::new(&local_file).parent() {
                fs::create_dir_all(parent).await?;
            }

            let temp_file = format!("{}.tmp", local_file);
            fs::write(&temp_file, &decompressed).await?;
            fs::rename(&temp_file, &local_file).await?;
            fs::write(&hash_file, remote_hash_hex.as_bytes()).await?;

            info!(ty, path = %local_file, "✓ Updated");
        }

        info!("all tag dbs are up to date");
        Ok(())
    }
}

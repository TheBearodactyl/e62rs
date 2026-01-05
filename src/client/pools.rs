//! client extensions for pool operations on the e6 api
use {
    crate::{
        client::E6Client,
        getopt,
        models::{E6PoolResponse, E6PoolsResponse, E6PostsResponse},
    },
    chrono::{Datelike, Days, Local},
    color_eyre::eyre::{Context, Result, bail},
    flate2::read::GzDecoder,
    sha2::{Digest, Sha256},
    std::{io::Read, path::Path},
    tokio::fs,
    tracing::{debug, info, instrument},
};

impl E6Client {
    #[instrument(skip(self), name = "update_pools")]
    /// update the local pool database
    pub async fn update_pools(&self) -> Result<()> {
        let local_file = getopt!(completion.pools);
        let local_hash_file = format!("{}.hash", local_file);

        let now = Local::now()
            .checked_sub_days(Days::new(1))
            .unwrap_or(Local::now());
        let url = format!(
            "https://e621.net/db_export/pools-{:04}-{:02}-{:02}.csv.gz",
            now.year(),
            now.month(),
            now.day()
        );

        self.download_and_update_file(&url, &local_file, &local_hash_file, "pools")
            .await
    }

    /// generic file download with hash-based update check
    async fn download_and_update_file(
        &self,
        url: &str,
        local_file: &str,
        hash_file: &str,
        file_type: &str,
    ) -> Result<()> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("Failed to fetch {}", file_type))?;

        if !response.status().is_success() {
            bail!(
                "Failed to download {}: HTTP {}",
                file_type,
                response.status()
            );
        }

        let remote_bytes = response.bytes().await?;
        let remote_hash_hex = {
            let mut hasher = Sha256::new();
            hasher.update(&remote_bytes);
            hex::encode(hasher.finalize())
        };

        let update_needed = match fs::read_to_string(hash_file).await {
            Ok(local_hash) => local_hash.trim() != remote_hash_hex,
            Err(_) => true,
        };

        if !update_needed {
            info!(file_type, "Local snapshot is up to date");
            return Ok(());
        }

        info!(file_type, "Updating local snapshot");

        let mut gz = GzDecoder::new(&remote_bytes[..]);
        let mut decompressed = Vec::new();
        gz.read_to_end(&mut decompressed)
            .with_context(|| format!("Failed to decompress {}", file_type))?;

        if let Some(parent) = Path::new(local_file).parent() {
            fs::create_dir_all(parent).await?;
        }

        let temp_file = format!("{}.tmp", local_file);

        fs::write(&temp_file, &decompressed).await?;
        fs::rename(&temp_file, local_file).await?;
        fs::write(hash_file, remote_hash_hex.as_bytes()).await?;

        info!(file_type, path = local_file, "Updated local snapshot");
        Ok(())
    }

    #[instrument(skip(self), fields(limit))]
    /// get n pools (default: 20)
    pub async fn get_pools(&self, limit: Option<u64>) -> Result<E6PoolsResponse> {
        let limit = limit.unwrap_or(20).min(320);
        let url = format!("{}/pools.json?limit={}", self.base_url, limit);

        let bytes = self.get_cached_or_fetch(&url).await?;
        serde_json::from_slice(&bytes).context("Failed to deserialize pools response")
    }

    #[instrument(skip(self))]
    /// get a pool by its id
    pub async fn get_pool_by_id(&self, id: i64) -> Result<E6PoolResponse> {
        let url = format!("{}/pools/{}.json", self.base_url, id);
        let bytes = self.get_cached_or_fetch(&url).await?;

        serde_json::from_slice(&bytes).with_context(|| format!("Failed to deserialize pool {}", id))
    }

    #[instrument(skip(self))]
    /// get all posts in a pool
    pub async fn get_pool_posts(&self, pool_id: i64) -> Result<E6PostsResponse> {
        let url = format!("{}/posts.json?tags=pool:{}", self.base_url, pool_id);
        let bytes = self.get_cached_or_fetch(&url).await?;

        let posts: E6PostsResponse =
            serde_json::from_slice(&bytes).context("Failed to deserialize pool posts")?;

        debug!(pool_id, count = posts.posts.len(), "Fetched pool posts");
        Ok(posts)
    }

    #[instrument(skip(self))]
    /// search for pools
    pub async fn search_pools(&self, query: &str, limit: Option<u64>) -> Result<E6PoolsResponse> {
        self.search_pools_internal("name_matches", query, limit)
            .await
    }

    #[instrument(skip(self))]
    /// search for pools by their creator
    pub async fn search_pools_by_creator(
        &self,
        creator_name: &str,
        limit: Option<u64>,
    ) -> Result<E6PoolsResponse> {
        self.search_pools_internal("creator_name", creator_name, limit)
            .await
    }

    #[instrument(skip(self))]
    /// search for pools by their description
    pub async fn search_pools_by_description(
        &self,
        query: &str,
        limit: Option<u64>,
    ) -> Result<E6PoolsResponse> {
        self.search_pools_internal("description_matches", query, limit)
            .await
    }

    /// search for pools with a given search type
    async fn search_pools_internal(
        &self,
        search_type: &str,
        query: &str,
        limit: Option<u64>,
    ) -> Result<E6PoolsResponse> {
        let limit = limit.unwrap_or(20).min(320);
        let url = format!(
            "{}/pools.json?search[{}]={}&limit={}",
            self.base_url,
            search_type,
            urlencoding::encode(query),
            limit
        );

        debug!(url, "Searching pools");
        let bytes = self.get_cached_or_fetch(&url).await?;

        serde_json::from_slice(&bytes).context("Failed to deserialize pool search response")
    }
}

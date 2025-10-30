use {
    crate::{
        client::E6Client,
        config::options::E62Rs,
        models::{E6PoolResponse, E6PoolsResponse, E6PostsResponse},
    },
    chrono::{Datelike, Local},
    color_eyre::eyre::{Context, Result, bail},
    flate2::read::GzDecoder,
    sha2::{Digest, Sha256},
    std::{io::Read, path::Path},
    tokio::{fs, io::AsyncWriteExt},
    tracing::{debug, info},
};

impl E6Client {
    pub async fn update_pools(&self) -> Result<()> {
        let cfg = E62Rs::get()?;
        let local_file_cfg = cfg.completion.pools;
        let local_file = local_file_cfg.as_str();
        let local_hash_file: &str = &format!("{}.hash", local_file);

        let now = Local::now();
        let url = format!(
            "https://e621.net/db_export/pools-{:04}-{:02}-{:02}.csv.gz",
            now.year(),
            now.month(),
            now.day()
        );

        let response = self.update_client.get(&url).send().await?;
        if !response.status().clone().is_success() {
            bail!("Failed to download pools: {}", response.status());
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
            info!("Updating local pools snapshot...");

            let mut gz = GzDecoder::new(&remote_bytes[..]);
            let mut decompressed_data = Vec::new();
            gz.read_to_end(&mut decompressed_data)?;

            fs::create_dir_all("data").await?;
            let mut file = fs::File::create(local_file).await?;
            file.write_all(&decompressed_data).await?;

            let mut hash_file = fs::File::create(local_hash_file).await?;
            hash_file.write_all(remote_hash_hex.as_bytes()).await?;

            info!("Updated local pools snapshot at {}", local_file);
        } else {
            info!("Local snapshot of `pools.csv` is up to date, continuing");
        }

        Ok(())
    }

    pub async fn get_pools(&self, limit: Option<u64>) -> Result<E6PoolsResponse> {
        let limit = limit.unwrap_or(20);
        let url = format!("{}/pools.json?limit={}", self.get_base_url(), limit);
        let bytes = self.get_cached_or_fetch(&url).await?;

        let pools: E6PoolsResponse =
            serde_json::from_slice(&bytes).context("Failed to deserialize pools response")?;

        debug!("Successfully fetched {} pools", pools.pools.len());
        Ok(pools)
    }

    pub async fn get_pool_by_id(&self, id: i64) -> Result<E6PoolResponse> {
        let url = format!("{}/pools/{}.json", self.base_url, id);
        let bytes = self.get_cached_or_fetch(&url).await?;

        let pool: E6PoolResponse = serde_json::from_slice(&bytes)
            .with_context(|| format!("Failed to deserialize pool {}", id))?;

        Ok(pool)
    }

    pub async fn get_pool_posts(&self, pool_id: i64) -> Result<E6PostsResponse> {
        let url = format!("{}/posts.json?tags=pool:{}", self.base_url, pool_id);
        let bytes = self.get_cached_or_fetch(&url).await?;

        let posts: E6PostsResponse =
            serde_json::from_slice(&bytes).context("Failed to deserialize pool posts response")?;

        debug!(
            "Successfully fetched {} posts from pool {}",
            posts.posts.len(),
            pool_id
        );
        Ok(posts)
    }

    pub async fn search_pools_by_creator(
        &self,
        creator_name: String,
        limit: Option<u64>,
    ) -> Result<E6PoolsResponse> {
        let limit = limit.unwrap_or(20);

        let query_url = format!(
            "{}/pools.json?search[creator_name]={}&limit={}",
            self.base_url,
            urlencoding::encode(&creator_name),
            limit
        );

        debug!("Searching pools by creator with URL: {}", query_url);
        let bytes = self.get_cached_or_fetch(&query_url).await?;

        let pools: E6PoolsResponse =
            serde_json::from_slice(&bytes).context("Failed to deserialize pool search response")?;

        Ok(pools)
    }

    pub async fn search_pools_by_description(
        &self,
        query: String,
        limit: Option<u64>,
    ) -> Result<E6PoolsResponse> {
        let limit = limit.unwrap_or(20);

        let query_url = format!(
            "{}/pools.json?search[description_matches]={}&limit={}",
            self.base_url,
            urlencoding::encode(&query),
            limit
        );

        debug!("Searching pools by description with URL: {}", query_url);
        let bytes = self.get_cached_or_fetch(&query_url).await?;

        let pools: E6PoolsResponse =
            serde_json::from_slice(&bytes).context("Failed to deserialize pool search response")?;

        Ok(pools)
    }

    pub async fn search_pools(&self, query: String, limit: Option<u64>) -> Result<E6PoolsResponse> {
        let limit = limit.unwrap_or(20);

        let query_url = format!(
            "{}/pools.json?search[name_matches]={}&limit={}",
            self.base_url,
            urlencoding::encode(&query),
            limit
        );

        debug!("Searching pools with URL: {}", query_url);
        let bytes = self.get_cached_or_fetch(&query_url).await?;

        let pools: E6PoolsResponse =
            serde_json::from_slice(&bytes).context("Failed to deserialize pool search response")?;

        debug!(
            "Successfully searched and found {} pools",
            pools.pools.len()
        );
        Ok(pools)
    }
}

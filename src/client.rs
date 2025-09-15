use crate::models::{E6PostResponse, E6PostsResponse};
use anyhow::{Context, Result};
use log::{debug, warn};
use reqwest::Client;
use std::time::Duration;

const BASE_URL: &str = "https://e621.net";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_LIMIT: u64 = 20;

#[derive(Debug, Clone)]
pub struct E6Client {
    client: Client,
    base_url: String,
}

impl Default for E6Client {
    fn default() -> Self {
        Self::new(BASE_URL).expect("Failed to create default E6Client")
    }
}

impl E6Client {
    pub fn new(base_url: &str) -> Result<Self> {
        let client = Client::builder()
            .user_agent(crate::USER_AGENT)
            .timeout(DEFAULT_TIMEOUT)
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            base_url: base_url.to_string(),
        })
    }

    pub fn with_client(client: Client, base_url: &str) -> Self {
        Self {
            client,
            base_url: base_url.to_string(),
        }
    }

    pub async fn get_latest_posts(&self) -> Result<E6PostsResponse> {
        let url = format!("{}/posts.json", self.base_url);
        debug!("Fetching latest posts from {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch latest posts")?;

        let status = response.status();
        if !status.is_success() {
            warn!("API returned error status: {}", status);
            anyhow::bail!("API returned error status: {}", status);
        }

        let posts = response
            .json::<E6PostsResponse>()
            .await
            .context("Failed to deserialize posts response")?;

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

        let response = self
            .client
            .get(&url)
            .query(&[("tags", tags.join(" ")), ("limit", limit.to_string())])
            .send()
            .await
            .context("Failed to search posts")?;

        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("API returned error status: {}", status);
        }

        let posts = response
            .json::<E6PostsResponse>()
            .await
            .context("Failed to deserialize search response")?;

        Ok(posts)
    }

    pub async fn get_post_by_id(&self, id: i64) -> Result<E6PostResponse> {
        let url = format!("{}/posts/{}.json", self.base_url, id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("Failed to fetch post with id {}", id))?;

        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("API returned error status: {} for post {}", status, id);
        }

        let post = response
            .json::<E6PostResponse>()
            .await
            .with_context(|| format!("Failed to deserialize post {}", id))?;

        Ok(post)
    }

    async fn execute_with_retry<T, F, Fut>(&self, max_retries: u32, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts <= max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    attempts += 1;

                    if attempts <= max_retries {
                        let delay = Duration::from_millis(100 * 2_u64.pow(attempts));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Retry failed without error")))
    }
}

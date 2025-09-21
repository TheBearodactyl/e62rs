#![allow(unused)]

use crate::{
    client::E6Client,
    models::{E6Post, E6PostResponse},
};
use anyhow::Result;
use futures_util::future;
use std::sync::Arc;

pub struct BatchOperations {
    client: Arc<E6Client>,
}

impl BatchOperations {
    pub fn new(client: E6Client) -> Self {
        Self {
            client: Arc::new(client),
        }
    }

    pub async fn fetch_posts_parallel(&self, post_ids: Vec<i64>) -> Result<Vec<E6PostResponse>> {
        let futures = post_ids.into_iter().map(|id| {
            let client = Arc::clone(&self.client);
            async move { client.get_post_by_id(id).await }
        });

        let results = future::join_all(futures).await;

        results.into_iter().collect()
    }

    pub async fn fetch_posts_with_limit(
        &self,
        post_ids: Vec<i64>,
        concurrent_limit: usize,
    ) -> Result<Vec<E6PostResponse>> {
        use futures_util::stream::{self, StreamExt};

        let client = Arc::clone(&self.client);

        let results: Vec<Result<E6PostResponse>> = stream::iter(post_ids)
            .map(move |id| {
                let client = Arc::clone(&client);
                async move { client.get_post_by_id(id).await }
            })
            .buffer_unordered(concurrent_limit)
            .collect()
            .await;

        results.into_iter().collect()
    }

    pub async fn prefetch_previews(&self, posts: &[E6Post]) -> Vec<Result<Vec<u8>>> {
        let futures = posts
            .iter()
            .filter_map(|post| post.preview.url.as_ref())
            .map(|url| {
                let url = url.clone();
                async move {
                    let response = reqwest::get(&url).await?;
                    response
                        .bytes()
                        .await
                        .map(|b| b.to_vec())
                        .map_err(Into::into)
                }
            });

        future::join_all(futures).await
    }
}

use std::{path::PathBuf, sync::Arc};

use crate::models::E6Post;
use crate::{
    check_e62rs_logging_enabled, check_e62rs_verbose, e62rs_debug as debug, e62rs_info as info,
    e62rs_warn as warn,
};
use anyhow::{Context, Result};
use bincode::config::standard;
use e6cfg::E62Rs;
use redb::{Database, ReadableDatabase, ReadableTableMetadata, TableDefinition};
use tokio::sync::RwLock;

const POSTS_TABLE: TableDefinition<i64, &[u8]> = TableDefinition::new("posts");

#[derive(Clone, Debug)]
pub struct PostCache {
    db: Arc<RwLock<Option<Database>>>,
    cache_path: PathBuf,
}

impl PostCache {
    pub fn new(cache_dir: &str) -> Result<Self> {
        let cache_path = PathBuf::from(cache_dir).join("posts.redb");

        std::fs::create_dir_all(cache_dir)
            .with_context(|| format!("Failed to create cache directory: {}", cache_dir))?;

        let db = Database::create(&cache_path)
            .with_context(|| format!("Failed to create post cache database at {:?}", cache_path))?;

        info!("Initialized post cache at {:?}", cache_path);

        Ok(Self {
            db: Arc::new(RwLock::new(Some(db))),
            cache_path,
        })
    }

    pub async fn get(&self, post_id: i64) -> Result<Option<E6Post>> {
        let db_guard = self.db.read().await;
        let db = match db_guard.as_ref() {
            Some(db) => db,
            None => return Ok(None),
        };

        let read_txn = db
            .begin_read()
            .context("Failed to begin read transaction")?;

        let table = match read_txn.open_table(POSTS_TABLE) {
            Ok(table) => table,
            Err(_) => return Ok(None),
        };

        match table.get(post_id) {
            Ok(Some(data)) => {
                let bytes = data.value();
                match bincode::serde::decode_from_slice::<E6Post, _>(bytes, standard()) {
                    Ok((post, _)) => {
                        debug!("Cache hit for post {}", post_id);
                        Ok(Some(post))
                    }
                    Err(e) => {
                        warn!("Failed to deserialize cached post {}: {}", post_id, e);
                        Ok(None)
                    }
                }
            }
            Ok(None) => {
                debug!("Cache miss for post {}", post_id);
                Ok(None)
            }
            Err(e) => {
                warn!("Error reading from cache for post {}: {}", post_id, e);
                Ok(None)
            }
        }
    }

    pub async fn insert(&self, post: &E6Post) -> Result<()> {
        let db_guard = self.db.read().await;
        let db = match db_guard.as_ref() {
            Some(db) => db,
            None => anyhow::bail!("Database not initialized"),
        };

        let serialized =
            bincode::serde::encode_to_vec(post, standard()).context("Failed to serialize post")?;

        let write_txn = db
            .begin_write()
            .context("Failed to begin write transaction")?;

        {
            let mut table = write_txn
                .open_table(POSTS_TABLE)
                .context("Failed to open posts table")?;

            table
                .insert(post.id, serialized.as_slice())
                .context("Failed to insert post into cache")?;
        }

        write_txn.commit().context("Failed to commit transaction")?;

        debug!("Cached post {}", post.id);
        Ok(())
    }

    pub async fn insert_batch(&self, posts: &[E6Post]) -> Result<()> {
        if posts.is_empty() {
            return Ok(());
        }

        let db_guard = self.db.read().await;
        let db = match db_guard.as_ref() {
            Some(db) => db,
            None => anyhow::bail!("Database not initialized"),
        };

        let write_txn = db
            .begin_write()
            .context("Failed to begin write transaction")?;

        {
            let mut table = write_txn
                .open_table(POSTS_TABLE)
                .context("Failed to open posts table")?;

            for post in posts {
                let serialized = bincode::serde::encode_to_vec(post, standard())
                    .context("Failed to serialize post")?;

                table
                    .insert(post.id, serialized.as_slice())
                    .with_context(|| format!("Failed to insert post {} into cache", post.id))?;
            }
        }

        write_txn
            .commit()
            .context("Failed to commit batch transaction")?;

        info!("Cached {} posts", posts.len());
        Ok(())
    }

    pub async fn get_batch(&self, post_ids: &[i64]) -> Result<Vec<Option<E6Post>>> {
        let db_guard = self.db.read().await;
        let db = match db_guard.as_ref() {
            Some(db) => db,
            None => return Ok(vec![None; post_ids.len()]),
        };

        let read_txn = db
            .begin_read()
            .context("Failed to begin read transaction")?;

        let table = match read_txn.open_table(POSTS_TABLE) {
            Ok(table) => table,
            Err(_) => return Ok(vec![None; post_ids.len()]),
        };

        let mut results = Vec::with_capacity(post_ids.len());

        for &post_id in post_ids {
            let post = match table.get(post_id) {
                Ok(Some(data)) => {
                    let bytes = data.value();
                    match bincode::serde::decode_from_slice::<E6Post, _>(bytes, standard()) {
                        Ok((post, _)) => Some(post),
                        Err(e) => {
                            warn!("Failed to deserialize cached post {}: {}", post_id, e);
                            None
                        }
                    }
                }
                Ok(None) => None,
                Err(e) => {
                    warn!("Error reading from cache for post {}: {}", post_id, e);
                    None
                }
            };
            results.push(post);
        }

        let hits = results.iter().filter(|p| p.is_some()).count();
        debug!("Batch cache: {}/{} hits", hits, post_ids.len());

        Ok(results)
    }

    pub async fn contains(&self, post_id: i64) -> bool {
        self.get(post_id)
            .await
            .map(|p| p.is_some())
            .unwrap_or(false)
    }

    pub async fn clear(&self) -> Result<()> {
        let mut db_guard = self.db.write().await;
        *db_guard = None;

        std::fs::remove_file(&self.cache_path).ok();

        let new_db =
            Database::create(&self.cache_path).context("Failed to recreate post cache database")?;

        *db_guard = Some(new_db);

        info!("Post cache cleared");
        Ok(())
    }

    pub async fn get_stats(&self) -> Result<PostCacheStats> {
        let db_guard = self.db.read().await;
        let db = match db_guard.as_ref() {
            Some(db) => db,
            None => return Ok(PostCacheStats::default()),
        };

        let read_txn = db
            .begin_read()
            .context("Failed to begin read transaction")?;

        let table = match read_txn.open_table(POSTS_TABLE) {
            Ok(table) => table,
            Err(_) => return Ok(PostCacheStats::default()),
        };

        let count = table.len()? as usize;

        let file_size = std::fs::metadata(&self.cache_path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(PostCacheStats {
            entry_count: count,
            file_size_bytes: file_size,
        })
    }
}

#[derive(Debug, Default)]
pub struct PostCacheStats {
    pub entry_count: usize,
    pub file_size_bytes: u64,
}

impl PostCacheStats {
    pub fn file_size_mb(&self) -> f64 {
        self.file_size_bytes as f64 / (1024.0 * 1024.0)
    }
}

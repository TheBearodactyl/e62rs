//! post cache management stuff
use {
    crate::{cache::stats::PostCacheStats, getopt, mkvec, models::E6Post},
    color_eyre::eyre::{Context, Result, bail},
    postcard::{from_bytes, to_allocvec},
    redb::{Database, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition},
    serde::{Deserialize, Serialize},
    std::{fs::create_dir_all, path::PathBuf, sync::Arc},
    tokio::sync::RwLock,
    tracing::{debug, info, warn},
};

/// the table of cached posts
const POSTS_TABLE: TableDefinition<i64, &[u8]> = TableDefinition::new("posts");

/// an entry in the post cache
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheEntry {
    /// the cached data
    pub data: Vec<u8>,
    /// the timestamp of when the entry was created
    pub timestamp: u64,
    /// the timestamp of when the entry was last accessed
    pub last_accessed: u64,
    /// the etag of the entry (optional)
    pub etag: Option<String>,
    /// the amount of times the entry has been accessed
    pub access_count: u64,
    /// whether the entry is compressed
    pub compressed: bool,
}

#[derive(Clone, Debug)]
/// the post cache
pub struct PostCache {
    /// the database itself
    db: Arc<RwLock<Option<Database>>>,
    /// the path to the cache file
    cache_path: PathBuf,
    /// the maximum number of posts allowed in the cache
    max_posts: usize,
    /// whether to automatically compact entries
    auto_compact: bool,
    /// the threshold at which to compact an entry
    compact_threshold: u8,
}

impl PostCache {
    /// initialize and/or load the post cache
    pub fn new(cache_dir: &str) -> Result<Self> {
        let cache_path = PathBuf::from(cache_dir).join("posts.redb");

        if !getopt!(cache.posts.enabled) {
            info!("Post cache disabled by config");

            return Ok(Self {
                db: Arc::new(RwLock::new(None)),
                cache_path,
                max_posts: 0,
                auto_compact: false,
                compact_threshold: 0,
            });
        }

        create_dir_all(cache_dir).context(format!("failed to make cache dir: {}", cache_dir))?;

        let cache_size_mb = getopt!(cache.max_size_mb);
        let cahce_size_bytes = ((cache_size_mb / 4) * 1024 * 1024) as usize;
        let db = Database::builder()
            .set_cache_size(cahce_size_bytes)
            .create(&cache_path)
            .context(format!("failed to make post cache db at {:?}", cache_path))?;
        let db = Arc::new(RwLock::new(Some(db)));
        let max_posts = getopt!(cache.posts.max_posts);
        let auto_compact = getopt!(cache.posts.auto_compact);
        let compact_threshold = getopt!(cache.posts.compact_threshold);

        info!(
            "initialized post cache at {:?} (max: {} posts, cache: {} MB)",
            cache_path, max_posts, cache_size_mb
        );

        Ok(Self {
            db,
            cache_path,
            max_posts,
            auto_compact,
            compact_threshold,
        })
    }

    /// print a list of entries currently in the post cache
    pub async fn list_entries(&self) -> Result<()> {
        let db_guard = self.db.read().await;
        let db = match db_guard.as_ref() {
            Some(db) => db,
            None => {
                warn!("post cache is not initialized");
                return Ok(());
            }
        };

        let read_txn = db.begin_read().context("failed to start read")?;
        let table_db = match read_txn.open_table(POSTS_TABLE) {
            Ok(t) => t,
            Err(_e) => {
                warn!("no posts table found in cache");
                return Ok(());
            }
        };

        println!("{:<12} {:<40} {:>10}", "ID", "Title", "Size (KB)");
        println!("{}", "-".repeat(65));

        const CHUNK_SIZE: usize = 100;

        mkvec!(chunk, (i64, Vec<u8>), CHUNK_SIZE);

        for entry in table_db.iter()? {
            if let Ok((id, data)) = entry {
                chunk.push((id.value(), data.value().to_vec()));
            }

            if chunk.len() >= CHUNK_SIZE {
                for (id, data) in chunk.drain(..) {
                    let size_kb = data.len() as f64 / 1024.0;
                    let title = match from_bytes::<E6Post>(&data) {
                        Ok(post) => post.description.clone(),
                        Err(_) => "<Failed to decode>".to_string(),
                    };
                    println!("{:<12} {:<40} {:>10.2}", id, title, size_kb);
                }
            }
        }

        for (id, data) in chunk.drain(..) {
            let size_kb = data.len() as f64 / 1024.0;
            let title = match from_bytes::<E6Post>(&data) {
                Ok(post) => post.description.clone(),
                Err(_) => "<Failed to decode>".to_string(),
            };
            println!("{:<12} {:<40} {:>10.2}", id, title, size_kb);
        }

        Ok(())
    }

    /// get a post in the cache
    pub async fn get(&self, post_id: i64) -> Result<Option<E6Post>> {
        let db_guard = self.db.read().await;
        let db = match db_guard.as_ref() {
            Some(db) => db,
            None => return Ok(None),
        };

        let read_txn = db
            .begin_read()
            .context("failed to begin read transaction")?;

        let table = match read_txn.open_table(POSTS_TABLE) {
            Ok(table) => table,
            Err(_) => return Ok(None),
        };

        match table.get(post_id) {
            Ok(Some(data)) => {
                let bytes = data.value();
                match from_bytes::<E6Post>(bytes) {
                    Ok(post) => {
                        debug!("Post cache hit for {}", post_id);
                        Ok(Some(post))
                    }
                    Err(e) => {
                        warn!("Failed to deserialize cached post {}: {}", post_id, e);
                        Ok(None)
                    }
                }
            }
            Ok(None) => {
                debug!("Post cache miss for {}", post_id);
                Ok(None)
            }
            Err(e) => {
                warn!("Error reading from cache for post {}: {}", post_id, e);
                Ok(None)
            }
        }
    }

    /// insert a post into the cache
    pub async fn insert(&self, post: &E6Post) -> Result<()> {
        let db_guard = self.db.read().await;
        let db = match db_guard.as_ref() {
            Some(db) => db,
            None => bail!("Database not initialized"),
        };

        let serialized = to_allocvec(post).context("Failed to serialize post")?;
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

        self.maybe_evict_old_entries().await?;
        self.maybe_compact().await?;

        Ok(())
    }

    /// insert multiple posts into the cache
    pub async fn insert_batch(&self, posts: &[E6Post]) -> Result<()> {
        if posts.is_empty() {
            return Ok(());
        }

        let db_guard = self.db.read().await;
        let db = match db_guard.as_ref() {
            Some(db) => db,
            None => bail!("Database not initialized"),
        };

        let write_txn = db
            .begin_write()
            .context("Failed to begin write transaction")?;

        {
            let mut table = write_txn
                .open_table(POSTS_TABLE)
                .context("Failed to open posts table")?;

            for post in posts {
                let serialized = to_allocvec(post).context("Failed to serialize post")?;

                table
                    .insert(post.id, serialized.as_slice())
                    .with_context(|| format!("Failed to insert post {} into cache", post.id))?;
            }
        }

        write_txn
            .commit()
            .context("Failed to commit batch transaction")?;

        self.maybe_evict_old_entries().await?;
        self.maybe_compact().await?;

        Ok(())
    }

    /// try to evict old entries from the cache
    async fn maybe_evict_old_entries(&self) -> Result<()> {
        if self.max_posts == 0 {
            return Ok(());
        }

        let stats = self.get_stats().await?;
        if stats.entry_count > self.max_posts {
            let to_remove = stats.entry_count - (self.max_posts * 9 / 10);
            self.evict_oldest_entries(to_remove).await?;
        }

        Ok(())
    }

    /// evict the oldest entries from the cache
    async fn evict_oldest_entries(&self, count: usize) -> Result<()> {
        let db_guard = self.db.read().await;
        let db = match db_guard.as_ref() {
            Some(db) => db,
            None => return Ok(()),
        };

        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(POSTS_TABLE)?;
        let mut entries: Vec<i64> = table
            .iter()?
            .filter_map(|result| result.ok())
            .map(|(id, _)| id.value())
            .collect();

        entries.sort_unstable();

        let keys_to_remove: Vec<i64> = entries.into_iter().take(count).collect();

        drop(table);
        drop(read_txn);

        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(POSTS_TABLE)?;
            for key in &keys_to_remove {
                table.remove(*key)?;
            }
        }
        write_txn.commit()?;

        info!("Evicted {} old post cache entries", keys_to_remove.len());
        Ok(())
    }

    /// try to compact the cache
    async fn maybe_compact(&self) -> Result<()> {
        if !self.auto_compact {
            return Ok(());
        }

        let metadata = std::fs::metadata(&self.cache_path)?;
        let file_size = metadata.len();

        let stats = self.get_stats().await?;
        let avg_entry_size = if stats.entry_count > 0 {
            file_size / stats.entry_count as u64
        } else {
            return Ok(());
        };

        let expected_size = avg_entry_size * stats.entry_count as u64;
        let wasted_space_percent = if file_size > expected_size {
            ((file_size - expected_size) as f64 / file_size as f64) * 100.0
        } else {
            0.0
        };

        if wasted_space_percent > self.compact_threshold as f64 {
            info!(
                "Compacting post cache ({:.1}% wasted space)",
                wasted_space_percent
            );
            self.compact().await?;
        }

        Ok(())
    }

    /// forcefully compact the cache
    async fn compact(&self) -> Result<()> {
        let db_guard = self.db.read().await;
        let db = match db_guard.as_ref() {
            Some(db) => db,
            None => return Ok(()),
        };

        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(POSTS_TABLE)?;
        let entries: Vec<(i64, Vec<u8>)> = table
            .iter()?
            .filter_map(|result| result.ok())
            .map(|(id, data)| (id.value(), data.value().to_vec()))
            .collect();

        drop(table);
        drop(read_txn);

        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(POSTS_TABLE)?;
            for (id, data) in entries {
                table.insert(id, data.as_slice())?;
            }
        }
        write_txn.commit()?;

        info!("Post cache compaction completed");
        Ok(())
    }

    /// get multiple posts by their ids
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
                    match from_bytes::<E6Post>(bytes) {
                        Ok(post) => Some(post),
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

    /// return whether there's an entry for a given post id
    pub async fn contains(&self, post_id: i64) -> bool {
        self.get(post_id)
            .await
            .map(|p| p.is_some())
            .unwrap_or(false)
    }

    /// clear the post cache
    pub async fn clear(&self) -> Result<()> {
        let mut db_guard = self.db.write().await;
        *db_guard = None;
        std::fs::remove_file(&self.cache_path).ok();
        let new = Database::create(&self.cache_path).context("failed to recreate post cache db")?;
        *db_guard = Some(new);
        info!("Post cache cleared");
        Ok(())
    }

    /// get the stats of the post cache
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
            max_entries: self.max_posts,
            auto_compact_enabled: self.auto_compact,
        })
    }

    /// remove a post from the cache
    pub async fn remove(&self, post_id: i64) -> Result<bool> {
        let db_guard = self.db.read().await;
        let db = match db_guard.as_ref() {
            Some(db) => db,
            None => return Ok(false),
        };

        let write_txn = db.begin_write()?;
        let removed = {
            let mut table = write_txn.open_table(POSTS_TABLE)?;
            table.remove(post_id)?.is_some()
        };
        write_txn.commit()?;

        if removed {
            debug!("Removed post {} from cache", post_id);
        }

        Ok(removed)
    }

    /// remove multiple posts from the cache
    pub async fn remove_batch(&self, post_ids: &[i64]) -> Result<usize> {
        let db_guard = self.db.read().await;
        let db = match db_guard.as_ref() {
            Some(db) => db,
            None => return Ok(0),
        };

        let write_txn = db.begin_write()?;
        let mut removed_count = 0;

        {
            let mut table = write_txn.open_table(POSTS_TABLE)?;
            for &post_id in post_ids {
                if table.remove(post_id)?.is_some() {
                    removed_count += 1;
                }
            }
        }

        write_txn.commit()?;

        if removed_count > 0 {
            info!("Removed {} posts from cache", removed_count);
        }

        Ok(removed_count)
    }
}

use {
    crate::{config::options::CacheConfig, models::E6Post},
    bincode::config::standard,
    color_eyre::eyre::{Context, Result, bail},
    redb::{Database, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition},
    std::{path::PathBuf, sync::Arc},
    tokio::sync::RwLock,
    tracing::{debug, info, warn},
};

const POSTS_TABLE: TableDefinition<i64, &[u8]> = TableDefinition::new("posts");

#[derive(Clone, Debug)]
pub struct PostCache {
    db: Arc<RwLock<Option<Database>>>,
    cache_path: PathBuf,
    max_posts: usize,
    auto_compact: bool,
    compact_threshold: u8,
}

impl PostCache {
    pub fn new(cache_dir: &str, cache_config: &CacheConfig) -> Result<Self> {
        let post_cache_config = cache_config.clone().post_cache;

        if !post_cache_config.enabled {
            info!("Post cache disabled by configuration");
            return Ok(Self {
                db: Arc::new(RwLock::new(None)),
                cache_path: PathBuf::from(cache_dir).join("posts.redb"),
                max_posts: 0,
                auto_compact: false,
                compact_threshold: 0,
            });
        }

        let cache_path = PathBuf::from(cache_dir).join("posts.redb");

        std::fs::create_dir_all(cache_dir)
            .with_context(|| format!("Failed to create cache directory: {}", cache_dir))?;

        let cache_size_mb = cache_config.max_size_mb;
        let cache_size_bytes = ((cache_size_mb / 4) * 1024 * 1024) as usize;

        let db = Database::builder()
            .set_cache_size(cache_size_bytes)
            .create(&cache_path)
            .with_context(|| format!("Failed to create post cache database at {:?}", cache_path))?;

        let max_posts = post_cache_config.max_posts;
        let auto_compact = post_cache_config.auto_compact;
        let compact_threshold = post_cache_config.compact_threshold_percent;

        info!(
            "Initialized post cache at {:?} (max: {} posts, cache: {} MB)",
            cache_path, max_posts, cache_size_mb
        );

        Ok(Self {
            db: Arc::new(RwLock::new(Some(db))),
            cache_path,
            max_posts,
            auto_compact,
            compact_threshold,
        })
    }

    pub async fn list_entries(&self) -> Result<()> {
        let db_guard = self.db.read().await;
        let db = match db_guard.as_ref() {
            Some(db) => db,
            None => {
                println!("Post cache is not initialized.");
                return Ok(());
            }
        };

        let read_txn = db
            .begin_read()
            .context("Failed to begin read transaction")?;
        let table_db = match read_txn.open_table(POSTS_TABLE) {
            Ok(t) => t,
            Err(_) => {
                println!("No posts table found in cache.");
                return Ok(());
            }
        };

        println!("{:<12} {:<40} {:>10}", "ID", "Title", "Size (KB)");
        println!("{}", "-".repeat(65));

        const CHUNK_SIZE: usize = 100;
        let mut chunk: Vec<(i64, Vec<u8>)> = Vec::with_capacity(CHUNK_SIZE);

        for entry in table_db.iter()? {
            if let Ok((id, data)) = entry {
                chunk.push((id.value(), data.value().to_vec()));
            }

            if chunk.len() >= CHUNK_SIZE {
                for (id, data) in chunk.drain(..) {
                    let size_kb = data.len() as f64 / 1024.0;
                    let title =
                        match bincode::serde::decode_from_slice::<E6Post, _>(&data, standard()) {
                            Ok((post, _)) => post.description.clone(),
                            Err(_) => "<Failed to decode>".to_string(),
                        };
                    println!("{:<12} {:<40} {:>10.2}", id, title, size_kb);
                }
            }
        }

        for (id, data) in chunk.drain(..) {
            let size_kb = data.len() as f64 / 1024.0;
            let title = match bincode::serde::decode_from_slice::<E6Post, _>(&data, standard()) {
                Ok((post, _)) => post.description.clone(),
                Err(_) => "<Failed to decode>".to_string(),
            };
            println!("{:<12} {:<40} {:>10.2}", id, title, size_kb);
        }

        Ok(())
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

    pub async fn insert(&self, post: &E6Post) -> Result<()> {
        let db_guard = self.db.read().await;
        let db = match db_guard.as_ref() {
            Some(db) => db,
            None => bail!("Database not initialized"),
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

        self.maybe_evict_old_entries().await?;
        self.maybe_compact().await?;

        Ok(())
    }

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

        self.maybe_evict_old_entries().await?;
        self.maybe_compact().await?;

        Ok(())
    }

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
            max_entries: self.max_posts,
            auto_compact_enabled: self.auto_compact,
        })
    }

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

#[derive(Debug, Default)]
pub struct PostCacheStats {
    pub entry_count: usize,
    pub file_size_bytes: u64,
    pub max_entries: usize,
    pub auto_compact_enabled: bool,
}

impl PostCacheStats {
    pub fn file_size_mb(&self) -> f64 {
        self.file_size_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn usage_percent(&self) -> f64 {
        if self.max_entries == 0 {
            0.0
        } else {
            (self.entry_count as f64 / self.max_entries as f64) * 100.0
        }
    }

    pub fn avg_entry_size_kb(&self) -> f64 {
        if self.entry_count == 0 {
            0.0
        } else {
            (self.file_size_bytes as f64 / self.entry_count as f64) / 1024.0
        }
    }
}

impl std::fmt::Display for PostCacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Post Cache Statistics:\n\
             - Entries: {} / {} ({:.1}% full)\n\
             - File Size: {:.2} MB\n\
             - Avg Entry Size: {:.2} KB\n\
             - Auto-Compact: {}",
            self.entry_count,
            self.max_entries,
            self.usage_percent(),
            self.file_size_mb(),
            self.avg_entry_size_kb(),
            if self.auto_compact_enabled {
                "enabled"
            } else {
                "disabled"
            }
        )
    }
}

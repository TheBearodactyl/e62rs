use color_eyre::eyre::Result;

use crate::{
    config::options::E62Rs,
    data::{Database, Entry},
    models::PoolEntry,
};

impl Entry for PoolEntry {
    fn name(&self) -> &str {
        &self.name
    }

    fn desc(&self) -> Option<&str> {
        Some(&self.description)
    }
}

#[derive(Clone, Default)]
pub struct PoolDatabase {
    pub pools: Database<PoolEntry>,
}

impl PoolDatabase {
    pub fn load() -> Result<Self> {
        let cfg = E62Rs::get()?;

        Ok(Self {
            pools: Database::from_csv(cfg.completion.pools.as_str())?,
        })
    }

    pub fn get_creator_name(&self, creator_id: i64) -> String {
        format!("User {}", creator_id)
    }

    /// # Safety
    #[inline(always)]
    pub unsafe fn iter_pools(&self) -> impl Iterator<Item = &PoolEntry> {
        let search_cfg = E62Rs::get_unsafe().search;
        let mut pools = unsafe {
            self.pools
                .buffer
                .iter()
                .filter(|pool| pool.post_ids.len() > search_cfg.min_posts_on_pool as usize)
                .filter(|pool| {
                    if search_cfg.show_inactive_pools {
                        true
                    } else {
                        pool.is_active
                    }
                })
                .collect::<Vec<&PoolEntry>>()
        };

        if search_cfg.sort_pools_by_post_count {
            pools.sort_by(|a, b| b.post_ids.len().cmp(&a.post_ids.len()));
        }

        pools.into_iter()
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<String> {
        self.pools.search(query, limit, 0.5)
    }

    pub fn autocomplete(&self, query: &str, limit: usize) -> Vec<String> {
        self.pools.autocomplete(query, limit)
    }

    pub fn exists(&self, pool_name: &str) -> bool {
        self.pools.exists(pool_name)
    }

    pub fn get_by_name(&self, pool_name: &str) -> Option<PoolEntry> {
        self.pools.get_by_name(pool_name)
    }

    pub fn list(&self) -> Vec<PoolEntry> {
        unsafe { self.iter_pools().cloned().collect::<Vec<_>>() }
    }
}

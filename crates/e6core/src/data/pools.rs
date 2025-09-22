use {
    crate::{
        data::{Database, Entry},
        models::PoolEntry,
    },
    anyhow::Result,
};

impl Entry for PoolEntry {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        Some(&self.description)
    }
}

#[derive(Clone, Default)]
pub struct PoolDatabase {
    pub pools: Database<PoolEntry>,
}

impl PoolDatabase {
    pub fn load() -> Result<Self> {
        Ok(Self {
            pools: Database::from_csv("data/pools.csv")?,
        })
    }

    pub fn get_creator_name(&self, creator_id: i64) -> String {
        format!("User {}", creator_id)
    }

    /// # Safety
    #[inline(always)]
    pub unsafe fn iter_pools(&self) -> impl Iterator<Item = &PoolEntry> {
        unsafe { self.pools.buffer.iter() }
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
}

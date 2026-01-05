//! pool db
use {
    crate::{data::Entry, getopt, models::PoolEntry},
    color_eyre::Result,
    hashbrown::HashSet,
    nucleo_matcher::{
        Config, Matcher,
        pattern::{CaseMatching, Normalization, Pattern},
    },
    radix_trie::{Trie, TrieCommon},
    std::{fs::File, sync::Arc},
};

impl Entry for PoolEntry {
    fn name(&self) -> &str {
        &self.name
    }

    fn desc(&self) -> Option<&str> {
        Some(&self.description)
    }
}

/// db for pool entries
#[derive(Clone, Default)]
pub struct PoolDb {
    /// the pools
    pool_trie: Trie<String, Arc<PoolEntry>>,
    /// presorted pools db
    sorted_pools: Vec<Arc<PoolEntry>>,
    /// the names of each pool
    pool_names: HashSet<String>,
}

impl PoolDb {
    /// loads pool data from the path set in `completion.pools`
    pub fn load() -> Result<Self> {
        let path = getopt!(completion.pools);
        let file = File::open(path.as_str())?;
        let mut rdr = csv::Reader::from_reader(file);

        let mut pools: Vec<PoolEntry> = Vec::new();
        for res in rdr.deserialize() {
            pools.push(res?);
        }

        let min_posts = getopt!(search.min_posts_on_pool) as usize;
        let show_inactive = getopt!(search.show_inactive_pools);

        let mut pool_trie = Trie::new();
        let mut pool_names = HashSet::with_capacity(pools.len());

        let mut sorted_pools: Vec<Arc<PoolEntry>> = pools
            .into_iter()
            .filter(|p| p.post_ids.len() > min_posts && (show_inactive || p.is_active))
            .map(|p| {
                let arc = Arc::new(p);
                pool_trie.insert(arc.name.to_lowercase(), arc.clone());
                pool_names.insert(arc.name.clone());
                arc
            })
            .collect();

        if getopt!(search.sort_pools_by_post_count) {
            sorted_pools.sort_by(|a, b| b.post_ids.len().cmp(&a.post_ids.len()));
        }

        Ok(Self {
            pool_trie,
            sorted_pools,
            pool_names,
        })
    }

    #[inline(always)]
    /// returns an iterator over pools matching cfg filters
    pub fn iter_pools(&self) -> impl Iterator<Item = &PoolEntry> {
        self.sorted_pools.iter().map(|a| a.as_ref())
    }

    /// searches for pools matching the given query (uses 0.5 sim threshold)
    pub fn search(&self, query: &str, limit: usize) -> Vec<String> {
        if query.is_empty() {
            return Vec::new();
        }

        let mut matcher = Matcher::new(Config::DEFAULT);
        let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);

        let mut scored: Vec<(u32, &str)> = self
            .sorted_pools
            .iter()
            .filter_map(|pool| {
                let mut buf = Vec::new();
                let name_score = pattern.score(
                    nucleo_matcher::Utf32Str::new(&pool.name, &mut buf),
                    &mut matcher,
                );
                let desc_score = pattern.score(
                    nucleo_matcher::Utf32Str::new(&pool.description, &mut buf),
                    &mut matcher,
                );
                name_score.max(desc_score).map(|s| (s, pool.name.as_str()))
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored
            .into_iter()
            .take(limit)
            .map(|(_, n)| n.to_string())
            .collect()
    }

    /// returns autocompletions for pool names
    pub fn autocomplete(&self, query: &str, limit: usize) -> Vec<String> {
        if query.is_empty() {
            return Vec::new();
        }

        let query_lower = query.to_lowercase();
        let mut results = Vec::with_capacity(limit);

        if let Some(subtrie) = self.pool_trie.get_raw_descendant(&query_lower) {
            for (_, pool) in subtrie.iter().take(limit) {
                results.push(pool.name.clone());
            }
        }

        results
    }

    /// checks if a pool with the given name exists
    pub fn exists(&self, pool_name: &str) -> bool {
        self.pool_names.contains(pool_name)
    }

    /// retrieves a pool by exact name match (returns None if no matches)
    pub fn get_by_name(&self, name: &str) -> Option<&PoolEntry> {
        self.pool_trie.get(&name.to_lowercase()).map(|a| a.as_ref())
    }

    /// returns all pools matching cfg filters
    pub fn list(&self) -> Vec<PoolEntry> {
        self.sorted_pools.iter().map(|a| (**a).clone()).collect()
    }
}

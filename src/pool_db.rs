use {
    crate::models::PoolEntry,
    anyhow::Result,
    std::{fs::File, slice, sync::Arc},
};

#[derive(Clone)]
pub struct PoolDatabase {
    pub pools: Arc<PoolBuffer>,
}

pub struct PoolBuffer {
    ptr: *const PoolEntry,
    len: usize,
}

unsafe impl Send for PoolBuffer {}
unsafe impl Sync for PoolBuffer {}

impl Drop for PoolBuffer {
    fn drop(&mut self) {
        if self.ptr.is_null() || self.len == 0 {
            return;
        }

        unsafe {
            let slice = slice::from_raw_parts_mut(self.ptr as *mut PoolEntry, self.len);
            drop(Box::from_raw(slice));
        }
    }
}

impl Default for PoolDatabase {
    fn default() -> Self {
        Self {
            pools: Arc::new(PoolBuffer {
                ptr: std::ptr::null(),
                len: 0,
            }),
        }
    }
}

impl PoolBuffer {
    pub fn len(&self) -> usize {
        self.len
    }
}

impl PoolDatabase {
    pub fn load() -> Result<Self> {
        let file = File::open("data/pools.csv")?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut pools = Vec::new();

        for res in rdr.deserialize() {
            let pool: PoolEntry = res?;
            pools.push(pool);
        }

        let boxed: Box<[PoolEntry]> = pools.into_boxed_slice();
        let len = boxed.len();
        let ptr = boxed.as_ptr();

        let _ = Box::into_raw(boxed);

        Ok(Self {
            pools: Arc::new(PoolBuffer { ptr, len }),
        })
    }

    pub fn get_creator_name(&self, creator_id: i64) -> String {
        format!("User {}", creator_id)
    }

    #[inline(always)]
    pub unsafe fn iter_pools(&self) -> impl Iterator<Item = &PoolEntry> { unsafe {
        slice::from_raw_parts(self.pools.ptr, self.pools.len).iter()
    }}

    #[inline(always)]
    fn lowercase(s: &str) -> String {
        if s.is_ascii() {
            let mut out = String::with_capacity(s.len());
            unsafe {
                let bytes = s.as_bytes();
                out.as_mut_vec().extend(bytes.iter().map(|&b| {
                    if b.is_ascii_uppercase() {
                        b + 32
                    } else {
                        b
                    }
                }));
                out.as_mut_vec().set_len(s.len());
            }
            out
        } else {
            s.to_lowercase()
        }
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<String> {
        let query_lower = Self::lowercase(query);
        let mut matches: Vec<(f64, String)> = Vec::new();

        unsafe {
            for entry in self.iter_pools() {
                let name_lower = Self::lowercase(&entry.name);
                let desc_lower = Self::lowercase(&entry.description);

                let name_similarity = strsim::jaro_winkler(&name_lower, &query_lower);
                let desc_similarity = strsim::jaro_winkler(&desc_lower, &query_lower);
                let max_similarity = name_similarity.max(desc_similarity);

                if max_similarity > 0.5
                    || name_lower.contains(&query_lower)
                    || desc_lower.contains(&query_lower)
                {
                    matches.push((max_similarity, entry.name.clone()));
                }
            }
        }

        matches.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        matches
            .into_iter()
            .take(limit)
            .map(|(_, name)| name)
            .collect()
    }

    pub fn autocomplete(&self, query: &str, limit: usize) -> Vec<String> {
        let query_lower = Self::lowercase(query);
        let mut results = Vec::new();

        unsafe {
            for entry in self.iter_pools() {
                if Self::lowercase(&entry.name).contains(&query_lower) {
                    results.push(entry.name.clone());
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        results
    }

    pub fn exists(&self, pool_name: &str) -> bool {
        unsafe { self.iter_pools().any(|entry| entry.name == pool_name) }
    }

    pub fn get_by_name(&self, pool_name: &str) -> Option<PoolEntry> {
        unsafe {
            self.iter_pools()
                .find(|entry| entry.name == pool_name)
                .cloned()
        }
    }
}

//! cache stats

use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
/// stats for the post cache
pub struct PostCacheStats {
    /// the number of entries in the cache
    pub entry_count: usize,
    /// the size of the post cache
    pub file_size_bytes: u64,
    /// the maximum number of entries allowed in the cache
    pub max_entries: usize,
    /// whether auto-compact is enabled
    pub auto_compact_enabled: bool,
}

impl PostCacheStats {
    /// return the size of the post cache in MB
    pub fn file_size_mb(&self) -> f64 {
        self.file_size_bytes as f64 / (1024.0 * 1024.0)
    }

    /// return the percentage of the cache that's currently used
    pub fn usage_percent(&self) -> f64 {
        if self.max_entries == 0 {
            0.0
        } else {
            (self.entry_count as f64 / self.max_entries as f64) * 100.0
        }
    }

    /// return the average size of an entry in KB
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
            "Post Cache Statistics:\n- Entries: {} / {} ({:.1}% full)\n- File Size: {:.2} MB\n- \
             Avg Entry Size: {:.2} KB\n- Auto-Compact: {}",
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

#[derive(Debug, Default)]
/// stats for the http cache
pub struct CacheStats {
    /// the number of times an entry has been found
    pub hits: AtomicU64,
    /// the number of times an entry hasn't been found
    pub misses: AtomicU64,
    /// the number of entries that have been evicted
    pub evictions: AtomicU64,
    /// the number of entries that have expired
    pub expired: AtomicU64,
}

impl CacheStats {
    /// get the rate at which cache searches result in a hit
    #[macroni_n_cheese::mathinator2000]
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let total = hits + self.misses.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            (hits / total) as f64
        }
    }

    /// reset stats
    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.evictions.store(0, Ordering::Relaxed);
        self.expired.store(0, Ordering::Relaxed);
    }
}

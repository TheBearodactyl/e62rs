//! db mangament stuff

use {
    crate::{getopt, sendsync},
    color_eyre::Result,
    rapidfuzz::fuzz,
    std::{fs::File, slice, sync::Arc},
};

pub mod pools;
pub mod tags;

/// a database entry
pub trait Entry: Clone + Send + Sync + for<'de> serde::Deserialize<'de> {
    /// the name of the entry
    fn name(&self) -> &str;
    /// an optional description of the entry (defaults to None)
    fn desc(&self) -> Option<&str> {
        None
    }
}

/// an entry buffer
#[derive(Debug)]
pub struct Buffer<T: Entry> {
    /// the entry slice
    ptr: *const T,
    /// the length of the buffer
    len: usize,
}

sendsync!(Buffer<T>);

impl<T: Entry> Drop for Buffer<T> {
    fn drop(&mut self) {
        if self.ptr.is_null() || self.len == 0 {
            return;
        }

        unsafe {
            let slice = slice::from_raw_parts_mut(self.ptr as *mut T, self.len);
            drop(Box::from_raw(slice));
        }
    }
}

impl<T: Entry> Buffer<T> {
    /// get the length of the buffer
    pub fn len(&self) -> usize {
        self.len
    }

    /// check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline(always)]
    /// returns an iterator over the entries
    ///
    /// # Safety
    /// caller needs to make sure the buffer isn't modified/dropped while iterating
    pub unsafe fn iter(&self) -> impl Iterator<Item = &T> {
        unsafe { slice::from_raw_parts(self.ptr, self.len).iter() }
    }
}

#[derive(Clone, Debug)]
/// a database
pub struct Db<T: Entry> {
    /// the db buffer
    pub buf: Arc<Buffer<T>>,
}

impl<T: Entry> Default for Db<T> {
    fn default() -> Self {
        Self {
            buf: Arc::new(Buffer {
                ptr: std::ptr::null(),
                len: 0,
            }),
        }
    }
}

impl<T: Entry> Db<T> {
    /// load a db from a csv file
    pub fn from_csv(file_path: &str) -> Result<Self> {
        let file = File::open(file_path)?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut entries = Vec::new();

        for res in rdr.deserialize() {
            let entry: T = res?;
            entries.push(entry);
        }

        let boxed: Box<[T]> = entries.into_boxed_slice();
        let len = boxed.len();
        let ptr = boxed.as_ptr();

        let _ = Box::into_raw(boxed);

        Ok(Self {
            buf: Arc::new(Buffer { ptr, len }),
        })
    }

    #[inline(always)]
    /// make a string lowercase
    fn lowercase(s: &str) -> String {
        if s.is_ascii() {
            let mut out = String::with_capacity(s.len());

            unsafe {
                let bytes = s.as_bytes();
                let out_bytes = out.as_mut_vec();

                out_bytes.extend(
                    bytes
                        .iter()
                        .map(|&b| if b.is_ascii_uppercase() { b + 32 } else { b }),
                );

                out_bytes.set_len(s.len());
            }

            out
        } else {
            s.to_lowercase()
        }
    }

    /// searches for entries matching the given query (fuzz: name and desc apply)
    pub fn search(&self, query: &str, limit: usize, sim_threshold: f64) -> Vec<String> {
        let query_lower = Self::lowercase(query);
        let mut matches: Vec<(f64, String)> = Vec::new();

        unsafe {
            for entry in self.buf.iter() {
                let name_lower = Self::lowercase(entry.name());
                let name_sim = fuzz::ratio(name_lower.chars(), query_lower.chars()) / 100.0;

                let max_sim = if let Some(desc) = entry.desc() {
                    let desc_lower = Self::lowercase(desc);
                    let desc_sim = fuzz::ratio(desc_lower.chars(), query_lower.chars()) / 100.0;

                    name_sim.max(desc_sim)
                } else {
                    name_sim
                };

                if max_sim > sim_threshold {
                    matches.push((max_sim, entry.name().to_string()));
                }
            }
        }

        matches.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        matches
            .into_iter()
            .take(limit)
            .map(|(_, name)| name)
            .collect()
    }

    /// returns autocompletions for the query (fuzzy: name and desc apply)
    pub fn autocomplete(&self, query: &str, limit: usize) -> Vec<String> {
        let query_lower = Self::lowercase(query);
        let mut results = Vec::new();

        unsafe {
            for entry in self.buf.iter() {
                let name_lower = Self::lowercase(entry.name());
                let name_sim = fuzz::ratio(name_lower.chars(), query_lower.chars());

                if name_sim > getopt!(completion.tag_similarity_threshold) {
                    results.push(entry.name().to_string());

                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        results
    }

    /// checks if an entry exists with the given name
    pub fn exists(&self, name: &str) -> bool {
        unsafe { self.buf.iter().any(|entry| entry.name() == name) }
    }

    /// retrieves an entry by exact name match (returns None if none exists)
    pub fn get_by_name(&self, name: &str) -> Option<T> {
        unsafe { self.buf.iter().find(|entry| entry.name() == name).cloned() }
    }
}

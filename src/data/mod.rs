use {
    crate::config::options::E62Rs,
    color_eyre::eyre::Result,
    rapidfuzz::fuzz,
    std::{fs::File, slice, sync::Arc},
};

pub mod pools;
pub mod tags;

pub trait Entry: Clone + Send + Sync + for<'de> serde::Deserialize<'de> {
    fn name(&self) -> &str;
    fn desc(&self) -> Option<&str> {
        None
    }
}

pub struct Buffer<T: Entry> {
    ptr: *const T,
    len: usize,
}

unsafe impl<T: Entry> Send for Buffer<T> {}
unsafe impl<T: Entry> Sync for Buffer<T> {}

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
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// # Safety
    #[inline(always)]
    pub unsafe fn iter(&self) -> impl Iterator<Item = &T> {
        unsafe { slice::from_raw_parts(self.ptr, self.len).iter() }
    }
}

#[derive(Clone)]
pub struct Database<T: Entry> {
    pub buffer: Arc<Buffer<T>>,
}

impl<T: Entry> Default for Database<T> {
    fn default() -> Self {
        Self {
            buffer: Arc::new(Buffer {
                ptr: std::ptr::null(),
                len: 0,
            }),
        }
    }
}

impl<T: Entry> Database<T> {
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
            buffer: Arc::new(Buffer { ptr, len }),
        })
    }

    #[inline(always)]
    fn lowercase(s: &str) -> String {
        if s.is_ascii() {
            let mut out = String::with_capacity(s.len());
            unsafe {
                let bytes = s.as_bytes();
                out.as_mut_vec().extend(
                    bytes
                        .iter()
                        .map(|&b| if b.is_ascii_uppercase() { b + 32 } else { b }),
                );
                out.as_mut_vec().set_len(s.len());
            }
            out
        } else {
            s.to_lowercase()
        }
    }

    pub fn search(&self, query: &str, limit: usize, similarity_threshold: f64) -> Vec<String> {
        let query_lower = Self::lowercase(query);
        let mut matches: Vec<(f64, String)> = Vec::new();

        unsafe {
            for entry in self.buffer.iter() {
                let name_lower = Self::lowercase(entry.name());
                let name_similarity = fuzz::ratio(name_lower.chars(), query_lower.chars()) / 100.0;

                let max_similarity = if let Some(desc) = entry.desc() {
                    let desc_lower = desc.to_lowercase();
                    let desc_similarity =
                        fuzz::ratio(desc_lower.chars(), query_lower.chars()) / 100.0;

                    name_similarity.max(desc_similarity)
                } else {
                    name_similarity
                };

                if max_similarity > similarity_threshold {
                    matches.push((max_similarity, entry.name().to_string()));
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
        let query_lower = query.to_lowercase();
        let completion_cfg = E62Rs::get_unsafe().completion;
        let mut results = Vec::new();

        unsafe {
            for entry in self.buffer.iter() {
                let name_lower = entry.name().to_lowercase();
                let name_similarity = fuzz::ratio(name_lower.chars(), query_lower.chars());

                if name_similarity > completion_cfg.tag_similarity_threshold {
                    results.push(entry.name().to_string());
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        results
    }

    pub fn exists(&self, name: &str) -> bool {
        unsafe { self.buffer.iter().any(|entry| entry.name() == name) }
    }

    pub fn get_by_name(&self, name: &str) -> Option<T> {
        unsafe {
            self.buffer
                .iter()
                .find(|entry| entry.name() == name)
                .cloned()
        }
    }
}

use {
    crate::models::TagEntry,
    anyhow::Result,
    std::{fs::File, slice, sync::Arc},
};

#[derive(Clone)]
pub struct TagDatabase {
    pub tags: Arc<TagBuffer>,
}

pub struct TagBuffer {
    ptr: *const TagEntry,
    len: usize,
}

unsafe impl Send for TagBuffer {}
unsafe impl Sync for TagBuffer {}

impl Drop for TagBuffer {
    fn drop(&mut self) {
        if self.ptr.is_null() || self.len == 0 {
            return;
        }

        unsafe {
            let slice = slice::from_raw_parts_mut(self.ptr as *mut TagEntry, self.len);
            drop(Box::from_raw(slice));
        }
    }
}

impl Default for TagDatabase {
    fn default() -> Self {
        Self {
            tags: Arc::new(TagBuffer {
                ptr: std::ptr::null(),
                len: 0,
            }),
        }
    }
}

impl TagBuffer {
    pub fn len(&self) -> usize {
        self.len
    }
}

impl TagDatabase {
    pub fn load() -> Result<Self> {
        let file = File::open("data/tags.csv")?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut tags = Vec::new();

        for res in rdr.deserialize() {
            let tag: TagEntry = res?;
            tags.push(tag);
        }

        let boxed: Box<[TagEntry]> = tags.into_boxed_slice();
        let len = boxed.len();
        let ptr = boxed.as_ptr();

        let _ = Box::into_raw(boxed);

        Ok(Self {
            tags: Arc::new(TagBuffer { ptr, len }),
        })
    }

    #[inline(always)]
    unsafe fn iter_tags(&self) -> impl Iterator<Item = &TagEntry> { unsafe {
        slice::from_raw_parts(self.tags.ptr, self.tags.len).iter()
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
            for entry in self.iter_tags() {
                let name_lower = Self::lowercase(&entry.name);
                let similarity = strsim::jaro_winkler(&name_lower, &query_lower);
                if similarity > 0.7 {
                    matches.push((similarity, entry.name.clone()));
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
            for entry in self.iter_tags() {
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

    pub fn exists(&self, tag: &str) -> bool {
        unsafe { self.iter_tags().any(|entry| entry.name == tag) }
    }
}

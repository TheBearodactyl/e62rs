use {
    crate::{
        data::{Database, Entry},
        models::TagEntry,
    },
    anyhow::Result,
};

impl Entry for TagEntry {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Default)]
pub struct TagDatabase {
    pub tags: Database<TagEntry>,
}

impl TagDatabase {
    pub fn load() -> Result<Self> {
        Ok(Self {
            tags: Database::from_csv("data/tags.csv")?,
        })
    }

    /// # Safety
    #[inline(always)]
    pub unsafe fn iter_tags(&self) -> impl Iterator<Item = &TagEntry> {
        unsafe { self.tags.buffer.iter() }
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<String> {
        self.tags.search(query, limit, 0.7)
    }

    pub fn autocomplete(&self, query: &str, limit: usize) -> Vec<String> {
        self.tags.autocomplete(query, limit)
    }

    pub fn exists(&self, tag: &str) -> bool {
        self.tags.exists(tag)
    }
}

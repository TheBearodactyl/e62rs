use color_eyre::eyre::Result;

use crate::{
    config::options::E62Rs,
    data::{Database, Entry},
    models::TagEntry,
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
        let cfg = E62Rs::get()?;

        Ok(Self {
            tags: Database::from_csv(cfg.completion.tags.as_str())?,
        })
    }

    /// # Safety
    #[inline(always)]
    pub unsafe fn iter_tags(&self) -> impl Iterator<Item = &TagEntry> {
        let search_cfg = E62Rs::get_unsafe().search;

        let mut tags = unsafe {
            self.tags
                .buffer
                .iter()
                .filter(|tag| tag.post_count > search_cfg.min_posts_on_tag as i64)
                .collect::<Vec<&TagEntry>>()
        };

        if search_cfg.sort_tags_by_post_count {
            tags.sort_by(|a, b| b.post_count.cmp(&a.post_count));
        }

        if search_cfg.reverse_tags_order {
            tags.reverse();
        }

        tags.into_iter()
    }

    pub fn list(&self) -> Vec<TagEntry> {
        unsafe { self.tags.buffer.iter().cloned().collect::<Vec<TagEntry>>() }
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

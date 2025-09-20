use crate::models::TagEntry;
use anyhow::Result;
use std::{fs::File, sync::Arc};

#[derive(Clone, Default)]
pub struct TagDatabase {
    tags: Arc<Vec<TagEntry>>,
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

        Ok(Self {
            tags: Arc::new(tags),
        })
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let mut matches: Vec<(f64, String)> = self
            .tags
            .iter()
            .filter_map(|entry| {
                let name_lower = entry.name.to_lowercase();
                let similarity = strsim::jaro_winkler(&name_lower, &query_lower);

                if similarity > 0.7 {
                    Some((similarity, entry.name.clone()))
                } else {
                    None
                }
            })
            .collect();

        matches.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        matches
            .into_iter()
            .take(limit)
            .map(|(_, name)| name)
            .collect()
    }

    pub fn autocomplete(&self, query: &str, limit: usize) -> Vec<String> {
        let query_lower = query.to_lowercase();

        self.tags
            .iter()
            .filter(|entry| entry.name.to_lowercase().contains(&query_lower))
            .take(limit)
            .map(|entry| entry.name.clone())
            .collect()
    }

    pub fn exists(&self, tag: &str) -> bool {
        self.tags.iter().any(|entry| entry.name == tag)
    }
}

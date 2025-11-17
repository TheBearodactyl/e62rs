use {
    crate::{
        config::options::E62Rs,
        data::{Database, Entry},
        models::{TagAliasEntry, TagEntry, TagImplicationEntry},
    },
    color_eyre::eyre::Result,
    std::collections::{HashMap, HashSet},
};

impl Entry for TagEntry {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Entry for TagAliasEntry {
    fn name(&self) -> &str {
        &self.antecedent_name
    }
}

impl Entry for TagImplicationEntry {
    fn name(&self) -> &str {
        &self.antecedent_name
    }
}

#[derive(Clone, Default)]
pub struct TagDatabase {
    pub tags: Database<TagEntry>,
    pub aliases: Database<TagAliasEntry>,
    pub implications: Database<TagImplicationEntry>,
    alias_map: HashMap<String, String>,
    implication_map: HashMap<String, Vec<String>>,
}

impl TagDatabase {
    pub fn load() -> Result<Self> {
        let cfg = E62Rs::get()?;

        let tags: Database<TagEntry> = Database::from_csv(&cfg.completion.tags)?;
        let aliases: Database<TagAliasEntry> = Database::from_csv(&cfg.completion.tag_aliases)?;
        let implications: Database<TagImplicationEntry> =
            Database::from_csv(&cfg.completion.tag_implications)?;

        let mut alias_map = HashMap::new();
        unsafe {
            for alias in aliases.buffer.iter() {
                if alias.status == "active" {
                    alias_map.insert(alias.antecedent_name.clone(), alias.consequent_name.clone());
                }
            }
        }

        let mut implication_map: HashMap<String, Vec<String>> = HashMap::new();
        unsafe {
            for implication in implications.buffer.iter() {
                if implication.status == "active" {
                    implication_map
                        .entry(implication.antecedent_name.clone())
                        .or_default()
                        .push(implication.consequent_name.clone());
                }
            }
        }

        Ok(Self {
            tags,
            aliases,
            implications,
            alias_map,
            implication_map,
        })
    }

    pub fn resolve_alias(&self, tag: &str) -> String {
        let mut current = tag.to_string();
        let mut visited = HashSet::new();

        while let Some(aliased) = self.alias_map.get(&current) {
            if !visited.insert(current.clone()) {
                break;
            }

            current = aliased.clone();
        }

        current
    }

    pub fn get_implications(&self, tag: &str) -> Vec<String> {
        let resolved = self.resolve_alias(tag);
        self.implication_map
            .get(&resolved)
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_all_implications(&self, tag: &str) -> Vec<String> {
        let mut all_implications = Vec::new();
        let mut visited = HashSet::new();
        let mut to_process = vec![self.resolve_alias(tag)];

        while let Some(curr_tag) = to_process.pop() {
            if !visited.insert(curr_tag.clone()) {
                continue;
            }

            if let Some(implications) = self.implication_map.get(&curr_tag) {
                for implied in implications {
                    if !all_implications.contains(implied) {
                        all_implications.push(implied.clone());
                        to_process.push(implied.clone());
                    }
                }
            }
        }

        all_implications
    }

    /// # Safety
    #[inline(always)]
    pub unsafe fn iter_tags(&self) -> Result<impl Iterator<Item = &TagEntry>> {
        let cfg = E62Rs::get()?;

        let mut tags = unsafe {
            self.tags
                .buffer
                .iter()
                .filter(|tag| tag.post_count > cfg.search.min_posts_on_tag as i64)
                .collect::<Vec<&TagEntry>>()
        };

        if cfg.search.sort_tags_by_post_count {
            tags.sort_by(|a, b| b.post_count.cmp(&a.post_count));
        }

        if cfg.search.reverse_tags_order {
            tags.reverse();
        }

        Ok(tags.into_iter())
    }

    pub fn list(&self) -> Vec<TagEntry> {
        let mut result = Vec::new();

        unsafe {
            result.extend(self.tags.buffer.iter().cloned());
        }

        unsafe {
            for alias in self.aliases.buffer.iter() {
                if alias.status == "active"
                    && let Some(consequent_tag) = self.tags.get_by_name(&alias.consequent_name)
                {
                    result.push(TagEntry {
                        id: alias.id,
                        name: alias.antecedent_name.clone(),
                        category: consequent_tag.category,
                        post_count: consequent_tag.post_count,
                    });
                }
            }
        }

        result
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<String> {
        let mut results = self.tags.search(query, limit, 0.7);
        let alias_results = self.aliases.search(query, limit, 0.7);

        for alias_name in alias_results {
            let resolved = self.resolve_alias(&alias_name);
            if !results.contains(&resolved) {
                results.push(resolved);
            }
        }

        results.truncate(limit);
        results
    }

    pub fn autocomplete(&self, query: &str, limit: usize) -> Vec<String> {
        let mut results = self.tags.autocomplete(query, limit);
        let alias_results = self.aliases.autocomplete(query, limit);

        for alias_name in alias_results {
            let resolved = self.resolve_alias(&alias_name);
            if !results.contains(&resolved) {
                results.push(resolved);
            }
        }

        results.truncate(limit);
        results
    }

    pub fn exists(&self, tag: &str) -> bool {
        if self.tags.exists(tag) {
            return true;
        }

        if self.alias_map.contains_key(tag) {
            return true;
        }

        false
    }

    pub fn get_canonical_name(&self, tag: &str) -> String {
        self.resolve_alias(tag)
    }
}

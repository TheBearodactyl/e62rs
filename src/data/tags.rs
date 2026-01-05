//! tag db with alias/implication resolution

use {
    crate::{
        data::Entry,
        getopt,
        models::{TagAliasEntry, TagEntry, TagImplicationEntry},
    },
    color_eyre::Result,
    hashbrown::{HashMap, HashSet},
    nucleo_matcher::{
        Config, Matcher,
        pattern::{CaseMatching, Normalization, Pattern},
    },
    radix_trie::{Trie, TrieCommon},
    std::{fs::File, sync::Arc},
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

/// tag db. manages tags, aliases, and implications
#[derive(Clone, Debug)]
pub struct TagDb {
    /// indexed tags for fast prefix lookup
    tag_trie: Trie<String, Arc<TagEntry>>,
    /// indexed aliases for fast prefix lookup
    alias_trie: Trie<String, String>,
    /// sorted tags by post count for iteration
    sorted_tags: Vec<Arc<TagEntry>>,
    /// alias -> canonical tag translation map
    alias_map: HashMap<String, String>,
    /// implication -> tag(s) translation map
    impl_map: HashMap<String, Vec<String>>,
    /// set of all tag names for O(1) existence checks
    tag_names: HashSet<String>,
}

impl Default for TagDb {
    fn default() -> Self {
        Self {
            tag_trie: Trie::new(),
            alias_trie: Trie::new(),
            sorted_tags: Vec::new(),
            alias_map: HashMap::new(),
            impl_map: HashMap::new(),
            tag_names: HashSet::new(),
        }
    }
}

impl TagDb {
    /// loads tag data from configured csv files
    pub fn load() -> Result<Self> {
        let tags_path = getopt!(completion.tags);
        let aliases_path = getopt!(completion.aliases);
        let impls_path = getopt!(completion.implications);
        let mut tags: Vec<TagEntry> = Vec::new();
        let file = File::open(&tags_path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for res in rdr.deserialize() {
            tags.push(res?);
        }

        let mut aliases: Vec<TagAliasEntry> = Vec::new();
        let file = File::open(&aliases_path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for res in rdr.deserialize() {
            aliases.push(res?);
        }

        let mut impls: Vec<TagImplicationEntry> = Vec::new();
        let file = File::open(&impls_path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for res in rdr.deserialize() {
            impls.push(res?);
        }

        let mut alias_map = HashMap::with_capacity(aliases.len());
        for alias in &aliases {
            if alias.status == "active" {
                alias_map.insert(alias.antecedent_name.clone(), alias.consequent_name.clone());
            }
        }

        let mut impl_map: HashMap<String, Vec<String>> = HashMap::new();
        for implication in &impls {
            if implication.status == "active" {
                impl_map
                    .entry(implication.antecedent_name.clone())
                    .or_default()
                    .push(implication.consequent_name.clone());
            }
        }

        let mut tag_trie = Trie::new();
        let mut tag_names = HashSet::with_capacity(tags.len());
        for tag in &tags {
            let arc_tag = Arc::new(tag.clone());
            tag_trie.insert(tag.name.clone(), arc_tag);
            tag_names.insert(tag.name.clone());
        }

        let mut alias_trie = Trie::new();
        for alias in &aliases {
            if alias.status == "active" {
                alias_trie.insert(alias.antecedent_name.clone(), alias.consequent_name.clone());
            }
        }

        let min_posts = getopt!(search.min_posts_on_tag) as i64;
        let mut sorted_tags: Vec<Arc<TagEntry>> = tags
            .into_iter()
            .filter(|t| t.post_count > min_posts)
            .map(Arc::new)
            .collect();

        if getopt!(search.sort_tags_by_post_count) {
            sorted_tags.sort_by(|a, b| b.post_count.cmp(&a.post_count));
        }

        if getopt!(search.reverse_tags_order) {
            sorted_tags.reverse();
        }

        Ok(Self {
            tag_trie,
            alias_trie,
            sorted_tags,
            alias_map,
            impl_map,
            tag_names,
        })
    }

    /// resolves a tag name through its alias chain
    #[inline]
    pub fn resolve_alias(&self, tag: &str) -> String {
        let mut curr = tag;
        let mut visited = HashSet::new();

        while let Some(aliased) = self.alias_map.get(curr) {
            if !visited.insert(curr) {
                break;
            }

            curr = aliased;
        }

        curr.to_string()
    }

    /// returns direct implications for a tag
    pub fn get_implications(&self, tag: &str) -> Vec<String> {
        let resolved = self.resolve_alias(tag);
        self.impl_map.get(&resolved).cloned().unwrap_or_default()
    }

    /// returns all implications recursively for a tag
    pub fn get_all_implications(&self, tag: &str) -> Vec<String> {
        let mut all_impls = Vec::new();
        let mut visited = HashSet::new();
        let mut to_process = vec![self.resolve_alias(tag)];

        while let Some(curr_tag) = to_process.pop() {
            if !visited.insert(curr_tag.clone()) {
                continue;
            }

            if let Some(implications) = self.impl_map.get(&curr_tag) {
                for implied in implications {
                    if !all_impls.contains(implied) {
                        all_impls.push(implied.clone());
                        to_process.push(implied.clone());
                    }
                }
            }
        }

        all_impls
    }

    /// returns an iterator over tags matching configured filters
    ///
    /// # Safety
    /// database must not be modified/dropped while iterating
    #[inline(always)]
    pub unsafe fn iter_tags(&self) -> Result<impl Iterator<Item = &TagEntry>> {
        Ok(self.sorted_tags.iter().map(|arc| arc.as_ref()))
    }

    /// returns all tags including resolved aliases
    pub fn list(&self) -> Vec<TagEntry> {
        let mut result: Vec<TagEntry> =
            self.sorted_tags.iter().map(|arc| (**arc).clone()).collect();

        for (alias_name, canonical) in &self.alias_map {
            if let Some(consequent) = self.tag_trie.get(canonical) {
                result.push(TagEntry {
                    id: 0,
                    name: alias_name.clone(),
                    category: consequent.category,
                    post_count: consequent.post_count,
                });
            }
        }

        result
    }

    /// searches for tags matching a query (searches: tags and aliases)
    pub fn search(&self, query: &str, limit: usize) -> Vec<String> {
        if query.is_empty() {
            return Vec::new();
        }

        let mut matcher = Matcher::new(Config::DEFAULT.match_paths());
        let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);
        let mut scored: Vec<(u32, String)> = Vec::new();

        for tag in &self.sorted_tags {
            let mut buf = Vec::new();
            if let Some(score) = pattern.score(
                nucleo_matcher::Utf32Str::new(&tag.name, &mut buf),
                &mut matcher,
            ) {
                scored.push((score, tag.name.clone()));
            }
        }

        for (alias, canonical) in &self.alias_map {
            let mut buf = Vec::new();
            if let Some(score) =
                pattern.score(nucleo_matcher::Utf32Str::new(alias, &mut buf), &mut matcher)
            {
                let resolved = self.resolve_alias(canonical);
                scored.push((score, resolved));
            }
        }

        scored.sort_by(|a, b| b.0.cmp(&a.0));

        let mut seen = HashSet::new();
        scored
            .into_iter()
            .filter_map(|(_, name)| {
                if seen.insert(name.clone()) {
                    Some(name)
                } else {
                    None
                }
            })
            .take(limit)
            .collect()
    }

    /// returns autocompletions for tag names
    /// (includes: tags and aliases, resolves to: canonical names)
    pub fn autocomplete(&self, query: &str, limit: usize) -> Vec<String> {
        if query.is_empty() {
            return Vec::new();
        }

        let query_lower = query.to_lowercase();
        let mut results = Vec::with_capacity(limit);
        let mut seen = HashSet::new();

        if let Some(subtrie) = self.tag_trie.get_raw_descendant(&query_lower) {
            for (key, _) in subtrie.iter().take(limit * 2) {
                if seen.insert(key.clone()) {
                    results.push(key.clone());
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        if results.len() < limit
            && let Some(subtrie) = self.alias_trie.get_raw_descendant(&query_lower)
        {
            for (_, canonical) in subtrie.iter().take(limit) {
                let resolved = self.resolve_alias(canonical);
                if seen.insert(resolved.clone()) {
                    results.push(resolved);
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        results.truncate(limit);
        results
    }

    /// checks if a tag/alias exists with the given name
    pub fn exists(&self, tag: &str) -> bool {
        self.tag_names.contains(tag) || self.alias_map.contains_key(tag)
    }

    /// returns the canonical name for a given tag/alias
    pub fn get_canon_name(&self, tag: &str) -> String {
        self.resolve_alias(tag)
    }
}

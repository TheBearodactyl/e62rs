use crate::utils::*;
use e6cfg::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E6PostsResponse {
    #[serde(default)]
    pub posts: Vec<E6Post>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E6PostResponse {
    #[serde(default)]
    pub post: E6Post,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct E6Post {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub file: File,
    #[serde(default)]
    pub preview: Preview,
    #[serde(default)]
    pub sample: Sample,
    #[serde(default)]
    pub score: Score,
    #[serde(default)]
    pub tags: Tags,
    #[serde(default)]
    pub locked_tags: Vec<String>,
    #[serde(default)]
    pub change_seq: i64,
    #[serde(default)]
    pub flags: Flags,
    #[serde(default)]
    pub rating: String,
    #[serde(default)]
    pub fav_count: i64,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub pools: Vec<i64>,
    #[serde(default)]
    pub relationships: Relationships,
    #[serde(default)]
    pub approver_id: Option<i64>,
    #[serde(default)]
    pub uploader_id: i64,
    #[serde(default)]
    pub uploader_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub comment_count: i64,
    #[serde(default)]
    pub is_favorited: bool,
    #[serde(default)]
    pub has_notes: bool,
    #[serde(default)]
    pub duration: Option<f64>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
pub struct File {
    #[serde(default)]
    pub width: i64,
    #[serde(default)]
    pub height: i64,
    #[serde(default)]
    pub ext: String,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub md5: String,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
pub struct Preview {
    #[serde(default)]
    pub width: i64,
    #[serde(default)]
    pub height: i64,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub alt: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sample {
    #[serde(default)]
    pub has: bool,
    #[serde(default)]
    pub width: i64,
    #[serde(default)]
    pub height: i64,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub alt: Option<String>,
    #[serde(default)]
    pub alternates: Alternates,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alternates {
    #[serde(default)]
    pub has: bool,
    #[serde(default)]
    pub original: Option<Original>,
    #[serde(default)]
    pub variants: Option<Variants>,
    #[serde(default)]
    pub samples: Option<Samples>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Original {
    #[serde(default)]
    pub fps: f64,
    #[serde(default)]
    pub codec: String,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub width: i64,
    #[serde(default)]
    pub height: i64,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variants {
    #[serde(default)]
    pub mp4: Option<Mp4>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mp4 {
    #[serde(default)]
    pub codec: String,
    #[serde(default)]
    pub fps: f64,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub width: i64,
    #[serde(default)]
    pub height: i64,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Samples(pub HashMap<String, Quality>);

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quality {
    #[serde(default)]
    pub fps: f64,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub codec: String,
    #[serde(default)]
    pub width: i64,
    #[serde(default)]
    pub height: i64,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Score {
    #[serde(default)]
    pub up: i64,
    #[serde(default)]
    pub down: i64,
    #[serde(default)]
    pub total: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tags {
    #[serde(default)]
    pub general: Vec<String>,
    #[serde(default)]
    pub artist: Vec<String>,
    #[serde(default)]
    pub contributor: Vec<String>,
    #[serde(default)]
    pub copyright: Vec<String>,
    #[serde(default)]
    pub character: Vec<String>,
    #[serde(default)]
    pub species: Vec<String>,
    #[serde(default)]
    pub invalid: Vec<String>,
    #[serde(default)]
    pub meta: Vec<String>,
    #[serde(default)]
    pub lore: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Flags {
    #[serde(default)]
    pub pending: bool,
    #[serde(default)]
    pub flagged: bool,
    #[serde(default)]
    pub note_locked: bool,
    #[serde(default)]
    pub status_locked: bool,
    #[serde(default)]
    pub rating_locked: bool,
    #[serde(default)]
    pub deleted: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relationships {
    #[serde(default)]
    pub parent_id: Option<i64>,
    #[serde(default)]
    pub has_children: bool,
    #[serde(default)]
    pub has_active_children: bool,
    #[serde(default)]
    pub children: Option<Vec<i64>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagEntry {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub category: i64,
    #[serde(default)]
    pub post_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E6PoolsResponse {
    #[serde(default)]
    pub pools: Vec<E6Pool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E6PoolResponse {
    #[serde(default)]
    pub pool: E6Pool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct E6Pool {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub creator_id: i64,
    #[serde(default)]
    pub creator_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub is_active: bool,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub post_ids: Vec<i64>,
    #[serde(default)]
    pub post_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoolEntry {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub creator_id: i64,
    #[serde(default)]
    pub description: String,
    #[serde(default, deserialize_with = "deserialize_bool_from_str")]
    pub is_active: bool,
    #[serde(default)]
    pub category: String,
    #[serde(default, deserialize_with = "deserialize_post_ids")]
    pub post_ids: Vec<i64>,
}

impl E6Post {
    pub fn is_blacklisted(&self) -> bool {
        if let Ok(config) = Cfg::get()
            && let Some(blacklist) = config.blacklist
        {
            if blacklist.is_empty() {
                return false;
            }

            let all_tags: Vec<&String> = self
                .tags
                .general
                .iter()
                .chain(self.tags.artist.iter())
                .chain(self.tags.contributor.iter())
                .chain(self.tags.copyright.iter())
                .chain(self.tags.character.iter())
                .chain(self.tags.species.iter())
                .chain(self.tags.meta.iter())
                .chain(self.tags.lore.iter())
                .collect();

            return all_tags.iter().any(|tag| blacklist.contains(tag));
        }

        false
    }

    pub fn search_includes_blacklisted(search_tags: &[String]) -> bool {
        if let Ok(config) = Cfg::get()
            && let Some(blacklist) = config.blacklist
        {
            return search_tags.iter().any(|tag| blacklist.contains(tag));
        }

        false
    }
}

impl E6PostsResponse {
    pub fn filter_blacklisted(mut self, search_tags: &[String]) -> Self {
        if E6Post::search_includes_blacklisted(search_tags) {
            return self;
        }

        self.posts.retain(|post| !post.is_blacklisted());
        self
    }
}

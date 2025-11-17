use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMetadata {
    pub id: i64,
    pub rating: String,
    pub score: i64,
    pub fav_count: i64,
    pub artists: Vec<String>,
    pub tags: Vec<String>,
    pub character_tags: Vec<String>,
    pub species_tags: Vec<String>,
    pub created_at: String,
    pub pools: Vec<i64>,
}

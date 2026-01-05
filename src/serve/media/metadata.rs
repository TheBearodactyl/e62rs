//! media metadata stuffuse serde::{Deserialize, Serialize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// a posts metadata
pub struct PostMetadata {
    /// the post id
    pub id: i64,
    /// the post content rating
    pub rating: String,
    /// the post score/votes
    pub score: i64,
    /// the posts favorite count
    pub fav_count: i64,
    /// the artist(s) of a post
    pub artists: Vec<String>,
    /// the tags of a post
    pub tags: Vec<String>,
    /// the characters in a post
    pub character_tags: Vec<String>,
    /// the species of a post
    pub species_tags: Vec<String>,
    /// the date a post was created at
    pub created_at: String,
    /// the pools a post is in
    pub pools: Vec<i64>,
}

//! All data types used by e621 for deserializing/serializing API responses.
//! (documentation for all models was AI generated)
use {
    crate::{
        getopt,
        utils::{deserialize_bool_from_str, deserialize_post_ids},
    },
    hashbrown::{HashMap, HashSet},
    serde::{Deserialize, Serialize},
};

/// Response from e621 API containing multiple posts, typically from a search or listing endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E6PostsResponse {
    /// List of posts returned by the API.
    #[serde(default)]
    pub posts: Vec<E6Post>,
}

/// Response from e621 API containing a single post.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E6PostResponse {
    /// The post object returned by the API.
    #[serde(default)]
    pub post: E6Post,
}

/// Represents a complete e621 post with all associated metadata.
/// This is the main data structure for individual posts on e621.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct E6Post {
    /// Unique identifier for the post.
    #[serde(default)]
    pub id: i64,
    /// ISO 8601 timestamp of when the post was created.
    #[serde(default)]
    pub created_at: String,
    /// ISO 8601 timestamp of when the post was last updated.
    #[serde(default)]
    pub updated_at: String,
    /// Information about the original uploaded file.
    #[serde(default)]
    pub file: File,
    /// Information about the preview/thumbnail image.
    #[serde(default)]
    pub preview: Preview,
    /// Information about the sample/resized version (if available).
    #[serde(default)]
    pub sample: Sample,
    /// Vote score statistics for the post.
    #[serde(default)]
    pub score: Score,
    /// Categorized tags associated with the post.
    #[serde(default)]
    pub tags: Tags,
    /// Tags that are locked and cannot be removed from the post.
    #[serde(default)]
    pub locked_tags: Vec<String>,
    /// Sequence number indicating the revision of the post (used for change tracking).
    #[serde(default)]
    pub change_seq: i64,
    /// Various status flags for the post (pending, flagged, deleted, etc.).
    #[serde(default)]
    pub flags: Flags,
    /// Content rating: "s" (safe), "q" (questionable), or "e" (explicit).
    #[serde(default)]
    pub rating: String,
    /// Number of users who have favorited this post.
    #[serde(default)]
    pub fav_count: i64,
    /// List of source URLs where the content originated from.
    #[serde(default)]
    pub sources: Vec<String>,
    /// IDs of pools that this post belongs to.
    #[serde(default)]
    pub pools: Vec<i64>,
    /// Parent/child relationship data for post hierarchies.
    #[serde(default)]
    pub relationships: Relationships,
    /// ID of the user who approved the post (if applicable).
    #[serde(default)]
    pub approver_id: Option<i64>,
    /// ID of the user who uploaded the post.
    #[serde(default)]
    pub uploader_id: i64,
    /// Username of the user who uploaded the post.
    #[serde(default)]
    pub uploader_name: String,
    /// Description or commentary for the post (may contain HTML).
    #[serde(default)]
    pub description: String,
    /// Number of comments on the post.
    #[serde(default)]
    pub comment_count: i64,
    /// Whether the currently authenticated user has favorited this post.
    #[serde(default)]
    pub is_favorited: bool,
    /// Whether the post has annotations/notes.
    #[serde(default)]
    pub has_notes: bool,
    /// Duration in seconds for video/audio posts (null for static images).
    #[serde(default)]
    pub duration: Option<f64>,
}

/// Contains metadata about the original uploaded file.
#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
pub struct File {
    /// Width of the original file in pixels.
    #[serde(default)]
    pub width: i64,
    /// Height of the original file in pixels.
    #[serde(default)]
    pub height: i64,
    /// File extension (e.g., "jpg", "png", "webm", "mp4").
    #[serde(default)]
    pub ext: String,
    /// File size in bytes.
    #[serde(default)]
    pub size: i64,
    /// MD5 hash of the file.
    #[serde(default)]
    pub md5: String,
    /// Direct URL to the original file (null if not available or requires higher privileges).
    #[serde(default)]
    pub url: Option<String>,
}

/// Contains metadata about the preview/thumbnail image.
#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
pub struct Preview {
    /// Width of the preview image in pixels.
    #[serde(default)]
    pub width: i64,
    /// Height of the preview image in pixels.
    #[serde(default)]
    pub height: i64,
    /// Direct URL to the preview image.
    #[serde(default)]
    pub url: Option<String>,
    /// Alt text for the preview image (typically null).
    #[serde(default)]
    pub alt: Option<String>,
}

/// Contains metadata about the sample/resized version of the post.
/// Samples are larger than previews but smaller than the original file.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sample {
    /// Whether a sample version is available for this post.
    #[serde(default)]
    pub has: bool,
    /// Width of the sample image in pixels.
    #[serde(default)]
    pub width: i64,
    /// Height of the sample image in pixels.
    #[serde(default)]
    pub height: i64,
    /// Direct URL to the sample image.
    #[serde(default)]
    pub url: Option<String>,
    /// Alt text for the sample image (typically null).
    #[serde(default)]
    pub alt: Option<String>,
    /// Alternate versions and qualities available (primarily for video posts).
    #[serde(default)]
    pub alternates: Alternates,
}

/// Contains alternate versions of the post, such as original video files and different quality levels.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alternates {
    /// Whether alternate versions are available.
    #[serde(default)]
    pub has: bool,
    /// Original video file information (for video posts).
    #[serde(default)]
    pub original: Option<Original>,
    /// Video variants in different formats (e.g., MP4).
    #[serde(default)]
    pub variants: Option<Variants>,
    /// Sample versions at different quality levels.
    #[serde(default)]
    pub samples: Option<Samples>,
}

/// Metadata for the original video file (when different from the main file).
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Original {
    /// Frames per second of the video.
    #[serde(default)]
    pub fps: f64,
    /// Video codec used (e.g., "vp8", "vp9", "h264").
    #[serde(default)]
    pub codec: String,
    /// File size in bytes.
    #[serde(default)]
    pub size: i64,
    /// Width in pixels.
    #[serde(default)]
    pub width: i64,
    /// Height in pixels.
    #[serde(default)]
    pub height: i64,
    /// Direct URL to the original video file.
    #[serde(default)]
    pub url: Option<String>,
}

/// Contains video variants in different formats.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variants {
    /// MP4 variant of the video.
    #[serde(default)]
    pub mp4: Option<Mp4>,
}

/// Metadata for an MP4 variant of a video post.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mp4 {
    /// Video codec used (e.g., "h264").
    #[serde(default)]
    pub codec: String,
    /// Frames per second.
    #[serde(default)]
    pub fps: f64,
    /// File size in bytes.
    #[serde(default)]
    pub size: i64,
    /// Width in pixels.
    #[serde(default)]
    pub width: i64,
    /// Height in pixels.
    #[serde(default)]
    pub height: i64,
    /// Direct URL to the MP4 file.
    #[serde(default)]
    pub url: Option<String>,
}

/// Map of sample quality names (e.g., "720p", "1080p") to their metadata.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Samples(pub HashMap<String, Quality>);

/// Metadata for a specific quality level of a sample.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quality {
    /// Frames per second (for video samples).
    #[serde(default)]
    pub fps: f64,
    /// File size in bytes.
    #[serde(default)]
    pub size: i64,
    /// Video codec used.
    #[serde(default)]
    pub codec: String,
    /// Width in pixels.
    #[serde(default)]
    pub width: i64,
    /// Height in pixels.
    #[serde(default)]
    pub height: i64,
    /// Direct URL to the sample of this quality.
    #[serde(default)]
    pub url: Option<String>,
}

/// Contains vote score statistics for a post.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Score {
    /// Number of upvotes.
    #[serde(default)]
    pub up: i64,
    /// Number of downvotes.
    #[serde(default)]
    pub down: i64,
    /// Total score (upvotes minus downvotes).
    #[serde(default)]
    pub total: i64,
}

/// Contains all tags for a post, organized by category.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tags {
    /// General descriptive tags.
    #[serde(default)]
    pub general: Vec<String>,
    /// Artist tags (creators of the content).
    #[serde(default)]
    pub artist: Vec<String>,
    /// Contributor tags (users who uploaded or edited the post).
    #[serde(default)]
    pub contributor: Vec<String>,
    /// Copyright tags (franchises, series, or original works).
    #[serde(default)]
    pub copyright: Vec<String>,
    /// Character tags (names of characters depicted).
    #[serde(default)]
    pub character: Vec<String>,
    /// Species tags (for furry content, the species of characters).
    #[serde(default)]
    pub species: Vec<String>,
    /// Invalid or deprecated tags that need to be corrected.
    #[serde(default)]
    pub invalid: Vec<String>,
    /// Meta tags (tags about the post itself, like "comic" or "high_res").
    #[serde(default)]
    pub meta: Vec<String>,
    /// Lore tags for background information and worldbuilding.
    #[serde(default)]
    pub lore: Vec<String>,
}

/// Contains various status flags for a post.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Flags {
    /// Whether the post is pending approval.
    #[serde(default)]
    pub pending: bool,
    /// Whether the post has been flagged for review.
    #[serde(default)]
    pub flagged: bool,
    /// Whether notes are locked (cannot be edited).
    #[serde(default)]
    pub note_locked: bool,
    /// Whether the status is locked (cannot be changed).
    #[serde(default)]
    pub status_locked: bool,
    /// Whether the rating is locked (cannot be changed).
    #[serde(default)]
    pub rating_locked: bool,
    /// Whether the post has been deleted.
    #[serde(default)]
    pub deleted: bool,
}

/// Contains parent/child relationship data for post hierarchies.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relationships {
    /// ID of the parent post (if this is a child post).
    #[serde(default)]
    pub parent_id: Option<i64>,
    /// Whether this post has any child posts.
    #[serde(default)]
    pub has_children: bool,
    /// Whether this post has active (non-deleted) child posts.
    #[serde(default)]
    pub has_active_children: bool,
    /// IDs of child posts (if any).
    #[serde(default)]
    pub children: Option<Vec<i64>>,
}

/// Represents a tag entry from e621's tag database.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagEntry {
    /// Unique identifier for the tag.
    #[serde(default)]
    pub id: i64,
    /// The tag name.
    #[serde(default)]
    pub name: String,
    /// Category ID for the tag (0=general, 1=artist, 3=copyright, 4=character, 5=species, etc.).
    #[serde(default)]
    pub category: i64,
    /// Number of posts that have this tag.
    #[serde(default)]
    pub post_count: i64,
}

/// Response from e621 API containing multiple pools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E6PoolsResponse {
    /// List of pools returned by the API.
    #[serde(default)]
    pub pools: Vec<E6Pool>,
}

/// Response from e621 API containing a single pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E6PoolResponse {
    /// The pool object returned by the API.
    #[serde(default)]
    pub pool: E6Pool,
}

/// Represents a pool (collection) of posts on e621.
/// Pools are used to group related posts, such as comic pages or themed collections.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct E6Pool {
    /// Unique identifier for the pool.
    #[serde(default)]
    pub id: i64,
    /// Name of the pool.
    #[serde(default)]
    pub name: String,
    /// ISO 8601 timestamp of when the pool was created.
    #[serde(default)]
    pub created_at: String,
    /// ISO 8601 timestamp of when the pool was last updated.
    #[serde(default)]
    pub updated_at: String,
    /// ID of the user who created the pool.
    #[serde(default)]
    pub creator_id: i64,
    /// Username of the user who created the pool.
    #[serde(default)]
    pub creator_name: String,
    /// Description of the pool's purpose or contents.
    #[serde(default)]
    pub description: String,
    /// Whether the pool is active (not deleted).
    #[serde(default)]
    pub is_active: bool,
    /// Category of the pool ("series", "collection", etc.).
    #[serde(default)]
    pub category: String,
    /// IDs of posts in the pool, in order.
    #[serde(default)]
    pub post_ids: Vec<i64>,
    /// Number of posts in the pool.
    #[serde(default)]
    pub post_count: i64,
}

/// Represents a pool entry, typically used in search results or listings.
/// Similar to E6Pool but with slightly different field conventions.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoolEntry {
    /// Unique identifier for the pool.
    #[serde(default)]
    pub id: i64,
    /// Name of the pool.
    #[serde(default)]
    pub name: String,
    /// ISO 8601 timestamp of when the pool was created.
    #[serde(default)]
    pub created_at: String,
    /// ISO 8601 timestamp of when the pool was last updated.
    #[serde(default)]
    pub updated_at: String,
    /// ID of the user who created the pool.
    #[serde(default)]
    pub creator_id: i64,
    /// Description of the pool's purpose or contents.
    #[serde(default)]
    pub description: String,
    /// Whether the pool is active (not deleted).
    #[serde(default, deserialize_with = "deserialize_bool_from_str")]
    pub is_active: bool,
    /// Category of the pool ("series", "collection", etc.).
    #[serde(default)]
    pub category: String,
    /// IDs of posts in the pool, in order.
    #[serde(default, deserialize_with = "deserialize_post_ids")]
    pub post_ids: Vec<i64>,
}

/// Represents a tag alias entry from e621.
/// Tag aliases automatically rename one tag to another.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagAliasEntry {
    /// Unique identifier for the alias.
    #[serde(default)]
    pub id: i64,
    /// The source tag name that will be aliased.
    #[serde(default)]
    pub antecedent_name: String,
    /// The destination tag name that the antecedent will be aliased to.
    #[serde(default)]
    pub consequent_name: String,
    /// ISO 8601 timestamp of when the alias was created.
    #[serde(default)]
    pub created_at: String,
    /// Status of the alias ("active", "pending", etc.).
    #[serde(default)]
    pub status: String,
}

/// Represents a tag implication entry from e621.
/// Tag implications automatically add additional tags when a tag is used.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagImplicationEntry {
    /// Unique identifier for the implication.
    #[serde(default)]
    pub id: i64,
    /// The source tag name that implies other tags.
    #[serde(default)]
    pub antecedent_name: String,
    /// The destination tag name that is implied.
    #[serde(default)]
    pub consequent_name: String,
    /// ISO 8601 timestamp of when the implication was created.
    #[serde(default)]
    pub created_at: String,
    /// Status of the implication ("active", "pending", etc.).
    #[serde(default)]
    pub status: String,
}

impl E6Post {
    /// parses a blacklist rule string into included, excluded, and wildcard tags
    ///
    /// # Arguments
    ///
    /// * `rule` - the rule to parse
    pub fn parse_blacklist_rule(rule: &str) -> (Vec<String>, Vec<String>) {
        let mut includes = Vec::new();
        let mut excludes = Vec::new();

        for part in rule.split_whitespace() {
            if let Some(tag) = part.strip_prefix('-') {
                if !tag.is_empty() {
                    excludes.push(tag.to_string());
                }
            } else if !part.is_empty() {
                includes.push(part.to_string());
            }
        }

        (includes, excludes)
    }

    /// checks if this post matches a given blacklist rule
    ///
    /// # Arguments
    ///
    /// * `rule` - check if a given post matches a rule in the blacklist
    pub fn matches_blacklist_rule(&self, rule: &str) -> bool {
        let (includes, excludes) = Self::parse_blacklist_rule(rule);

        let all: HashSet<&String> = self
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

        let includes_matched = includes.is_empty() || includes.iter().all(|tag| all.contains(tag));
        let excludes_matched = excludes.iter().any(|tag| all.contains(tag));

        if includes.is_empty() && !excludes.is_empty() {
            !excludes_matched
        } else {
            includes_matched && !excludes_matched
        }
    }

    /// checks if this post is blacklisted
    pub fn is_blacklisted(&self) -> bool {
        let blacklist = getopt!(search.blacklist);
        if blacklist.is_empty() {
            return false;
        }

        for rule in &blacklist {
            if self.matches_blacklist_rule(rule) {
                return true;
            }
        }

        false
    }

    /// checks if any of the search tags are blacklisted
    ///
    /// # Arguments
    ///
    /// * `search_tags` - a list of tags to compare to the blacklist
    pub fn search_includes_blacklisted(search_tags: &[String]) -> bool {
        let blacklist = getopt!(search.blacklist);
        for search_tag in search_tags {
            for rule in &blacklist {
                let (include_tags, exclude_tags) = Self::parse_blacklist_rule(rule);
                if include_tags.len() == 1
                    && exclude_tags.is_empty()
                    && include_tags[0] == *search_tag
                {
                    return true;
                }
            }
        }

        false
    }

    /// returns whether the post meets the configured requirements
    pub fn meets_score_requirements(&self) -> bool {
        let min_score = getopt!(search.min_post_score);
        let max_score = getopt!(search.max_post_score);

        self.score.total >= min_score && self.score.total <= max_score
    }
}

impl E6PostsResponse {
    /// filter blacklisted posts from the api response unless explicitly searched for
    ///
    /// # Arguments
    ///
    /// * `search_tags` - the tags included in the search
    pub fn filter_blacklisted(mut self, search_tags: &[String]) -> Self {
        if E6Post::search_includes_blacklisted(search_tags) {
            return self;
        }

        self.posts.retain(|post| !post.is_blacklisted());
        self
    }

    /// filters out posts that don't meet the min socre requirements
    pub fn filter_score(mut self) -> Self {
        self.posts.retain(|post| post.meets_score_requirements());
        self
    }
}

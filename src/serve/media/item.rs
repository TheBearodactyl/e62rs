//! media item stuff
use {
    crate::serve::media::{metadata::PostMetadata, types::MediaType},
    serde::{Deserialize, Serialize},
};

/// a media item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    /// file path relative to media root
    path: String,
    /// display name of the item
    name: String,
    /// type of media (image, video, etc)
    #[serde(rename = "media_type")]
    media_type: MediaType,
    /// file size in bytes
    size: u64,
    /// optional post metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<PostMetadata>,
}

impl MediaItem {
    /// make a new media item
    pub fn new(path: String, name: String, media_type: MediaType, size: u64) -> Self {
        Self {
            path,
            name,
            media_type,
            size,
            metadata: None,
        }
    }

    /// set the post metadata
    pub fn with_metadata(mut self, metadata: PostMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// get the file path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// get the display name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// get the media type
    pub fn media_type(&self) -> &MediaType {
        &self.media_type
    }

    /// get the file size in bytes
    pub fn size(&self) -> u64 {
        self.size
    }

    /// get the post metadata if present
    pub fn metadata(&self) -> Option<&PostMetadata> {
        self.metadata.as_ref()
    }

    /// check if this item matches a search query
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();

        if self.name.to_lowercase().contains(&query_lower) {
            return true;
        }

        if let Some(meta) = &self.metadata {
            if meta.id.to_string().contains(&query_lower) {
                return true;
            }

            if meta
                .artists
                .iter()
                .any(|a| a.to_lowercase().contains(&query_lower))
            {
                return true;
            }

            if meta
                .tags
                .iter()
                .any(|t| t.to_lowercase().contains(&query_lower))
                || meta
                    .character_tags
                    .iter()
                    .any(|t| t.to_lowercase().contains(&query_lower))
                || meta
                    .species_tags
                    .iter()
                    .any(|t| t.to_lowercase().contains(&query_lower))
            {
                return true;
            }
        }

        false
    }
}

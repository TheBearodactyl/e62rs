use {
    crate::serve::media::{metadata::PostMetadata, types::MediaType},
    serde::{Deserialize, Serialize},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    path: String,
    name: String,
    #[serde(rename = "media_type")]
    media_type: MediaType,
    size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<PostMetadata>,
}

impl MediaItem {
    pub fn new(path: String, name: String, media_type: MediaType, size: u64) -> Self {
        Self {
            path,
            name,
            media_type,
            size,
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: PostMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn media_type(&self) -> &MediaType {
        &self.media_type
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn metadata(&self) -> Option<&PostMetadata> {
        self.metadata.as_ref()
    }

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

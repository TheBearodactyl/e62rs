use {crate::serve::media::item::MediaItem, serde::Deserialize};

#[derive(Debug, Clone, Deserialize)]
pub struct MediaFilter {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub media_type: Option<String>,
    #[serde(default)]
    pub rating: Option<String>,
    #[serde(default)]
    pub artist: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub min_score: Option<i64>,
    #[serde(default)]
    pub max_score: Option<i64>,
    #[serde(default)]
    pub extension: Option<String>,
    #[serde(default)]
    pub post_id: Option<i64>,
}

impl MediaFilter {
    pub fn matches(&self, item: &MediaItem) -> bool {
        if let Some(ref media_type_filter) = self.media_type
            && media_type_filter != item.media_type().as_str()
        {
            return false;
        }

        if let Some(ref ext_filter) = self.extension
            && !item
                .name()
                .to_lowercase()
                .ends_with(&format!(".{}", ext_filter.to_lowercase()))
        {
            return false;
        }

        if let Some(ref search_query) = self.search
            && !item.matches_search(search_query)
        {
            return false;
        }

        if let Some(metadata) = item.metadata() {
            if let Some(ref rating_filter) = self.rating
                && &metadata.rating != rating_filter
            {
                return false;
            }

            if let Some(ref artist_filter) = self.artist {
                let artist_lower = artist_filter.to_lowercase();
                if !metadata
                    .artists
                    .iter()
                    .any(|a| a.to_lowercase().contains(&artist_lower))
                {
                    return false;
                }
            }

            if let Some(ref tag_filter) = self.tag {
                let tag_lower = tag_filter.to_lowercase();
                let has_tag = metadata
                    .tags
                    .iter()
                    .any(|t| t.to_lowercase().contains(&tag_lower))
                    || metadata
                        .character_tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&tag_lower))
                    || metadata
                        .species_tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&tag_lower));

                if !has_tag {
                    return false;
                }
            }

            if let Some(min_score) = self.min_score
                && metadata.score < min_score
            {
                return false;
            }

            if let Some(max_score) = self.max_score
                && metadata.score > max_score
            {
                return false;
            }

            if let Some(post_id) = self.post_id
                && metadata.id != post_id
            {
                return false;
            }
        } else if self.rating.is_some()
            || self.artist.is_some()
            || self.tag.is_some()
            || self.min_score.is_some()
            || self.max_score.is_some()
            || self.post_id.is_some()
        {
            return false;
        }

        true
    }
}

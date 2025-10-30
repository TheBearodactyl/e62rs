use {serde::Serialize, std::collections::HashMap};

use crate::serve::media::{item::MediaItem, types::MediaType};

#[derive(Debug, Default, Serialize)]
pub struct FilterStats {
    pub total_images: usize,
    pub total_videos: usize,
    pub safe_count: usize,
    pub questionable_count: usize,
    pub explicit_count: usize,
    pub unique_artists: std::collections::HashSet<String>,
    pub unique_tags: std::collections::HashSet<String>,
    pub extensions: HashMap<String, usize>,
}

impl FilterStats {
    pub fn from_items(items: &[MediaItem]) -> Self {
        let mut stats = Self::default();

        for item in items {
            match item.media_type() {
                MediaType::Image => stats.total_images += 1,
                MediaType::Video => stats.total_videos += 1,
            }

            if let Some(meta) = &item.metadata() {
                match meta.rating.as_str() {
                    "s" => stats.safe_count += 1,
                    "q" => stats.questionable_count += 1,
                    "e" => stats.explicit_count += 1,
                    _ => {}
                }

                for artist in &meta.artists {
                    stats.unique_artists.insert(artist.clone());
                }

                for tag in &meta.tags {
                    stats.unique_tags.insert(tag.clone());
                }

                if let Some(ext) = item.name().rsplit('.').next() {
                    *stats.extensions.entry(ext.to_string()).or_insert(0) += 1;
                }
            }
        }

        stats
    }
}

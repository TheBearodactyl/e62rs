//! media statistics stuff
use {
    crate::serve::media::{item::MediaItem, types::MediaType},
    hashbrown::{HashMap, HashSet},
    serde::Serialize,
};

/// filter stats
#[derive(Debug, Default, Serialize)]
pub struct FilterStats {
    /// total number of images
    pub total_images: usize,
    /// total number of videos
    pub total_videos: usize,
    /// the number of safe items
    pub safe_count: usize,
    /// the number of questionable items
    pub questionable_count: usize,
    /// the number of explicit items
    pub explicit_count: usize,
    /// a list of unique artists
    pub unique_artists: HashSet<String>,
    /// a list of unique tags
    pub unique_tags: HashSet<String>,
    /// a list of extensions
    pub extensions: HashMap<String, usize>,
}

impl FilterStats {
    /// make filter stats based on an array of media items
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

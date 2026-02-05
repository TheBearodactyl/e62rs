//! media gallery stuff
use {
    crate::{
        getopt,
        serve::media::{
            filter::MediaFilter,
            item::MediaItem,
            scanner::{FsScanner, MediaScanner},
            stats::FilterStats,
            types::MediaType,
        },
    },
    std::path::{Path, PathBuf},
};

/// the media gallery
pub struct MediaGallery {
    /// the media scanner
    scanner: Box<dyn MediaScanner>,
    /// the directory to make a gallery from
    directory: PathBuf,
    /// the cached items
    cached_items: Option<Vec<MediaItem>>,
}

impl MediaGallery {
    /// initialize the media gallery
    pub fn new(directory: PathBuf, load_metadata: bool) -> Self {
        Self {
            scanner: Box::new(FsScanner::with_threads(
                load_metadata,
                getopt!(gallery.load_threads),
            )),
            directory,
            cached_items: None,
        }
    }

    /// initialize the media gallery with a custom scanner
    pub fn with_scanner(directory: PathBuf, scanner: Box<dyn MediaScanner>) -> Self {
        Self {
            scanner,
            directory,
            cached_items: None,
        }
    }

    /// get all media items
    pub async fn get_items(&mut self) -> Result<&[MediaItem], std::io::Error> {
        if self.cached_items.is_none() {
            let items = self.scanner.scan(&self.directory).await?;
            self.cached_items = Some(items);
        }

        Ok(self.cached_items.as_ref().expect("cached_items is None"))
    }

    /// get all media items with the given filters applied
    pub async fn get_filtered_items(
        &mut self,
        filter: &MediaFilter,
    ) -> Result<Vec<MediaItem>, std::io::Error> {
        Ok(self
            .get_items()
            .await?
            .iter()
            .filter(|item| filter.matches(item))
            .cloned()
            .collect())
    }

    /// refresh the gallery
    pub async fn refresh(&mut self) -> Result<&[MediaItem], std::io::Error> {
        self.cached_items = Some(self.scanner.scan(&self.directory).await?);
        Ok(self.cached_items.as_ref().expect("cached_items is None"))
    }

    /// filter media items by type
    pub fn filter_by_type(&self, media_type: &MediaType) -> Vec<MediaItem> {
        self.cached_items
            .as_ref()
            .map(|items| {
                items
                    .iter()
                    .filter(|item| item.media_type() == media_type)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// search all media items based on a given query
    pub fn search(&self, query: &str) -> Vec<MediaItem> {
        self.cached_items
            .as_ref()
            .map(|items| {
                items
                    .iter()
                    .filter(|item| item.matches_search(query))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// get the directory being used for the gallery
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// get the filter stats of the current loaded items
    pub fn get_filter_stats(&self) -> FilterStats {
        FilterStats::from_items(self.cached_items.as_deref().unwrap_or(&[]))
    }
}

use {
    crate::{
        config::options::E62Rs,
        serve::media::{
            filter::MediaFilter,
            item::MediaItem,
            scanner::{FileSystemScanner, MediaScanner},
            stats::FilterStats,
            types::MediaType,
        },
    },
    std::path::{Path, PathBuf},
};

pub struct MediaGallery {
    scanner: Box<dyn MediaScanner>,
    directory: PathBuf,
    cached_items: Option<Vec<MediaItem>>,
}

impl MediaGallery {
    pub fn new(directory: PathBuf, load_metadata: bool) -> Self {
        let gallery_cfg = E62Rs::get_unsafe().gallery;

        Self {
            scanner: Box::new(FileSystemScanner::with_threads(
                load_metadata,
                gallery_cfg.load_threads,
            )),
            directory,
            cached_items: None,
        }
    }

    pub fn with_scanner(directory: PathBuf, scanner: Box<dyn MediaScanner>) -> Self {
        Self {
            scanner,
            directory,
            cached_items: None,
        }
    }

    pub async fn get_items(&mut self) -> Result<&[MediaItem], std::io::Error> {
        if self.cached_items.is_none() {
            let items = self.scanner.scan(&self.directory).await?;
            self.cached_items = Some(items);
        }
        Ok(self.cached_items.as_ref().unwrap())
    }

    pub async fn get_filtered_items(
        &mut self,
        filter: &MediaFilter,
    ) -> Result<Vec<MediaItem>, std::io::Error> {
        let items = self.get_items().await?;
        Ok(items
            .iter()
            .filter(|item| filter.matches(item))
            .cloned()
            .collect())
    }

    pub async fn refresh(&mut self) -> Result<&[MediaItem], std::io::Error> {
        let items = self.scanner.scan(&self.directory).await?;
        self.cached_items = Some(items);
        Ok(self.cached_items.as_ref().unwrap())
    }

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

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    pub fn get_filter_stats(&self) -> FilterStats {
        let items = self.cached_items.as_deref().unwrap_or(&[]);
        FilterStats::from_items(items)
    }
}

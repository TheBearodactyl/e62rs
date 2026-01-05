//! media scanning stuff
use {
    crate::serve::media::{item::MediaItem, metadata::PostMetadata, types::MediaType},
    jwalk::WalkDir,
    rayon::iter::{IntoParallelRefIterator, ParallelIterator},
    std::{fs::OpenOptions, io::Read, path::Path},
    tracing::info,
};

/// a media scanner
#[async_trait::async_trait]
pub trait MediaScanner: Send + Sync {
    /// scan a directory
    async fn scan(&self, dir: &Path) -> Result<Vec<MediaItem>, std::io::Error>;
}

/// a filesystem scanner
pub struct FsScanner {
    /// whether to load post metadata
    load_metadata: bool,
    /// how many threads to load with
    num_threads: usize,
}

impl FsScanner {
    /// make a new filesystem scanner
    pub fn new(load_metadata: bool) -> Self {
        Self {
            load_metadata,
            num_threads: num_cpus::get().max(4),
        }
    }

    /// make a new filesystem scanner with a custom thread count
    pub fn with_threads(load_metadata: bool, num_threads: usize) -> Self {
        Self {
            load_metadata,
            num_threads,
        }
    }

    /// read a files metadata from JSON
    fn read_metadata_from_json(&self, file_path: &Path) -> Option<PostMetadata> {
        let json_path = file_path.with_extension(format!(
            "{}.json",
            file_path.extension().and_then(|e| e.to_str()).unwrap_or("")
        ));

        if !json_path.exists() {
            return None;
        }

        let contents = std::fs::read_to_string(&json_path).ok()?;
        self.parse_metadata(&contents)
    }

    /// read a files metadata from an NTFS ADS stream
    #[cfg(target_os = "windows")]
    fn read_metadata_from_ads(&self, file_path: &Path) -> Option<PostMetadata> {
        let ads_path = format!("{}:metadata", file_path.display());
        let mut file = OpenOptions::new().read(true).open(&ads_path).ok()?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).ok()?;

        self.parse_metadata(&contents)
    }

    /// parse a JSON string into PostMetadata
    fn parse_metadata(&self, contents: &str) -> Option<PostMetadata> {
        let post = serde_json::from_str::<crate::models::E6Post>(contents).ok()?;

        Some(PostMetadata {
            id: post.id,
            rating: post.rating,
            score: post.score.total,
            fav_count: post.fav_count,
            artists: post.tags.artist,
            tags: post.tags.general,
            character_tags: post.tags.character,
            species_tags: post.tags.species,
            created_at: post.created_at,
            pools: post.pools,
        })
    }
}

impl Default for FsScanner {
    fn default() -> Self {
        Self::new(true)
    }
}

#[async_trait::async_trait]
impl MediaScanner for FsScanner {
    async fn scan(&self, directory: &Path) -> Result<Vec<MediaItem>, std::io::Error> {
        let directory = directory.to_path_buf();
        let load_meta = self.load_metadata;
        let num_threads = self.num_threads;

        let media_items = tokio::task::spawn_blocking(move || {
            let entries: Vec<_> = WalkDir::new(&directory)
                .skip_hidden(false)
                .parallelism(jwalk::Parallelism::RayonNewPool(num_threads))
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .collect();

            entries
                .par_iter()
                .filter_map(|entry| {
                    let path = entry.path();
                    let ext = path.extension()?;
                    let media_type = MediaType::from_extension(&ext.to_string_lossy())?;
                    let rel_path = path.strip_prefix(&directory).ok()?;
                    let size = entry.metadata().ok()?.len();

                    let mut item = MediaItem::new(
                        format!("/files/{}", rel_path.display()),
                        path.file_name()?.to_string_lossy().to_string(),
                        media_type,
                        size,
                    );

                    if load_meta && let Some(metadata) = Self::read_metadata_static(path.as_path())
                    {
                        item = item.with_metadata(metadata);
                        info!("Found item: {}", item.name());
                    }

                    Some(item)
                })
                .collect()
        })
        .await
        .map_err(std::io::Error::other)?;

        Ok(media_items)
    }
}

impl FsScanner {
    #[cfg(target_os = "windows")]
    /// read a files metadata into a PostMetadata
    fn read_metadata_static(file_path: &Path) -> Option<PostMetadata> {
        let scanner = Self::new(true);
        scanner
            .read_metadata_from_ads(file_path)
            .or_else(|| scanner.read_metadata_from_json(file_path))
    }

    #[cfg(not(target_os = "windows"))]
    /// read a files metadata into a PostMetadata
    fn read_metadata_static(file_path: &Path) -> Option<PostMetadata> {
        let scanner = Self::new(true);
        scanner.read_metadata_from_json(file_path)
    }
}

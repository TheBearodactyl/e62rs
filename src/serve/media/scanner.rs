use {
    crate::serve::media::{item::MediaItem, metadata::PostMetadata, types::MediaType},
    jwalk::WalkDir,
    rayon::iter::{IntoParallelRefIterator, ParallelIterator},
    serde_json::Value,
    std::path::Path,
    tracing::info,
};

#[async_trait::async_trait]
pub trait MediaScanner: Send + Sync {
    async fn scan(&self, directory: &Path) -> Result<Vec<MediaItem>, std::io::Error>;
}

pub struct FileSystemScanner {
    load_metadata: bool,
    num_threads: usize,
}

impl FileSystemScanner {
    pub fn new(load_metadata: bool) -> Self {
        Self {
            load_metadata,
            num_threads: num_cpus::get().max(4),
        }
    }

    pub fn with_threads(load_metadata: bool, num_threads: usize) -> Self {
        Self {
            load_metadata,
            num_threads,
        }
    }

    fn read_metadata_from_json(&self, file_path: &Path) -> Option<PostMetadata> {
        use std::fs;

        let json_path = file_path.with_extension(format!(
            "{}.json",
            file_path.extension().and_then(|e| e.to_str()).unwrap_or("")
        ));

        if !json_path.exists() {
            return None;
        }

        let contents = fs::read_to_string(&json_path).ok()?;
        self.parse_metadata(&contents)
    }

    #[cfg(target_os = "windows")]
    fn read_metadata_from_ads(&self, file_path: &Path) -> Option<PostMetadata> {
        use std::{fs::OpenOptions, io::Read};

        let ads_path = format!("{}:metadata", file_path.display());
        let mut file = OpenOptions::new().read(true).open(&ads_path).ok()?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).ok()?;

        self.parse_metadata(&contents)
    }

    fn parse_metadata(&self, contents: &str) -> Option<PostMetadata> {
        let post: Value = serde_json::from_str(contents).ok()?;

        Some(PostMetadata {
            id: post["id"].as_i64().unwrap_or(0),
            rating: post["rating"].as_str().unwrap_or("").to_string(),
            score: post["score"]["total"].as_i64().unwrap_or(0),
            fav_count: post["fav_count"].as_i64().unwrap_or(0),
            artists: post["tags"]["artist"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            tags: post["tags"]["general"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            character_tags: post["tags"]["character"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            species_tags: post["tags"]["species"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            created_at: post["created_at"].as_str().unwrap_or("").to_string(),
        })
    }

    fn read_metadata(&self, file_path: &Path) -> Option<PostMetadata> {
        #[cfg(target_os = "windows")]
        {
            self.read_metadata_from_ads(file_path)
                .or_else(|| self.read_metadata_from_json(file_path))
        }

        #[cfg(not(target_os = "windows"))]
        {
            self.read_metadata_from_json(file_path)
        }
    }
}

impl Default for FileSystemScanner {
    fn default() -> Self {
        Self::new(true)
    }
}

#[async_trait::async_trait]
impl MediaScanner for FileSystemScanner {
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

impl FileSystemScanner {
    #[cfg(target_os = "windows")]
    fn read_metadata_static(file_path: &Path) -> Option<PostMetadata> {
        let scanner = Self::new(true);
        scanner
            .read_metadata_from_ads(file_path)
            .or_else(|| scanner.read_metadata_from_json(file_path))
    }

    #[cfg(not(target_os = "windows"))]
    fn read_metadata_static(file_path: &Path) -> Option<PostMetadata> {
        let scanner = Self::new(true);
        scanner.read_metadata_from_json(file_path)
    }
}

use {
    e6cfg::E62Rs,
    jwalk::WalkDir,
    rayon::iter::{IntoParallelRefIterator, ParallelIterator},
    serde::{Deserialize, Serialize},
    std::{
        collections::HashMap,
        path::{Path, PathBuf},
    },
    tracing::info,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Image,
    Video,
}

impl MediaType {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "bmp" | "ico" => Some(Self::Image),
            "mp4" | "webm" | "mov" | "avi" | "mkv" => Some(Self::Video),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Image => "image",
            Self::Video => "video",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMetadata {
    pub id: i64,
    pub rating: String,
    pub score: i64,
    pub fav_count: i64,
    pub artists: Vec<String>,
    pub tags: Vec<String>,
    pub character_tags: Vec<String>,
    pub species_tags: Vec<String>,
    pub created_at: String,
}

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
            && media_type_filter != item.media_type.as_str()
        {
            return false;
        }

        if let Some(ref ext_filter) = self.extension
            && !item
                .name
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

        if let Some(ref metadata) = item.metadata {
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

    #[cfg(target_os = "windows")]
    fn read_metadata_static(file_path: &Path) -> Option<PostMetadata> {
        use std::{fs::OpenOptions, io::Read};

        let ads_path = format!("{}:metadata", file_path.display());
        let mut file = OpenOptions::new().read(true).open(&ads_path).ok()?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).ok()?;

        let e6post: serde_json::Value = serde_json::from_str(&contents).ok()?;

        Some(PostMetadata {
            id: e6post["id"].as_i64().unwrap_or(0),
            rating: e6post["rating"].as_str().unwrap_or("").to_string(),
            score: e6post["score"]["total"].as_i64().unwrap_or(0),
            fav_count: e6post["fav_count"].as_i64().unwrap_or(0),
            artists: e6post["tags"]["artist"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            tags: e6post["tags"]["general"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            character_tags: e6post["tags"]["character"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            species_tags: e6post["tags"]["species"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            created_at: e6post["created_at"].as_str().unwrap_or("").to_string(),
        })
    }

    #[cfg(not(target_os = "windows"))]
    fn read_metadata_static(file_path: &Path) -> Option<PostMetadata> {
        use std::fs;

        let json_path = file_path.with_extension(format!(
            "{}.json",
            file_path.extension().and_then(|e| e.to_str()).unwrap_or("")
        ));

        if !json_path.exists() {
            return None;
        }

        let contents = fs::read_to_string(&json_path).ok()?;
        let e6post: serde_json::Value = serde_json::from_str(&contents).ok()?;

        Some(PostMetadata {
            id: e6post["id"].as_i64().unwrap_or(0),
            rating: e6post["rating"].as_str().unwrap_or("").to_string(),
            score: e6post["score"]["total"].as_i64().unwrap_or(0),
            fav_count: e6post["fav_count"].as_i64().unwrap_or(0),
            artists: e6post["tags"]["artist"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            tags: e6post["tags"]["general"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            character_tags: e6post["tags"]["character"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            species_tags: e6post["tags"]["species"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            created_at: e6post["created_at"].as_str().unwrap_or("").to_string(),
        })
    }

    #[cfg(target_os = "windows")]
    fn read_metadata(&self, file_path: &Path) -> Option<PostMetadata> {
        use std::{fs::OpenOptions, io::Read};

        let ads_path = format!("{}:metadata", file_path.display());
        let mut file = OpenOptions::new().read(true).open(&ads_path).ok()?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).ok()?;

        let e6post: serde_json::Value = serde_json::from_str(&contents).ok()?;

        Some(PostMetadata {
            id: e6post["id"].as_i64().unwrap_or(0),
            rating: e6post["rating"].as_str().unwrap_or("").to_string(),
            score: e6post["score"]["total"].as_i64().unwrap_or(0),
            fav_count: e6post["fav_count"].as_i64().unwrap_or(0),
            artists: e6post["tags"]["artist"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            tags: e6post["tags"]["general"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            character_tags: e6post["tags"]["character"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            species_tags: e6post["tags"]["species"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            created_at: e6post["created_at"].as_str().unwrap_or("").to_string(),
        })
    }

    #[cfg(not(target_os = "windows"))]
    fn read_metadata(&self, file_path: &Path) -> Option<PostMetadata> {
        use std::fs;

        let json_path = file_path.with_extension(format!(
            "{}.json",
            file_path.extension().and_then(|e| e.to_str()).unwrap_or("")
        ));

        if !json_path.exists() {
            return None;
        }

        let contents = fs::read_to_string(&json_path).ok()?;
        let e6post: serde_json::Value = serde_json::from_str(&contents).ok()?;

        Some(PostMetadata {
            id: e6post["id"].as_i64().unwrap_or(0),
            rating: e6post["rating"].as_str().unwrap_or("").to_string(),
            score: e6post["score"]["total"].as_i64().unwrap_or(0),
            fav_count: e6post["fav_count"].as_i64().unwrap_or(0),
            artists: e6post["tags"]["artist"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            tags: e6post["tags"]["general"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            character_tags: e6post["tags"]["character"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            species_tags: e6post["tags"]["species"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            created_at: e6post["created_at"].as_str().unwrap_or("").to_string(),
        })
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

        let mut stats = FilterStats::default();

        for item in items {
            match item.media_type {
                MediaType::Image => stats.total_images += 1,
                MediaType::Video => stats.total_videos += 1,
            }

            if let Some(meta) = &item.metadata {
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

                if let Some(ext) = item.name.rsplit('.').next() {
                    *stats.extensions.entry(ext.to_string()).or_insert(0) += 1;
                }
            }
        }

        stats
    }
}

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

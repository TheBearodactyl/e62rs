//! reorganization stuff
use {
    crate::{
        config::format::FormatTemplate,
        getopt,
        models::E6Post,
        ui::{E6Ui, progress::ProgressManager, themes::ROSE_PINE},
    },
    color_eyre::eyre::{Context, Result, bail},
    demand::{Confirm, DemandOption, Input, Select},
    hashbrown::HashMap,
    smart_default::SmartDefault,
    std::{
        fs::{self, OpenOptions},
        io::Read,
        path::{Path, PathBuf},
        sync::Arc,
    },
    tracing::{debug, warn},
    url::Url,
};

#[derive(Debug, Clone, Copy)]
/// methods for conflict resolution
pub enum ConflictResolution {
    /// skip conflicts
    Skip,
    /// overwrite conflicts
    Overwrite,
    /// auto-rename to bypass conflicts
    AutoRename,
}

#[derive(Debug, Clone, SmartDefault)]
/// options for reorganization
pub struct ReorganizeOptions {
    #[default(false)]
    /// whether to just do a dry-run (only show what would change)
    pub dry_run: bool,

    #[default(ConflictResolution::Skip)]
    /// the conflict resolution method
    pub conflict_resolution: ConflictResolution,

    #[default(None)]
    /// the output format to use
    pub output_format: Option<String>,
}

#[derive(Debug)]
/// the results of running a reorganization
pub struct ReorganizeResult {
    /// total files moved
    pub total_files: usize,
    /// total successful moves
    pub successful: usize,
    /// total skipped moves
    pub skipped: usize,
    /// total failed moves
    pub failed: usize,
    /// errors
    pub errors: Vec<(PathBuf, String)>,
}

#[derive(Default)]
/// the reorganizer
pub struct FileReorganizer {
    /// the progress bar manager
    progress_manager: Arc<ProgressManager>,
}

impl FileReorganizer {
    /// make a new reorganizer
    pub fn new() -> Self {
        Self {
            progress_manager: Arc::new(ProgressManager::new()),
        }
    }

    #[cfg(target_os = "windows")]
    /// read file metadata into an E6Post
    fn read_metadata_from_ads(&self, file_path: &Path) -> Result<E6Post> {
        let ads_path = format!("{}:metadata", file_path.display());

        let mut file = OpenOptions::new()
            .read(true)
            .open(&ads_path)
            .with_context(|| format!("Failed to open ADS metadata for {}", file_path.display()))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .with_context(|| format!("Failed to read ADS metadata for {}", file_path.display()))?;

        serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse metadata for {}", file_path.display()))
    }

    #[cfg(not(target_os = "windows"))]
    /// read file metadata into an E6Post
    fn read_metadata_from_json(&self, file_path: &Path) -> Result<E6Post> {
        let json_path = file_path.with_extension(format!(
            "{}.json",
            file_path.extension().and_then(|e| e.to_str()).unwrap_or("")
        ));

        if !json_path.exists() {
            bail!("Metadata file not found: {}", json_path.display());
        }

        let contents = fs::read_to_string(&json_path)
            .with_context(|| format!("Failed to read metadata file {}", json_path.display()))?;

        serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse metadata for {}", file_path.display()))
    }

    /// search a directory for any files with valid metadata
    pub fn find_files_with_metadata(&self, directory: &Path) -> Result<Vec<PathBuf>> {
        let mut files_with_metadata = Vec::new();

        if !directory.exists() {
            bail!("Directory does not exist: {}", directory.display());
        }

        for entry in fs::read_dir(directory)
            .with_context(|| format!("Failed to read directory {}", directory.display()))?
        {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                #[cfg(target_os = "windows")]
                {
                    let ads_path = format!("{}:metadata", path.display());
                    if OpenOptions::new().read(true).open(&ads_path).is_ok() {
                        files_with_metadata.push(path);
                    }
                }

                #[cfg(not(target_os = "windows"))]
                {
                    let json_path = path.with_extension(format!(
                        "{}.json",
                        path.extension().and_then(|e| e.to_str()).unwrap_or("")
                    ));
                    if json_path.exists() {
                        files_with_metadata.push(path);
                    }
                }
            }
        }

        Ok(files_with_metadata)
    }

    /// recursively find files
    pub fn find_files_recursive(&self, directory: &Path) -> Result<Vec<PathBuf>> {
        let mut all_files = Vec::new();

        if !directory.exists() {
            bail!("Directory does not exist: {}", directory.display());
        }

        self.find_files_recursive_impl(directory, &mut all_files)?;
        Ok(all_files)
    }

    /// recursively find files (internal)
    fn find_files_recursive_impl(&self, directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(directory)
            .with_context(|| format!("Failed to read directory {}", directory.display()))?
        {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.find_files_recursive_impl(&path, files)?;
            } else if path.is_file() {
                #[cfg(target_os = "windows")]
                {
                    let ads_path = format!("{}:metadata", path.display());
                    if OpenOptions::new().read(true).open(&ads_path).is_ok() {
                        files.push(path);
                    }
                }

                #[cfg(not(target_os = "windows"))]
                {
                    let json_path = path.with_extension(format!(
                        "{}.json",
                        path.extension().and_then(|e| e.to_str()).unwrap_or("")
                    ));
                    if json_path.exists() {
                        files.push(path);
                    }
                }
            }
        }

        Ok(())
    }

    /// read a file into an E6Post (cross-plat)
    fn read_metadata(&self, file_path: &Path) -> Result<E6Post> {
        #[cfg(target_os = "windows")]
        {
            self.read_metadata_from_ads(file_path)
        }

        #[cfg(not(target_os = "windows"))]
        {
            self.read_metadata_from_json(file_path)
        }
    }

    /// build placeholder context from post metadata
    fn build_post_context(
        &self,
        post: &E6Post,
    ) -> (HashMap<String, String>, HashMap<String, Vec<String>>) {
        let mut simple = HashMap::new();
        let mut arrays = HashMap::new();

        let artist = post
            .tags
            .artist
            .first()
            .map(|s| s.as_str())
            .unwrap_or("unknown");

        let aspect_ratio = if post.file.height > 0 {
            post.file.width as f64 / post.file.height as f64
        } else {
            0.0
        };

        let orientation = if post.file.width > post.file.height {
            "landscape"
        } else if post.file.width < post.file.height {
            "portrait"
        } else {
            "square"
        };

        let megapixels = (post.file.width * post.file.height) as f64 / 1_000_000.0;

        let resolution = match (post.file.width, post.file.height) {
            (w, h) if w >= 7680 || h >= 4320 => "8K",
            (w, h) if w >= 3840 || h >= 2160 => "4K",
            (w, h) if w >= 2560 || h >= 1440 => "QHD",
            (w, h) if w >= 1920 || h >= 1080 => "FHD",
            (w, h) if w >= 1280 || h >= 720 => "HD",
            _ => "SD",
        };

        let size_mb = post.file.size as f64 / (1024.0 * 1024.0);
        let size_kb = post.file.size as f64 / 1024.0;

        let file_type = match post.file.ext.as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => "image",
            "mp4" | "webm" | "mov" | "avi" | "mkv" => "video",
            "swf" => "flash",
            _ => "unknown",
        };

        let duration_formatted = if let Some(duration) = post.duration {
            let total_seconds = duration as i64;
            let hours = total_seconds / 3600;
            let minutes = (total_seconds % 3600) / 60;
            let seconds = total_seconds % 60;

            if hours > 0 {
                format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
            } else {
                format!("{:02}:{:02}", minutes, seconds)
            }
        } else {
            "N/A".to_string()
        };

        let tag_count = post.tags.general.len()
            + post.tags.artist.len()
            + post.tags.character.len()
            + post.tags.species.len()
            + post.tags.copyright.len()
            + post.tags.meta.len()
            + post.tags.lore.len();

        let now = chrono::Local::now();
        let rating_first = post
            .rating
            .chars()
            .next()
            .unwrap_or('u')
            .to_lowercase()
            .to_string();

        let (year, month, day, hour, minute, second, timestamp) =
            if let Ok(created_date) = chrono::DateTime::parse_from_rfc3339(&post.created_at) {
                (
                    created_date.format("%Y").to_string(),
                    created_date.format("%m").to_string(),
                    created_date.format("%d").to_string(),
                    created_date.format("%H").to_string(),
                    created_date.format("%M").to_string(),
                    created_date.format("%S").to_string(),
                    created_date.timestamp().to_string(),
                )
            } else {
                (
                    now.format("%Y").to_string(),
                    now.format("%m").to_string(),
                    now.format("%d").to_string(),
                    now.format("%H").to_string(),
                    now.format("%M").to_string(),
                    now.format("%S").to_string(),
                    now.timestamp().to_string(),
                )
            };

        let (year_updated, month_updated, day_updated) =
            if let Ok(updated_date) = chrono::DateTime::parse_from_rfc3339(&post.updated_at) {
                (
                    updated_date.format("%Y").to_string(),
                    updated_date.format("%m").to_string(),
                    updated_date.format("%d").to_string(),
                )
            } else {
                (year.clone(), month.clone(), day.clone())
            };

        let rating_full = match post.rating.as_str() {
            "e" => "explicit",
            "q" => "questionable",
            "s" => "safe",
            _ => "unknown",
        };

        let pool_ids = post
            .pools
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let approver_id = post
            .approver_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "none".to_string());

        // Simple placeholders
        simple.insert("id".to_string(), post.id.to_string());
        simple.insert("rating".to_string(), rating_full.to_string());
        simple.insert("rating_first".to_string(), rating_first);
        simple.insert("score".to_string(), post.score.total.to_string());
        simple.insert("score_up".to_string(), post.score.up.to_string());
        simple.insert("score_down".to_string(), post.score.down.to_string());
        simple.insert("fav_count".to_string(), post.fav_count.to_string());
        simple.insert("comment_count".to_string(), post.comment_count.to_string());
        simple.insert("md5".to_string(), post.file.md5.clone());
        simple.insert("ext".to_string(), post.file.ext.clone());
        simple.insert("width".to_string(), post.file.width.to_string());
        simple.insert("height".to_string(), post.file.height.to_string());
        simple.insert("aspect_ratio".to_string(), format!("{:.2}", aspect_ratio));
        simple.insert("orientation".to_string(), orientation.to_string());
        simple.insert("resolution".to_string(), resolution.to_string());
        simple.insert("megapixels".to_string(), format!("{:.1}", megapixels));
        simple.insert("size".to_string(), post.file.size.to_string());
        simple.insert("size_mb".to_string(), format!("{:.2}", size_mb));
        simple.insert("size_kb".to_string(), format!("{:.2}", size_kb));
        simple.insert("artist".to_string(), artist.to_string());
        simple.insert(
            "artist_count".to_string(),
            post.tags.artist.len().to_string(),
        );
        simple.insert("tag_count".to_string(), tag_count.to_string());
        simple.insert(
            "tag_count_general".to_string(),
            post.tags.general.len().to_string(),
        );
        simple.insert(
            "tag_count_character".to_string(),
            post.tags.character.len().to_string(),
        );
        simple.insert(
            "tag_count_species".to_string(),
            post.tags.species.len().to_string(),
        );
        simple.insert(
            "tag_count_copyright".to_string(),
            post.tags.copyright.len().to_string(),
        );
        simple.insert("pool_ids".to_string(), pool_ids);
        simple.insert("pool_count".to_string(), post.pools.len().to_string());
        simple.insert("uploader".to_string(), post.uploader_name.clone());
        simple.insert("uploader_id".to_string(), post.uploader_id.to_string());
        simple.insert("approver_id".to_string(), approver_id);
        simple.insert(
            "has_children".to_string(),
            if post.relationships.has_children {
                "yes"
            } else {
                "no"
            }
            .to_string(),
        );
        simple.insert(
            "parent_id".to_string(),
            post.relationships
                .parent_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "none".to_string()),
        );
        simple.insert("year".to_string(), year.clone());
        simple.insert("month".to_string(), month.clone());
        simple.insert("day".to_string(), day.clone());
        simple.insert("hour".to_string(), hour.clone());
        simple.insert("minute".to_string(), minute.clone());
        simple.insert("second".to_string(), second.clone());
        simple.insert("date".to_string(), format!("{}-{}-{}", year, month, day));
        simple.insert(
            "time".to_string(),
            format!("{}-{}-{}", hour, minute, second),
        );
        simple.insert(
            "datetime".to_string(),
            format!("{}-{}-{} {}-{}-{}", year, month, day, hour, minute, second),
        );
        simple.insert("timestamp".to_string(), timestamp);
        simple.insert("year_updated".to_string(), year_updated.clone());
        simple.insert("month_updated".to_string(), month_updated.clone());
        simple.insert("day_updated".to_string(), day_updated.clone());
        simple.insert(
            "date_updated".to_string(),
            format!("{}-{}-{}", year_updated, month_updated, day_updated),
        );
        simple.insert("now_year".to_string(), now.format("%Y").to_string());
        simple.insert("now_month".to_string(), now.format("%m").to_string());
        simple.insert("now_day".to_string(), now.format("%d").to_string());
        simple.insert("now_hour".to_string(), now.format("%H").to_string());
        simple.insert("now_minute".to_string(), now.format("%M").to_string());
        simple.insert("now_second".to_string(), now.format("%S").to_string());
        simple.insert("now_date".to_string(), now.format("%Y-%m-%d").to_string());
        simple.insert("now_time".to_string(), now.format("%H-%M-%S").to_string());
        simple.insert(
            "now_datetime".to_string(),
            now.format("%Y-%m-%d %H-%M-%S").to_string(),
        );
        simple.insert("now_timestamp".to_string(), now.timestamp().to_string());
        simple.insert(
            "is_pending".to_string(),
            if post.flags.pending { "yes" } else { "no" }.to_string(),
        );
        simple.insert(
            "is_flagged".to_string(),
            if post.flags.flagged { "yes" } else { "no" }.to_string(),
        );
        simple.insert(
            "is_deleted".to_string(),
            if post.flags.deleted { "yes" } else { "no" }.to_string(),
        );
        simple.insert(
            "has_notes".to_string(),
            if post.has_notes { "yes" } else { "no" }.to_string(),
        );
        simple.insert(
            "duration".to_string(),
            post.duration
                .map(|d| d.to_string())
                .unwrap_or_else(|| "0".to_string()),
        );
        simple.insert("duration_formatted".to_string(), duration_formatted);
        simple.insert("file_type".to_string(), file_type.to_string());

        arrays.insert("tags".to_string(), post.tags.general.clone());
        arrays.insert("artists".to_string(), post.tags.artist.clone());
        arrays.insert("characters".to_string(), post.tags.character.clone());
        arrays.insert("species".to_string(), post.tags.species.clone());
        arrays.insert("copyright".to_string(), post.tags.copyright.clone());
        arrays.insert(
            "sources".to_string(),
            post.sources
                .iter()
                .map(|source| {
                    Url::parse(source)
                        .ok()
                        .and_then(|u| u.domain().map(String::from))
                        .unwrap_or_else(|| "unknown".to_string())
                })
                .collect(),
        );

        (simple, arrays)
    }

    /// format a filename based on a format template
    fn format_filename(&self, post: &E6Post, out_fmt: &str) -> Result<String> {
        let template = FormatTemplate::parse(out_fmt)
            .with_context(|| format!("Failed to parse output format: {}", out_fmt))?;

        let (simple_ctx, array_ctx) = self.build_post_context(post);

        template
            .render_with_arrays(&simple_ctx, &array_ctx)
            .with_context(|| format!("Failed to render filename for post {}", post.id))
    }

    /// move a file based on its metadata
    fn move_file_with_metadata(
        &self,
        old_path: &Path,
        new_path: &Path,
        conflict_resolution: ConflictResolution,
    ) -> Result<PathBuf> {
        if let Some(parent) = new_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        let final_path = if new_path.exists() {
            match conflict_resolution {
                ConflictResolution::Skip => {
                    bail!("File already exists: {}", new_path.display());
                }
                ConflictResolution::Overwrite => new_path.to_path_buf(),
                ConflictResolution::AutoRename => self.find_unique_path(new_path)?,
            }
        } else {
            new_path.to_path_buf()
        };

        if fs::rename(old_path, &final_path).is_err() {
            fs::copy(old_path, &final_path).with_context(|| {
                format!(
                    "Failed to copy {} to {}",
                    old_path.display(),
                    final_path.display()
                )
            })?;

            fs::remove_file(old_path).with_context(|| {
                format!("Failed to remove original file {}", old_path.display())
            })?;
        }

        #[cfg(target_os = "windows")]
        {
            let new_ads_path = format!("{}:metadata", final_path.display());
            if OpenOptions::new().read(true).open(&new_ads_path).is_err() {
                warn!(
                    "Metadata ADS may not have moved with file: {}",
                    final_path.display()
                );
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let old_json = old_path.with_extension(format!(
                "{}.json",
                old_path.extension().and_then(|e| e.to_str()).unwrap_or("")
            ));

            let new_json = final_path.with_extension(format!(
                "{}.json",
                final_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
            ));

            if old_json.exists() {
                if fs::rename(&old_json, &new_json).is_err() {
                    fs::copy(&old_json, &new_json)?;
                    fs::remove_file(&old_json)?;
                }
            }
        }

        Ok(final_path)
    }

    /// find a unique path using incrementation
    fn find_unique_path(&self, path: &Path) -> Result<PathBuf> {
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let parent = path.parent().unwrap_or(Path::new("."));

        for i in 1..10000 {
            let new_name = if extension.is_empty() {
                format!("{}_{}", stem, i)
            } else {
                format!("{}_{}.{}", stem, i, extension)
            };

            let new_path = parent.join(new_name);
            if !new_path.exists() {
                return Ok(new_path);
            }
        }

        bail!("Could not find unique filename for {}", path.display())
    }

    /// reorganize a directory
    pub async fn reorganize_directory(
        &self,
        directory: &Path,
        options: ReorganizeOptions,
        recursive: bool,
    ) -> Result<ReorganizeResult> {
        let files = if recursive {
            self.find_files_recursive(directory)?
        } else {
            self.find_files_with_metadata(directory)?
        };

        if files.is_empty() {
            println!("No files with metadata found in {}", directory.display());
            return Ok(ReorganizeResult {
                total_files: 0,
                successful: 0,
                skipped: 0,
                failed: 0,
                errors: Vec::new(),
            });
        }

        println!("Found {} files with metadata", files.len());

        let download_dir: String = getopt!(download.path);
        let default_format: String = getopt!(download.format);
        let output_format = options.output_format.clone().unwrap_or(default_format);
        let base_path = Path::new(&download_dir);

        let pb = self
            .progress_manager
            .create_count_bar("reorganize", files.len() as u64, "Reorganizing files")
            .await?;

        let mut result = ReorganizeResult {
            total_files: files.len(),
            successful: 0,
            skipped: 0,
            failed: 0,
            errors: Vec::new(),
        };

        for file_path in files {
            pb.set_message(format!("Processing {}", file_path.display()));

            match self.process_file(&file_path, base_path, &output_format, &options) {
                Ok(_) => {
                    result.successful += 1;
                }
                Err(e) => {
                    if e.to_string().contains("already exists")
                        || e.to_string().contains("already in correct location")
                    {
                        result.skipped += 1;
                        debug!("Skipped {}: {}", file_path.display(), e);
                    } else {
                        result.failed += 1;
                        result.errors.push((file_path.clone(), e.to_string()));
                        warn!("Failed to process {}: {}", file_path.display(), e);
                    }
                }
            }

            pb.inc(1);
        }

        pb.finish_with_message(format!(
            "Reorganization complete: {} successful, {} skipped, {} failed",
            result.successful, result.skipped, result.failed
        ));

        Ok(result)
    }

    /// process and move a file
    fn process_file(
        &self,
        file_path: &Path,
        base_path: &Path,
        output_format: &str,
        options: &ReorganizeOptions,
    ) -> Result<PathBuf> {
        let post = self.read_metadata(file_path)?;
        let new_filename = self.format_filename(&post, output_format)?;
        let new_path = base_path.join(&new_filename);

        if file_path == new_path {
            bail!("File already in correct location");
        }

        if !options.dry_run {
            self.move_file_with_metadata(file_path, &new_path, options.conflict_resolution)?;
        }

        Ok(new_path)
    }
}

impl E6Ui {
    /// downloads reorganizer
    pub async fn reorganize_downloads(&self) -> Result<()> {
        println!("\n=== Downloads Reorganizer ===\n");
        println!("This will reorganize your downloaded files based on the current output format.");
        println!("Files will be moved to match the format specified in your config.\n");

        let download_dir: String = getopt!(download.path);
        let default_format: String = getopt!(download.format);

        let directory = Input::new("Enter directory to reorganize:")
            .theme(&ROSE_PINE)
            .default_value(&download_dir)
            .run()?;

        let directory = Path::new(&directory);
        if !directory.exists() {
            bail!("Directory does not exist: {}", directory.display());
        }

        let recursive = Confirm::new("Search subdirectories recursively?")
            .affirmative("Yes")
            .negative("No")
            .theme(&ROSE_PINE)
            .run()?;

        let use_current_format = Confirm::new("Use current output format from config?")
            .affirmative("Yes")
            .negative("No")
            .theme(&ROSE_PINE)
            .run()?;

        let output_format = if !use_current_format {
            Some(
                Input::new("Enter output format:")
                    .default_value(&default_format)
                    .theme(&ROSE_PINE)
                    .run()?,
            )
        } else {
            None
        };

        let conflict_options = [
            "Skip existing files",
            "Overwrite existing files",
            "Auto-rename duplicates",
        ];

        let conflict_choice = Select::new("How should conflicts be handled?")
            .options(
                conflict_options
                    .iter()
                    .map(DemandOption::new)
                    .collect::<Vec<_>>(),
            )
            .description("Choose what to do when target file already exists")
            .theme(&ROSE_PINE)
            .run()?;

        let conflict_resolution = match *conflict_choice {
            "Skip existing files" => ConflictResolution::Skip,
            "Overwrite existing files" => ConflictResolution::Overwrite,
            "Auto-rename duplicates" => ConflictResolution::AutoRename,
            _ => ConflictResolution::Skip,
        };

        let dry_run = Confirm::new("Perform dry run? (preview changes without moving files)")
            .affirmative("Yes")
            .negative("No")
            .theme(&ROSE_PINE)
            .run()?;

        let options = ReorganizeOptions {
            dry_run,
            conflict_resolution,
            output_format,
        };

        let reorganizer = FileReorganizer::new();
        let result = reorganizer
            .reorganize_directory(directory, options.clone(), recursive)
            .await?;

        println!("\n=== Reorganization Summary ===");
        println!("Total files: {}", result.total_files);
        println!("Successful: {}", result.successful);
        println!("Skipped: {}", result.skipped);
        println!("Failed: {}", result.failed);

        if !result.errors.is_empty() {
            println!("\nErrors:");
            for (path, error) in &result.errors {
                println!("  {}: {}", path.display(), error);
            }
        }

        if options.dry_run && result.successful > 0 {
            println!("\nThis was a dry run. No files were actually moved.");
            let proceed = Confirm::new("Would you like to perform the reorganization for real?")
                .affirmative("Yes")
                .negative("No")
                .theme(&ROSE_PINE)
                .run()?;

            if proceed {
                let real_options = ReorganizeOptions {
                    dry_run: false,
                    ..options
                };

                let final_result = reorganizer
                    .reorganize_directory(directory, real_options, recursive)
                    .await?;

                println!("\n=== Final Summary ===");
                println!("Successfully reorganized: {}", final_result.successful);
                println!("Skipped: {}", final_result.skipped);
                println!("Failed: {}", final_result.failed);
            }
        }

        Ok(())
    }
}

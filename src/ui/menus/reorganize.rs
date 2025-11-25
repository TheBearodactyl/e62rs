use {
    crate::{
        config::options::E62Rs,
        models::E6Post,
        ui::{E6Ui, ROSE_PINE, progress::ProgressManager},
    },
    color_eyre::eyre::{Context, Result, bail},
    demand::{Confirm, DemandOption, Input, Select},
    std::{
        fs::{self, OpenOptions},
        io::Read,
        path::{Path, PathBuf},
        sync::Arc,
    },
    tracing::*,
    url::Url,
};

#[derive(Debug, Clone, Copy)]
pub enum ConflictResolution {
    Skip,
    Overwrite,
    AutoRename,
}

#[derive(Debug, Clone)]
pub struct ReorganizeOptions {
    pub dry_run: bool,
    pub conflict_resolution: ConflictResolution,
    pub output_format: Option<String>,
}

impl Default for ReorganizeOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            conflict_resolution: ConflictResolution::Skip,
            output_format: None,
        }
    }
}

#[derive(Debug)]
pub struct ReorganizeResult {
    pub total_files: usize,
    pub successful: usize,
    pub skipped: usize,
    pub failed: usize,
    pub errors: Vec<(PathBuf, String)>,
}

#[derive(Default)]
pub struct FileReorganizer {
    progress_manager: Arc<ProgressManager>,
}

impl FileReorganizer {
    pub fn new() -> Self {
        Self {
            progress_manager: Arc::new(ProgressManager::new()),
        }
    }

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

    #[cfg(target_os = "windows")]
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
    fn read_metadata_from_json(&self, file_path: &Path) -> Result<E6Post> {
        let json_path = file_path.with_extension(format!(
            "{}.json",
            file_path.extension().and_then(|e| e.to_str()).unwrap_or("")
        ));

        if !json_path.exists() {
            anyhow::bail!("Metadata file not found: {}", json_path.display());
        }

        let contents = fs::read_to_string(&json_path)
            .with_context(|| format!("Failed to read metadata file {}", json_path.display()))?;

        serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse metadata for {}", file_path.display()))
    }

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

    pub fn find_files_recursive(&self, directory: &Path) -> Result<Vec<PathBuf>> {
        let mut all_files = Vec::new();

        if !directory.exists() {
            bail!("Directory does not exist: {}", directory.display());
        }

        self.find_files_recursive_impl(directory, &mut all_files)?;
        Ok(all_files)
    }

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

    fn format_filename(&self, post: &E6Post, out_fmt: &str) -> Result<String> {
        let artist = post
            .tags
            .artist
            .first()
            .map(|s| s.as_str())
            .unwrap_or("unknown");

        let tags_re = regex::Regex::new(r"\$tags\[(\d+)\]").unwrap();
        let artists_re = regex::Regex::new(r"\$artists\[(\d+)\]").unwrap();
        let characters_re = regex::Regex::new(r"\$characters\[(\d+)\]").unwrap();
        let species_re = regex::Regex::new(r"\$species\[(\d+)\]").unwrap();
        let copyright_re = regex::Regex::new(r"\$copyright\[(\d+)\]").unwrap();
        let sources_re = regex::Regex::new(r"\$sources\[(\d+)\]").unwrap();

        let mut formatted = out_fmt.to_string();

        for cap in tags_re.captures_iter(out_fmt) {
            if let Some(num_match) = cap.get(1)
                && let Ok(num_tags) = num_match.as_str().parse::<usize>()
            {
                let tags = post
                    .tags
                    .general
                    .iter()
                    .take(num_tags)
                    .cloned()
                    .collect::<Vec<String>>()
                    .join(", ");
                formatted = formatted.replace(&cap[0], &tags);
            }
        }

        for cap in artists_re.captures_iter(out_fmt) {
            if let Some(num_match) = cap.get(1)
                && let Ok(num_artists) = num_match.as_str().parse::<usize>()
            {
                let artists = post
                    .tags
                    .artist
                    .iter()
                    .take(num_artists)
                    .cloned()
                    .collect::<Vec<String>>()
                    .join(", ");
                formatted = formatted.replace(&cap[0], &artists);
            }
        }

        for cap in characters_re.captures_iter(out_fmt) {
            if let Some(num_match) = cap.get(1)
                && let Ok(num_chars) = num_match.as_str().parse::<usize>()
            {
                let characters = post
                    .tags
                    .character
                    .iter()
                    .take(num_chars)
                    .cloned()
                    .collect::<Vec<String>>()
                    .join(", ");
                formatted = formatted.replace(&cap[0], &characters);
            }
        }

        for cap in species_re.captures_iter(out_fmt) {
            if let Some(num_match) = cap.get(1)
                && let Ok(num_species) = num_match.as_str().parse::<usize>()
            {
                let species = post
                    .tags
                    .species
                    .iter()
                    .take(num_species)
                    .cloned()
                    .collect::<Vec<String>>()
                    .join(", ");
                formatted = formatted.replace(&cap[0], &species);
            }
        }

        for cap in copyright_re.captures_iter(out_fmt) {
            if let Some(num_match) = cap.get(1)
                && let Ok(num_copyright) = num_match.as_str().parse::<usize>()
            {
                let copyright = post
                    .tags
                    .copyright
                    .iter()
                    .take(num_copyright)
                    .cloned()
                    .collect::<Vec<String>>()
                    .join(", ");
                formatted = formatted.replace(&cap[0], &copyright);
            }
        }

        for cap in sources_re.captures_iter(out_fmt) {
            if let Some(num_match) = cap.get(1)
                && let Ok(num_sources) = num_match.as_str().parse::<usize>()
            {
                let sources = post
                    .sources
                    .iter()
                    .take(num_sources)
                    .map(|source| {
                        Url::parse(source)
                            .ok()
                            .and_then(|u| u.domain().map(String::from))
                            .unwrap_or_else(|| "unknown".to_string())
                    })
                    .collect::<Vec<String>>()
                    .join(", ");
                formatted = formatted.replace(&cap[0], &sources);
            }
        }

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

        let formatted = formatted
            .replace("$id", &post.id.to_string())
            .replace("$rating", rating_full)
            .replace("$rating_first", &rating_first)
            .replace("$score", &post.score.total.to_string())
            .replace("$score_up", &post.score.up.to_string())
            .replace("$score_down", &post.score.down.to_string())
            .replace("$fav_count", &post.fav_count.to_string())
            .replace("$comment_count", &post.comment_count.to_string())
            .replace("$md5", &post.file.md5)
            .replace("$ext", &post.file.ext)
            .replace("$width", &post.file.width.to_string())
            .replace("$height", &post.file.height.to_string())
            .replace("$aspect_ratio", &format!("{:.2}", aspect_ratio))
            .replace("$orientation", orientation)
            .replace("$resolution", resolution)
            .replace("$megapixels", &format!("{:.1}", megapixels))
            .replace("$size", &post.file.size.to_string())
            .replace("$size_mb", &format!("{:.2}", size_mb))
            .replace("$size_kb", &format!("{:.2}", size_kb))
            .replace("$artist", artist)
            .replace("$artist_count", &post.tags.artist.len().to_string())
            .replace("$tag_count", &tag_count.to_string())
            .replace("$tag_count_general", &post.tags.general.len().to_string())
            .replace(
                "$tag_count_character",
                &post.tags.character.len().to_string(),
            )
            .replace("$tag_count_species", &post.tags.species.len().to_string())
            .replace(
                "$tag_count_copyright",
                &post.tags.copyright.len().to_string(),
            )
            .replace("$pool_ids", &pool_ids)
            .replace("$pool_count", &post.pools.len().to_string())
            .replace("$uploader", &post.uploader_name)
            .replace("$uploader_id", &post.uploader_id.to_string())
            .replace("$approver_id", &approver_id)
            .replace(
                "$has_children",
                if post.relationships.has_children {
                    "yes"
                } else {
                    "no"
                },
            )
            .replace(
                "$parent_id",
                &post
                    .relationships
                    .parent_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "none".to_string()),
            )
            .replace("$year", &year)
            .replace("$month", &month)
            .replace("$day", &day)
            .replace("$hour", &hour)
            .replace("$minute", &minute)
            .replace("$second", &second)
            .replace("$date", &format!("{}-{}-{}", year, month, day))
            .replace("$time", &format!("{}-{}-{}", hour, minute, second))
            .replace(
                "$datetime",
                &format!("{}-{}-{} {}-{}-{}", year, month, day, hour, minute, second),
            )
            .replace("$timestamp", &timestamp)
            .replace("$year_updated", &year_updated)
            .replace("$month_updated", &month_updated)
            .replace("$day_updated", &day_updated)
            .replace(
                "$date_updated",
                &format!("{}-{}-{}", year_updated, month_updated, day_updated),
            )
            .replace("$now_year", &now.format("%Y").to_string())
            .replace("$now_month", &now.format("%m").to_string())
            .replace("$now_day", &now.format("%d").to_string())
            .replace("$now_hour", &now.format("%H").to_string())
            .replace("$now_minute", &now.format("%M").to_string())
            .replace("$now_second", &now.format("%S").to_string())
            .replace("$now_date", &now.format("%Y-%m-%d").to_string())
            .replace("$now_time", &now.format("%H-%M-%S").to_string())
            .replace(
                "$now_datetime",
                &now.format("%Y-%m-%d %H-%M-%S").to_string(),
            )
            .replace("$now_timestamp", &now.timestamp().to_string())
            .replace("$is_pending", if post.flags.pending { "yes" } else { "no" })
            .replace("$is_flagged", if post.flags.flagged { "yes" } else { "no" })
            .replace("$is_deleted", if post.flags.deleted { "yes" } else { "no" })
            .replace("$has_notes", if post.has_notes { "yes" } else { "no" })
            .replace(
                "$duration",
                &post
                    .duration
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "0".to_string()),
            )
            .replace("$duration_formatted", &duration_formatted)
            .replace("$file_type", file_type);

        Ok(formatted)
    }

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
            use std::fs::OpenOptions;

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
                if let Err(_) = fs::rename(&old_json, &new_json) {
                    fs::copy(&old_json, &new_json)?;
                    fs::remove_file(&old_json)?;
                }
            }
        }

        Ok(final_path)
    }

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

        let cfg = E62Rs::get()?;
        let dl_cfg = cfg.download;
        let output_format = options.clone().output_format;
        let download_dir = dl_cfg.download_dir;
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

            match self.process_file(
                &file_path,
                base_path,
                output_format
                    .clone()
                    .unwrap_or("$id.$ext".to_owned())
                    .as_str(),
                &options,
            ) {
                Ok(_) => {
                    result.successful += 1;
                }
                Err(e) => {
                    if e.to_string().contains("already exists") {
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
    pub async fn reorganize_downloads(&self) -> Result<()> {
        println!("\n=== Downloads Reorganizer ===\n");
        println!("This will reorganize your downloaded files based on the current output format.");
        println!("Files will be moved to match the format specified in your config.\n");

        let cfg = E62Rs::get()?;
        let dl_cfg = cfg.download;
        let download_dir = dl_cfg.download_dir;

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
                    .default_value(dl_cfg.output_format.as_str())
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

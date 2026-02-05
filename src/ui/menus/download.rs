//! post downloading stuff
//!
//! provides utilities for downloading from e(621/926), with support for:  
//! * batch downloads
//! * progress tracking
//! * metadata storage
//! * highly customizable filename formatting
use {
    crate::{
        bail,
        config::format::FormatTemplate,
        error::*,
        getopt,
        models::E6Post,
        ui::progress::ProgressManager,
        utils::{self, MutableStatic as MutStatic},
    },
    color_eyre::eyre::Context,
    futures::StreamExt,
    hashbrown::HashMap,
    indicatif::ProgressBar,
    miette::Context as _,
    reqwest::Client,
    std::{
        path::{Path, PathBuf},
        sync::Arc,
    },
    tokio::{fs::File, io::AsyncWriteExt},
    tracing::warn,
    url::Url,
};

/// the download progress for a given post
#[derive(Clone)]
pub struct DownloadProgress {
    /// the file path of the download
    pub path: PathBuf,

    /// whether or not the download failed
    pub failed: bool,

    /// id for this download
    id: u64,
}

/// all currently progressing downloads
pub static IN_PROGRESS_DOWNLOADS: MutStatic<Vec<DownloadProgress>> = MutStatic::new(Vec::new());
/// atomic counter for unique dl ids
static DOWNLOAD_ID_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[ctor::dtor]
unsafe fn terminate() {
    IN_PROGRESS_DOWNLOADS.with(|downloads| {
        for download in downloads.iter() {
            if download.failed && download.path.exists() {
                if let Err(e) = std::fs::remove_file(&download.path) {
                    eprintln!(
                        "Failed to remove incomplete download '{}': {}",
                        download.path.display(),
                        e
                    );
                } else {
                    eprintln!(
                        "Cleaned up incomplete download: {}",
                        download.path.display()
                    );
                }
            }
        }
    });
}

/// raii guard to make sure downloads are cleaned up on panic or early return
struct DownloadGuard {
    /// the id of this guard
    id: u64,
    /// the path to this download
    path: PathBuf,
}

impl DownloadGuard {
    /// make a new download guard
    fn new(path: PathBuf) -> Self {
        let id = DOWNLOAD_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        IN_PROGRESS_DOWNLOADS.update(|items| {
            items.push(DownloadProgress {
                path: path.clone(),
                failed: true,
                id,
            });
        });

        Self { id, path }
    }

    /// mark the download as successful
    fn mark_success(&self) {
        IN_PROGRESS_DOWNLOADS.update(|items| {
            if let Some(download) = items.iter_mut().find(|d| d.id == self.id) {
                download.failed = false;
            }
        });
    }

    /// clean up the file if it exists and is marked as failed
    fn cleanup_if_failed(&self) {
        let should_remove = IN_PROGRESS_DOWNLOADS.map(|items| {
            items
                .iter()
                .find(|d| d.id == self.id)
                .is_some_and(|d| d.failed)
        });

        if should_remove
            && self.path.exists()
            && let Err(e) = std::fs::remove_file(&self.path)
        {
            warn!(
                "Failed to clean up incomplete download '{}': {}",
                self.path.display(),
                e
            );
        }
    }
}

impl Drop for DownloadGuard {
    fn drop(&mut self) {
        self.cleanup_if_failed();
    }
}

/// a post downloader
///
/// handles downloading posts from e(621/926)
#[derive(Default, Clone)]
pub struct PostDownloader {
    /// the http client
    ///
    /// used for making requests to the e6 api
    pub client: Client,

    /// the path to download to
    ///
    /// specifies the base directory where downloaded files are stored
    pub download_dir: Option<PathBuf>,

    /// the format template to use for output
    ///
    /// the way filenames should be formatted using post metadata
    pub output_format: Option<String>,

    /// the progress bar manager
    ///
    /// manages and displays progress bars for download operations
    pub progress_manager: Arc<ProgressManager>,
}

/// sanitize a value for use in filenames (before template substitution)
///
/// replaces fs-unsafe chars with safe alts, using full-width unicode eqs on windows for characters
/// like `<>:"|?*` and replaces slashes with `" on "`
///
/// # Examples
///
/// ```
/// use e62rs::ui::menus::download::sanitize_value;
///
/// let sanitized = sanitize_value("artist/name");
/// assert_eq!(sanitized, "artist on name");
/// ```
#[bearive::argdoc]
pub fn sanitize_value<S: AsRef<str>>(
    /// the string to sanitize
    input: S,
) -> String {
    let s = input.as_ref();
    let mut sanitized = String::with_capacity(s.len());

    for ch in s.chars() {
        let chstr = ch.to_string();
        sanitized.push_str(match ch {
            '/' | '\\' => " on ",
            #[cfg(target_os = "windows")]
            '<' => "＜",
            #[cfg(target_os = "windows")]
            '>' => "＞",
            #[cfg(target_os = "windows")]
            ':' => "：",
            #[cfg(target_os = "windows")]
            '"' => "＂",
            #[cfg(target_os = "windows")]
            '|' => "｜",
            #[cfg(target_os = "windows")]
            '?' => "？",
            #[cfg(target_os = "windows")]
            '*' => "＊",
            _ => &chstr,
        });
    }

    sanitized
}

/// sanitize a path for fs compatibility (only OS-specific issues, not `/`)
///
/// removes/replaces chars that're invalid on the current OS but preserves path seps. primarily
/// handles windows-specific restrictions like `<>:"|?*`
///
/// # Platform-specific behavior
///
/// - **Windows**: replaces `<>:"|?*` with full-width unicode equivalents
/// - **Unix**: returns the path unchanged
#[bearive::argdoc]
pub fn sanitize_path<S: AsRef<str>>(
    /// the path to sanitize
    input: S,
) -> PathBuf {
    let s = input.as_ref();

    #[cfg(target_os = "windows")]
    {
        let mut sanitized = String::with_capacity(s.len());
        for ch in s.chars() {
            let chstr = ch.to_string();
            sanitized.push_str(match ch {
                '<' => "＜",
                '>' => "＞",
                ':' => "：",
                '"' => "＂",
                '|' => "｜",
                '?' => "？",
                '*' => "＊",
                _ => &chstr,
            });
        }
        PathBuf::from(sanitized)
    }

    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from(s)
    }
}

impl PostDownloader {
    /// make a new post downloader with a given download dir and output format
    ///
    /// makes a downloader instance configured with a specific dir and optional custom filename
    /// format. also initializes an http client and progress manager
    #[bearive::argdoc]
    pub fn with_download_dir_and_format<T>(
        /// the base directory for downloads
        download_dir: T,
        /// optional custom output format template
        output_format: Option<String>,
    ) -> Self
    where
        T: AsRef<Path>,
        PathBuf: From<T>,
    {
        Self {
            client: Client::new(),
            download_dir: Some(download_dir.into()),
            output_format,
            progress_manager: Arc::new(ProgressManager::new()),
        }
    }

    /// download multiple posts
    ///
    /// concurrently downloads a list of posts with a configurable thread limit. creates a progress
    /// bar to track overall progress and handles errs for individual downloads without stopping
    #[bearive::argdoc]
    #[error = "the progress bar cannot be created (individual download failures are logged as \
               warnings)"]
    pub async fn download_posts(
        self: Arc<Self>,
        /// the posts to download
        posts: Vec<E6Post>,
    ) -> Result<()> {
        let concurrent_limit = getopt!(download.threads);

        let total_pb = self
            .progress_manager
            .create_count_bar("total", posts.len() as u64, "Total Downloads")
            .await?;

        let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrent_limit));

        let tasks: Vec<_> = posts
            .into_iter()
            .enumerate()
            .map(|(i, post)| {
                let downloader = Arc::clone(&self);
                let semaphore = Arc::clone(&semaphore);
                let total_pb = total_pb.clone();

                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    let result = downloader.download_post(post, i).await;
                    total_pb.inc(1);
                    result
                })
            })
            .collect();

        let results = futures_util::future::join_all(tasks).await;

        total_pb.finish_with_message("✓ All downloads completed");

        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(Ok(_)) => {}
                Ok(Err(_)) => {}
                Err(e) => warn!("task {} failed: {}", i, e),
            }
        }

        Ok(())
    }

    /// download an individual post
    ///
    /// downloads a single post, saves it to disk with formatted filename, and optionally
    /// stoers metadata. creates and manages a progress bar for tracking the download
    #[bearive::argdoc]
    #[error = "the post has no downloadable url"]
    #[error = "the filename cannot be formatted"]
    #[error = "the http request fails"]
    #[error = "the file cannot be saved"]
    pub async fn download_post(
        &self,
        /// the post to download
        post: E6Post,
        /// the index of this post in a batch download
        index: usize,
    ) -> Result<()> {
        let url = post
            .file
            .url
            .clone()
            .context("post has no downloadable file url")
            .map_err(Report::new)?;

        let filename = self.format_filename(&post)?;
        let filepath = self.get_filepath(&filename)?;
        let guard = DownloadGuard::new(filepath.clone());

        let prog_message = match getopt!(ui.progress.message).as_str() {
            "id" => post.id.to_string(),
            "filename" => filename.clone(),
            _ => post.id.to_string(),
        };

        let pb_key = format!("download_{}", index);
        let pb = self
            .progress_manager
            .mk_dl_bar(&pb_key, 0, &format!("Downloading {}", prog_message))
            .await?;

        let response = match self.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                pb.finish_with_message(format!("✗ Failed: {}", filename));
                self.progress_manager.remove_bar(&pb_key).await;
                return Err(e.into());
            }
        };

        let total_size = response.content_length().unwrap_or(0);
        pb.set_length(total_size);

        let response = match response.error_for_status() {
            Ok(r) => r,
            Err(e) => {
                pb.finish_with_message(format!("✗ Server error: {}", filename));
                self.progress_manager.remove_bar(&pb_key).await;
                return Err(e.into());
            }
        };

        match self
            .save_to_file(response, &filepath, pb.clone(), &post)
            .await
        {
            Ok(_) => {
                guard.mark_success();
                pb.finish_with_message(format!("✓ Downloaded {}", filename));
                self.progress_manager.remove_bar(&pb_key).await;
                Ok(())
            }
            Err(e) => {
                pb.finish_with_message(format!("✗ Save failed: {}", filename));
                self.progress_manager.remove_bar(&pb_key).await;
                Err(e)
            }
        }
    }

    /// save a post to a file
    ///
    /// streams the http response to disk while updating a progress bar, optionally saves metadata
    /// to an ADS (Windows) or JSON file (Unix)
    #[bearive::argdoc]
    #[error = "returns an error if"]
    pub async fn save_to_file(
        &self,
        /// the http response to stream from
        response: reqwest::Response,
        /// the path where the file will be saved
        filepath: &Path,
        /// progress bar for download progress tracking
        pb: ProgressBar,
        /// post metadata to save alongside the file
        post: &E6Post,
    ) -> Result<()> {
        let temp_path = filepath.with_extension("tmp");
        let mut file = File::create(&temp_path)
            .await
            .context(format!(
                "failed to create temp file '{}'",
                temp_path.display()
            ))
            .map_err(Report::new)?;

        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;
        let mut last_update = 0u64;
        const UPDATE_THRESHOLD: u64 = 8192;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Error reading chunk from response")?;
            downloaded += chunk.len() as u64;

            file.write_all(&chunk)
                .await
                .with_context(|| format!("Error writing to temp file '{}'", temp_path.display()))?;

            if downloaded - last_update >= UPDATE_THRESHOLD {
                pb.set_position(downloaded);
                last_update = downloaded;
            }
        }

        file.flush().await.context("Failed to flush file to disk")?;
        file.sync_all()
            .await
            .context("Failed to sync file to disk")?;

        drop(file);

        tokio::fs::rename(&temp_path, filepath)
            .await
            .with_context(|| {
                format!(
                    "Failed to move temp file '{}' to '{}'",
                    temp_path.display(),
                    filepath.display()
                )
            })
            .map_err(Report::new)?;

        if getopt!(download.save_metadata) {
            #[cfg(target_os = "windows")]
            {
                if let Err(e) = utils::write_to_ads(filepath, "metadata", post) {
                    warn!(
                        "Failed to write metadata ADS for '{}': {}",
                        filepath.display(),
                        e
                    );
                }
            }

            #[cfg(not(target_os = "windows"))]
            {
                if let Err(e) = utils::write_to_json(filepath, post) {
                    warn!(
                        "Failed to write metadata JSON for '{}': {}",
                        filepath.display(),
                        e
                    );
                }
            }
        }

        Ok(())
    }

    /// format the filename based on the post metadata
    ///
    /// applies a format template to post metadata to gen a filename. uses the configured output
    /// format or a default if non is specified
    #[bearive::argdoc]
    #[error = "the format template can't be parsed"]
    #[error = "the template can't be rendered with the post data"]
    pub fn format_filename(
        &self,
        /// the post to generate a filename for
        post: &E6Post,
    ) -> Result<String> {
        let out_fmt = self.output_format.as_deref().unwrap_or("$id.$ext");
        let template = FormatTemplate::parse(out_fmt).context("failed to parse output format")?;
        let (simple_context, array_context) = build_context_from_post(post);
        let formatted = template
            .render_with_arrays(&simple_context, &array_context)
            .context("Failed to render filename template")?;

        Ok(formatted)
    }

    /// get the path to a file
    ///
    /// constructs the full path for a file, making parent dirs as needed. checks if the file
    /// already exists and returns an error if it does
    #[bearive::argdoc]
    #[error = "parent directories can't be made"]
    #[error = "the file already exists"]
    pub fn get_filepath(
        &self,
        /// the filename to construct a path for
        filename: &str,
    ) -> Result<PathBuf> {
        let filename = sanitize_path(filename);

        let path = if let Some(ref dir) = self.download_dir {
            dir.join(filename)
        } else {
            filename
        };

        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent)
                .context(format!("Failed to create directory: {}", parent.display()))?;
        }

        if path.exists() {
            bail!("File '{}' already exists", path.display());
        }

        Ok(path)
    }

    /// make a new downloader for a pool, saving to `<download_dir>/<pool_name>`
    ///
    /// creates a downloader configured to save files in a subdir named after the given
    /// pool. sanitizes the pool name to ensure fs compat
    #[bearive::argdoc]
    pub fn for_pool<T, S>(
        /// the base dir for pool downloads
        base_download_dir: T,
        /// the name of the pool being downloaded
        pool_name: S,
    ) -> Self
    where
        T: AsRef<Path>,
        S: AsRef<str>,
    {
        let sanitized_name = sanitize_pool_name(pool_name.as_ref());
        let pool_dir = base_download_dir.as_ref().join(sanitized_name);

        Self {
            client: Client::new(),
            download_dir: Some(pool_dir),
            output_format: None,
            progress_manager: Arc::new(ProgressManager::new()),
        }
    }

    /// download pool posts with sequential naming based on pool order
    ///
    /// download posts from a pool using seq numbering (001, 002, 003, etc.) to preserve the pool's
    /// intended order. handles concurrent downloads while maintaining filename order
    #[bearive::argdoc]
    #[error = "progress bar creation fails"]
    pub async fn download_pool_posts(
        self: Arc<Self>,
        /// the posts to download in seq order
        posts: Vec<E6Post>,
    ) -> Result<()> {
        let concurrent_limit = getopt!(download.threads);
        let total = posts.len();
        let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrent_limit));
        let pad_width = total.to_string().len().max(3);
        let total_pb = self
            .progress_manager
            .create_count_bar("total", total as u64, "Total Downloads")
            .await?;

        let tasks: Vec<_> = posts
            .into_iter()
            .enumerate()
            .map(|(i, post)| {
                let downloader = Arc::clone(&self);
                let semaphore = Arc::clone(&semaphore);
                let total_pb = total_pb.clone();
                let sequence_num = i + 1;

                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    let result = downloader
                        .download_pool_post(post, sequence_num, pad_width)
                        .await;
                    total_pb.inc(1);
                    result
                })
            })
            .collect();

        let results = futures_util::future::join_all(tasks).await;

        total_pb.finish_with_message("✓ All downloads completed");

        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(Ok(_)) => {}
                Ok(Err(_)) => {}
                Err(e) => warn!("task {} failed: {}", i + 1, e),
            }
        }

        Ok(())
    }

    /// download a single pool post with sequential naming
    ///
    /// downloads one post from a pool using a 0-padded seq number as the filename. preserves
    /// metadata alongside the downloaded file
    #[bearive::argdoc]
    #[error = "the post has no downloadable url"]
    #[error = "the download fails"]
    #[error = "the file cannot be saved"]
    pub async fn download_pool_post(
        &self,
        /// the post to download
        post: E6Post,
        /// the 1-indexed position in the pool
        sequence_num: usize,
        /// the number of digits for 0-padding
        pad_width: usize,
    ) -> Result<()> {
        let url = post
            .file
            .url
            .clone()
            .context("Post has no downloadable file URL")?;

        let filename = format!(
            "{:0width$}.{}",
            sequence_num,
            post.file.ext,
            width = pad_width
        );
        let filepath = self.get_filepath(&filename)?;

        let guard = DownloadGuard::new(filepath.clone());
        let pb_key = format!("download_{}", sequence_num);
        let pb = self
            .progress_manager
            .mk_dl_bar(&pb_key, 0, &format!("Downloading {}", filename))
            .await?;

        let response = match self.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                pb.finish_with_message(format!("✗ Failed: {}", filename));
                self.progress_manager.remove_bar(&pb_key).await;
                return Err(e.into());
            }
        };

        let total_size = response.content_length().unwrap_or(0);
        pb.set_length(total_size);

        let response = match response.error_for_status() {
            Ok(r) => r,
            Err(e) => {
                pb.finish_with_message(format!("✗ Server error: {}", filename));
                self.progress_manager.remove_bar(&pb_key).await;
                return Err(e.into());
            }
        };

        match self
            .save_to_file(response, &filepath, pb.clone(), &post)
            .await
        {
            Ok(_) => {
                guard.mark_success();
                pb.finish_with_message(format!("✓ Downloaded {}", filename));
                self.progress_manager.remove_bar(&pb_key).await;
                Ok(())
            }
            Err(e) => {
                pb.finish_with_message(format!("✗ Save failed: {}", filename));
                self.progress_manager.remove_bar(&pb_key).await;
                Err(e)
            }
        }
    }
}

/// build template context based on post metadata
///
/// extracts metadata from a post and organizes it into simple str maps and arr maps for use in
/// filename templates. includes computed vals like aspect ratio, res cat, and formatted data
///
/// # Returns
///
/// a tuple of:
/// - simple string subs (id, rating, score, etc.)
/// - array subs (tags, artists, characters, etc.)
///
/// # Available template variables
///
/// **Basic Post Information:**
///
/// - `$id` → post ID
/// - `$rating` → rating (e.g. `"safe"`, `"questionable"`, `"explicit"`)
/// - `$rating_first` → first character of rating (`s`, `q`, `e`)
/// - `$md5` → MD5 hash of file
/// - `$ext` → file extension
///
/// **Scores & Engagement:**
///
/// - `$score` → total post score
/// - `$score_up` → upvote score
/// - `$score_down` → downvote score
/// - `$fav_count` → number of favorites
/// - `$comment_count` → number of comments
///
/// **File Metadata:**
///
/// - `$width` / `$height` → file dimensions in pixels
/// - `$aspect_ratio` → aspect ratio (width/height)
/// - `$orientation` → `"portrait"`, `"landscape"`, or `"square"`
/// - `$resolution` → resolution category (`"SD"`, `"HD"`, `"FHD"`, `"QHD"`, `"4K"`, `"8K"`)
/// - `$megapixels` → megapixel count (rounded to 1 decimal)
/// - `$size` → file size in bytes
/// - `$size_mb` → file size in megabytes (rounded to 2 decimals)
/// - `$size_kb` → file size in kilobytes (rounded to 2 decimals)
/// - `$file_type` → media type (`"image"`, `"video"`, `"flash"`, `"unknown"`)
///
/// **Video-Specific:**
///
/// - `$duration` → video duration in seconds (0 if not applicable)
/// - `$duration_formatted` → video duration as `MM:SS` or `HH:MM:SS`
///
/// **User Information:**
///
/// - `$artist` → first listed artist (or `"unknown"`)
/// - `$uploader` → uploader username
/// - `$uploader_id` → uploader user ID
/// - `$approver_id` → approver ID (or `"none"`)
///
/// **Tag Counts:**
///
/// - `$tag_count` → total number of tags
/// - `$artist_count` → number of artist tags
/// - `$tag_count_general` → number of general tags
/// - `$tag_count_character` → number of character tags
/// - `$tag_count_species` → number of species tags
/// - `$tag_count_copyright` → number of copyright tags
///
/// **Pool Information:**
///
/// - `$pool_ids` → comma-separated list of pool IDs
/// - `$pool_count` → number of pools the post is in
///
/// **Relationships:**
///
/// - `$has_children` → `"yes"` if post has children, `"no"` otherwise
/// - `$parent_id` → parent post ID (or `"none"`)
///
/// **Flags:**
///
/// - `$is_pending` → `"yes"` if pending approval, `"no"` otherwise
/// - `$is_flagged` → `"yes"` if flagged, `"no"` otherwise
/// - `$is_deleted` → `"yes"` if deleted, `"no"` otherwise
/// - `$has_notes` → `"yes"` if has notes, `"no"` otherwise
///
/// ### Date/Time Placeholders
///
/// **Post Creation Date:**
///
/// - `$year`, `$month`, `$day` → creation date components
/// - `$hour`, `$minute`, `$second` → creation time components
/// - `$date` → shorthand for `$year-$month-$day`
/// - `$time` → shorthand for `$hour-$minute-$second`
/// - `$datetime` → shorthand for `$year-$month-$day $hour-$minute-$second`
/// - `$timestamp` → Unix timestamp of upload
///
/// **Post Update Date:**
///
/// - `$year_updated`, `$month_updated`, `$day_updated` → last update date components
/// - `$date_updated` → shorthand for `$year_updated-$month_updated-$day_updated`
///
/// **Download Time:**
///
/// - `$now_year`, `$now_month`, `$now_day` → download date components
/// - `$now_hour`, `$now_minute`, `$now_second` → download time components
/// - `$now_date` → shorthand for `$now_year-$now_month-$now_day`
/// - `$now_time` → shorthand for `$now_hour-$now_minute-$now_second`
/// - `$now_datetime` → shorthand for `$now_year-$now_month-$now_day $now_hour-$now_minute-$now_second`
/// - `$now_timestamp` → Unix timestamp at download time
///
/// ### Indexed Placeholders
///
/// These placeholders allow you to extract multiple items from lists. They support several range syntaxes:
///
/// **Syntax:**
///
/// - `$key[N]` → first N items (e.g., `$tags[5]`)
/// - `$key[L..R]` → items from index L to R (exclusive) (e.g., `$tags[2..5]`)
/// - `$key[N..]` → all items from index N onwards (e.g., `$artists[1..]`)
/// - `$key[..N]` → items from start to index N (exclusive) (e.g., `$sources[..3]`)
///
/// **Available indexed placeholders:**
///
/// - `$tags[...]` → general tags, joined by commas
/// - `$artists[...]` → artist tags, joined by commas
/// - `$characters[...]` → character tags, joined by commas
/// - `$species[...]` → species tags, joined by commas
/// - `$copyright[...]` → copyright tags, joined by commas
/// - `$sources[...]` → source domains, joined by commas
#[bearive::argdoc]
pub fn build_context_from_post(
    /// the post to extract metadata from
    post: &E6Post,
) -> (HashMap<String, String>, HashMap<String, Vec<String>>) {
    let mut simple = HashMap::new();
    let mut arrays = HashMap::new();
    let now = chrono::Local::now();

    let mut insert = |key: &str, value: String| {
        simple.insert(key.to_string(), sanitize_value(&value));
    };

    insert("id", post.id.to_string());
    insert("md5", post.file.md5.to_string());
    insert("ext", post.file.ext.to_string());
    insert("width", post.file.width.to_string());
    insert("height", post.file.height.to_string());
    insert("size", post.file.size.to_string());
    insert(
        "size_mb",
        format!("{:.2}", post.file.size as f64 / (1024.0 * 1024.0)),
    );
    insert("size_kb", format!("{:.2}", post.file.size as f64 / 1024.0));

    let rating_full = match post.rating.as_str() {
        "e" => "explicit",
        "q" => "questionable",
        "s" => "safe",
        _ => "unknown",
    };
    insert("rating", rating_full.to_string());
    insert(
        "rating_first",
        post.rating
            .chars()
            .next()
            .unwrap_or('u')
            .to_lowercase()
            .to_string(),
    );

    insert("score", post.score.total.to_string());
    insert("score_up", post.score.up.to_string());
    insert("score_down", post.score.down.to_string());
    insert("fav_count", post.fav_count.to_string());
    insert("comment_count", post.comment_count.to_string());

    let ratio = if post.file.height > 0 {
        post.file.width as f64 / post.file.height as f64
    } else {
        0.0
    };
    insert("aspect_ratio", format!("{:.2}", ratio));

    let orientation = if post.file.width > post.file.height {
        "landscape"
    } else if post.file.width < post.file.height {
        "portrait"
    } else {
        "square"
    };
    insert("orientation", orientation.to_string());

    let resolution = match (post.file.width, post.file.height) {
        (w, h) if w >= 7680 || h >= 4320 => "8K",
        (w, h) if w >= 3840 || h >= 2160 => "4K",
        (w, h) if w >= 2560 || h >= 1440 => "QHD",
        (w, h) if w >= 1920 || h >= 1080 => "FHD",
        (w, h) if w >= 1280 || h >= 720 => "HD",
        _ => "SD",
    };
    insert("resolution", resolution.to_string());
    insert(
        "megapixels",
        format!(
            "{:.1}",
            (post.file.width * post.file.height) as f64 / 1_000_000.0
        ),
    );

    insert(
        "artist",
        post.tags
            .artist
            .first()
            .cloned()
            .unwrap_or_else(|| "unknown".to_string()),
    );
    insert("artist_count", post.tags.artist.len().to_string());

    let total_tags = post.tags.general.len()
        + post.tags.artist.len()
        + post.tags.character.len()
        + post.tags.species.len()
        + post.tags.copyright.len()
        + post.tags.meta.len()
        + post.tags.lore.len();
    insert("tag_count", total_tags.to_string());
    insert("tag_count_general", post.tags.general.len().to_string());
    insert("tag_count_character", post.tags.character.len().to_string());
    insert("tag_count_species", post.tags.species.len().to_string());
    insert("tag_count_copyright", post.tags.copyright.len().to_string());

    insert("pool_count", post.pools.len().to_string());
    insert(
        "pool_ids",
        post.pools
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(","),
    );

    insert("uploader", post.uploader_name.clone());
    insert("uploader_id", post.uploader_id.to_string());
    insert(
        "approver_id",
        post.approver_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "none".to_string()),
    );

    insert(
        "has_children",
        if post.relationships.has_children {
            "yes"
        } else {
            "no"
        }
        .to_string(),
    );
    insert(
        "parent_id",
        post.relationships
            .parent_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "none".to_string()),
    );

    insert(
        "is_pending",
        if post.flags.pending { "yes" } else { "no" }.to_string(),
    );
    insert(
        "is_flagged",
        if post.flags.flagged { "yes" } else { "no" }.to_string(),
    );
    insert(
        "is_deleted",
        if post.flags.deleted { "yes" } else { "no" }.to_string(),
    );
    insert(
        "has_notes",
        if post.has_notes { "yes" } else { "no" }.to_string(),
    );

    if let Some(duration) = post.duration {
        insert("duration", duration.to_string());
        let total_seconds = duration as i64;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        let formatted = if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        };
        insert("duration_formatted", formatted);
    } else {
        insert("duration", "0".to_string());
        insert("duration_formatted", "N/A".to_string());
    }

    let file_type = match post.file.ext.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => "image",
        "mp4" | "webm" | "mov" | "avi" | "mkv" => "video",
        "swf" => "flash",
        _ => "unknown",
    };
    insert("file_type", file_type.to_string());

    if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&post.created_at) {
        insert("year", created.format("%Y").to_string());
        insert("month", created.format("%m").to_string());
        insert("day", created.format("%d").to_string());
        insert("hour", created.format("%H").to_string());
        insert("minute", created.format("%M").to_string());
        insert("second", created.format("%S").to_string());
        insert("date", created.format("%Y-%m-%d").to_string());
        insert("time", created.format("%H-%M-%S").to_string());
        insert("datetime", created.format("%Y-%m-%d %H-%M-%S").to_string());
        insert("timestamp", created.timestamp().to_string());
    }

    if let Ok(updated) = chrono::DateTime::parse_from_rfc3339(&post.updated_at) {
        insert("year_updated", updated.format("%Y").to_string());
        insert("month_updated", updated.format("%m").to_string());
        insert("day_updated", updated.format("%d").to_string());
        insert("date_updated", updated.format("%Y-%m-%d").to_string());
    }

    insert("now_year", now.format("%Y").to_string());
    insert("now_month", now.format("%m").to_string());
    insert("now_day", now.format("%d").to_string());
    insert("now_hour", now.format("%H").to_string());
    insert("now_minute", now.format("%M").to_string());
    insert("now_second", now.format("%S").to_string());
    insert("now_date", now.format("%Y-%m-%d").to_string());
    insert("now_time", now.format("%H-%M-%S").to_string());
    insert("now_datetime", now.format("%Y-%m-%d %H-%M-%S").to_string());
    insert("now_timestamp", now.timestamp().to_string());

    arrays.insert(
        "tags".to_string(),
        post.tags.general.iter().map(sanitize_value).collect(),
    );

    if post.tags.artist.is_empty() {
        arrays.insert("artists".to_string(), vec!["NA".to_string()]);
    } else {
        arrays.insert(
            "artists".to_string(),
            post.tags.artist.iter().map(sanitize_value).collect(),
        );
    }

    arrays.insert(
        "characters".to_string(),
        post.tags.character.iter().map(sanitize_value).collect(),
    );
    arrays.insert(
        "species".to_string(),
        post.tags.species.iter().map(sanitize_value).collect(),
    );
    arrays.insert(
        "copyright".to_string(),
        post.tags.copyright.iter().map(sanitize_value).collect(),
    );

    let sources: Vec<String> = post
        .sources
        .iter()
        .filter_map(|source| {
            Url::parse(source)
                .ok()
                .and_then(|u| u.domain().map(sanitize_value))
        })
        .collect();
    arrays.insert("sources".to_string(), sources);

    (simple, arrays)
}

/// sanitize a pool name for use as a directory name
///
/// removes/replaces chars that're problematic in dir names. collapses multiple consecutive
/// underscores and trims leading/trailing underscores
///
/// # Examples
///
/// ```
/// let sanitized = e62rs::ui::menus::download::sanitize_pool_name("I'm A Pool, Motherfucker: Part 2");
/// assert_eq!(sanitized, "I'm_A_Pool,_Motherfucker_Part_2");
/// ```
#[bearive::argdoc]
pub fn sanitize_pool_name<S: AsRef<str>>(
    /// the pool name to sanatize
    name: S,
) -> String {
    let s = name.as_ref();
    let mut sanitized = String::with_capacity(s.len());

    for ch in s.chars() {
        let replacement = match ch {
            '<' | '>' | ':' | '"' | '|' | '?' | '*' => '_',
            '/' | '\\' => '_',
            c if c.is_control() => continue,
            ' ' => '_',
            _ => ch,
        };
        sanitized.push(replacement);
    }

    let mut result = String::with_capacity(sanitized.len());
    let mut last_was_underscore = false;

    for ch in sanitized.chars() {
        if ch == '_' {
            if !last_was_underscore {
                result.push(ch);
            }

            last_was_underscore = true;
        } else {
            result.push(ch);
            last_was_underscore = false;
        }
    }

    result.trim_matches('_').to_string()
}

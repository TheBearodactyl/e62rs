//! post viewing stuff
use {
    crate::{
        display::{
            dtext::parser::format_text,
            image::{
                animation::{AnimatedImage, is_animated_format, load_animated},
                dimensions::ImageDimensions,
                encoder::SixelEncoder,
                processor::ImageProcessor,
                source::ImageSource,
            },
        },
        getopt,
        models::E6Post,
        ui::E6Ui,
    },
    color_eyre::eyre::{Context, Result, bail},
    std::{
        io::{self, Write},
        path::Path,
        thread,
        time::Duration,
    },
};

/// load an animation from bytes with explicit extension
///
/// # Arguments
///
/// * `bytes` - the bytes of the animation file
/// * `ext` - the extension to load as (webp/gif)
fn load_animated_from_bytes_with_ext(bytes: &[u8], ext: &str) -> Result<AnimatedImage> {
    match ext.to_lowercase().as_str() {
        "gif" => AnimatedImage::from_gif_bytes(bytes),
        "webp" => AnimatedImage::from_webp_bytes(bytes),
        _ => bail!("Unsupported animation format: {}", ext),
    }
}

/// play an animation in the terminal
///
/// # Arguments
///
/// * `animated` - the animated image to play
/// * `processor` - an image processor
/// * `encoder` - a sixel encoder
fn play_animation(
    animated: AnimatedImage,
    processor: &ImageProcessor,
    encoder: &SixelEncoder,
) -> Result<()> {
    if animated.frame_count() == 1 {
        let frame = animated.get_frame(0).unwrap();
        let sixel_str = encoder
            .encode(&frame.data)
            .context("failed to encode frame to sixel")?;
        print!("{}", sixel_str);
        io::stdout().flush()?;
        return Ok(());
    }

    let processed = processor
        .process_animated(animated)
        .context("failed to process animation")?;

    #[allow(clippy::manual_div_ceil, reason = "bro")]
    let term_lines = (processed.height + 5) / 6;

    let mut encoded_frames = Vec::with_capacity(processed.frames.len());
    for frame in &processed.frames {
        let sixel_str = encoder
            .encode(&frame.data)
            .context("failed to encode frame to sixel")?;
        encoded_frames.push((sixel_str, frame.delay));
    }

    print!("\x1b[s");
    io::stdout().flush()?;

    let is_infinite = processed.is_infinite_loop();
    let loop_count = if is_infinite {
        1000
    } else {
        processed.loop_count.max(1)
    };

    for loop_idx in 0..loop_count {
        for (frame_idx, (sixel_str, delay)) in encoded_frames.iter().enumerate() {
            if loop_idx > 0 || frame_idx > 0 {
                print!("\x1B[{}A\x1B[G", term_lines);
            }

            print!("{}", sixel_str);
            println!();
            io::stdout().flush()?;

            thread::sleep(*delay);
        }

        if !is_infinite {
            break;
        }
    }

    thread::sleep(Duration::from_millis(100));
    print!("\x1b[u");
    io::stdout().flush()?;

    Ok(())
}

/// fetch a post image and display it in the terminal
///
/// # Arguments
///
/// * `post` - the post to fetch and display
#[allow(clippy::await_holding_lock)]
pub async fn print_post_to_terminal(post: E6Post) -> Result<()> {
    let post_url = post.file.url.unwrap();
    let cfg = crate::config::instance::config()?;
    let processor = ImageProcessor::with_dimensions(ImageDimensions::from_cfg(&cfg)?);
    let encoder = SixelEncoder::new();
    let source = ImageSource::from_url(&post_url)
        .await
        .context("failed to fetch image")?;

    let url_path = Path::new(&post_url);
    if let (true, Some(ext)) = (is_animated_format(url_path), url_path.extension())
        && let ImageSource::Bytes(bytes) = &source
    {
        let ext_str = ext.to_string_lossy();
        match load_animated_from_bytes_with_ext(bytes, &ext_str) {
            Ok(animated) => {
                return play_animation(animated, &processor, &encoder);
            }
            Err(e) => {
                eprintln!(
                    "warning: file appears to be animated but failed to load as animation ({}), \
                     trying as static image...",
                    e
                );
            }
        }
    }

    let image_data = processor
        .process(source)
        .context("failed to process image")?;
    let sixel_str = encoder
        .encode(&image_data)
        .context("failed to encode image to sixel")?;

    print!("{}", sixel_str);
    println!();

    Ok(())
}

/// print an image to the terminal
///
/// # Arguments
///
/// * `path` - the path to the image to display
pub fn print_dl_to_terminal(path: &Path) -> Result<()> {
    let cfg = getopt!()?;
    let processor = ImageProcessor::with_dimensions(ImageDimensions::from_cfg(&cfg)?);
    let encoder = SixelEncoder::new();

    if is_animated_format(path) {
        match load_animated(path) {
            Ok(animated) => {
                return play_animation(animated, &processor, &encoder);
            }
            Err(e) => {
                eprintln!(
                    "warning: file appears to be animated but failed to load as animation ({}), \
                     trying as static image...",
                    e
                );
            }
        }
    }

    let source = ImageSource::from_path(path).context("failed to load image")?;
    let image_data = processor
        .process(source)
        .context("failed to process image")?;
    let sixel_str = encoder
        .encode(&image_data)
        .context("failed to encode to sixel")?;

    print!("{}", sixel_str);
    println!();

    Ok(())
}

/// fetch multiple posts and display them in the terminal
///
/// # Arguments
///
/// * `posts` - a list of posts to fetch and print
pub async fn print_posts_to_terminal(posts: Vec<E6Post>) -> Result<()> {
    for post in posts {
        print_post_to_terminal(post).await?;
    }

    Ok(())
}

/// functions for viewing posts
pub trait ViewMenu {
    /// open a post in the users default browser
    ///
    /// # Arguments
    ///
    /// * `post` - the post to open
    fn open_in_browser(&self, post: &E6Post) -> Result<()>;

    /// open multiple posts in the users default browser
    ///
    /// # Arguments
    ///
    /// * `posts` - a list of posts to open
    fn open_posts_in_browser(&self, posts: &[E6Post]) -> Result<()>;

    /// display post info
    ///
    /// # Arguments
    ///
    /// * `posts` - a list of posts to display the info for
    /// * `column_width` - the width for each column of the displayed info
    fn display_posts_row(&self, posts: &[E6Post], column_width: usize);

    /// print a post field
    ///
    /// # Arguments
    ///
    /// * `posts` - a list of posts to display the info for
    /// * `column_width` - the width for each column of the displayed info
    /// * `field_fn` - a function that returns the data to display in the field
    fn print_posts_field<F>(&self, posts: &[E6Post], column_width: usize, field_fn: F)
    where
        F: Fn(&E6Post) -> String;

    /// print a row separator
    ///
    /// # Arguments
    ///
    /// * `count` - the number of separators to make
    /// * `column_width` - the width for each column of the displayed info
    /// * `left` - the character to use for the left of the separator
    /// * `mid` - the character to use for the middle of the separator
    /// * `right` - the character to use for the right of the separator
    /// * `fill` - the character to use for filling in blank space
    fn print_row_separator(
        &self,
        count: usize,
        column_width: usize,
        left: &str,
        mid: &str,
        right: &str,
        fill: &str,
    );

    /// truncate a string
    ///
    /// # Arguments
    ///
    /// * `s` - the string to truncate
    /// * `max_width` - the width to truncate it to
    fn truncate_string(&self, s: &str, max_width: usize) -> String;

    /// display multiple posts
    ///
    /// # Arguments
    ///
    /// * `posts` - a list of posts to display
    fn display_posts(&self, posts: &[E6Post]);

    /// display the latest posts
    fn display_latest_posts(&self) -> impl Future<Output = Result<()>>;

    /// display an individual post
    ///
    /// # Arguments
    ///
    /// * `post` - the post to display
    fn display_post(&self, post: &E6Post);
}

impl ViewMenu for E6Ui {
    /// open a post in the users default browser
    ///
    /// # Arguments
    ///
    /// * `post` - the post to open
    fn open_in_browser(&self, post: &E6Post) -> Result<()> {
        if post.id <= 0 {
            bail!("invalid post id: {}", post.id);
        }

        let url = format!("https://e621.net/posts/{}", post.id);
        open::that(&url).context("failed to open post in browser")?;
        println!("Opened post in browser: {}", url);
        Ok(())
    }

    /// open multiple posts in the users default browser
    ///
    /// # Arguments
    ///
    /// * `posts` - a list of posts to open
    fn open_posts_in_browser(&self, posts: &[E6Post]) -> Result<()> {
        println!("Opening {} posts in browser...", posts.len());
        for post in posts {
            let url = format!("https://e621.net/posts/{}", post.id);
            open::that(&url).context("Failed to open post in browser")?;
            println!("Opened post {} in browser", post.id);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        println!("✓ Opened all {} posts in browser", posts.len());
        Ok(())
    }

    /// display post info
    ///
    /// # Arguments
    ///
    /// * `posts` - a list of posts to display the info for
    /// * `column_width` - the width for each column of the displayed info
    fn display_posts_row(&self, posts: &[E6Post], column_width: usize) {
        self.print_row_separator(posts.len(), column_width, "┌", "┬", "┐", "─");
        self.print_posts_field(posts, column_width, |post| format!("ID: {}", post.id));
        self.print_row_separator(posts.len(), column_width, "├", "┼", "┤", "─");
        self.print_posts_field(posts, column_width, |post| {
            format!("Rating: {} | Score: {}", post.rating, post.score.total)
        });
        self.print_posts_field(posts, column_width, |post| {
            let uploader = self.truncate_string(&post.uploader_name, 15);
            format!(" {} | By: {}", post.fav_count, uploader)
        });
        self.print_posts_field(posts, column_width, |post| {
            if !post.tags.artist.is_empty() {
                let artists = post.tags.artist.join(", ");
                format!(" {}", self.truncate_string(&artists, column_width - 4))
            } else {
                "󰏫 Unknown artist".to_string()
            }
        });

        self.print_posts_field(posts, column_width, |post| {
            let tags = post
                .tags
                .general
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            format!("󰓹 {}", self.truncate_string(&tags, column_width - 4))
        });

        self.print_row_separator(posts.len(), column_width, "└", "┴", "┘", "─");
    }

    /// print a post field
    ///
    /// # Arguments
    ///
    /// * `posts` - a list of posts to display the info for
    /// * `column_width` - the width for each column of the displayed info
    /// * `field_fn` - a function that returns the data to display in the field
    fn print_posts_field<F>(&self, posts: &[E6Post], column_width: usize, field_fn: F)
    where
        F: Fn(&E6Post) -> String,
    {
        print!("│");
        for (i, post) in posts.iter().enumerate() {
            let content = field_fn(post);
            let truncated = self.truncate_string(&content, column_width - 2);
            print!(" {:width$} ", truncated, width = column_width - 2);
            print!("│");
            if i < posts.len() - 1 {}
        }
        println!();
    }

    /// print a row separator
    ///
    /// # Arguments
    ///
    /// * `count` - the number of separators to make
    /// * `column_width` - the width for each column of the displayed info
    /// * `left` - the character to use for the left of the separator
    /// * `mid` - the character to use for the middle of the separator
    /// * `right` - the character to use for the right of the separator
    /// * `fill` - the character to use for filling in blank space
    fn print_row_separator(
        &self,
        count: usize,
        column_width: usize,
        left: &str,
        mid: &str,
        right: &str,
        fill: &str,
    ) {
        print!("{}", left);
        for i in 0..count {
            print!("{}", fill.repeat(column_width));
            if i < count - 1 {
                print!("{}", mid);
            }
        }
        print!("{}", right);
        println!();
    }

    /// truncate a string
    ///
    /// # Arguments
    ///
    /// * `s` - the string to truncate
    /// * `max_width` - the width to truncate it to
    fn truncate_string(&self, s: &str, max_width: usize) -> String {
        if s.len() <= max_width {
            format!("{:width$}", s, width = max_width)
        } else {
            format!("{}...", &s[..max_width.saturating_sub(3)])
        }
    }

    /// display multiple posts
    ///
    /// # Arguments
    ///
    /// * `posts` - a list of posts to display
    fn display_posts(&self, posts: &[E6Post]) {
        let posts_per_row = 3;
        let column_width = 28;

        for chunk in posts.chunks(posts_per_row) {
            self.display_posts_row(chunk, column_width);
            println!();
        }
    }

    /// display the latest posts
    async fn display_latest_posts(&self) -> Result<()> {
        let results = self
            .client
            .get_latest_posts()
            .await
            .context("Failed to fetch latest posts")?;

        if results.posts.is_empty() {
            println!("No latest posts found.");
            return Ok(());
        }

        println!("\n Latest Posts:");
        self.display_posts(&results.posts);
        Ok(())
    }

    /// display an individual post
    ///
    /// # Arguments
    ///
    /// * `post` - the post to display
    fn display_post(&self, post: &E6Post) {
        println!("\n{}", "=".repeat(50));
        println!("Post ID: {}", post.id);
        println!("Rating: {}", post.rating);
        println!(
            "Score: ↑{} ↓{} = {}",
            post.score.up, post.score.down, post.score.total
        );
        println!("Tags: {}", post.tags.general[0..=3].join(", "));
        println!("Favorites: {}", post.fav_count);
        println!("Uploaded by: {}", post.uploader_name);

        if !post.tags.artist.is_empty() {
            println!("Artists: {}", post.tags.artist.join(", "));
        }

        if !post.description.is_empty() {
            println!("Description: {}", format_text(&post.description));
        }

        println!("{}", "=".repeat(50));
    }
}

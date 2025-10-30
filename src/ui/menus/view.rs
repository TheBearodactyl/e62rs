use {
    crate::{
        display::{dtext::format_text, image::*},
        models::E6Post,
        ui::E6Ui,
    },
    color_eyre::{Result, eyre::Context},
};

pub async fn print_post_to_terminal(post: E6Post) -> Result<()> {
    let post_url = post.file.url.unwrap();

    fetch_and_display_image_as_sixel(&post_url)
        .await
        .context("Failed to view image")
}

impl E6Ui {
    pub fn open_in_browser(&self, post: &E6Post) -> Result<()> {
        let url = format!("https://e621.net/posts/{}", post.id);
        open::that(&url).context("Failed to open post in browser")?;
        println!("Opened post in browser: {}", url);
        Ok(())
    }

    pub fn open_posts_in_browser(&self, posts: &[E6Post]) -> Result<()> {
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

    pub fn display_posts_row(&self, posts: &[E6Post], column_width: usize) {
        self.print_row_separator(posts.len(), column_width, "┌", "┬", "┐", "─");
        self.print_posts_field(posts, column_width, |post| format!("ID: {}", post.id));
        self.print_row_separator(posts.len(), column_width, "├", "┼", "┤", "─");
        self.print_posts_field(posts, column_width, |post| {
            format!("Rating: {} | Score: {}", post.rating, post.score.total)
        });
        self.print_posts_field(posts, column_width, |post| {
            let uploader = self.truncate_string(&post.uploader_name, 15);
            format!("❤️ {} | By: {}", post.fav_count, uploader)
        });
        self.print_posts_field(posts, column_width, |post| {
            if !post.tags.artist.is_empty() {
                let artists = post.tags.artist.join(", ");
                format!("🎨 {}", self.truncate_string(&artists, column_width - 4))
            } else {
                "🎨 Unknown artist".to_string()
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
            format!("🏷️ {}", self.truncate_string(&tags, column_width - 4))
        });

        self.print_row_separator(posts.len(), column_width, "└", "┴", "┘", "─");
    }

    pub fn print_posts_field<F>(&self, posts: &[E6Post], column_width: usize, field_fn: F)
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

    pub fn print_row_separator(
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

    pub fn truncate_string(&self, s: &str, max_width: usize) -> String {
        if s.len() <= max_width {
            format!("{:width$}", s, width = max_width)
        } else {
            format!("{}...", &s[..max_width.saturating_sub(3)])
        }
    }

    pub fn display_posts(&self, posts: &[E6Post]) {
        let posts_per_row = 3;
        let column_width = 28;

        for chunk in posts.chunks(posts_per_row) {
            self.display_posts_row(chunk, column_width);
            println!();
        }
    }

    pub async fn display_latest_posts(&self) -> Result<()> {
        let results = self
            .client
            .get_latest_posts()
            .await
            .context("Failed to fetch latest posts")?;

        if results.posts.is_empty() {
            println!("No latest posts found.");
            return Ok(());
        }

        println!("\n📋 Latest Posts:");
        self.display_posts(&results.posts);
        Ok(())
    }

    pub fn display_post(&self, post: &E6Post) {
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

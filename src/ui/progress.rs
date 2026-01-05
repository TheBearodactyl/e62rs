//! progress bar management stuff
use {
    crate::getopt,
    color_eyre::eyre::Result,
    hashbrown::HashMap,
    indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressState, ProgressStyle},
    std::{sync::Arc, time::Duration},
    tokio::sync::RwLock,
};

#[derive(Default, Debug)]
/// the progress bar manager
pub struct ProgressManager {
    /// the bar group
    multi: MultiProgress,
    /// the bar(s)
    bars: Arc<RwLock<HashMap<String, ProgressBar>>>,
}

impl ProgressManager {
    /// make a new progress bar manager
    pub fn new() -> Self {
        let refresh_rate = getopt!(ui.progress_refresh_rate);
        let multi = MultiProgress::new();

        multi.set_draw_target(ProgressDrawTarget::stderr_with_hz(
            refresh_rate.clamp(5, 240) as u8,
        ));

        Self {
            multi,
            bars: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// make a progress bar for a download
    ///
    /// # Arguments
    ///
    /// * `key` - a key for the bar to be able to access it later
    /// * `len` - the length of the bar
    /// * `msg` - a message to show to the right of the bar
    pub async fn mk_dl_bar(&self, key: &str, len: u64, msg: &str) -> Result<ProgressBar> {
        let size_fmt = getopt!(ui.progress_format);
        let detailed = getopt!(ui.detailed_progress);
        let template = if detailed {
            "{spinner:.bright_cyan} [{elapsed_precise}] [{wide_bar:.bright_cyan/blue}] \
             {pos_size:>10}/{len_size} ({percent}%) {msg}"
        } else {
            "{spinner:.bright_cyan} [{wide_bar:.bright_cyan/blue}] {pos_size:>10}/{len_size} {msg}"
        };

        let style = ProgressStyle::with_template(template)?
            .with_key(
                "eta",
                |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                    write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap_or(())
                },
            )
            .with_key(
                "pos_size",
                move |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                    write!(w, "{}", size_fmt.format_size(state.pos()).trim()).unwrap_or(())
                },
            )
            .with_key(
                "len_size",
                move |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                    write!(
                        w,
                        "{}",
                        size_fmt.format_size(state.len().unwrap_or(0)).trim()
                    )
                    .unwrap_or(())
                },
            )
            .progress_chars("━╸─");

        let pb = self.multi.add(ProgressBar::new(len));

        pb.set_style(style);
        pb.set_message(msg.to_string());
        pb.enable_steady_tick(Duration::from_millis(10));

        let mut bars = self.bars.write().await;
        bars.insert(key.to_string(), pb.clone());

        Ok(pb)
    }

    /// make a new progress bar for a countdown
    ///
    /// # Arguments
    ///
    /// * `key` - a key for the bar to be able to access it later
    /// * `len` - the length of the bar
    /// * `msg` - a message to show to the right of the bar
    pub async fn create_count_bar(
        &self,
        key: &str,
        len: u64,
        message: &str,
    ) -> Result<ProgressBar> {
        let detailed = getopt!(ui.detailed_progress);
        let template = if detailed {
            "{spinner:.bright_cyan} [{elapsed_precise}] [{wide_bar:.bright_cyan/blue}] {pos}/{len} \
             ({percent}%) {msg}"
        } else {
            "{spinner:.bright_cyan} [{wide_bar:.bright_cyan/blue}] {pos}/{len} {msg}"
        };

        let style = ProgressStyle::with_template(template)?
            .with_key(
                "eta",
                |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                    write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap_or(())
                },
            )
            .progress_chars("━╸─");

        let pb = self.multi.add(ProgressBar::new(len));

        pb.set_style(style);
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(10));

        let mut bars = self.bars.write().await;

        bars.insert(key.to_string(), pb.clone());

        Ok(pb)
    }

    /// make a new bar
    ///
    /// # Arguments
    ///
    /// * `key` - a key for the bar to be able to access it later
    /// * `len` - the length of the bar
    /// * `msg` - a message to show to the right of the bar
    pub async fn create_bar(&self, key: &str, len: u64, message: &str) -> Result<ProgressBar> {
        self.create_count_bar(key, len, message).await
    }

    /// get one of the bars in the multi
    ///
    /// # Arguments
    ///
    /// * `key` - the key of the bar to search for
    pub async fn get_bar(&self, key: &str) -> Result<Option<ProgressBar>> {
        let bars = self.bars.read().await;
        Ok(bars.get(key).cloned())
    }

    /// remove a bar from the multi
    ///
    /// # Arguments
    ///
    /// * `key` - the key of the bar to remove
    pub async fn remove_bar(&self, key: &str) {
        let mut bars = self.bars.write().await;

        if let Some(pb) = bars.remove(key) {
            pb.finish_and_clear();
        }
    }

    /// make a spinner progress bar
    ///
    /// # Arguments
    ///
    /// * `message` - a message to go along with the spinner
    pub fn create_spinner(&self, message: &str) -> ProgressBar {
        let style = ProgressStyle::with_template("{spinner:.bright_cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]);

        let pb = self.multi.add(ProgressBar::new_spinner());
        pb.set_style(style);
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));

        pb
    }

    /// finish all active progress bars
    pub async fn finish_all(&self) {
        let bars = self.bars.read().await;
        for (_, pb) in bars.iter() {
            pb.finish_and_clear();
        }
    }
}

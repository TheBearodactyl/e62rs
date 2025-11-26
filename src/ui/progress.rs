use {
    crate::config::options::E62Rs,
    color_eyre::eyre::Result,
    indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressState, ProgressStyle},
    std::{collections::HashMap, fmt::Write, sync::Arc, time::Duration},
    tokio::sync::RwLock,
};

#[derive(Default, Debug)]
pub struct ProgressManager {
    multi: MultiProgress,
    bars: Arc<RwLock<HashMap<String, ProgressBar>>>,
}

impl ProgressManager {
    pub fn new() -> Self {
        let cfg = E62Rs::get_unsafe();
        let refresh_rate = cfg.ui.progress_refresh_rate;
        let multi = MultiProgress::new();

        multi.set_draw_target(ProgressDrawTarget::stderr_with_hz(
            refresh_rate.clamp(5, 60) as u8,
        ));

        Self {
            multi,
            bars: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_download_bar(
        &self,
        key: &str,
        len: u64,
        message: &str,
    ) -> Result<ProgressBar> {
        let cfg = E62Rs::get()?;
        let size_format = cfg.progress_format;
        let detailed = cfg.ui.detailed_progress;

        let template = if detailed {
            "{spinner:.bright_cyan} [{elapsed_precise}] [{wide_bar:.bright_cyan/blue}] \
             {pos_size:>10}/{len_size} ({percent}%) {msg}"
        } else {
            "{spinner:.bright_cyan} [{wide_bar:.bright_cyan/blue}] {pos_size:>10}/{len_size} {msg}"
        };

        let style = ProgressStyle::with_template(template)?
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
                write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap_or(())
            })
            .with_key(
                "pos_size",
                move |state: &ProgressState, w: &mut dyn Write| {
                    write!(w, "{}", size_format.format_size(state.pos()).trim()).unwrap_or(())
                },
            )
            .with_key(
                "len_size",
                move |state: &ProgressState, w: &mut dyn Write| {
                    write!(
                        w,
                        "{}",
                        size_format.format_size(state.len().unwrap_or(0)).trim()
                    )
                    .unwrap_or(())
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

    pub async fn create_count_bar(
        &self,
        key: &str,
        len: u64,
        message: &str,
    ) -> Result<ProgressBar> {
        let cfg = E62Rs::get()?;
        let detailed = cfg.ui.detailed_progress;

        let template = if detailed {
            "{spinner:.bright_cyan} [{elapsed_precise}] [{wide_bar:.bright_cyan/blue}] {pos}/{len} \
             ({percent}%) {msg}"
        } else {
            "{spinner:.bright_cyan} [{wide_bar:.bright_cyan/blue}] {pos}/{len} {msg}"
        };

        let style = ProgressStyle::with_template(template)?
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
                write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap_or(())
            })
            .progress_chars("━╸─");

        let pb = self.multi.add(ProgressBar::new(len));
        pb.set_style(style);
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(10));

        let mut bars = self.bars.write().await;
        bars.insert(key.to_string(), pb.clone());

        Ok(pb)
    }

    pub async fn create_bar(&self, key: &str, len: u64, message: &str) -> Result<ProgressBar> {
        self.create_count_bar(key, len, message).await
    }

    pub async fn get_bar(&self, key: &str) -> Result<Option<ProgressBar>> {
        let bars = self.bars.read().await;
        Ok(bars.get(key).cloned())
    }

    pub async fn remove_bar(&self, key: &str) {
        let mut bars = self.bars.write().await;

        if let Some(pb) = bars.remove(key) {
            pb.finish_and_clear();
        }
    }

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

    pub async fn finish_all(&self) {
        let bars = self.bars.read().await;
        for (_, pb) in bars.iter() {
            pb.finish_and_clear();
        }
    }
}

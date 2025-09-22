use anyhow::Result;
use e6cfg::Cfg;
use indicatif::{MultiProgress, ProgressBar, ProgressState, ProgressStyle};
use std::{collections::HashMap, fmt::Write, sync::Arc};
use tokio::sync::RwLock;

#[derive(Default, Debug)]
pub struct ProgressManager {
    multi: MultiProgress,
    bars: Arc<RwLock<HashMap<String, ProgressBar>>>,
}

impl ProgressManager {
    pub fn new() -> Self {
        let config = Cfg::get().unwrap_or_default();
        let refresh_rate = config
            .ui
            .as_ref()
            .and_then(|ui| ui.progress_refresh_rate)
            .unwrap_or(20);

        let multi = MultiProgress::new();

        multi.set_draw_target(indicatif::ProgressDrawTarget::stderr_with_hz(
            refresh_rate as u8,
        ));

        Self {
            multi,
            bars: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_bar(&self, key: &str, len: u64, message: &str) -> Result<ProgressBar> {
        let config = Cfg::get().unwrap_or_default();
        let detailed = config
            .ui
            .as_ref()
            .and_then(|ui| ui.detailed_progress)
            .unwrap_or(true);

        let template = if detailed {
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg} ({eta_precise})"
        } else {
            "{bar:40.cyan/blue} {pos:>7}/{len:7} {msg}"
        };

        let style = ProgressStyle::with_template(template)
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
                write!(w, "{:.1}s", state.eta().as_secs_f64()).expect("Failed");
            })
            .progress_chars("##-");

        let pb = self.multi.add(ProgressBar::new(len));
        pb.set_style(style);
        pb.set_message(message.to_string());

        let mut bars = self.bars.write().await;
        bars.insert(key.to_string(), pb.clone());

        Ok(pb)
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
        let style = ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

        let pb = self.multi.add(ProgressBar::new_spinner());
        pb.set_style(style);
        pb.set_message(message.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(120));

        pb
    }

    pub async fn finish_all(&self) {
        let bars = self.bars.read().await;
        for (_, pb) in bars.iter() {
            pb.finish_and_clear();
        }
    }
}

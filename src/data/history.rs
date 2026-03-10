//! opt-in search history
use {
    color_eyre::Result,
    std::{
        fs,
        path::PathBuf,
    },
};

const MAX_ENTRIES: usize = 100;

/// persisted search history
#[derive(Clone, Debug)]
pub struct SearchHistory {
    /// the entries (most recent first)
    entries: Vec<String>,
    /// the path to the history file
    path: PathBuf,
}

impl Default for SearchHistory {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            path: dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("e62rs_history.txt"),
        }
    }
}

impl SearchHistory {
    /// load history from the config dir
    pub fn load() -> Result<Self> {
        let path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("e62rs_history.txt");

        let entries = if path.exists() {
            fs::read_to_string(&path)
                .unwrap_or_default()
                .lines()
                .filter(|l| !l.is_empty())
                .map(String::from)
                .collect()
        } else {
            Vec::new()
        };

        Ok(Self { entries, path })
    }

    /// add a query to history (deduped, most recent first)
    pub fn add(&mut self, query: &str) {
        let query = query.trim().to_string();
        if query.is_empty() {
            return;
        }

        self.entries.retain(|e| e != &query);
        self.entries.insert(0, query);
        self.entries.truncate(MAX_ENTRIES);
    }

    /// get suggestions matching a prefix
    pub fn suggestions(&self, prefix: &str) -> Vec<String> {
        if prefix.is_empty() {
            return self.entries.iter().take(10).cloned().collect();
        }

        let lower = prefix.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.to_lowercase().contains(&lower))
            .take(10)
            .cloned()
            .collect()
    }

    /// save history to disk
    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, self.entries.join("\n"))?;
        Ok(())
    }
}

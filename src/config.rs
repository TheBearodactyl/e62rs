use config::Config;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Cfg {
    /// The directory to download posts to
    pub download_dir: String,

    /// The amount of posts to show in a search
    pub post_count: u64,

    /// The base URL of the API (defaults to https://e621.net)
    pub base_url: String
}

pub fn get_config() -> anyhow::Result<Cfg> {
    let settings = Config::builder()
        .add_source(config::File::with_name("e62rs"))
        .add_source(config::Environment::with_prefix("E62RS"))
        .build()?;

    settings
        .try_deserialize::<Cfg>()
        .map_err(anyhow::Error::new)
}

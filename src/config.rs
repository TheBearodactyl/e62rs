use config::Config;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ImageDisplay {
    /// The max width of displayed images
    pub width: Option<u64>,
    /// The max height of displayed images
    pub height: Option<u64>,
    /// Whether or not to display the image of a post when displaying its info
    pub image_when_info: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Cfg {
    /// The directory to download posts to
    pub download_dir: Option<String>,

    /// The output format for downloaded files
    pub output_format: Option<String>,

    /// The amount of posts to show in a search
    pub post_count: Option<u64>,

    /// The base URL of the API (defaults to https://e621.net)
    pub base_url: Option<String>,

    /// Post viewing settings
    pub display: Option<ImageDisplay>,

    /// The path to `tags.csv` that's used for tag
    /// searching/autocompletion
    pub tags: Option<String>,
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

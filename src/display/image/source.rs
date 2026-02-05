//! image loading stuff
use {
    color_eyre::eyre::{Context, Result, bail},
    image::{DynamicImage, ImageReader},
    macroni_n_cheese::Construct,
    std::{io::Cursor, path::Path},
};

/// raw rgba888 image data with dimensions
#[derive(Debug, Clone, Construct)]
pub struct ImageData {
    /// raw rgba888 pixel data
    pub rgb_data: Vec<u8>,
    /// image width (px)
    pub width: usize,
    /// image height (px)
    pub height: usize,
}

impl ImageData {
    /// convert `DynamicImage` to `ImageData`
    pub fn from_dynamic_image(img: DynamicImage) -> Self {
        let rgba8 = img.to_rgba8();
        let (width, height) = rgba8.dimensions();
        let rgb_data = rgba8.into_raw();

        Self {
            rgb_data,
            width: width as usize,
            height: height as usize,
        }
    }
}

/// source of image data
pub enum ImageSource {
    /// image data as bytes
    Bytes(Vec<u8>),
    /// image from a file path
    Path(std::path::PathBuf),
    /// preloaded image
    Image(DynamicImage),
}

impl ImageSource {
    /// load an image from a url
    pub async fn from_url(url: &str) -> Result<Self> {
        let bytes = fetch_remote_file(url).await?;
        Ok(Self::Bytes(bytes))
    }

    /// load an image from a path
    pub fn from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            bail!("File does not exist: {}", path.display());
        }
        Ok(Self::Path(path.to_path_buf()))
    }

    /// create a source from raw bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self::Bytes(bytes)
    }

    /// create a source from an already-loaded image
    pub fn from_image(image: DynamicImage) -> Self {
        Self::Image(image)
    }

    /// load the image into a `DynamicImage`
    pub fn load(self) -> Result<DynamicImage> {
        match self {
            Self::Bytes(bytes) => {
                let cursor = Cursor::new(bytes);
                ImageReader::new(cursor)
                    .with_guessed_format()
                    .context("Failed to guess image format")?
                    .decode()
                    .context("Failed to decode image")
            }
            Self::Path(path) => ImageReader::open(&path)
                .with_context(|| format!("Failed to open {}", path.display()))?
                .decode()
                .with_context(|| format!("Failed to decode {}", path.display())),
            Self::Image(img) => Ok(img),
        }
    }
}

/// fetch a file from a remote url
pub async fn fetch_remote_file(url: &str) -> Result<Vec<u8>> {
    let response = reqwest::get(url)
        .await
        .with_context(|| format!("Failed to fetch {}", url))?;

    if !response.status().is_success() {
        bail!(
            "HTTP request failed with status: {} for URL: {}",
            response.status(),
            url
        );
    }

    response
        .bytes()
        .await
        .context("Failed to read response body")
        .map(|bytes| bytes.to_vec())
}

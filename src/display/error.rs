//! display errors
use thiserror::Error;

/// an image related error
#[derive(Error, Debug)]
pub enum ImageError {
    /// error fetching image
    #[error("Failed to fetch image from {url}: {source}")]
    FetchError {
        /// the url of the image
        url: String,
        /// the error itself
        #[source]
        source: reqwest::Error,
    },
    /// error decoding image
    #[error("Failed to decode image: {0}")]
    DecodeError(#[from] image::ImageError),
    /// error encoding to sixel
    #[error("Failed to encode sixel: {0}")]
    SixelEncodeError(String),
    /// invalid dimensions
    #[error("Invalid dimensions: {0}")]
    InvalidDimensions(String),
    /// io error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// error parsing dtext
#[derive(Error, Debug)]
pub enum DTextError {
    /// invalid color
    #[error("Invalid color format: {0}")]
    InvalidColor(String),
    /// unclosed tag
    #[error("Unclosed tag: {0}")]
    UnclosedTag(String),
}

/// woah
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

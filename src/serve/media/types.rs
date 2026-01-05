//! types for the media manager
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
/// a type of media
pub enum MediaType {
    /// an image
    Image,
    /// a video
    Video,
}

impl MediaType {
    /// infer the type of media from its extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "bmp" | "ico" => Some(Self::Image),
            "mp4" | "webm" | "mov" | "avi" | "mkv" => Some(Self::Video),
            _ => None,
        }
    }

    /// convert a MediaType to a string
    pub fn as_str(&self) -> &str {
        match self {
            Self::Image => "image",
            Self::Video => "video",
        }
    }
}

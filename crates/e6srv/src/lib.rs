#![allow(unused)]

mod cfg;
mod media;
mod routes;
mod server;
mod theme;

pub use cfg::{ServerConfig, ServerConfigBuilder};
pub use media::{MediaGallery, MediaItem, MediaScanner, MediaType};
pub use server::MediaServer;
pub use theme::*;

pub mod prelude {
    pub use crate::cfg::{ServerConfig, ServerConfigBuilder};
    pub use crate::media::{MediaGallery, MediaItem, MediaScanner, MediaType};
    pub use crate::server::MediaServer;
    pub use crate::theme::*;
}

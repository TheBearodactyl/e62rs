pub static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub mod client;
pub mod data;
pub mod display;
pub mod formatting;
pub mod image;
pub mod macros;
pub mod models;
pub mod utils;

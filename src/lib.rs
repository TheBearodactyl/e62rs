//! e62rs is a CLI e621/926 client
#![forbid(
    clippy::missing_docs_in_private_items,
    missing_docs,
    rustdoc::missing_crate_level_docs
)]

pub mod app;
pub mod cache;
pub mod client;
pub mod config;
pub mod data;
pub mod display;
pub mod models;
pub mod serve;
pub mod ui;
pub mod utils;

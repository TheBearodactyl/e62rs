//! e62rs is a CLI e621/926 client.
#![deny(
    clippy::perf,
    clippy::clone_on_copy,
    clippy::missing_docs_in_private_items,
    clippy::empty_docs,
    //clippy::missing_errors_doc,
    clippy::missing_safety_doc,
    //clippy::missing_panics_doc
)]
pub mod app;
pub mod cache;
pub mod client;
pub mod config;
pub mod data;
pub mod display;
pub mod macros;
pub mod models;
pub mod serve;
pub mod ui;
pub mod utils;

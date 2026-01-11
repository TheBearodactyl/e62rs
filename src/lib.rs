//! e62rs is a CLI e621/926 client
//!
//! features include:  
//! * 100+ options with sane defaults (see [`crate::config::options`] for all sections and fields)
//! * fully offline post explorer
//! * really fast post and pool downloader with batch support
//! * highly customizable downloads organization
//! * in-terminal image viewer with support for animations
//! * **__everything__** is documented
#![allow(uncommon_codepoints, confusable_idents)]
#![deny(
    clippy::perf,
    clippy::clone_on_copy,
    clippy::missing_docs_in_private_items,
    clippy::empty_docs,
    clippy::missing_safety_doc,
    clippy::style,
    unused,
    missing_docs
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

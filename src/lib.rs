//! e62rs is a CLI e621/926 client.
#![deny(
    clippy::perf,
    clippy::clone_on_copy,
    clippy::missing_docs_in_private_items,
    clippy::empty_docs,
    clippy::missing_safety_doc,
    unused
)]
#![allow(uncommon_codepoints, confusable_idents)]
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

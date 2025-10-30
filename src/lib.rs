#![allow(unused)]

pub static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub mod app;
pub mod cache;
pub mod cli;
pub mod client;
pub mod config;
pub mod data;
pub mod display;
pub mod models;
pub mod serve;
pub mod ui;
pub mod utils;

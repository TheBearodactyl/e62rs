#![allow(unused)]

use crate::{cli::Cli, client::E6Client, tag_db::TagDatabase, ui::E6Ui};
use anyhow::{Context, Result};
use clap::Parser;
use env_logger::{Builder, Env};
use log::{error, info};
use std::{process, sync::Arc, time::Instant};

pub static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

mod batch;
mod cli;
mod client;
mod config;
mod error;
mod formatting;
mod models;
mod progress;
mod tag_db;
mod ui;

#[derive(inquiry::Choice, PartialEq, PartialOrd, Debug, Clone, Copy)]
enum MainMenu {
    /// Search for posts
    Search,
    /// View the latest posts
    ViewLatest,
}

#[tokio::main]
async fn main() -> Result<()> {
    Builder::from_env(Env::default().default_filter_or("info")).init();

    if let Err(e) = run().await {
        error!("Application error: {:#}", e);
        process::exit(1);
    }

    info!("Application finished successfully");
    Ok(())
}

async fn run() -> Result<()> {
    let argv = Cli::parse();
    let start = Instant::now();
    let tag_db = Arc::new(
        TagDatabase::load()
            .context("Failed to load tag database. Please ensure data/tags.csv exists")?,
    );
    let load_time = start.elapsed().as_millis();

    println!(
        "Loaded {} tags in {} milliseconds",
        tag_db.tags.len(),
        load_time
    );

    let client = if argv.e926 {
        info!(
            "Starting {} v{} using e926",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );
        Arc::new(E6Client::new("https://e926.net")?)
    } else {
        info!(
            "Starting {} v{} using e621",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );
        Arc::new(E6Client::default())
    };

    let selection = MainMenu::choice("What would you like to do?")?;
    let ui = E6Ui::new(client, tag_db);

    match selection {
        MainMenu::Search => ui.search().await,
        MainMenu::ViewLatest => ui.display_latest_posts().await,
    }
}

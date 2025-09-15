use anyhow::Result;
use clap::Parser;
use log::{error, info};
use std::process;

mod batch;
mod cli;
mod client;
mod models;
mod ui;
mod formatting;

use crate::{cli::Cli, client::E6Client, ui::E6Ui};

pub static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    if let Err(e) = run().await {
        error!("Application error: {:#}", e);
        process::exit(1);
    }

    info!("Application finished successfully");
    Ok(())
}
async fn run() -> Result<()> {
    let argv = Cli::parse();
    let client = if argv.e926 {
        info!(
            "Starting {} v{} using e926",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );

        E6Client::new("https://e926.net")?
    } else {
        info!(
            "Starting {} v{} using e621",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );

        E6Client::default()
    };

    let ui = E6Ui::new(client);
    ui.search().await
}

#![allow(unused)]

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    e62rs::app::setup_logging()?;
    e62rs::app::run().await
}

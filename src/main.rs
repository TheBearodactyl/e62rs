#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let app = e62rs::app::E6App::init().await?;
    app.run().await
}

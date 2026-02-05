#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> e62rs::error::Result<()> {
    let app = e62rs::app::E6App::init().await?;
    app.run().await
}

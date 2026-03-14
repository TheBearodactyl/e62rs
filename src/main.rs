#[cfg(not(feature = "cli"))]
compile_error!("the `cli` feature is required to build the e62rs binary");

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(feature = "cli")]
#[tokio::main]
async fn main() -> e62rs::error::Result<()> {
    let app = e62rs::app::E6App::init().await?;
    app.run().await
}

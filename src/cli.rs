use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    /// Use e926 instead of e621
    #[arg(short, long)]
    pub e926: bool,

    /// Where to download posts to
    #[arg(short, long)]
    pub output: Option<String>
}

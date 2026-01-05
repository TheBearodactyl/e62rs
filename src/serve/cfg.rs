//! server configuration stuff
use {
    color_eyre::eyre::Result,
    std::{net::SocketAddr, path::PathBuf},
};

#[derive(Debug, Clone)]
/// configuration for the server
pub struct ServerConfig {
    /// the path to the downloaded media
    pub media_directory: PathBuf,
    /// the address to bind to
    pub bind_address: SocketAddr,
    /// the max file size to index
    pub max_file_size: Option<u64>,
    /// whether to enable filtering by metadata
    pub enable_metadata_filtering: bool,
    /// whether to cache metadata
    pub cache_metadata: bool,
    /// the number of threads to use when loading
    pub num_threads: usize,
}

impl ServerConfig {
    /// make a builder
    pub fn builder() -> ServerConfigBuilder {
        ServerConfigBuilder::default()
    }

    /// get the current set media directory
    pub fn media_directory(&self) -> &PathBuf {
        &self.media_directory
    }

    /// get the current bind addr
    pub fn bind_address(&self) -> &SocketAddr {
        &self.bind_address
    }
}

#[derive(Default)]
/// a builder for the server configuration
pub struct ServerConfigBuilder {
    /// the path to the downloaded media
    media_directory: Option<PathBuf>,
    /// the address to bind to
    bind_address: Option<SocketAddr>,
    /// the max file size to index
    max_file_size: Option<u64>,
    /// whether to enable filtering by metadata
    enable_metadata_filtering: bool,
    /// whether to cache metadata
    cache_metadata: bool,
    /// the number of threads to use when loading
    num_threads: usize,
}

impl ServerConfigBuilder {
    /// make a new ServerConfigBuilder
    pub fn new() -> Self {
        Self::default()
    }

    /// set the media directory
    pub fn media_directory(mut self, dir: PathBuf) -> Self {
        self.media_directory = Some(dir);
        self
    }

    /// set the bind address
    pub fn bind_address(mut self, addr: SocketAddr) -> Self {
        self.bind_address = Some(addr);
        self
    }

    /// set the max file size
    pub fn max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = Some(size);
        self
    }

    /// set whether to enable metadata filtering
    pub fn enable_metadata_filtering(mut self, enabled: bool) -> Self {
        self.enable_metadata_filtering = enabled;
        self
    }

    /// set the metadata caching mode
    pub fn cache_metadata(mut self, enabled: bool) -> Self {
        self.cache_metadata = enabled;
        self
    }

    /// set the number of load threads
    pub fn num_threads(mut self, threads: usize) -> Self {
        self.num_threads = threads;
        self
    }

    /// build the ServerConfigBuilder into a ServerConfig
    pub fn build(self) -> Result<ServerConfig, String> {
        let media_directory = self
            .media_directory
            .ok_or_else(|| "Media directory is required".to_string())?;

        if !media_directory.exists() {
            return Err(format!(
                "Media directory does not exist: {}",
                media_directory.display()
            ));
        }

        let bind_address = self
            .bind_address
            .unwrap_or_else(|| "127.0.0.1:23794".parse().unwrap());

        Ok(ServerConfig {
            media_directory,
            bind_address,
            max_file_size: self.max_file_size,
            enable_metadata_filtering: self.enable_metadata_filtering,
            cache_metadata: self.cache_metadata,
            num_threads: self.num_threads,
        })
    }
}

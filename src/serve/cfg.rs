use {
    color_eyre::eyre::Result,
    std::{net::SocketAddr, path::PathBuf},
};

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub media_directory: PathBuf,
    pub bind_address: SocketAddr,
    pub max_file_size: Option<u64>,
    pub enable_metadata_filtering: bool,
    pub cache_metadata: bool,
    pub num_threads: usize,
}

impl ServerConfig {
    pub fn builder() -> ServerConfigBuilder {
        ServerConfigBuilder::default()
    }

    pub fn media_directory(&self) -> &PathBuf {
        &self.media_directory
    }

    pub fn bind_address(&self) -> &SocketAddr {
        &self.bind_address
    }
}

#[derive(Default)]
pub struct ServerConfigBuilder {
    media_directory: Option<PathBuf>,
    bind_address: Option<SocketAddr>,
    max_file_size: Option<u64>,
    enable_metadata_filtering: bool,
    cache_metadata: bool,
    num_threads: usize,
}

impl ServerConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn media_directory(mut self, dir: PathBuf) -> Self {
        self.media_directory = Some(dir);
        self
    }

    pub fn bind_address(mut self, addr: SocketAddr) -> Self {
        self.bind_address = Some(addr);
        self
    }

    pub fn max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = Some(size);
        self
    }

    pub fn enable_metadata_filtering(mut self, enabled: bool) -> Self {
        self.enable_metadata_filtering = enabled;
        self
    }

    pub fn cache_metadata(mut self, enabled: bool) -> Self {
        self.cache_metadata = enabled;
        self
    }

    pub fn num_threads(mut self, threads: usize) -> Self {
        self.num_threads = threads;
        self
    }

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

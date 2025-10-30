use {
    crate::config::messages::config::LogMessageConfig,
    color_eyre::eyre::{Context, Result},
    config::{Config, Environment, File},
    std::path::Path,
};

pub struct MessagesConfigBuilder {
    builder: config::ConfigBuilder<config::builder::DefaultState>,
}

impl MessagesConfigBuilder {
    pub fn new() -> Self {
        Self {
            builder: Config::builder(),
        }
    }

    pub fn add_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.builder = self.builder.add_source(File::from(path.as_ref()));
        self
    }

    pub fn add_env_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.builder = self
            .builder
            .add_source(Environment::with_prefix(&prefix.into()));

        self
    }

    pub fn build(self) -> Result<LogMessageConfig> {
        let cfg = self
            .builder
            .build()
            .wrap_err("Failed to build configuration")?;

        let log_config: LogMessageConfig = cfg
            .try_deserialize()
            .wrap_err("Failed to deserialize configuration")?;

        log_config.validate()?;

        Ok(log_config)
    }
}

impl Default for MessagesConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

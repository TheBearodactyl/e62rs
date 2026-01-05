//! config singleton management stuff
use {
    crate::config::options::E62Rs,
    color_eyre::{Result, eyre::Context},
    std::sync::{LazyLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

/// global config instance
static CONFIG: LazyLock<RwLock<E62Rs>> = LazyLock::new(|| {
    RwLock::new(E62Rs::load().expect("!!!Failed to load configuration, this should NOT happen!!!"))
});

/// init the config explicitly
pub fn init_config() -> Result<()> {
    let _l = config()?;
    Ok(())
}

/// get a ro ref to the config
pub fn config() -> Result<RwLockReadGuard<'static, E62Rs>> {
    CONFIG
        .read()
        .map_err(|e| color_eyre::eyre::eyre!("Configuration lock poisoned: {}", e))
}

/// get a rw ref to the config
pub fn config_mut() -> Result<RwLockWriteGuard<'static, E62Rs>> {
    CONFIG
        .write()
        .map_err(|e| color_eyre::eyre::eyre!("Configuration lock poisoned: {}", e))
}

/// reload cfg from disk
pub fn reload_config() -> Result<()> {
    let new_config = E62Rs::load().wrap_err("Failed to reload config from disk")?;
    let mut config = config_mut().wrap_err("failed to acquire write lock for cfg reload")?;

    *config = new_config;

    Ok(())
}

/// get a specific config value with a default fallback
pub fn get_or_default<T, F>(getter: F, default: T) -> T
where
    F: FnOnce(&E62Rs) -> Option<T>,
    T: Clone,
{
    config()
        .ok()
        .and_then(|cfg| getter(&cfg))
        .unwrap_or(default)
}

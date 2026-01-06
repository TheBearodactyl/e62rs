//! blacklist management stuff
use {
    crate::config::instance::{config, *},
    color_eyre::{Result, eyre::Context},
};

/// add a tag to the blacklist
pub fn add_to_blacklist(tag: String) -> Result<()> {
    let mut cfg = config_mut().wrap_err("failed to get write lock for config")?;

    if let Some(ref mut search_cfg) = cfg.search
        && let Some(blacklist) = search_cfg.blacklist.as_mut()
        && !blacklist.contains(&tag)
    {
        blacklist.push(tag);
        blacklist.sort();

        cfg.save().wrap_err("failed to save config")?;
    }
    Ok(())
}

/// remove a tag from the blacklist
pub fn remove_from_blacklist(tag: &str) -> Result<bool> {
    let mut cfg = config_mut().wrap_err("failed to get write lock for config")?;

    if let Some(ref mut search_cfg) = cfg.search
        && let Some(blacklist) = search_cfg.blacklist.as_mut()
        && let Some(pos) = blacklist.iter().position(|x| x == tag)
    {
        blacklist.remove(pos);
        cfg.save().wrap_err("failed to save config")?;

        return Ok(true);
    }

    Ok(false)
}

/// clear the blacklist
pub fn clear_blacklist() -> Result<()> {
    let mut cfg = config_mut().wrap_err("failed to get write lock for config")?;

    if let Some(ref mut search_cfg) = cfg.search
        && let Some(blacklist) = search_cfg.blacklist.as_mut()
    {
        blacklist.clear();
        cfg.save().wrap_err("failed to save config")?;
    }

    Ok(())
}

/// get a copy of the current blacklist
pub fn get_blacklist() -> Result<Vec<String>> {
    let cfg = config().wrap_err("failed to get read lock for config")?;
    if let Some(ref search_cfg) = cfg.search
        && let Some(blacklist) = search_cfg.blacklist.clone()
    {
        return Ok(blacklist);
    }

    Ok(Vec::new())
}

/// check if a tag is blacklisted
pub fn is_blacklisted(tag: &str) -> bool {
    get_blacklist()
        .ok()
        .map(|b| b.iter().any(|t| t == tag))
        .unwrap_or(false)
}

//! Configuration management for Vibe Downloader

mod settings;

pub use settings::*;

use anyhow::Result;
use std::fs;
use std::path::PathBuf;

/// Get the configuration directory path
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("vibe-downloader")
}

/// Get the configuration file path
pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

/// Load configuration from file or create default
pub fn load_or_create_default() -> Result<Settings> {
    let path = config_path();
    
    if path.exists() {
        let content = fs::read_to_string(&path)?;
        let settings: Settings = toml::from_str(&content)?;
        Ok(settings)
    } else {
        let settings = Settings::default();
        save(&settings)?;
        Ok(settings)
    }
}

/// Save configuration to file
pub fn save(settings: &Settings) -> Result<()> {
    let path = config_path();
    
    // Ensure config directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let content = toml::to_string_pretty(settings)?;
    fs::write(&path, content)?;
    
    Ok(())
}

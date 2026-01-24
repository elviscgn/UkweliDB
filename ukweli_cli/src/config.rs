use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub db_path: PathBuf,
}

impl Config {
    /// Get (~/.ukweli)
    pub fn ukweli_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        Ok(home.join(".ukweli"))
    }

    pub fn default_db_path() -> Result<PathBuf> {
        Ok(Self::ukweli_dir()?.join("default.ukweli"))
    }

    pub fn users_dir() -> Result<PathBuf> {
        Ok(Self::ukweli_dir()?.join("users"))
    }

    pub fn workflows_dir() -> Result<PathBuf> {
        Ok(Self::ukweli_dir()?.join("workflows"))
    }

    pub fn config_file() -> Result<PathBuf> {
        Ok(Self::ukweli_dir()?.join("config.json"))
    }

    pub fn load_or_default() -> Result<Self> {
        let config_path = Self::config_file()?;

        if config_path.exists() {
            let content =
                std::fs::read_to_string(&config_path).context("Failed to read config file")?;
            let config: Config =
                serde_json::from_str(&content).context("Failed to parse config file")?;
            Ok(config)
        } else {
            Ok(Config {
                db_path: Self::default_db_path()?,
            })
        }
    }

    // pub fn save(&self) -> Result<()> {
    //     let config_path = Self::config_file()?;

    //     if let Some(parent) = config_path.parent() {
    //         std::fs::create_dir_all(parent).context("Failed to create config directory")?;
    //     }

    //     let content = serde_json::to_string_pretty(self).context("Failed to serialize config")?;

    //     std::fs::write(&config_path, content).context("Failed to write config file")?;

    //     Ok(())
    // }
}

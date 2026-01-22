use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub rpc_url: Option<String>,
    #[serde(default)]
    pub recent_searches: Vec<String>,
}

impl Config {
    /// Returns the config directory path (~/.config/tbex on Linux/macOS)
    fn config_dir() -> Result<PathBuf> {
        dirs::config_dir()
            .map(|p| p.join("tbex"))
            .context("Could not determine config directory")
    }

    /// Returns the config file path
    fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    /// Load config from disk, or return default if not found
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config from {path:?}"))?;

        toml::from_str(&contents).context("Failed to parse config file")
    }

    /// Save config to disk
    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir()?;
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create config directory {dir:?}"))?;

        let path = Self::config_path()?;
        let contents = toml::to_string_pretty(self).context("Failed to serialize config")?;

        std::fs::write(&path, contents)
            .with_context(|| format!("Failed to write config to {path:?}"))?;

        Ok(())
    }

    /// Set the RPC URL and persist
    pub fn set_rpc(&mut self, url: String) -> Result<()> {
        self.rpc_url = Some(url);
        self.save()
    }

    /// Add a search to recent history (keeps last 10)
    pub fn add_recent_search(&mut self, query: String) -> Result<()> {
        // Remove if already exists to avoid duplicates
        self.recent_searches.retain(|s| s != &query);
        // Add to front
        self.recent_searches.insert(0, query);
        // Keep only last 10
        self.recent_searches.truncate(10);
        self.save()
    }
}

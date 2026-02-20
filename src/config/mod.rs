use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::scanner::ProjectKind;

/// Persistent configuration for devclean.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevCleanConfig {
    /// Directories to always ignore during scanning.
    #[serde(default)]
    pub ignore_paths: Vec<PathBuf>,

    /// Project kinds to exclude from scanning.
    #[serde(default)]
    pub exclude_kinds: Vec<ProjectKind>,

    /// Default scan roots.
    #[serde(default)]
    pub default_roots: Vec<PathBuf>,

    /// Maximum directory depth to scan.
    #[serde(default)]
    pub max_depth: Option<usize>,
}

impl Default for DevCleanConfig {
    fn default() -> Self {
        Self {
            ignore_paths: Vec::new(),
            exclude_kinds: Vec::new(),
            default_roots: Vec::new(),
            max_depth: None,
        }
    }
}

impl DevCleanConfig {
    /// Load config from the default location (~/.config/devclean/config.json).
    pub fn load() -> Self {
        let config_path = Self::config_path();
        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        }
    }

    /// Save config to the default location.
    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, json)?;
        Ok(())
    }

    /// Get the default config file path.
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("devclean")
            .join("config.json")
    }
}

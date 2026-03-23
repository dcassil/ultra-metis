//! Project configuration stored in `.cadre/config.toml`

use crate::error::{Result, StoreError};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const CONFIG_FILE: &str = "config.toml";
const PROJECT_DIR: &str = ".cadre";

/// Project configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Short code prefix (e.g., "PROJ")
    pub prefix: String,
    /// Next counter value for short code generation
    pub next_counter: u32,
}

impl ProjectConfig {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_uppercase(),
            next_counter: 1,
        }
    }

    /// Get the `.cadre/` directory path for a project root
    pub fn project_dir(project_path: &Path) -> PathBuf {
        project_path.join(PROJECT_DIR)
    }

    /// Get the config file path
    pub fn config_path(project_path: &Path) -> PathBuf {
        Self::project_dir(project_path).join(CONFIG_FILE)
    }

    /// Load config from disk
    pub fn load(project_path: &Path) -> Result<Self> {
        let config_path = Self::config_path(project_path);
        if !config_path.exists() {
            return Err(StoreError::NotInitialized {
                path: project_path.display().to_string(),
            });
        }
        let content = std::fs::read_to_string(&config_path)?;
        toml::from_str(&content).map_err(|e| StoreError::Config(e.to_string()))
    }

    /// Save config to disk
    pub fn save(&self, project_path: &Path) -> Result<()> {
        let config_path = Self::config_path(project_path);
        let content =
            toml::to_string_pretty(self).map_err(|e| StoreError::Config(e.to_string()))?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    /// Allocate the next short code for a given document type prefix letter
    pub fn next_short_code(&mut self, type_prefix: &str) -> String {
        let code = format!("{}-{}-{:04}", self.prefix, type_prefix, self.next_counter);
        self.next_counter += 1;
        code
    }
}

/// The directory name used for cadre projects
pub fn project_dir_name() -> &'static str {
    PROJECT_DIR
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_roundtrip() {
        let dir = tempdir().unwrap();
        let project_path = dir.path();

        // Create the .cadre directory
        std::fs::create_dir_all(ProjectConfig::project_dir(project_path)).unwrap();

        let mut config = ProjectConfig::new("TEST");
        assert_eq!(config.prefix, "TEST");
        assert_eq!(config.next_counter, 1);

        let code = config.next_short_code("V");
        assert_eq!(code, "TEST-V-0001");
        assert_eq!(config.next_counter, 2);

        config.save(project_path).unwrap();
        let loaded = ProjectConfig::load(project_path).unwrap();
        assert_eq!(loaded.prefix, "TEST");
        assert_eq!(loaded.next_counter, 2);
    }

    #[test]
    fn test_not_initialized() {
        let dir = tempdir().unwrap();
        let result = ProjectConfig::load(dir.path());
        assert!(result.is_err());
    }
}

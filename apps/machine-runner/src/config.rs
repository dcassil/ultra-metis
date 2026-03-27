use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::discovery;

#[derive(Debug, Deserialize)]
pub struct RunnerConfig {
    pub control_service_url: String,
    pub machine_name: String,
    pub api_token: String,
    #[serde(default = "default_heartbeat_interval")]
    pub heartbeat_interval_secs: u64,
    #[serde(default)]
    pub repo_directories: Vec<String>,
}

fn default_heartbeat_interval() -> u64 {
    20
}

/// Default config file path: `~/.config/cadre/machine-runner.toml`
fn default_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|c| c.join("cadre").join("machine-runner.toml"))
}

impl RunnerConfig {
    /// Load config from the specified path, or the default path
    /// (`~/.config/cadre/machine-runner.toml`).
    ///
    /// # Errors
    ///
    /// Returns an error if the config file cannot be read or parsed.
    pub fn load(path: Option<&Path>) -> anyhow::Result<Self> {
        let config_path = match path {
            Some(p) => p.to_path_buf(),
            None => default_config_path()
                .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?,
        };

        let contents = std::fs::read_to_string(&config_path).map_err(|e| {
            anyhow::anyhow!(
                "Failed to read config file at {}: {}",
                config_path.display(),
                e
            )
        })?;

        Self::from_toml(&contents)
    }

    /// Parse config from a TOML string.
    ///
    /// # Errors
    ///
    /// Returns an error if the TOML string cannot be parsed into a valid config.
    pub fn from_toml(toml_str: &str) -> anyhow::Result<Self> {
        let config: Self = toml::from_str(toml_str)?;
        Ok(config)
    }

    /// Get `repo_directories` as expanded `PathBuf`s, with `~` resolved to the
    /// home directory.
    #[must_use]
    pub fn repo_paths(&self) -> Vec<PathBuf> {
        self.repo_directories
            .iter()
            .map(|s| discovery::expand_home_public(Path::new(s)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_config() {
        let toml = r#"
control_service_url = "http://localhost:3000"
machine_name = "test-machine"
api_token = "test-token"
heartbeat_interval_secs = 30
repo_directories = ["~/projects", "/opt/repos"]
"#;

        let config = RunnerConfig::from_toml(toml).unwrap();
        assert_eq!(config.control_service_url, "http://localhost:3000");
        assert_eq!(config.machine_name, "test-machine");
        assert_eq!(config.api_token, "test-token");
        assert_eq!(config.heartbeat_interval_secs, 30);
        assert_eq!(config.repo_directories.len(), 2);
        assert_eq!(config.repo_directories[0], "~/projects");
        assert_eq!(config.repo_directories[1], "/opt/repos");
    }

    #[test]
    fn uses_default_heartbeat_interval() {
        let toml = r#"
control_service_url = "http://localhost:3000"
machine_name = "test-machine"
api_token = "test-token"
"#;

        let config = RunnerConfig::from_toml(toml).unwrap();
        assert_eq!(config.heartbeat_interval_secs, 20);
    }

    #[test]
    fn defaults_to_empty_repo_directories() {
        let toml = r#"
control_service_url = "http://localhost:3000"
machine_name = "test-machine"
api_token = "test-token"
"#;

        let config = RunnerConfig::from_toml(toml).unwrap();
        assert!(config.repo_directories.is_empty());
    }

    #[test]
    fn repo_paths_expands_tilde() {
        let toml = r#"
control_service_url = "http://localhost:3000"
machine_name = "test-machine"
api_token = "test-token"
repo_directories = ["~/projects"]
"#;

        let config = RunnerConfig::from_toml(toml).unwrap();
        let paths = config.repo_paths();
        assert_eq!(paths.len(), 1);

        if let Some(home) = dirs::home_dir() {
            assert_eq!(paths[0], home.join("projects"));
        }
    }

    #[test]
    fn repo_paths_keeps_absolute_paths() {
        let toml = r#"
control_service_url = "http://localhost:3000"
machine_name = "test-machine"
api_token = "test-token"
repo_directories = ["/absolute/path"]
"#;

        let config = RunnerConfig::from_toml(toml).unwrap();
        let paths = config.repo_paths();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], PathBuf::from("/absolute/path"));
    }

    #[test]
    fn rejects_invalid_toml() {
        let toml = "this is not valid toml {{{";
        let result = RunnerConfig::from_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_missing_required_fields() {
        let toml = r#"
machine_name = "test-machine"
"#;

        let result = RunnerConfig::from_toml(toml);
        assert!(result.is_err());
    }
}

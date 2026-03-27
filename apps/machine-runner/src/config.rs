use std::path::PathBuf;

use serde::Deserialize;

/// Configuration for the machine runner, loaded from a TOML file.
#[derive(Debug, Clone, Deserialize)]
pub struct RunnerConfig {
    pub control_service_url: String,
    pub machine_name: String,
    pub api_token: String,
    #[serde(default = "default_heartbeat_interval")]
    pub heartbeat_interval_secs: u64,
    #[serde(default)]
    pub repo_directories: Vec<String>,
}

const fn default_heartbeat_interval() -> u64 {
    30
}

impl RunnerConfig {
    /// Load configuration from a TOML file.
    ///
    /// If no path is provided, looks for `~/.config/cadre/machine-runner.toml`.
    ///
    /// # Errors
    ///
    /// Returns an error if the config file cannot be read or parsed.
    pub fn load(path: Option<&str>) -> anyhow::Result<Self> {
        let config_path = match path {
            Some(p) => PathBuf::from(p),
            None => default_config_path()?,
        };

        let contents = std::fs::read_to_string(&config_path)
            .map_err(|e| anyhow::anyhow!("Failed to read config at {}: {e}", config_path.display()))?;

        let config: Self = toml::from_str(&contents)
            .map_err(|e| anyhow::anyhow!("Failed to parse config: {e}"))?;

        Ok(config)
    }

    /// Convert repo directory strings to `PathBuf` values.
    #[must_use]
    pub fn repo_paths(&self) -> Vec<PathBuf> {
        self.repo_directories
            .iter()
            .map(|d| {
                if d.starts_with('~') {
                    if let Some(home) = dirs::home_dir() {
                        return home.join(d.strip_prefix("~/").unwrap_or(d));
                    }
                }
                PathBuf::from(d)
            })
            .collect()
    }
}

fn default_config_path() -> anyhow::Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
    Ok(config_dir.join("cadre").join("machine-runner.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_config() {
        let toml_str = r#"
control_service_url = "https://api.cadre.dev"
machine_name = "test-machine"
api_token = "tok_abc123"
heartbeat_interval_secs = 15
repo_directories = ["/home/user/projects", "~/code"]
"#;
        let config: RunnerConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.control_service_url, "https://api.cadre.dev");
        assert_eq!(config.machine_name, "test-machine");
        assert_eq!(config.api_token, "tok_abc123");
        assert_eq!(config.heartbeat_interval_secs, 15);
        assert_eq!(config.repo_directories.len(), 2);
    }

    #[test]
    fn test_default_heartbeat_interval() {
        let toml_str = r#"
control_service_url = "http://localhost:8080"
machine_name = "dev"
api_token = "tok_dev"
"#;
        let config: RunnerConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.heartbeat_interval_secs, 30);
    }

    #[test]
    fn test_repo_paths_expands_tilde() {
        let config = RunnerConfig {
            control_service_url: String::new(),
            machine_name: String::new(),
            api_token: String::new(),
            heartbeat_interval_secs: 30,
            repo_directories: vec![
                "/absolute/path".to_string(),
                "~/relative/path".to_string(),
            ],
        };
        let paths = config.repo_paths();
        assert_eq!(paths[0], PathBuf::from("/absolute/path"));
        // The tilde-expanded path should not start with '~'
        assert!(!paths[1].to_string_lossy().starts_with('~'));
    }
}

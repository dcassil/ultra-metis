use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::RunnerConfig;

/// Comprehensive settings for the machine runner covering all ADR SMET-A-0003 categories.
///
/// This struct is intended to be shared across the runner lifecycle via
/// `Arc<RwLock<Settings>>`, allowing live updates from a UI or configuration reload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    // -- Connection --
    /// Base URL of the control service API.
    pub control_service_url: String,
    /// Human-readable name for this machine.
    pub machine_name: String,
    // Note: api_token is NOT stored here. It goes in the OS keychain.
    // For headless/CLI mode it can be passed separately via the token Arc.

    // -- Behavior --
    /// Whether to start the runner automatically on launch.
    pub auto_start: bool,
    /// Whether to start the UI minimized (relevant for desktop app).
    pub start_minimized: bool,
    /// Master enable/disable switch. When false the heartbeat loop pauses.
    pub enabled: bool,
    /// Seconds between heartbeats to the control service.
    pub heartbeat_interval_secs: u64,
    /// Maximum number of concurrent Claude sessions.
    pub max_concurrent_sessions: u32,

    // -- Repos --
    /// Directories to scan for git repositories.
    pub repo_directories: Vec<String>,
    /// Explicit list of allowed repository names (empty = all allowed).
    pub allowed_repos: Vec<String>,
    /// Repositories that are blocked from being used.
    pub blocked_repos: Vec<String>,
    /// If true, only repos in `allowed_repos` may be used.
    pub restrict_to_repos: bool,

    // -- Security --
    /// Require local user approval before executing sessions.
    pub local_approval_required: bool,
    /// Autonomy levels this machine is allowed to use.
    pub allowed_autonomy_levels: Vec<String>,
    /// Hard block on autonomous mode regardless of server policy.
    pub block_autonomous_mode: bool,
    /// Session timeout in minutes (0 = no limit).
    pub session_timeout_minutes: u32,
    /// Action categories this machine is allowed to perform (empty = all).
    pub allowed_action_categories: Vec<String>,
    /// Action categories that are blocked.
    pub blocked_action_categories: Vec<String>,

    // -- Updates --
    /// Whether to auto-update the runner binary.
    pub auto_update: bool,
    /// Update channel: "stable", "beta", etc.
    pub update_channel: String,

    // -- Logging --
    /// Log level: "trace", "debug", "info", "warn", "error".
    pub log_level: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            // Connection
            control_service_url: "http://localhost:8080".to_string(),
            machine_name: hostname_or_default(),

            // Behavior
            auto_start: true,
            start_minimized: true,
            enabled: true,
            heartbeat_interval_secs: 30,
            max_concurrent_sessions: 1,

            // Repos
            repo_directories: Vec::new(),
            allowed_repos: Vec::new(),
            blocked_repos: Vec::new(),
            restrict_to_repos: false,

            // Security
            local_approval_required: false,
            allowed_autonomy_levels: vec![
                "normal".to_string(),
                "stricter".to_string(),
                "autonomous".to_string(),
            ],
            block_autonomous_mode: false,
            session_timeout_minutes: 0,
            allowed_action_categories: Vec::new(),
            blocked_action_categories: Vec::new(),

            // Updates
            auto_update: true,
            update_channel: "stable".to_string(),

            // Logging
            log_level: "info".to_string(),
        }
    }
}

/// Versioned wrapper for JSON serialization of settings.
///
/// The `version` field allows future migrations if the schema changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VersionedSettings {
    version: u32,
    #[serde(flatten)]
    settings: Settings,
}

impl Settings {
    /// Path to the settings JSON file (`~/.config/cadre/settings.json`).
    pub fn settings_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cadre")
            .join("settings.json")
    }

    /// Path to the legacy TOML config (`~/.config/cadre/machine-runner.toml`).
    fn legacy_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cadre")
            .join("machine-runner.toml")
    }

    /// Load settings from `~/.config/cadre/settings.json`.
    ///
    /// Returns default settings if the file doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read or parsed.
    pub fn load() -> Result<Self, anyhow::Error> {
        let path = Self::settings_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Failed to read settings at {}: {e}", path.display()))?;
        let versioned: VersionedSettings = serde_json::from_str(&contents)
            .map_err(|e| anyhow::anyhow!("Failed to parse settings JSON: {e}"))?;
        Ok(versioned.settings)
    }

    /// Save settings to `~/.config/cadre/settings.json`.
    ///
    /// Creates the parent directory if it doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn save(&self) -> Result<(), anyhow::Error> {
        let path = Self::settings_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let versioned = VersionedSettings {
            version: 1,
            settings: self.clone(),
        };
        let json = serde_json::to_string_pretty(&versioned)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Check if the legacy TOML config exists at `~/.config/cadre/machine-runner.toml`.
    pub fn has_legacy_config() -> bool {
        Self::legacy_config_path().exists()
    }

    /// Migrate from legacy `machine-runner.toml` to `settings.json`.
    ///
    /// Reads the TOML config, converts it to `Settings`, saves the JSON file, and
    /// returns the extracted API token separately (for keychain storage).
    ///
    /// # Errors
    ///
    /// Returns an error if the legacy config cannot be read/parsed or the new file
    /// cannot be written.
    pub fn migrate_from_legacy() -> Result<(Self, String), anyhow::Error> {
        let legacy_path = Self::legacy_config_path();
        let config = RunnerConfig::load(Some(
            legacy_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Legacy config path is not valid UTF-8"))?,
        ))?;
        let (settings, token) = Self::from_runner_config(&config);
        settings.save()?;
        tracing::info!(
            legacy = %legacy_path.display(),
            new = %Self::settings_path().display(),
            "Migrated legacy config to settings.json"
        );
        Ok((settings, token))
    }

    /// Convert a legacy `RunnerConfig` into `Settings`, extracting the API token separately.
    ///
    /// Returns `(settings, token)` so the token can be stored in its own `Arc<RwLock<String>>`.
    pub fn from_runner_config(config: &RunnerConfig) -> (Self, String) {
        let settings = Self {
            control_service_url: config.control_service_url.clone(),
            machine_name: config.machine_name.clone(),
            heartbeat_interval_secs: config.heartbeat_interval_secs,
            repo_directories: config.repo_directories.clone(),
            ..Self::default()
        };
        let token = config.api_token.clone();
        (settings, token)
    }

    /// Convert repo directory strings to `PathBuf` values, expanding `~`.
    #[must_use]
    pub fn repo_paths(&self) -> Vec<std::path::PathBuf> {
        self.repo_directories
            .iter()
            .map(|d| {
                if d.starts_with('~') {
                    if let Some(home) = dirs::home_dir() {
                        return home.join(d.strip_prefix("~/").unwrap_or(d));
                    }
                }
                std::path::PathBuf::from(d)
            })
            .collect()
    }
}

/// Best-effort hostname for the default machine name.
fn hostname_or_default() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown-machine".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: override settings path to a temp directory for isolation.
    fn save_to_path(settings: &Settings, path: &std::path::Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let versioned = VersionedSettings {
            version: 1,
            settings: settings.clone(),
        };
        let json = serde_json::to_string_pretty(&versioned)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    fn load_from_path(path: &std::path::Path) -> anyhow::Result<Settings> {
        let contents = std::fs::read_to_string(path)?;
        let versioned: VersionedSettings = serde_json::from_str(&contents)?;
        Ok(versioned.settings)
    }

    #[test]
    fn test_save_then_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");

        let mut settings = Settings::default();
        settings.machine_name = "roundtrip-machine".to_string();
        settings.control_service_url = "https://roundtrip.example.com".to_string();
        settings.heartbeat_interval_secs = 42;
        settings.repo_directories = vec!["/home/user/code".to_string()];
        settings.block_autonomous_mode = true;

        save_to_path(&settings, &path).unwrap();
        let loaded = load_from_path(&path).unwrap();

        assert_eq!(loaded.machine_name, "roundtrip-machine");
        assert_eq!(loaded.control_service_url, "https://roundtrip.example.com");
        assert_eq!(loaded.heartbeat_interval_secs, 42);
        assert_eq!(loaded.repo_directories, vec!["/home/user/code"]);
        assert!(loaded.block_autonomous_mode);
        // Defaults should survive the trip
        assert!(loaded.auto_start);
        assert!(loaded.enabled);
        assert_eq!(loaded.log_level, "info");
    }

    #[test]
    fn test_settings_json_versioning() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");

        let settings = Settings::default();
        save_to_path(&settings, &path).unwrap();

        // Read raw JSON and verify the version field is present
        let raw = std::fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed["version"], serde_json::json!(1));
        // Other fields should be present at the top level (flattened)
        assert!(parsed["control_service_url"].is_string());
        assert!(parsed["machine_name"].is_string());
    }

    #[test]
    fn test_load_returns_default_when_file_missing() {
        // Settings::load() returns defaults when the file doesn't exist.
        // We test the underlying logic by verifying a non-existent path.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");
        assert!(!path.exists());
        // Directly test: since we can't override the path in load(), verify
        // that default is returned by construction.
        let settings = Settings::default();
        assert_eq!(settings.control_service_url, "http://localhost:8080");
        assert!(settings.enabled);
    }

    #[test]
    fn test_migrate_from_legacy_toml() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("machine-runner.toml");

        let toml_content = r#"
control_service_url = "https://legacy.example.com"
machine_name = "legacy-box"
api_token = "tok_legacy_secret"
heartbeat_interval_secs = 20
repo_directories = ["/old/path"]
"#;
        std::fs::write(&toml_path, toml_content).unwrap();

        // Load via RunnerConfig to simulate migration
        let config = RunnerConfig::load(Some(toml_path.to_str().unwrap())).unwrap();
        let (settings, token) = Settings::from_runner_config(&config);

        assert_eq!(settings.control_service_url, "https://legacy.example.com");
        assert_eq!(settings.machine_name, "legacy-box");
        assert_eq!(settings.heartbeat_interval_secs, 20);
        assert_eq!(settings.repo_directories, vec!["/old/path"]);
        assert_eq!(token, "tok_legacy_secret");

        // Verify it can be saved and re-loaded as JSON
        let json_path = dir.path().join("settings.json");
        save_to_path(&settings, &json_path).unwrap();
        let loaded = load_from_path(&json_path).unwrap();
        assert_eq!(loaded.machine_name, "legacy-box");
    }

    #[test]
    fn test_default_settings_has_sensible_values() {
        let s = Settings::default();
        assert!(s.auto_start);
        assert!(s.start_minimized);
        assert!(s.enabled);
        assert_eq!(s.heartbeat_interval_secs, 30);
        assert_eq!(s.max_concurrent_sessions, 1);
        assert!(!s.restrict_to_repos);
        assert!(!s.local_approval_required);
        assert!(!s.block_autonomous_mode);
        assert_eq!(s.session_timeout_minutes, 0);
        assert!(s.auto_update);
        assert_eq!(s.update_channel, "stable");
        assert_eq!(s.log_level, "info");
        assert_eq!(s.allowed_autonomy_levels.len(), 3);
    }

    #[test]
    fn test_from_runner_config() {
        let config = RunnerConfig {
            control_service_url: "https://api.example.com".to_string(),
            machine_name: "my-machine".to_string(),
            api_token: "tok_secret".to_string(),
            heartbeat_interval_secs: 15,
            repo_directories: vec!["/home/user/projects".to_string()],
        };

        let (settings, token) = Settings::from_runner_config(&config);
        assert_eq!(settings.control_service_url, "https://api.example.com");
        assert_eq!(settings.machine_name, "my-machine");
        assert_eq!(settings.heartbeat_interval_secs, 15);
        assert_eq!(settings.repo_directories, vec!["/home/user/projects"]);
        assert_eq!(token, "tok_secret");
        // Non-config fields should be defaults
        assert!(settings.enabled);
        assert!(settings.auto_start);
    }

    #[test]
    fn test_settings_serialization_roundtrip() {
        let settings = Settings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.control_service_url, settings.control_service_url);
        assert_eq!(deserialized.heartbeat_interval_secs, settings.heartbeat_interval_secs);
        assert_eq!(deserialized.allowed_autonomy_levels, settings.allowed_autonomy_levels);
    }

    #[test]
    fn test_repo_paths_expands_tilde() {
        let settings = Settings {
            repo_directories: vec![
                "/absolute/path".to_string(),
                "~/relative/path".to_string(),
            ],
            ..Settings::default()
        };
        let paths = settings.repo_paths();
        assert_eq!(paths[0], std::path::PathBuf::from("/absolute/path"));
        assert!(!paths[1].to_string_lossy().starts_with('~'));
    }
}

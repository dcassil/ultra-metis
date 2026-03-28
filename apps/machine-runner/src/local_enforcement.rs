use crate::settings::Settings;
use serde_json::Value;

/// A violation of local security settings.
///
/// Returned when a session command fails validation against the machine's
/// local settings (as opposed to the server-side policy cache).
#[derive(Debug, Clone)]
pub struct LocalViolation {
    /// Human-readable explanation of why the command was rejected.
    pub reason: String,
    /// The name of the setting that caused the rejection.
    pub setting: String,
}

/// Validate a `start_session` command payload against local settings.
///
/// This enforces the owner's local security preferences *before* the session
/// is dispatched to the process supervisor, complementing the server-side
/// policy checked via `LocalPolicyCache`.
///
/// # Errors
///
/// Returns `Err(LocalViolation)` if the payload violates any local setting.
pub fn validate_session_command(
    settings: &Settings,
    payload: &Value,
) -> Result<(), LocalViolation> {
    let autonomy = payload
        .get("autonomy_level")
        .and_then(|v| v.as_str())
        .unwrap_or("normal");
    let repo_path = payload
        .get("repo_path")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Check block_autonomous_mode
    if settings.block_autonomous_mode && autonomy == "autonomous" {
        return Err(LocalViolation {
            reason: "Autonomous mode is blocked by local settings".into(),
            setting: "block_autonomous_mode".into(),
        });
    }

    // Check allowed_autonomy_levels
    if !settings.allowed_autonomy_levels.is_empty()
        && !settings.allowed_autonomy_levels.iter().any(|a| a == autonomy)
    {
        return Err(LocalViolation {
            reason: format!("Autonomy level '{}' is not in allowed list", autonomy),
            setting: "allowed_autonomy_levels".into(),
        });
    }

    // Check restrict_to_repos + allowed_repos
    if settings.restrict_to_repos && !settings.allowed_repos.is_empty() {
        if !settings
            .allowed_repos
            .iter()
            .any(|r| repo_path.starts_with(r.as_str()))
        {
            return Err(LocalViolation {
                reason: format!("Repo '{}' is not in allowed repos list", repo_path),
                setting: "restrict_to_repos".into(),
            });
        }
    }

    // Check blocked_repos
    if settings
        .blocked_repos
        .iter()
        .any(|r| repo_path.starts_with(r.as_str()))
    {
        return Err(LocalViolation {
            reason: format!("Repo '{}' is in blocked repos list", repo_path),
            setting: "blocked_repos".into(),
        });
    }

    Ok(())
}

/// Check whether the local settings require user approval before starting sessions.
pub fn requires_local_approval(settings: &Settings) -> bool {
    settings.local_approval_required
}

/// A request sent through the approval channel to ask for local user approval.
///
/// The runner sends this when `local_approval_required` is true. The Tauri
/// desktop app (or any other UI host) listens on the receiving end and
/// presents a dialog. For headless/CLI mode the channel is left unset,
/// causing auto-approval.
#[derive(Debug, Clone)]
pub struct ApprovalRequest {
    /// The session ID that needs approval.
    pub session_id: String,
    /// Repository path for the session.
    pub repo_path: String,
    /// Requested autonomy level.
    pub autonomy_level: String,
    /// The instruction prompt (may be truncated for display).
    pub instructions_preview: String,
}

/// The response to an `ApprovalRequest`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalResponse {
    /// The user approved the session.
    Approved,
    /// The user denied the session.
    Denied,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;

    #[test]
    fn test_block_autonomous_mode_rejects_autonomous() {
        let settings = Settings {
            block_autonomous_mode: true,
            ..Settings::default()
        };
        let payload = serde_json::json!({
            "session_id": "s-1",
            "repo_path": "/home/user/project",
            "autonomy_level": "autonomous"
        });
        let result = validate_session_command(&settings, &payload);
        assert!(result.is_err());
        let violation = result.unwrap_err();
        assert_eq!(violation.setting, "block_autonomous_mode");
        assert!(violation.reason.contains("blocked"));
    }

    #[test]
    fn test_block_autonomous_mode_allows_normal() {
        let settings = Settings {
            block_autonomous_mode: true,
            ..Settings::default()
        };
        let payload = serde_json::json!({
            "session_id": "s-1",
            "repo_path": "/home/user/project",
            "autonomy_level": "normal"
        });
        let result = validate_session_command(&settings, &payload);
        assert!(result.is_ok());
    }

    #[test]
    fn test_block_autonomous_mode_false_allows_autonomous() {
        let settings = Settings {
            block_autonomous_mode: false,
            ..Settings::default()
        };
        let payload = serde_json::json!({
            "session_id": "s-1",
            "repo_path": "/home/user/project",
            "autonomy_level": "autonomous"
        });
        let result = validate_session_command(&settings, &payload);
        assert!(result.is_ok());
    }

    #[test]
    fn test_allowed_autonomy_levels_filters_correctly() {
        let settings = Settings {
            allowed_autonomy_levels: vec!["normal".to_string(), "stricter".to_string()],
            ..Settings::default()
        };

        // normal is allowed
        let payload = serde_json::json!({
            "session_id": "s-1",
            "repo_path": "/home/user/project",
            "autonomy_level": "normal"
        });
        assert!(validate_session_command(&settings, &payload).is_ok());

        // stricter is allowed
        let payload = serde_json::json!({
            "session_id": "s-1",
            "repo_path": "/home/user/project",
            "autonomy_level": "stricter"
        });
        assert!(validate_session_command(&settings, &payload).is_ok());

        // autonomous is NOT allowed
        let payload = serde_json::json!({
            "session_id": "s-1",
            "repo_path": "/home/user/project",
            "autonomy_level": "autonomous"
        });
        let result = validate_session_command(&settings, &payload);
        assert!(result.is_err());
        let violation = result.unwrap_err();
        assert_eq!(violation.setting, "allowed_autonomy_levels");
        assert!(violation.reason.contains("not in allowed list"));
    }

    #[test]
    fn test_empty_allowed_autonomy_levels_allows_all() {
        let settings = Settings {
            allowed_autonomy_levels: vec![],
            ..Settings::default()
        };
        let payload = serde_json::json!({
            "session_id": "s-1",
            "repo_path": "/home/user/project",
            "autonomy_level": "autonomous"
        });
        assert!(validate_session_command(&settings, &payload).is_ok());
    }

    #[test]
    fn test_restrict_to_repos_with_allowed_list() {
        let settings = Settings {
            restrict_to_repos: true,
            allowed_repos: vec![
                "/home/user/allowed-project".to_string(),
                "/home/user/other-project".to_string(),
            ],
            ..Settings::default()
        };

        // Allowed repo
        let payload = serde_json::json!({
            "session_id": "s-1",
            "repo_path": "/home/user/allowed-project",
            "autonomy_level": "normal"
        });
        assert!(validate_session_command(&settings, &payload).is_ok());

        // Subdirectory of allowed repo should also work
        let payload = serde_json::json!({
            "session_id": "s-1",
            "repo_path": "/home/user/allowed-project/sub/dir",
            "autonomy_level": "normal"
        });
        assert!(validate_session_command(&settings, &payload).is_ok());

        // Not in allowed list
        let payload = serde_json::json!({
            "session_id": "s-1",
            "repo_path": "/home/user/forbidden-project",
            "autonomy_level": "normal"
        });
        let result = validate_session_command(&settings, &payload);
        assert!(result.is_err());
        let violation = result.unwrap_err();
        assert_eq!(violation.setting, "restrict_to_repos");
        assert!(violation.reason.contains("not in allowed repos list"));
    }

    #[test]
    fn test_restrict_to_repos_false_allows_any_repo() {
        let settings = Settings {
            restrict_to_repos: false,
            allowed_repos: vec!["/home/user/allowed-project".to_string()],
            ..Settings::default()
        };
        let payload = serde_json::json!({
            "session_id": "s-1",
            "repo_path": "/home/user/any-project",
            "autonomy_level": "normal"
        });
        assert!(validate_session_command(&settings, &payload).is_ok());
    }

    #[test]
    fn test_blocked_repos_rejects_matching_repo() {
        let settings = Settings {
            blocked_repos: vec![
                "/home/user/secret-project".to_string(),
                "/srv/production".to_string(),
            ],
            ..Settings::default()
        };

        // Blocked repo
        let payload = serde_json::json!({
            "session_id": "s-1",
            "repo_path": "/home/user/secret-project",
            "autonomy_level": "normal"
        });
        let result = validate_session_command(&settings, &payload);
        assert!(result.is_err());
        let violation = result.unwrap_err();
        assert_eq!(violation.setting, "blocked_repos");
        assert!(violation.reason.contains("blocked repos list"));

        // Subdirectory of blocked repo
        let payload = serde_json::json!({
            "session_id": "s-1",
            "repo_path": "/srv/production/app",
            "autonomy_level": "normal"
        });
        assert!(validate_session_command(&settings, &payload).is_err());
    }

    #[test]
    fn test_blocked_repos_allows_non_matching_repo() {
        let settings = Settings {
            blocked_repos: vec!["/home/user/secret-project".to_string()],
            ..Settings::default()
        };
        let payload = serde_json::json!({
            "session_id": "s-1",
            "repo_path": "/home/user/open-project",
            "autonomy_level": "normal"
        });
        assert!(validate_session_command(&settings, &payload).is_ok());
    }

    #[test]
    fn test_all_checks_pass_with_permissive_defaults() {
        let settings = Settings::default();
        let payload = serde_json::json!({
            "session_id": "s-1",
            "repo_path": "/any/repo/path",
            "autonomy_level": "autonomous"
        });
        assert!(validate_session_command(&settings, &payload).is_ok());
    }

    #[test]
    fn test_missing_autonomy_level_defaults_to_normal() {
        let settings = Settings {
            allowed_autonomy_levels: vec!["normal".to_string()],
            ..Settings::default()
        };
        // No autonomy_level in payload => defaults to "normal"
        let payload = serde_json::json!({
            "session_id": "s-1",
            "repo_path": "/any/repo"
        });
        assert!(validate_session_command(&settings, &payload).is_ok());
    }

    #[test]
    fn test_missing_repo_path_defaults_to_empty_string() {
        let settings = Settings {
            restrict_to_repos: true,
            allowed_repos: vec!["/allowed".to_string()],
            ..Settings::default()
        };
        // No repo_path in payload => defaults to ""
        let payload = serde_json::json!({
            "session_id": "s-1",
            "autonomy_level": "normal"
        });
        let result = validate_session_command(&settings, &payload);
        assert!(result.is_err());
    }

    #[test]
    fn test_requires_local_approval_true() {
        let settings = Settings {
            local_approval_required: true,
            ..Settings::default()
        };
        assert!(requires_local_approval(&settings));
    }

    #[test]
    fn test_requires_local_approval_false() {
        let settings = Settings::default();
        assert!(!requires_local_approval(&settings));
    }

    #[test]
    fn test_blocked_repos_checked_even_when_restrict_to_repos_false() {
        // Even with restrict_to_repos=false, blocked_repos should still block
        let settings = Settings {
            restrict_to_repos: false,
            blocked_repos: vec!["/blocked".to_string()],
            ..Settings::default()
        };
        let payload = serde_json::json!({
            "session_id": "s-1",
            "repo_path": "/blocked/sub",
            "autonomy_level": "normal"
        });
        let result = validate_session_command(&settings, &payload);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().setting, "blocked_repos");
    }

    #[test]
    fn test_block_autonomous_checked_before_allowed_levels() {
        // block_autonomous_mode should trigger before allowed_autonomy_levels
        let settings = Settings {
            block_autonomous_mode: true,
            allowed_autonomy_levels: vec![
                "normal".to_string(),
                "autonomous".to_string(),
            ],
            ..Settings::default()
        };
        let payload = serde_json::json!({
            "session_id": "s-1",
            "repo_path": "/repo",
            "autonomy_level": "autonomous"
        });
        let result = validate_session_command(&settings, &payload);
        assert!(result.is_err());
        // Should be the block_autonomous_mode setting, not allowed_autonomy_levels
        assert_eq!(result.unwrap_err().setting, "block_autonomous_mode");
    }
}

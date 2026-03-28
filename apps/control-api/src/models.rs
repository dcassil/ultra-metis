//! Domain types for the Control API.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Status of a machine in the fleet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MachineStatus {
    Pending,
    Trusted,
    Revoked,
}

impl MachineStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Trusted => "trusted",
            Self::Revoked => "revoked",
        }
    }
}

impl std::fmt::Display for MachineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for MachineStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "trusted" => Ok(Self::Trusted),
            "revoked" => Ok(Self::Revoked),
            other => Err(format!("unknown machine status: {other}")),
        }
    }
}

/// Trust tier for a machine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrustTier {
    Untrusted,
    Basic,
    Elevated,
}

impl TrustTier {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Untrusted => "untrusted",
            Self::Basic => "basic",
            Self::Elevated => "elevated",
        }
    }
}

impl std::str::FromStr for TrustTier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "untrusted" => Ok(Self::Untrusted),
            "basic" => Ok(Self::Basic),
            "elevated" => Ok(Self::Elevated),
            other => Err(format!("unknown trust tier: {other}")),
        }
    }
}

/// Connectivity status derived from heartbeat recency.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectivityStatus {
    Online,
    Degraded,
    Offline,
    Unknown,
}

/// A registered machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Machine {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub platform: String,
    pub status: MachineStatus,
    pub trust_tier: TrustTier,
    pub capabilities: Option<String>,
    pub last_heartbeat: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Machine {
    /// Compute connectivity status from heartbeat timestamp.
    #[must_use]
    pub fn connectivity_status(&self) -> ConnectivityStatus {
        let Some(ref hb) = self.last_heartbeat else {
            return ConnectivityStatus::Unknown;
        };

        let Ok(last) = hb.parse::<DateTime<Utc>>() else {
            return ConnectivityStatus::Unknown;
        };

        let elapsed = Utc::now().signed_duration_since(last);
        if elapsed.num_seconds() < 120 {
            ConnectivityStatus::Online
        } else if elapsed.num_seconds() < 600 {
            ConnectivityStatus::Degraded
        } else {
            ConnectivityStatus::Offline
        }
    }
}

/// A repo tracked on a machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineRepo {
    pub id: String,
    pub machine_id: String,
    pub repo_path: String,
    pub repo_name: Option<String>,
    pub last_seen: String,
}

// -- Session types --

/// State of a session in its lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Starting,
    Running,
    WaitingForInput,
    Paused,
    Completed,
    Failed,
    Stopped,
}

impl SessionState {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Starting => "starting",
            Self::Running => "running",
            Self::WaitingForInput => "waiting_for_input",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Stopped => "stopped",
        }
    }

    /// Returns true if this is a terminal state (no further transitions allowed).
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Stopped)
    }

    /// Check if transitioning from `self` to `to` is a valid state transition.
    #[must_use]
    pub fn is_valid_transition(&self, to: &Self) -> bool {
        matches!(
            (self, to),
            (Self::Starting, Self::Running)
                | (Self::Starting, Self::Failed)
                | (Self::Running, Self::WaitingForInput)
                | (Self::Running, Self::Paused)
                | (Self::Running, Self::Completed)
                | (Self::Running, Self::Failed)
                | (Self::Running, Self::Stopped)
                | (Self::WaitingForInput, Self::Running)
                | (Self::Paused, Self::Running)
        )
    }
}

impl std::fmt::Display for SessionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for SessionState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "starting" => Ok(Self::Starting),
            "running" => Ok(Self::Running),
            "waiting_for_input" => Ok(Self::WaitingForInput),
            "paused" => Ok(Self::Paused),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "stopped" => Ok(Self::Stopped),
            other => Err(format!("unknown session state: {other}")),
        }
    }
}

/// Autonomy level for a session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AutonomyLevel {
    Normal,
    Stricter,
    Autonomous,
}

impl AutonomyLevel {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Stricter => "stricter",
            Self::Autonomous => "autonomous",
        }
    }
}

impl std::fmt::Display for AutonomyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for AutonomyLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "normal" => Ok(Self::Normal),
            "stricter" => Ok(Self::Stricter),
            "autonomous" => Ok(Self::Autonomous),
            other => Err(format!("unknown autonomy level: {other}")),
        }
    }
}

/// A session record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub machine_id: String,
    pub repo_path: String,
    pub title: String,
    pub instructions: String,
    pub autonomy_level: AutonomyLevel,
    pub work_item_id: Option<String>,
    pub context: Option<String>,
    pub state: SessionState,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// A session state transition event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    pub id: String,
    pub session_id: String,
    pub from_state: Option<String>,
    pub to_state: String,
    pub timestamp: String,
    pub metadata: Option<String>,
}

/// Command type for session commands queued for the runner.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommandType {
    StartSession,
    Stop,
    ForceStop,
    Pause,
    Resume,
}

impl CommandType {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StartSession => "start_session",
            Self::Stop => "stop",
            Self::ForceStop => "force_stop",
            Self::Pause => "pause",
            Self::Resume => "resume",
        }
    }
}

impl std::str::FromStr for CommandType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "start_session" => Ok(Self::StartSession),
            "stop" => Ok(Self::Stop),
            "force_stop" => Ok(Self::ForceStop),
            "pause" => Ok(Self::Pause),
            "resume" => Ok(Self::Resume),
            other => Err(format!("unknown command type: {other}")),
        }
    }
}

/// Status of a command in the queue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommandStatus {
    Pending,
    Delivered,
    Executed,
}

impl CommandStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Delivered => "delivered",
            Self::Executed => "executed",
        }
    }
}

impl std::str::FromStr for CommandStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "delivered" => Ok(Self::Delivered),
            "executed" => Ok(Self::Executed),
            other => Err(format!("unknown command status: {other}")),
        }
    }
}

/// A command queued for a runner to pick up.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCommand {
    pub id: String,
    pub session_id: String,
    pub machine_id: String,
    pub command_type: String,
    pub payload: Option<String>,
    pub status: String,
    pub created_at: String,
    pub delivered_at: Option<String>,
}

// -- Request / Response types --

/// Request body for POST /api/machines/register.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub platform: String,
    pub capabilities: Option<String>,
    pub repos: Option<Vec<RepoInfo>>,
}

/// Repo info sent during registration or heartbeat.
#[derive(Debug, Deserialize)]
pub struct RepoInfo {
    pub path: String,
    pub name: Option<String>,
}

/// Request body for POST /api/machines/{id}/heartbeat.
#[derive(Debug, Deserialize)]
pub struct HeartbeatRequest {
    pub repos: Option<Vec<RepoInfo>>,
}

/// Response for a single machine.
#[derive(Debug, Serialize)]
pub struct MachineResponse {
    pub id: String,
    pub name: String,
    pub platform: String,
    pub status: MachineStatus,
    pub trust_tier: TrustTier,
    pub connectivity_status: ConnectivityStatus,
    pub capabilities: Option<String>,
    pub last_heartbeat: Option<String>,
    pub repos_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Response for a machine with full repo detail.
#[derive(Debug, Serialize)]
pub struct MachineDetailResponse {
    pub id: String,
    pub name: String,
    pub platform: String,
    pub status: MachineStatus,
    pub trust_tier: TrustTier,
    pub connectivity_status: ConnectivityStatus,
    pub capabilities: Option<String>,
    pub last_heartbeat: Option<String>,
    pub repos: Vec<MachineRepo>,
    pub created_at: String,
    pub updated_at: String,
}

/// Minimal response after registration.
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub id: String,
    pub status: MachineStatus,
}

/// Request body for POST /api/sessions.
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub machine_id: String,
    pub repo_path: String,
    pub title: String,
    pub instructions: String,
    pub autonomy_level: Option<AutonomyLevel>,
    pub work_item_id: Option<String>,
    pub context: Option<String>,
}

/// Query params for GET /api/sessions.
#[derive(Debug, Deserialize)]
pub struct ListSessionsQuery {
    pub machine_id: Option<String>,
    pub repo_path: Option<String>,
    pub state: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Request body for POST /api/sessions/{id}/state (runner reports state change).
#[derive(Debug, Deserialize)]
pub struct ReportStateRequest {
    pub state: SessionState,
    pub metadata: Option<serde_json::Value>,
}

/// Response for a single session.
#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: String,
    pub machine_id: String,
    pub repo_path: String,
    pub title: String,
    pub instructions: String,
    pub autonomy_level: AutonomyLevel,
    pub work_item_id: Option<String>,
    pub context: Option<String>,
    pub state: SessionState,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// Response for session list.
#[derive(Debug, Serialize)]
pub struct SessionListResponse {
    pub sessions: Vec<SessionResponse>,
    pub total: i64,
}

/// Response after session creation.
#[derive(Debug, Serialize)]
pub struct CreateSessionResponse {
    pub id: String,
    pub state: SessionState,
}

/// Response for a command in the queue.
#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub command_id: String,
    pub command_type: String,
    pub payload: Option<serde_json::Value>,
}

/// Generic JSON error body.
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine_status_roundtrip() {
        for status in [MachineStatus::Pending, MachineStatus::Trusted, MachineStatus::Revoked] {
            let s = status.as_str();
            let parsed: MachineStatus = s.parse().unwrap();
            assert_eq!(parsed, status);
        }
    }

    #[test]
    fn test_trust_tier_roundtrip() {
        for tier in [TrustTier::Untrusted, TrustTier::Basic, TrustTier::Elevated] {
            let s = tier.as_str();
            let parsed: TrustTier = s.parse().unwrap();
            assert_eq!(parsed, tier);
        }
    }

    #[test]
    fn test_connectivity_unknown_when_no_heartbeat() {
        let machine = Machine {
            id: "m1".into(),
            user_id: "u1".into(),
            name: "test".into(),
            platform: "linux".into(),
            status: MachineStatus::Trusted,
            trust_tier: TrustTier::Basic,
            capabilities: None,
            last_heartbeat: None,
            created_at: String::new(),
            updated_at: String::new(),
        };
        assert_eq!(machine.connectivity_status(), ConnectivityStatus::Unknown);
    }

    #[test]
    fn test_connectivity_online_when_recent() {
        let now = Utc::now().to_rfc3339();
        let machine = Machine {
            id: "m1".into(),
            user_id: "u1".into(),
            name: "test".into(),
            platform: "linux".into(),
            status: MachineStatus::Trusted,
            trust_tier: TrustTier::Basic,
            capabilities: None,
            last_heartbeat: Some(now),
            created_at: String::new(),
            updated_at: String::new(),
        };
        assert_eq!(machine.connectivity_status(), ConnectivityStatus::Online);
    }

    #[test]
    fn test_session_state_roundtrip() {
        for state in [
            SessionState::Starting,
            SessionState::Running,
            SessionState::WaitingForInput,
            SessionState::Paused,
            SessionState::Completed,
            SessionState::Failed,
            SessionState::Stopped,
        ] {
            let s = state.as_str();
            let parsed: SessionState = s.parse().unwrap();
            assert_eq!(parsed, state);
        }
    }

    #[test]
    fn test_session_state_terminal() {
        assert!(!SessionState::Starting.is_terminal());
        assert!(!SessionState::Running.is_terminal());
        assert!(!SessionState::WaitingForInput.is_terminal());
        assert!(!SessionState::Paused.is_terminal());
        assert!(SessionState::Completed.is_terminal());
        assert!(SessionState::Failed.is_terminal());
        assert!(SessionState::Stopped.is_terminal());
    }

    #[test]
    fn test_autonomy_level_roundtrip() {
        for level in [
            AutonomyLevel::Normal,
            AutonomyLevel::Stricter,
            AutonomyLevel::Autonomous,
        ] {
            let s = level.as_str();
            let parsed: AutonomyLevel = s.parse().unwrap();
            assert_eq!(parsed, level);
        }
    }

    #[test]
    fn test_command_type_roundtrip() {
        for ct in [
            CommandType::StartSession,
            CommandType::Stop,
            CommandType::ForceStop,
            CommandType::Pause,
            CommandType::Resume,
        ] {
            let s = ct.as_str();
            let parsed: CommandType = s.parse().unwrap();
            assert_eq!(parsed, ct);
        }
    }

    #[test]
    fn test_command_status_roundtrip() {
        for cs in [
            CommandStatus::Pending,
            CommandStatus::Delivered,
            CommandStatus::Executed,
        ] {
            let s = cs.as_str();
            let parsed: CommandStatus = s.parse().unwrap();
            assert_eq!(parsed, cs);
        }
    }

    #[test]
    fn test_valid_transitions() {
        // All valid transitions
        assert!(SessionState::Starting.is_valid_transition(&SessionState::Running));
        assert!(SessionState::Starting.is_valid_transition(&SessionState::Failed));
        assert!(SessionState::Running.is_valid_transition(&SessionState::WaitingForInput));
        assert!(SessionState::Running.is_valid_transition(&SessionState::Paused));
        assert!(SessionState::Running.is_valid_transition(&SessionState::Completed));
        assert!(SessionState::Running.is_valid_transition(&SessionState::Failed));
        assert!(SessionState::Running.is_valid_transition(&SessionState::Stopped));
        assert!(SessionState::WaitingForInput.is_valid_transition(&SessionState::Running));
        assert!(SessionState::Paused.is_valid_transition(&SessionState::Running));
    }

    #[test]
    fn test_invalid_transitions() {
        // From terminal states
        assert!(!SessionState::Completed.is_valid_transition(&SessionState::Running));
        assert!(!SessionState::Failed.is_valid_transition(&SessionState::Running));
        assert!(!SessionState::Stopped.is_valid_transition(&SessionState::Running));
        // Invalid from starting
        assert!(!SessionState::Starting.is_valid_transition(&SessionState::Paused));
        assert!(!SessionState::Starting.is_valid_transition(&SessionState::Completed));
        // Invalid from paused
        assert!(!SessionState::Paused.is_valid_transition(&SessionState::Completed));
        // Self-transitions
        assert!(!SessionState::Running.is_valid_transition(&SessionState::Running));
    }

    #[test]
    fn test_invalid_session_state() {
        assert!("bogus".parse::<SessionState>().is_err());
    }

    #[test]
    fn test_invalid_autonomy_level() {
        assert!("bogus".parse::<AutonomyLevel>().is_err());
    }
}

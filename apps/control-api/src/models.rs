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

// -- Policy types --

/// Action categories that can be allowed or blocked by policy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ActionCategory {
    ReadFiles,
    WriteFiles,
    RunTests,
    RunBuilds,
    GitOperations,
    InstallPackages,
    NetworkAccess,
    WorktreeOperations,
    ShellExecution,
}

impl ActionCategory {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ReadFiles => "read_files",
            Self::WriteFiles => "write_files",
            Self::RunTests => "run_tests",
            Self::RunBuilds => "run_builds",
            Self::GitOperations => "git_operations",
            Self::InstallPackages => "install_packages",
            Self::NetworkAccess => "network_access",
            Self::WorktreeOperations => "worktree_operations",
            Self::ShellExecution => "shell_execution",
        }
    }

    /// All available action categories.
    #[must_use]
    pub fn all() -> Vec<Self> {
        vec![
            Self::ReadFiles,
            Self::WriteFiles,
            Self::RunTests,
            Self::RunBuilds,
            Self::GitOperations,
            Self::InstallPackages,
            Self::NetworkAccess,
            Self::WorktreeOperations,
            Self::ShellExecution,
        ]
    }
}

impl std::fmt::Display for ActionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ActionCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "read_files" => Ok(Self::ReadFiles),
            "write_files" => Ok(Self::WriteFiles),
            "run_tests" => Ok(Self::RunTests),
            "run_builds" => Ok(Self::RunBuilds),
            "git_operations" => Ok(Self::GitOperations),
            "install_packages" => Ok(Self::InstallPackages),
            "network_access" => Ok(Self::NetworkAccess),
            "worktree_operations" => Ok(Self::WorktreeOperations),
            "shell_execution" => Ok(Self::ShellExecution),
            other => Err(format!("unknown action category: {other}")),
        }
    }
}

/// Session mode — visible indicator of policy restrictiveness.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionMode {
    Normal,
    Restricted,
    Elevated,
}

impl SessionMode {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Restricted => "restricted",
            Self::Elevated => "elevated",
        }
    }
}

impl std::str::FromStr for SessionMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "normal" => Ok(Self::Normal),
            "restricted" => Ok(Self::Restricted),
            "elevated" => Ok(Self::Elevated),
            other => Err(format!("unknown session mode: {other}")),
        }
    }
}

/// Machine-level policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachinePolicy {
    pub id: String,
    pub machine_id: String,
    pub allowed_categories: Vec<ActionCategory>,
    pub blocked_categories: Vec<ActionCategory>,
    pub max_autonomy_level: AutonomyLevel,
    pub session_mode: SessionMode,
    pub require_approval_for: Vec<ActionCategory>,
    pub created_at: String,
    pub updated_at: String,
}

/// Repo-level policy (narrows machine policy for a specific repo).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoPolicy {
    pub id: String,
    pub machine_id: String,
    pub repo_path: String,
    pub allowed_categories: Vec<ActionCategory>,
    pub blocked_categories: Vec<ActionCategory>,
    pub max_autonomy_level: Option<AutonomyLevel>,
    pub require_approval_for: Vec<ActionCategory>,
    pub created_at: String,
    pub updated_at: String,
}

/// Policy violation — recorded when an action is blocked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub reason: String,
    pub policy_scope: String,
    pub blocked_action: String,
}

/// Check if an action is allowed under a machine policy and optional repo policy.
/// Blocked categories always take precedence. Repo policy narrows machine policy.
pub fn is_action_allowed(
    action: &ActionCategory,
    machine_policy: &MachinePolicy,
    repo_policy: Option<&RepoPolicy>,
) -> Result<(), PolicyViolation> {
    // Check machine blocked list first (deny wins)
    if machine_policy.blocked_categories.contains(action) {
        return Err(PolicyViolation {
            reason: format!("action '{}' is blocked by machine policy", action),
            policy_scope: "machine".into(),
            blocked_action: action.to_string(),
        });
    }

    // Check repo blocked list
    if let Some(rp) = repo_policy {
        if rp.blocked_categories.contains(action) {
            return Err(PolicyViolation {
                reason: format!("action '{}' is blocked by repo policy", action),
                policy_scope: "repo".into(),
                blocked_action: action.to_string(),
            });
        }
    }

    // Check machine allowed list
    if !machine_policy.allowed_categories.contains(action) {
        return Err(PolicyViolation {
            reason: format!("action '{}' is not in machine allowed list", action),
            policy_scope: "machine".into(),
            blocked_action: action.to_string(),
        });
    }

    // Check repo allowed list (if repo policy exists and has allowed categories)
    if let Some(rp) = repo_policy {
        if !rp.allowed_categories.is_empty() && !rp.allowed_categories.contains(action) {
            return Err(PolicyViolation {
                reason: format!("action '{}' is not in repo allowed list", action),
                policy_scope: "repo".into(),
                blocked_action: action.to_string(),
            });
        }
    }

    Ok(())
}

/// Check if a requested autonomy level is allowed by policy.
pub fn is_autonomy_allowed(
    requested: &AutonomyLevel,
    machine_policy: &MachinePolicy,
    repo_policy: Option<&RepoPolicy>,
) -> Result<(), PolicyViolation> {
    let autonomy_rank = |level: &AutonomyLevel| -> u8 {
        match level {
            AutonomyLevel::Normal => 0,
            AutonomyLevel::Stricter => 0, // stricter is more restrictive, always allowed
            AutonomyLevel::Autonomous => 2,
        }
    };

    if autonomy_rank(requested) > autonomy_rank(&machine_policy.max_autonomy_level) {
        return Err(PolicyViolation {
            reason: format!(
                "requested autonomy '{}' exceeds machine max '{}'",
                requested, machine_policy.max_autonomy_level
            ),
            policy_scope: "machine".into(),
            blocked_action: format!("autonomy:{requested}"),
        });
    }

    if let Some(rp) = repo_policy {
        if let Some(ref repo_max) = rp.max_autonomy_level {
            if autonomy_rank(requested) > autonomy_rank(repo_max) {
                return Err(PolicyViolation {
                    reason: format!(
                        "requested autonomy '{}' exceeds repo max '{}'",
                        requested, repo_max
                    ),
                    policy_scope: "repo".into(),
                    blocked_action: format!("autonomy:{requested}"),
                });
            }
        }
    }

    Ok(())
}

/// Request body for PUT /api/machines/{id}/policy.
#[derive(Debug, Deserialize)]
pub struct UpdateMachinePolicyRequest {
    pub allowed_categories: Option<Vec<ActionCategory>>,
    pub blocked_categories: Option<Vec<ActionCategory>>,
    pub max_autonomy_level: Option<AutonomyLevel>,
    pub session_mode: Option<SessionMode>,
    pub require_approval_for: Option<Vec<ActionCategory>>,
}

/// Request body for PUT /api/machines/{id}/repos/{repo_path}/policy.
#[derive(Debug, Deserialize)]
pub struct UpdateRepoPolicyRequest {
    pub allowed_categories: Option<Vec<ActionCategory>>,
    pub blocked_categories: Option<Vec<ActionCategory>>,
    pub max_autonomy_level: Option<AutonomyLevel>,
    pub require_approval_for: Option<Vec<ActionCategory>>,
}

/// Response for a policy violation record.
#[derive(Debug, Serialize)]
pub struct PolicyViolationRecord {
    pub id: String,
    pub session_id: Option<String>,
    pub machine_id: String,
    pub user_id: String,
    pub action: String,
    pub policy_scope: String,
    pub reason: String,
    pub repo_path: Option<String>,
    pub timestamp: String,
}

/// Query params for repo-policy endpoints (repo_path as query param).
#[derive(Debug, Deserialize)]
pub struct RepoPolicyQuery {
    pub repo_path: String,
}

/// Query params for GET /api/machines/{id}/policy/effective.
#[derive(Debug, Deserialize)]
pub struct EffectivePolicyQuery {
    pub repo_path: Option<String>,
}

/// Merged effective policy returned by the effective endpoint.
#[derive(Debug, Serialize)]
pub struct EffectivePolicy {
    pub allowed_categories: Vec<ActionCategory>,
    pub blocked_categories: Vec<ActionCategory>,
    pub max_autonomy_level: AutonomyLevel,
    pub session_mode: SessionMode,
    pub require_approval_for: Vec<ActionCategory>,
}

/// Query params for GET /api/policy-violations.
#[derive(Debug, Deserialize)]
pub struct ListViolationsQuery {
    pub machine_id: Option<String>,
    pub session_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Response for policy violations list.
#[derive(Debug, Serialize)]
pub struct ViolationsListResponse {
    pub violations: Vec<PolicyViolationRecord>,
    pub total: i64,
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
    /// Optional machine ID for re-registration of a previously registered machine.
    pub machine_id: Option<String>,
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
    pub outcome: Option<String>,
    pub search: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
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
    pub outcome_status: Option<String>,
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

// -- Session output event types --

/// Type of session output event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionOutputEventType {
    OutputLine,
    ApprovalRequest,
    ApprovalResponse,
    GuidanceInjected,
    StateChanged,
    PolicyViolation,
}

impl SessionOutputEventType {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OutputLine => "output_line",
            Self::ApprovalRequest => "approval_request",
            Self::ApprovalResponse => "approval_response",
            Self::GuidanceInjected => "guidance_injected",
            Self::StateChanged => "state_changed",
            Self::PolicyViolation => "policy_violation",
        }
    }
}

impl std::fmt::Display for SessionOutputEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for SessionOutputEventType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "output_line" => Ok(Self::OutputLine),
            "approval_request" => Ok(Self::ApprovalRequest),
            "approval_response" => Ok(Self::ApprovalResponse),
            "guidance_injected" => Ok(Self::GuidanceInjected),
            "state_changed" => Ok(Self::StateChanged),
            "policy_violation" => Ok(Self::PolicyViolation),
            other => Err(format!("unknown session output event type: {other}")),
        }
    }
}

/// Category for output line events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputCategory {
    Info,
    Warning,
    Error,
    Summary,
}

impl OutputCategory {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Summary => "summary",
        }
    }
}

impl std::fmt::Display for OutputCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for OutputCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "info" => Ok(Self::Info),
            "warning" => Ok(Self::Warning),
            "error" => Ok(Self::Error),
            "summary" => Ok(Self::Summary),
            other => Err(format!("unknown output category: {other}")),
        }
    }
}

/// Type of guidance injection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InjectionType {
    Normal,
    SideNote,
    Interrupt,
}

impl InjectionType {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::SideNote => "side_note",
            Self::Interrupt => "interrupt",
        }
    }
}

impl std::fmt::Display for InjectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for InjectionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "normal" => Ok(Self::Normal),
            "side_note" => Ok(Self::SideNote),
            "interrupt" => Ok(Self::Interrupt),
            other => Err(format!("unknown injection type: {other}")),
        }
    }
}

/// Approval status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Responded,
    Expired,
}

impl ApprovalStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Responded => "responded",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ApprovalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ApprovalStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "responded" => Ok(Self::Responded),
            "expired" => Ok(Self::Expired),
            other => Err(format!("unknown approval status: {other}")),
        }
    }
}

/// A session output event (high-volume event stream).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionOutputEvent {
    pub id: String,
    pub session_id: String,
    pub event_type: SessionOutputEventType,
    pub category: Option<OutputCategory>,
    pub content: String,
    pub metadata: Option<String>,
    pub sequence_num: i64,
    pub timestamp: String,
}

/// A pending approval request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingApproval {
    pub id: String,
    pub session_id: String,
    pub question: String,
    pub options: String,
    pub context: Option<String>,
    pub status: ApprovalStatus,
    pub response_choice: Option<String>,
    pub response_note: Option<String>,
    pub created_at: String,
    pub responded_at: Option<String>,
}

/// Single event in an ingestion batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestEventItem {
    pub event_type: SessionOutputEventType,
    pub category: Option<OutputCategory>,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
}

/// Request body for POST /api/sessions/{id}/events (batch ingestion).
#[derive(Debug, Deserialize)]
pub struct IngestEventsRequest {
    pub events: Vec<IngestEventItem>,
}

/// Request for POST /api/sessions/{id}/respond.
#[derive(Debug, Deserialize)]
pub struct RespondToApprovalRequest {
    pub approval_id: String,
    pub choice: String,
    pub note: Option<String>,
}

/// Request for POST /api/sessions/{id}/inject.
#[derive(Debug, Deserialize)]
pub struct InjectGuidanceRequest {
    pub message: String,
    pub injection_type: InjectionType,
}

/// Query params for GET /api/sessions/{id}/events.
#[derive(Debug, Deserialize)]
pub struct QueryEventsParams {
    pub since_sequence: Option<i64>,
    pub limit: Option<i64>,
    pub event_type: Option<String>,
}

/// Outcome record written when a session reaches a terminal state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionOutcome {
    pub id: String,
    pub session_id: String,
    pub status: String,
    pub summary: String,
    pub artifacts: String,
    pub next_steps: String,
    pub event_count: i64,
    pub intervention_count: i64,
    pub duration_seconds: i64,
    pub created_at: String,
}

// -- Notification types --

/// A notification for a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub user_id: String,
    pub session_id: Option<String>,
    pub notification_type: String,
    pub priority: String,
    pub title: String,
    pub body: String,
    pub deep_link: Option<String>,
    pub read_at: Option<String>,
    pub dismissed_at: Option<String>,
    pub created_at: String,
}

/// A registered device token for push notifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceToken {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub platform: String,
    pub created_at: String,
}

/// Query params for GET /api/notifications.
#[derive(Debug, Deserialize)]
pub struct ListNotificationsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Request body for POST /api/devices.
#[derive(Debug, Deserialize)]
pub struct RegisterDeviceRequest {
    pub token: String,
    pub platform: String,
}

/// Response for GET /api/notifications/unread-count.
#[derive(Debug, Serialize)]
pub struct UnreadCountResponse {
    pub count: i64,
}

// -- Machine log types --

/// A machine-level log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineLogEntry {
    pub id: String,
    pub machine_id: String,
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
    pub fields_json: Option<String>,
}

/// Single log in an ingestion batch.
#[derive(Debug, Deserialize)]
pub struct IngestLogItem {
    pub level: String,
    pub target: String,
    pub message: String,
    pub fields: Option<serde_json::Value>,
    pub timestamp: Option<String>,
}

/// Request body for POST /api/machines/{id}/logs.
#[derive(Debug, Deserialize)]
pub struct IngestLogsRequest {
    pub logs: Vec<IngestLogItem>,
}

/// Query params for GET /api/machines/{id}/logs.
#[derive(Debug, Deserialize)]
pub struct QueryLogsParams {
    pub level: Option<String>,
    pub since: Option<String>,
    pub target: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
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

    #[test]
    fn test_action_category_roundtrip() {
        for cat in ActionCategory::all() {
            let s = cat.as_str();
            let parsed: ActionCategory = s.parse().unwrap();
            assert_eq!(parsed, cat);
        }
    }

    #[test]
    fn test_session_mode_roundtrip() {
        for mode in [SessionMode::Normal, SessionMode::Restricted, SessionMode::Elevated] {
            let s = mode.as_str();
            let parsed: SessionMode = s.parse().unwrap();
            assert_eq!(parsed, mode);
        }
    }

    #[test]
    fn test_is_action_allowed_basic() {
        let policy = MachinePolicy {
            id: "p1".into(),
            machine_id: "m1".into(),
            allowed_categories: ActionCategory::all(),
            blocked_categories: vec![],
            max_autonomy_level: AutonomyLevel::Autonomous,
            session_mode: SessionMode::Normal,
            require_approval_for: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        assert!(is_action_allowed(&ActionCategory::ReadFiles, &policy, None).is_ok());
        assert!(is_action_allowed(&ActionCategory::WriteFiles, &policy, None).is_ok());
    }

    #[test]
    fn test_is_action_blocked_by_machine() {
        let policy = MachinePolicy {
            id: "p1".into(),
            machine_id: "m1".into(),
            allowed_categories: ActionCategory::all(),
            blocked_categories: vec![ActionCategory::InstallPackages],
            max_autonomy_level: AutonomyLevel::Autonomous,
            session_mode: SessionMode::Normal,
            require_approval_for: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        let result = is_action_allowed(&ActionCategory::InstallPackages, &policy, None);
        assert!(result.is_err());
        let violation = result.unwrap_err();
        assert_eq!(violation.policy_scope, "machine");
    }

    #[test]
    fn test_is_action_blocked_by_repo() {
        let machine_policy = MachinePolicy {
            id: "p1".into(),
            machine_id: "m1".into(),
            allowed_categories: ActionCategory::all(),
            blocked_categories: vec![],
            max_autonomy_level: AutonomyLevel::Autonomous,
            session_mode: SessionMode::Normal,
            require_approval_for: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        let repo_policy = RepoPolicy {
            id: "rp1".into(),
            machine_id: "m1".into(),
            repo_path: "/project".into(),
            allowed_categories: vec![],
            blocked_categories: vec![ActionCategory::GitOperations],
            max_autonomy_level: None,
            require_approval_for: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        let result = is_action_allowed(&ActionCategory::GitOperations, &machine_policy, Some(&repo_policy));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().policy_scope, "repo");
    }

    #[test]
    fn test_repo_narrows_allowed_list() {
        let machine_policy = MachinePolicy {
            id: "p1".into(),
            machine_id: "m1".into(),
            allowed_categories: ActionCategory::all(),
            blocked_categories: vec![],
            max_autonomy_level: AutonomyLevel::Autonomous,
            session_mode: SessionMode::Normal,
            require_approval_for: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        let repo_policy = RepoPolicy {
            id: "rp1".into(),
            machine_id: "m1".into(),
            repo_path: "/project".into(),
            allowed_categories: vec![ActionCategory::ReadFiles, ActionCategory::WriteFiles],
            blocked_categories: vec![],
            max_autonomy_level: None,
            require_approval_for: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        // ReadFiles allowed by both
        assert!(is_action_allowed(&ActionCategory::ReadFiles, &machine_policy, Some(&repo_policy)).is_ok());
        // GitOperations allowed by machine but not in repo allowed list
        assert!(is_action_allowed(&ActionCategory::GitOperations, &machine_policy, Some(&repo_policy)).is_err());
    }

    #[test]
    fn test_autonomy_allowed() {
        let policy = MachinePolicy {
            id: "p1".into(),
            machine_id: "m1".into(),
            allowed_categories: vec![],
            blocked_categories: vec![],
            max_autonomy_level: AutonomyLevel::Autonomous,
            session_mode: SessionMode::Normal,
            require_approval_for: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        assert!(is_autonomy_allowed(&AutonomyLevel::Normal, &policy, None).is_ok());
        assert!(is_autonomy_allowed(&AutonomyLevel::Autonomous, &policy, None).is_ok());
    }

    #[test]
    fn test_autonomy_blocked_by_machine() {
        let policy = MachinePolicy {
            id: "p1".into(),
            machine_id: "m1".into(),
            allowed_categories: vec![],
            blocked_categories: vec![],
            max_autonomy_level: AutonomyLevel::Normal,
            session_mode: SessionMode::Normal,
            require_approval_for: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        assert!(is_autonomy_allowed(&AutonomyLevel::Normal, &policy, None).is_ok());
        assert!(is_autonomy_allowed(&AutonomyLevel::Autonomous, &policy, None).is_err());
    }

    #[test]
    fn test_session_output_event_type_roundtrip() {
        for t in [
            SessionOutputEventType::OutputLine,
            SessionOutputEventType::ApprovalRequest,
            SessionOutputEventType::ApprovalResponse,
            SessionOutputEventType::GuidanceInjected,
            SessionOutputEventType::StateChanged,
            SessionOutputEventType::PolicyViolation,
        ] {
            let s = t.as_str();
            let parsed: SessionOutputEventType = s.parse().unwrap();
            assert_eq!(parsed, t);
        }
    }

    #[test]
    fn test_output_category_roundtrip() {
        for c in [
            OutputCategory::Info,
            OutputCategory::Warning,
            OutputCategory::Error,
            OutputCategory::Summary,
        ] {
            let s = c.as_str();
            let parsed: OutputCategory = s.parse().unwrap();
            assert_eq!(parsed, c);
        }
    }

    #[test]
    fn test_injection_type_roundtrip() {
        for t in [
            InjectionType::Normal,
            InjectionType::SideNote,
            InjectionType::Interrupt,
        ] {
            let s = t.as_str();
            let parsed: InjectionType = s.parse().unwrap();
            assert_eq!(parsed, t);
        }
    }

    #[test]
    fn test_approval_status_roundtrip() {
        for s in [
            ApprovalStatus::Pending,
            ApprovalStatus::Responded,
            ApprovalStatus::Expired,
        ] {
            let str_val = s.as_str();
            let parsed: ApprovalStatus = str_val.parse().unwrap();
            assert_eq!(parsed, s);
        }
    }

    #[test]
    fn test_ingest_event_item_serialization() {
        let item = IngestEventItem {
            event_type: SessionOutputEventType::OutputLine,
            category: Some(OutputCategory::Info),
            content: "hello world".into(),
            metadata: Some(serde_json::json!({"line": 42})),
        };
        let json = serde_json::to_string(&item).unwrap();
        let parsed: IngestEventItem = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.event_type, SessionOutputEventType::OutputLine);
        assert_eq!(parsed.category, Some(OutputCategory::Info));
        assert_eq!(parsed.content, "hello world");
        assert_eq!(parsed.metadata, Some(serde_json::json!({"line": 42})));
    }

    #[test]
    fn test_ingest_event_item_minimal() {
        let json = r#"{"event_type":"state_changed","content":"running","category":null,"metadata":null}"#;
        let parsed: IngestEventItem = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.event_type, SessionOutputEventType::StateChanged);
        assert_eq!(parsed.content, "running");
        assert!(parsed.category.is_none());
        assert!(parsed.metadata.is_none());
    }

    #[test]
    fn test_autonomy_blocked_by_repo() {
        let machine_policy = MachinePolicy {
            id: "p1".into(),
            machine_id: "m1".into(),
            allowed_categories: vec![],
            blocked_categories: vec![],
            max_autonomy_level: AutonomyLevel::Autonomous,
            session_mode: SessionMode::Normal,
            require_approval_for: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        let repo_policy = RepoPolicy {
            id: "rp1".into(),
            machine_id: "m1".into(),
            repo_path: "/project".into(),
            allowed_categories: vec![],
            blocked_categories: vec![],
            max_autonomy_level: Some(AutonomyLevel::Normal),
            require_approval_for: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        assert!(is_autonomy_allowed(&AutonomyLevel::Autonomous, &machine_policy, Some(&repo_policy)).is_err());
    }
}

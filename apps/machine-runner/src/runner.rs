use std::sync::Arc;
use std::time::Duration;

use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;

use crate::client::{ClientError, CommandResponse, ControlClient, HeartbeatRequest, RegisterRequest};
use crate::config::RunnerConfig;
use crate::discovery;
use crate::injection;
use crate::output_capture::{OutputCapture, OutputEvent};
use crate::policy::LocalPolicyCache;
use crate::supervisor::ProcessSupervisor;

/// The possible states of the machine runner lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunnerState {
    Registering,
    Pending,
    Active,
    Stopped,
}

/// The machine runner, responsible for registration and heartbeat lifecycle.
pub struct Runner {
    config: RunnerConfig,
    client: Arc<ControlClient>,
    state: RunnerState,
    machine_id: Option<String>,
    supervisor: ProcessSupervisor,
    policy_cache: LocalPolicyCache,
    output_event_tx: tokio::sync::mpsc::Sender<(String, Vec<OutputEvent>)>,
}

impl Runner {
    /// Create a new runner from the given configuration.
    ///
    /// Returns the runner and a receiver for supervisor lifecycle events.
    /// Also spawns a background task that forwards output events from captured
    /// process streams to the control service.
    pub fn new(config: RunnerConfig) -> (Self, tokio::sync::mpsc::Receiver<crate::supervisor::SupervisorEvent>) {
        let client = Arc::new(ControlClient::new(&config.control_service_url, &config.api_token));
        let (supervisor, event_rx) = ProcessSupervisor::new();
        let policy_cache = LocalPolicyCache::new(300);

        // Channel for output events from captured process streams
        let (output_tx, mut output_rx) =
            tokio::sync::mpsc::channel::<(String, Vec<OutputEvent>)>(128);

        // Spawn a background task to forward output events to the control service
        let forwarder_client = Arc::clone(&client);
        tokio::spawn(async move {
            while let Some((session_id, events)) = output_rx.recv().await {
                if let Err(e) = forwarder_client
                    .post_session_events(&session_id, &events)
                    .await
                {
                    tracing::warn!(
                        session_id = %session_id,
                        event_count = events.len(),
                        error = %e,
                        "Failed to post session output events, dropping batch"
                    );
                }
            }
            tracing::debug!("Output event forwarder task exiting");
        });

        let runner = Self {
            config,
            client,
            state: RunnerState::Registering,
            machine_id: None,
            supervisor,
            policy_cache,
            output_event_tx: output_tx,
        };
        (runner, event_rx)
    }

    /// Returns the current state of the runner.
    #[must_use]
    pub fn state(&self) -> &RunnerState {
        &self.state
    }

    /// Returns the machine ID assigned by the control service, if registered.
    #[must_use]
    pub fn machine_id(&self) -> Option<&str> {
        self.machine_id.as_deref()
    }

    /// Run the full lifecycle: register, then heartbeat loop.
    ///
    /// # Errors
    ///
    /// Returns an error if registration fails fatally or if the heartbeat
    /// loop terminates due to an unrecoverable error.
    pub async fn run(&mut self) -> anyhow::Result<()> {
        self.register().await?;

        // Fetch initial policy if already active after registration
        if self.state == RunnerState::Active {
            self.refresh_policy().await;
        }

        self.heartbeat_loop().await
    }

    async fn register(&mut self) -> anyhow::Result<()> {
        let repos = discovery::discover_repos(&self.config.repo_paths())?;
        let request = build_register_request(&self.config.machine_name, &repos);

        tracing::info!(
            name = %self.config.machine_name,
            repos = repos.len(),
            "Registering machine with control service"
        );

        let response = self.client.register(&request).await?;

        self.machine_id = Some(response.id.clone());
        self.state = match response.status.as_str() {
            "trusted" | "active" => RunnerState::Active,
            _ => RunnerState::Pending,
        };

        tracing::info!(
            machine_id = %response.id,
            status = %response.status,
            state = ?self.state,
            "Registration complete"
        );

        Ok(())
    }

    async fn heartbeat_loop(&mut self) -> anyhow::Result<()> {
        let interval = Duration::from_secs(self.config.heartbeat_interval_secs);
        let mut backoff = BackoffState::new();

        while self.state == RunnerState::Pending || self.state == RunnerState::Active {
            tokio::time::sleep(interval).await;

            let result = self.send_heartbeat().await;
            match result {
                Ok(()) => {
                    backoff.reset();
                    // Poll for and process commands when active
                    if self.state == RunnerState::Active {
                        // Refresh policy if stale
                        if self.policy_cache.needs_refresh() {
                            self.refresh_policy().await;
                        }
                        self.poll_and_process_commands().await;
                    }
                }
                Err(HeartbeatOutcome::Revoked) => {
                    tracing::warn!("Machine revoked by control service, stopping");
                    self.state = RunnerState::Stopped;
                    break;
                }
                Err(HeartbeatOutcome::Pending) => {
                    tracing::debug!("Machine still pending approval");
                    // State stays as Pending; no backoff needed
                }
                Err(HeartbeatOutcome::NetworkError(e)) => {
                    let delay = backoff.next_delay();
                    tracing::warn!(
                        error = %e,
                        retry_in_secs = delay.as_secs(),
                        "Heartbeat failed, retrying with backoff"
                    );
                    tokio::time::sleep(delay).await;
                }
            }
        }

        tracing::info!(state = ?self.state, "Runner heartbeat loop exited");
        Ok(())
    }

    /// Fetch the machine policy from the control service and update the local cache.
    ///
    /// Logs a warning if the fetch fails but does not propagate the error,
    /// allowing the runner to continue with the previously cached policy.
    async fn refresh_policy(&mut self) {
        let machine_id = match self.machine_id.as_deref() {
            Some(id) => id.to_string(),
            None => return,
        };

        match self.client.fetch_policy(&machine_id).await {
            Ok(policy) => {
                tracing::info!(
                    machine_id = %machine_id,
                    policy_id = %policy.id,
                    max_autonomy = %policy.max_autonomy_level,
                    "Refreshed machine policy"
                );
                self.policy_cache.update(policy);
            }
            Err(e) => {
                tracing::warn!(
                    machine_id = %machine_id,
                    error = %e,
                    "Failed to fetch machine policy, using cached version"
                );
            }
        }
    }

    async fn send_heartbeat(&mut self) -> Result<(), HeartbeatOutcome> {
        let machine_id = self
            .machine_id
            .as_deref()
            .expect("machine_id must be set before heartbeat loop");

        let repos = discovery::discover_repos(&self.config.repo_paths())
            .map_err(|e| HeartbeatOutcome::NetworkError(e.to_string()))?;

        let request = HeartbeatRequest { repos };

        match self.client.heartbeat(machine_id, &request).await {
            Ok(response) => {
                handle_heartbeat_success(&mut self.state, &response.status);
                Ok(())
            }
            Err(ClientError::MachineRevoked) => Err(HeartbeatOutcome::Revoked),
            Err(ClientError::MachinePending) => Err(HeartbeatOutcome::Pending),
            Err(e) => Err(HeartbeatOutcome::NetworkError(e.to_string())),
        }
    }

    async fn poll_and_process_commands(&mut self) {
        let machine_id = self
            .machine_id
            .as_deref()
            .expect("machine_id must be set before polling commands")
            .to_string();

        let commands = match self.client.fetch_commands(&machine_id).await {
            Ok(cmds) => cmds,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to fetch commands");
                return;
            }
        };

        for cmd in commands {
            self.process_command(&machine_id, &cmd).await;
        }
    }

    async fn process_command(&mut self, machine_id: &str, cmd: &CommandResponse) {
        tracing::info!(
            command_id = %cmd.command_id,
            command_type = %cmd.command_type,
            "Received command"
        );

        let result = match cmd.command_type.as_str() {
            "start_session" => self.handle_start_session(cmd).await,
            "stop" => self.handle_session_command(cmd, SessionAction::Stop).await,
            "force_stop" => self.handle_session_command(cmd, SessionAction::ForceStop).await,
            "pause" => self.handle_session_command(cmd, SessionAction::Pause).await,
            "resume" => self.handle_session_command(cmd, SessionAction::Resume).await,
            "respond" => self.handle_respond(cmd).await,
            "inject" => self.handle_inject(cmd).await,
            other => {
                tracing::warn!(
                    command_id = %cmd.command_id,
                    command_type = other,
                    "Unknown command type, acknowledging anyway"
                );
                Ok(())
            }
        };

        if let Err(e) = &result {
            tracing::error!(
                command_id = %cmd.command_id,
                command_type = %cmd.command_type,
                error = %e,
                "Failed to execute command"
            );
        }

        // Acknowledge the command regardless of outcome
        if let Err(e) = self.client.ack_command(machine_id, &cmd.command_id).await {
            tracing::warn!(
                command_id = %cmd.command_id,
                error = %e,
                "Failed to acknowledge command"
            );
        }
    }

    async fn handle_start_session(&mut self, cmd: &CommandResponse) -> anyhow::Result<()> {
        let payload = cmd
            .payload
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("start_session command missing payload"))?;

        let session_id = payload
            .get("session_id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("start_session payload missing session_id"))?;

        let repo_path = payload
            .get("repo_path")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("start_session payload missing repo_path"))?;

        let instructions = payload
            .get("instructions")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");

        let autonomy_level = payload
            .get("autonomy_level")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("normal");

        // Enforce local policy: validate autonomy level
        if let Err(detail) = self.policy_cache.validate_autonomy(autonomy_level) {
            tracing::warn!(
                command_id = %cmd.command_id,
                session_id = session_id,
                autonomy = autonomy_level,
                detail = %detail,
                "Session rejected by local policy"
            );

            // Report failure to the control service
            let _ = self
                .client
                .report_session_state(
                    session_id,
                    "failed",
                    Some(serde_json::json!({
                        "reason": "local_policy_violation",
                        "detail": detail
                    })),
                )
                .await;

            return Ok(());
        }

        tracing::info!(
            command_id = %cmd.command_id,
            session_id = session_id,
            repo_path = repo_path,
            autonomy = autonomy_level,
            "Starting session via supervisor"
        );

        let (stdout, stderr) = self
            .supervisor
            .start_session(session_id, repo_path, instructions, autonomy_level)
            .await?;

        // Wire up output capture for the session's stdout/stderr
        let capture = OutputCapture::new(
            session_id.to_string(),
            stdout,
            stderr,
            self.output_event_tx.clone(),
        );
        capture.start();

        tracing::info!(
            session_id = session_id,
            "Output capture started for session"
        );

        Ok(())
    }

    async fn handle_session_command(
        &mut self,
        cmd: &CommandResponse,
        action: SessionAction,
    ) -> anyhow::Result<()> {
        let session_id = cmd
            .payload
            .as_ref()
            .and_then(|p| p.get("session_id"))
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("{:?} command missing session_id in payload", action))?;

        tracing::info!(
            command_id = %cmd.command_id,
            session_id = session_id,
            action = ?action,
            "Executing session command via supervisor"
        );

        match action {
            SessionAction::Stop => self.supervisor.stop_session(session_id).await,
            SessionAction::ForceStop => self.supervisor.force_stop_session(session_id).await,
            SessionAction::Pause => self.supervisor.pause_session(session_id),
            SessionAction::Resume => self.supervisor.resume_session(session_id),
        }
    }

    /// Handle an approval response command by writing the chosen response to
    /// the session's stdin.
    ///
    /// Expected payload:
    /// ```json
    /// {
    ///   "session_id": "...",
    ///   "approval_id": "...",
    ///   "choice": "yes",
    ///   "note": "optional note"
    /// }
    /// ```
    async fn handle_respond(&mut self, cmd: &CommandResponse) -> anyhow::Result<()> {
        let payload = cmd
            .payload
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("respond command missing payload"))?;

        let session_id = payload
            .get("session_id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("respond payload missing session_id"))?;

        let approval_id = payload
            .get("approval_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");

        let choice = payload
            .get("choice")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("respond payload missing choice"))?;

        let note = payload
            .get("note")
            .and_then(serde_json::Value::as_str);

        tracing::info!(
            command_id = %cmd.command_id,
            session_id = session_id,
            approval_id = approval_id,
            choice = choice,
            "Writing approval response to session stdin"
        );

        let formatted = injection::format_approval_response(choice);
        self.supervisor
            .write_to_stdin(session_id, formatted.as_bytes())
            .await?;

        // Emit a confirmation event
        let events = vec![OutputEvent {
            event_type: "approval_response".to_string(),
            category: Some("info".to_string()),
            content: format!("Approval {approval_id} responded with: {choice}"),
            metadata: Some(serde_json::json!({
                "approval_id": approval_id,
                "choice": choice,
                "note": note,
            })),
            sequence: 0,
        }];

        if let Err(e) = self.output_event_tx.send((session_id.to_string(), events)).await {
            tracing::warn!(
                session_id = session_id,
                error = %e,
                "Failed to emit approval response confirmation event"
            );
        }

        Ok(())
    }

    /// Handle a guidance injection command by writing a formatted message to
    /// the session's stdin.
    ///
    /// Expected payload:
    /// ```json
    /// {
    ///   "session_id": "...",
    ///   "message": "...",
    ///   "injection_type": "normal|side_note|interrupt"
    /// }
    /// ```
    async fn handle_inject(&mut self, cmd: &CommandResponse) -> anyhow::Result<()> {
        let payload = cmd
            .payload
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("inject command missing payload"))?;

        let session_id = payload
            .get("session_id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("inject payload missing session_id"))?;

        let message = payload
            .get("message")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("inject payload missing message"))?;

        let injection_type = payload
            .get("injection_type")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("normal");

        tracing::info!(
            command_id = %cmd.command_id,
            session_id = session_id,
            injection_type = injection_type,
            "Writing injection to session stdin"
        );

        let formatted = injection::format_injection(message, injection_type);
        self.supervisor
            .write_to_stdin(session_id, formatted.as_bytes())
            .await?;

        // For interrupt type, optionally send SIGUSR1 to nudge the process
        if injection_type == "interrupt" {
            if let Some(raw_pid) = self.supervisor.session_pid(session_id) {
                let pid = Pid::from_raw(raw_pid as i32);
                if let Err(e) = signal::kill(pid, Signal::SIGUSR1) {
                    tracing::debug!(
                        session_id = session_id,
                        error = %e,
                        "Failed to send SIGUSR1 for interrupt injection (non-fatal)"
                    );
                }
            }
        }

        // Emit a confirmation event
        let events = vec![OutputEvent {
            event_type: "injection".to_string(),
            category: Some("info".to_string()),
            content: format!("Injected {injection_type} message into session"),
            metadata: Some(serde_json::json!({
                "injection_type": injection_type,
                "message": message,
            })),
            sequence: 0,
        }];

        if let Err(e) = self.output_event_tx.send((session_id.to_string(), events)).await {
            tracing::warn!(
                session_id = session_id,
                error = %e,
                "Failed to emit injection confirmation event"
            );
        }

        Ok(())
    }
}

/// Actions that can be performed on a supervised session.
#[derive(Debug)]
enum SessionAction {
    Stop,
    ForceStop,
    Pause,
    Resume,
}

fn build_register_request(
    machine_name: &str,
    repos: &[discovery::RepoInfo],
) -> RegisterRequest {
    let platform = format!("{}/{}", std::env::consts::OS, std::env::consts::ARCH);
    RegisterRequest {
        name: machine_name.to_string(),
        platform,
        capabilities: Some("claude_code".to_string()),
        repos: repos.to_vec(),
    }
}

fn handle_heartbeat_success(state: &mut RunnerState, response_status: &str) {
    if *state == RunnerState::Pending && response_status != "pending" {
        tracing::info!(
            new_status = response_status,
            "Machine approved, transitioning to Active"
        );
        *state = RunnerState::Active;
    }
}

/// Internal outcome type for heartbeat attempts.
enum HeartbeatOutcome {
    Revoked,
    Pending,
    NetworkError(String),
}

/// Manages exponential backoff for network retries.
struct BackoffState {
    current_secs: u64,
}

impl BackoffState {
    const INITIAL_SECS: u64 = 1;
    const MAX_SECS: u64 = 60;

    fn new() -> Self {
        Self {
            current_secs: Self::INITIAL_SECS,
        }
    }

    fn next_delay(&mut self) -> Duration {
        let delay = Duration::from_secs(self.current_secs);
        self.current_secs = (self.current_secs * 2).min(Self::MAX_SECS);
        delay
    }

    fn reset(&mut self) {
        self.current_secs = Self::INITIAL_SECS;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initial_state_is_registering() {
        let config = test_config();
        let (runner, _rx) = Runner::new(config);
        assert_eq!(*runner.state(), RunnerState::Registering);
        assert!(runner.machine_id().is_none());
    }

    #[test]
    fn test_runner_state_equality() {
        assert_eq!(RunnerState::Registering, RunnerState::Registering);
        assert_eq!(RunnerState::Pending, RunnerState::Pending);
        assert_eq!(RunnerState::Active, RunnerState::Active);
        assert_eq!(RunnerState::Stopped, RunnerState::Stopped);
        assert_ne!(RunnerState::Registering, RunnerState::Active);
    }

    #[test]
    fn test_heartbeat_success_transitions_pending_to_active() {
        let mut state = RunnerState::Pending;
        handle_heartbeat_success(&mut state, "ok");
        assert_eq!(state, RunnerState::Active);
    }

    #[test]
    fn test_heartbeat_success_keeps_pending_when_still_pending() {
        let mut state = RunnerState::Pending;
        handle_heartbeat_success(&mut state, "pending");
        assert_eq!(state, RunnerState::Pending);
    }

    #[test]
    fn test_heartbeat_success_leaves_active_unchanged() {
        let mut state = RunnerState::Active;
        handle_heartbeat_success(&mut state, "ok");
        assert_eq!(state, RunnerState::Active);
    }

    #[test]
    fn test_build_register_request() {
        let repos = vec![crate::discovery::RepoInfo {
            repo_name: "test".to_string(),
            repo_path: "/path/test".to_string(),
            cadre_managed: true,
        }];

        let req = build_register_request("my-machine", &repos);
        assert_eq!(req.name, "my-machine");
        assert!(!req.platform.is_empty());
        assert!(req.platform.contains('/'));
        assert_eq!(req.repos.len(), 1);
    }

    #[test]
    fn test_backoff_exponential_with_cap() {
        let mut backoff = BackoffState::new();

        assert_eq!(backoff.next_delay(), Duration::from_secs(1));
        assert_eq!(backoff.next_delay(), Duration::from_secs(2));
        assert_eq!(backoff.next_delay(), Duration::from_secs(4));
        assert_eq!(backoff.next_delay(), Duration::from_secs(8));
        assert_eq!(backoff.next_delay(), Duration::from_secs(16));
        assert_eq!(backoff.next_delay(), Duration::from_secs(32));
        assert_eq!(backoff.next_delay(), Duration::from_secs(60));
        // Should stay at 60 (the cap)
        assert_eq!(backoff.next_delay(), Duration::from_secs(60));
    }

    #[test]
    fn test_backoff_reset() {
        let mut backoff = BackoffState::new();
        let _ = backoff.next_delay(); // 1
        let _ = backoff.next_delay(); // 2
        let _ = backoff.next_delay(); // 4
        backoff.reset();
        assert_eq!(backoff.next_delay(), Duration::from_secs(1));
    }

    fn test_config() -> RunnerConfig {
        RunnerConfig {
            control_service_url: "http://localhost:8080".to_string(),
            machine_name: "test-machine".to_string(),
            api_token: "test-token".to_string(),
            heartbeat_interval_secs: 5,
            repo_directories: vec![],
        }
    }
}

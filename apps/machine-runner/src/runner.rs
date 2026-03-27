use std::time::Duration;

use crate::client::{ClientError, ControlClient, HeartbeatRequest, RegisterRequest};
use crate::config::RunnerConfig;
use crate::discovery;

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
    client: ControlClient,
    state: RunnerState,
    machine_id: Option<String>,
}

impl Runner {
    /// Create a new runner from the given configuration.
    pub fn new(config: RunnerConfig) -> Self {
        let client = ControlClient::new(&config.control_service_url, &config.api_token);
        Self {
            config,
            client,
            state: RunnerState::Registering,
            machine_id: None,
        }
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

    #[test]
    fn test_initial_state_is_registering() {
        let config = test_config();
        let runner = Runner::new(config);
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

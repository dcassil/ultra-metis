use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::{watch, RwLock};
use tokio::task::JoinHandle;

use crate::client::ControlClient;
use crate::runner::Runner;
use crate::settings::Settings;
use crate::state_reporter::StateReporter;

/// The external-facing status of the machine runner.
///
/// Consumers (CLI, Tauri UI, etc.) subscribe to status changes via a
/// `watch::Receiver<RunnerStatus>` obtained from `RunnerHandle::subscribe_status`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunnerStatus {
    /// The runner is not running.
    Stopped,
    /// The runner is attempting to register with the control service.
    Registering,
    /// Registration succeeded but the machine is awaiting approval.
    PendingApproval,
    /// The machine is approved and actively heartbeating.
    Active {
        machine_id: String,
        connected: bool,
        active_sessions: u32,
    },
    /// An error occurred in the runner lifecycle.
    Error(String),
}

/// A handle to the machine runner lifecycle, suitable for embedding in both
/// CLI binaries and GUI applications.
///
/// The `RunnerHandle` owns shared settings and token state, and can start/stop
/// the runner lifecycle as a background tokio task.
pub struct RunnerHandle {
    settings: Arc<RwLock<Settings>>,
    token: Arc<RwLock<String>>,
    status_tx: watch::Sender<RunnerStatus>,
    status_rx: watch::Receiver<RunnerStatus>,
    task_handle: Option<JoinHandle<()>>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl RunnerHandle {
    /// Create a new handle with initial `Stopped` status.
    pub fn new(settings: Settings, token: String) -> Self {
        let (status_tx, status_rx) = watch::channel(RunnerStatus::Stopped);
        Self {
            settings: Arc::new(RwLock::new(settings)),
            token: Arc::new(RwLock::new(token)),
            status_tx,
            status_rx,
            task_handle: None,
            shutdown_tx: None,
        }
    }

    /// Spawn the runner lifecycle as a background tokio task.
    ///
    /// The task performs registration, then enters the heartbeat loop.
    /// Status updates are published through the watch channel.
    ///
    /// # Errors
    ///
    /// Returns an error if the runner is already running.
    pub async fn start(&mut self) -> anyhow::Result<()> {
        if self.is_running() {
            anyhow::bail!("Runner is already running");
        }

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        self.shutdown_tx = Some(shutdown_tx);

        let settings = Arc::clone(&self.settings);
        let token = Arc::clone(&self.token);
        let status_tx = self.status_tx.clone();

        let handle = tokio::spawn(async move {
            run_lifecycle(settings, token, status_tx, shutdown_rx).await;
        });

        self.task_handle = Some(handle);
        Ok(())
    }

    /// Send a shutdown signal and wait for the runner task to complete.
    ///
    /// # Errors
    ///
    /// Returns an error if the runner is not running.
    pub async fn stop(&mut self) -> anyhow::Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            // It's OK if the receiver is already gone (task finished on its own).
            let _ = tx.send(());
        }

        if let Some(handle) = self.task_handle.take() {
            // Wait for the task to finish, but don't propagate panics.
            let _ = handle.await;
        }

        let _ = self.status_tx.send(RunnerStatus::Stopped);
        Ok(())
    }

    /// Returns `true` if the background task is still active.
    pub fn is_running(&self) -> bool {
        self.task_handle
            .as_ref()
            .is_some_and(|h| !h.is_finished())
    }

    /// Read the current status.
    pub fn status(&self) -> RunnerStatus {
        self.status_rx.borrow().clone()
    }

    /// Get a watch receiver for live status updates.
    pub fn subscribe_status(&self) -> watch::Receiver<RunnerStatus> {
        self.status_rx.clone()
    }

    /// Replace the shared settings. The runner will pick up changes on its
    /// next iteration through the heartbeat loop.
    pub async fn update_settings(&self, new_settings: Settings) {
        let mut guard = self.settings.write().await;
        *guard = new_settings;
    }

    /// Read a snapshot of the current settings.
    pub async fn get_settings(&self) -> Settings {
        self.settings.read().await.clone()
    }

    /// Update the API token.
    pub async fn update_token(&self, new_token: String) {
        let mut guard = self.token.write().await;
        *guard = new_token;
    }
}

/// The core lifecycle function that runs in a spawned task.
async fn run_lifecycle(
    settings: Arc<RwLock<Settings>>,
    token: Arc<RwLock<String>>,
    status_tx: watch::Sender<RunnerStatus>,
    shutdown_rx: tokio::sync::oneshot::Receiver<()>,
) {
    let (mut runner, mut event_rx) =
        Runner::new(Arc::clone(&settings), Arc::clone(&token), status_tx.clone());

    // Create a StateReporter with its own client snapshot to report session state changes.
    let (url, tok) = {
        let s = settings.read().await;
        let t = token.read().await;
        (s.control_service_url.clone(), t.clone())
    };
    let reporter_client = ControlClient::new(&url, &tok);
    let mut reporter = StateReporter::new(reporter_client);

    // Spawn a task to consume supervisor events and report state changes.
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            tracing::info!(?event, "Supervisor event");
            reporter.handle_event(event).await;
        }
    });

    // Run the main lifecycle; propagate the shutdown receiver.
    if let Err(e) = runner.run(shutdown_rx).await {
        tracing::error!(error = %e, "Runner lifecycle exited with error");
        let _ = status_tx.send(RunnerStatus::Error(e.to_string()));
    } else {
        let _ = status_tx.send(RunnerStatus::Stopped);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_status_serde_roundtrip_stopped() {
        let status = RunnerStatus::Stopped;
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: RunnerStatus = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, RunnerStatus::Stopped));
    }

    #[test]
    fn test_runner_status_serde_roundtrip_active() {
        let status = RunnerStatus::Active {
            machine_id: "m-123".to_string(),
            connected: true,
            active_sessions: 2,
        };
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: RunnerStatus = serde_json::from_str(&json).unwrap();
        match deserialized {
            RunnerStatus::Active {
                machine_id,
                connected,
                active_sessions,
            } => {
                assert_eq!(machine_id, "m-123");
                assert!(connected);
                assert_eq!(active_sessions, 2);
            }
            other => panic!("Expected Active, got {other:?}"),
        }
    }

    #[test]
    fn test_runner_status_serde_roundtrip_error() {
        let status = RunnerStatus::Error("something broke".to_string());
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: RunnerStatus = serde_json::from_str(&json).unwrap();
        match deserialized {
            RunnerStatus::Error(msg) => assert_eq!(msg, "something broke"),
            other => panic!("Expected Error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_new_starts_stopped() {
        let handle = RunnerHandle::new(Settings::default(), "tok".to_string());
        assert!(matches!(handle.status(), RunnerStatus::Stopped));
        assert!(!handle.is_running());
    }

    #[tokio::test]
    async fn test_handle_update_and_get_settings() {
        let handle = RunnerHandle::new(Settings::default(), "tok".to_string());
        let mut s = handle.get_settings().await;
        assert_eq!(s.heartbeat_interval_secs, 30);

        s.heartbeat_interval_secs = 10;
        handle.update_settings(s).await;

        let updated = handle.get_settings().await;
        assert_eq!(updated.heartbeat_interval_secs, 10);
    }

    #[tokio::test]
    async fn test_handle_update_token() {
        let handle = RunnerHandle::new(Settings::default(), "old_token".to_string());
        handle.update_token("new_token".to_string()).await;

        let t = handle.token.read().await;
        assert_eq!(*t, "new_token");
    }

    #[tokio::test]
    async fn test_handle_subscribe_status() {
        let handle = RunnerHandle::new(Settings::default(), "tok".to_string());
        let rx = handle.subscribe_status();
        assert!(matches!(*rx.borrow(), RunnerStatus::Stopped));
    }

    #[tokio::test]
    async fn test_handle_stop_when_not_running() {
        let mut handle = RunnerHandle::new(Settings::default(), "tok".to_string());
        // Stopping when not running should not error.
        let result = handle.stop().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_start_twice_errors() {
        let mut handle = RunnerHandle::new(Settings::default(), "tok".to_string());
        // Start will spawn a task that will fail to connect, but that's fine --
        // we just need the task handle to be set.
        handle.start().await.unwrap();
        let result = handle.start().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already running"));
        // Cleanup
        handle.stop().await.ok();
    }
}

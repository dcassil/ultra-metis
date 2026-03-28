use std::collections::HashMap;
use std::process::Stdio;
use std::time::Duration;

use chrono::{DateTime, Utc};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use serde::{Deserialize, Serialize};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

/// Events emitted by the process supervisor when session state changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupervisorEvent {
    /// A session's process state has changed.
    StateChanged {
        session_id: String,
        new_state: String,
        metadata: Option<serde_json::Value>,
    },
}

/// The lifecycle state of a supervised process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessState {
    Running,
    Paused,
    Stopped,
}

impl std::fmt::Display for ProcessState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Running => write!(f, "running"),
            Self::Paused => write!(f, "paused"),
            Self::Stopped => write!(f, "stopped"),
        }
    }
}

/// A handle to a supervised child process.
///
/// The `Child` is owned by a background monitor task that calls `wait()`.
/// The supervisor retains only the PID for signal-based control.
#[allow(dead_code)] // session_id and started_at stored for logging/future session queries
struct ProcessHandle {
    session_id: String,
    pid: u32,
    started_at: DateTime<Utc>,
    state: ProcessState,
}

/// Supervises Claude CLI child processes for remote sessions.
///
/// Tracks active sessions and emits lifecycle events through an mpsc channel.
/// MVP: supports one session at a time, though the `HashMap` allows future
/// expansion to concurrent sessions.
pub struct ProcessSupervisor {
    sessions: HashMap<String, ProcessHandle>,
    event_tx: mpsc::Sender<SupervisorEvent>,
}

impl ProcessSupervisor {
    /// Create a new supervisor and the receiver end of its event channel.
    pub fn new() -> (Self, mpsc::Receiver<SupervisorEvent>) {
        let (tx, rx) = mpsc::channel(64);
        let supervisor = Self {
            sessions: HashMap::new(),
            event_tx: tx,
        };
        (supervisor, rx)
    }

    /// Spawn a Claude CLI session as a child process.
    ///
    /// The process runs `claude --print --output-format json -p "<instructions>"`
    /// in `repo_path`. Autonomy level is mapped to additional CLI flags.
    ///
    /// A background monitor task calls `child.wait()` and emits a
    /// `SupervisorEvent::StateChanged` with state `"stopped"` when the process exits.
    ///
    /// # Errors
    ///
    /// Returns an error if the process fails to spawn or if a session with the
    /// same ID is already active.
    pub async fn start_session(
        &mut self,
        session_id: &str,
        repo_path: &str,
        instructions: &str,
        autonomy_level: &str,
    ) -> anyhow::Result<()> {
        if self.sessions.contains_key(session_id) {
            anyhow::bail!("Session {session_id} is already active");
        }

        let mut cmd = Command::new("claude");
        cmd.arg("--print")
            .arg("--output-format")
            .arg("json")
            .arg("-p")
            .arg(instructions)
            .current_dir(repo_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        apply_autonomy_flags(&mut cmd, autonomy_level);

        let child = cmd.spawn()?;
        let pid = child
            .id()
            .ok_or_else(|| anyhow::anyhow!("Failed to get PID for spawned process"))?;

        tracing::info!(
            session_id = session_id,
            pid = pid,
            repo_path = repo_path,
            autonomy = autonomy_level,
            "Spawned Claude CLI session"
        );

        self.register_process(session_id, child, pid).await;
        Ok(())
    }

    /// Spawn a custom command as a session (used for testing).
    ///
    /// Same lifecycle management as `start_session`, but runs an arbitrary
    /// command instead of the Claude CLI.
    #[cfg(test)]
    async fn start_session_with_command(
        &mut self,
        session_id: &str,
        mut cmd: Command,
    ) -> anyhow::Result<()> {
        if self.sessions.contains_key(session_id) {
            anyhow::bail!("Session {session_id} is already active");
        }

        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = cmd.spawn()?;
        let pid = child
            .id()
            .ok_or_else(|| anyhow::anyhow!("Failed to get PID for spawned process"))?;

        self.register_process(session_id, child, pid).await;
        Ok(())
    }

    /// Internal helper: store handle, emit Running event, spawn monitor task.
    async fn register_process(&mut self, session_id: &str, child: Child, pid: u32) {
        let handle = ProcessHandle {
            session_id: session_id.to_string(),
            pid,
            started_at: Utc::now(),
            state: ProcessState::Running,
        };
        self.sessions.insert(session_id.to_string(), handle);

        // Emit started event
        let _ = self
            .event_tx
            .send(SupervisorEvent::StateChanged {
                session_id: session_id.to_string(),
                new_state: ProcessState::Running.to_string(),
                metadata: Some(serde_json::json!({ "pid": pid })),
            })
            .await;

        // Spawn a monitor task that owns the Child and calls wait().
        // This properly reaps the child process (avoids zombies) and emits
        // the stopped event when the process exits naturally.
        let monitor_session_id = session_id.to_string();
        let monitor_tx = self.event_tx.clone();
        tokio::spawn(async move {
            let exit_status = wait_for_child(child).await;
            tracing::info!(
                session_id = %monitor_session_id,
                exit = ?exit_status,
                "Monitored process exited"
            );
            let _ = monitor_tx
                .send(SupervisorEvent::StateChanged {
                    session_id: monitor_session_id,
                    new_state: ProcessState::Stopped.to_string(),
                    metadata: exit_status.map(|code| serde_json::json!({ "exit_code": code })),
                })
                .await;
        });
    }

    /// Gracefully stop a session: SIGTERM, wait up to 10 seconds, then SIGKILL.
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found.
    pub async fn stop_session(&mut self, session_id: &str) -> anyhow::Result<()> {
        let handle = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session {session_id} not found"))?;

        if handle.state == ProcessState::Stopped {
            self.sessions.remove(session_id);
            return Ok(());
        }

        let pid = Pid::from_raw(handle.pid as i32);

        // If paused, resume first so it can receive SIGTERM
        if handle.state == ProcessState::Paused {
            let _ = signal::kill(pid, Signal::SIGCONT);
        }

        tracing::info!(session_id = session_id, pid = handle.pid, "Stopping session (SIGTERM)");

        // Send SIGTERM
        let _ = signal::kill(pid, Signal::SIGTERM);

        // Wait up to 10 seconds for the process to exit
        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        loop {
            if signal::kill(pid, None).is_err() {
                // Process is gone
                break;
            }
            if tokio::time::Instant::now() >= deadline {
                tracing::warn!(
                    session_id = session_id,
                    pid = handle.pid,
                    "Session did not exit after SIGTERM, sending SIGKILL"
                );
                let _ = signal::kill(pid, Signal::SIGKILL);
                // Brief wait for SIGKILL to take effect
                tokio::time::sleep(Duration::from_millis(100)).await;
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        self.sessions.remove(session_id);
        Ok(())
    }

    /// Immediately SIGKILL a session.
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found.
    pub async fn force_stop_session(&mut self, session_id: &str) -> anyhow::Result<()> {
        let handle = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session {session_id} not found"))?;

        if handle.state == ProcessState::Stopped {
            self.sessions.remove(session_id);
            return Ok(());
        }

        tracing::info!(session_id = session_id, pid = handle.pid, "Force-stopping session (SIGKILL)");

        let _ = signal::kill(Pid::from_raw(handle.pid as i32), Signal::SIGKILL);
        // Brief wait for SIGKILL to take effect
        tokio::time::sleep(Duration::from_millis(100)).await;

        self.sessions.remove(session_id);
        Ok(())
    }

    /// Pause a running session by sending SIGSTOP.
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found or not in a running state.
    pub fn pause_session(&mut self, session_id: &str) -> anyhow::Result<()> {
        let handle = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session {session_id} not found"))?;

        if handle.state != ProcessState::Running {
            anyhow::bail!(
                "Cannot pause session {session_id}: state is {:?}",
                handle.state
            );
        }

        tracing::info!(session_id = session_id, pid = handle.pid, "Pausing session (SIGSTOP)");
        signal::kill(Pid::from_raw(handle.pid as i32), Signal::SIGSTOP)?;
        handle.state = ProcessState::Paused;
        Ok(())
    }

    /// Resume a paused session by sending SIGCONT.
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found or not in a paused state.
    pub fn resume_session(&mut self, session_id: &str) -> anyhow::Result<()> {
        let handle = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session {session_id} not found"))?;

        if handle.state != ProcessState::Paused {
            anyhow::bail!(
                "Cannot resume session {session_id}: state is {:?}",
                handle.state
            );
        }

        tracing::info!(session_id = session_id, pid = handle.pid, "Resuming session (SIGCONT)");
        signal::kill(Pid::from_raw(handle.pid as i32), Signal::SIGCONT)?;
        handle.state = ProcessState::Running;
        Ok(())
    }

    /// Returns the number of active (non-stopped) sessions.
    pub fn active_session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Gracefully shut down all active sessions.
    ///
    /// Sends SIGTERM to each, waits briefly, then SIGKILL stragglers.
    pub async fn shutdown(&mut self) -> anyhow::Result<()> {
        let session_ids: Vec<String> = self.sessions.keys().cloned().collect();

        tracing::info!(
            count = session_ids.len(),
            "Shutting down all supervised sessions"
        );

        for id in session_ids {
            if let Err(e) = self.stop_session(&id).await {
                tracing::warn!(session_id = %id, error = %e, "Error stopping session during shutdown");
            }
        }

        Ok(())
    }
}

/// Map autonomy level strings to additional Claude CLI flags.
fn apply_autonomy_flags(cmd: &mut Command, autonomy_level: &str) {
    match autonomy_level {
        "autonomous" => {
            cmd.arg("--dangerously-skip-permissions");
        }
        "stricter" => {
            cmd.arg("--allowedTools")
                .arg("Edit,Read,Write,Glob,Grep");
        }
        // "normal" or anything else: no extra flags
        _ => {}
    }
}

/// Wait for a child process to exit and return its exit code (if available).
///
/// This properly reaps the child process via `waitpid`, preventing zombies.
async fn wait_for_child(mut child: Child) -> Option<i32> {
    match child.wait().await {
        Ok(status) => status.code(),
        Err(e) => {
            tracing::warn!(error = %e, "Error waiting for child process");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_supervisor_has_zero_sessions() {
        let (supervisor, _rx) = ProcessSupervisor::new();
        assert_eq!(supervisor.active_session_count(), 0);
    }

    #[tokio::test]
    async fn test_start_session_tracks_process() {
        let (mut supervisor, mut rx) = ProcessSupervisor::new();

        let mut cmd = Command::new("echo");
        cmd.arg("hello");

        supervisor
            .start_session_with_command("sess-1", cmd)
            .await
            .expect("start_session_with_command should succeed");

        assert_eq!(supervisor.active_session_count(), 1);

        // Should receive a Running event
        let event = rx.recv().await.expect("should receive an event");
        match event {
            SupervisorEvent::StateChanged {
                session_id,
                new_state,
                ..
            } => {
                assert_eq!(session_id, "sess-1");
                assert_eq!(new_state, "running");
            }
        }

        // Wait a bit for the echo process to exit and the monitor to fire
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Should receive a Stopped event from the monitor
        let event = rx.recv().await.expect("should receive stopped event");
        match event {
            SupervisorEvent::StateChanged {
                session_id,
                new_state,
                ..
            } => {
                assert_eq!(session_id, "sess-1");
                assert_eq!(new_state, "stopped");
            }
        }
    }

    #[tokio::test]
    async fn test_duplicate_session_id_rejected() {
        let (mut supervisor, _rx) = ProcessSupervisor::new();

        let mut cmd1 = Command::new("sleep");
        cmd1.arg("10");
        supervisor
            .start_session_with_command("dup-1", cmd1)
            .await
            .expect("first start should succeed");

        let mut cmd2 = Command::new("sleep");
        cmd2.arg("10");
        let result = supervisor
            .start_session_with_command("dup-1", cmd2)
            .await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("already active")
        );

        // Cleanup
        supervisor.force_stop_session("dup-1").await.ok();
    }

    #[tokio::test]
    async fn test_stop_session_removes_handle() {
        let (mut supervisor, _rx) = ProcessSupervisor::new();

        let mut cmd = Command::new("sleep");
        cmd.arg("60");

        supervisor
            .start_session_with_command("stop-1", cmd)
            .await
            .expect("start should succeed");

        assert_eq!(supervisor.active_session_count(), 1);

        supervisor
            .stop_session("stop-1")
            .await
            .expect("stop should succeed");

        assert_eq!(supervisor.active_session_count(), 0);
    }

    #[tokio::test]
    async fn test_force_stop_removes_handle() {
        let (mut supervisor, _rx) = ProcessSupervisor::new();

        let mut cmd = Command::new("sleep");
        cmd.arg("60");

        supervisor
            .start_session_with_command("force-1", cmd)
            .await
            .expect("start should succeed");

        assert_eq!(supervisor.active_session_count(), 1);

        supervisor
            .force_stop_session("force-1")
            .await
            .expect("force_stop should succeed");

        assert_eq!(supervisor.active_session_count(), 0);
    }

    #[tokio::test]
    async fn test_stop_nonexistent_session_returns_error() {
        let (mut supervisor, _rx) = ProcessSupervisor::new();
        let result = supervisor.stop_session("no-such-session").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pause_and_resume() {
        let (mut supervisor, _rx) = ProcessSupervisor::new();

        let mut cmd = Command::new("sleep");
        cmd.arg("60");

        supervisor
            .start_session_with_command("pr-1", cmd)
            .await
            .expect("start should succeed");

        supervisor
            .pause_session("pr-1")
            .expect("pause should succeed");

        // Verify internal state is Paused
        assert_eq!(
            supervisor.sessions.get("pr-1").unwrap().state,
            ProcessState::Paused
        );

        supervisor
            .resume_session("pr-1")
            .expect("resume should succeed");

        // Verify internal state is Running again
        assert_eq!(
            supervisor.sessions.get("pr-1").unwrap().state,
            ProcessState::Running
        );

        // Cleanup
        supervisor.force_stop_session("pr-1").await.ok();
    }

    #[tokio::test]
    async fn test_shutdown_stops_all_sessions() {
        let (mut supervisor, _rx) = ProcessSupervisor::new();

        for i in 0..3 {
            let mut cmd = Command::new("sleep");
            cmd.arg("60");
            supervisor
                .start_session_with_command(&format!("shutdown-{i}"), cmd)
                .await
                .expect("start should succeed");
        }

        assert_eq!(supervisor.active_session_count(), 3);

        supervisor
            .shutdown()
            .await
            .expect("shutdown should succeed");

        assert_eq!(supervisor.active_session_count(), 0);
    }

    #[test]
    fn test_apply_autonomy_flags_normal() {
        let mut cmd = Command::new("claude");
        apply_autonomy_flags(&mut cmd, "normal");
        // Normal adds no extra flags; we just verify it doesn't panic.
        let built = cmd.as_std();
        let args: Vec<_> = built.get_args().collect();
        assert!(args.is_empty());
    }

    #[test]
    fn test_apply_autonomy_flags_autonomous() {
        let mut cmd = Command::new("claude");
        apply_autonomy_flags(&mut cmd, "autonomous");
        let built = cmd.as_std();
        let args: Vec<_> = built.get_args().collect();
        assert_eq!(args, vec!["--dangerously-skip-permissions"]);
    }

    #[test]
    fn test_apply_autonomy_flags_stricter() {
        let mut cmd = Command::new("claude");
        apply_autonomy_flags(&mut cmd, "stricter");
        let built = cmd.as_std();
        let args: Vec<_> = built.get_args().collect();
        assert_eq!(args, vec!["--allowedTools", "Edit,Read,Write,Glob,Grep"]);
    }

    #[test]
    fn test_process_state_display() {
        assert_eq!(ProcessState::Running.to_string(), "running");
        assert_eq!(ProcessState::Paused.to_string(), "paused");
        assert_eq!(ProcessState::Stopped.to_string(), "stopped");
    }
}

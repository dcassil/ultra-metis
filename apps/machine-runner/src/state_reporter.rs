use chrono::{DateTime, Utc};

use crate::client::ControlClient;
use crate::supervisor::SupervisorEvent;

/// A report that failed to send and should be retried.
#[allow(dead_code)] // failed_at stored for future retry-age-out logic
struct PendingReport {
    session_id: String,
    state: String,
    metadata: Option<serde_json::Value>,
    failed_at: DateTime<Utc>,
}

/// Reports session state changes to the control service.
///
/// Consumes `SupervisorEvent` messages and translates them into
/// `report_session_state` API calls. Failed reports are queued for
/// retry via `retry_pending`.
pub struct StateReporter {
    client: ControlClient,
    pending_reports: Vec<PendingReport>,
}

impl StateReporter {
    /// Create a new state reporter using the given control client.
    pub fn new(client: ControlClient) -> Self {
        Self {
            client,
            pending_reports: Vec::new(),
        }
    }

    /// Handle a supervisor event by reporting the corresponding state
    /// to the control service.
    ///
    /// If the report fails, it is queued for later retry.
    pub async fn handle_event(&mut self, event: SupervisorEvent) {
        let (session_id, api_state, metadata) = match event {
            SupervisorEvent::StateChanged {
                session_id,
                new_state,
                metadata,
            } => map_supervisor_state(&session_id, &new_state, metadata),
        };

        tracing::info!(
            session_id = %session_id,
            state = %api_state,
            "Reporting session state to control service"
        );

        if let Err(e) = self
            .client
            .report_session_state(&session_id, &api_state, metadata.clone())
            .await
        {
            tracing::warn!(
                session_id = %session_id,
                state = %api_state,
                error = %e,
                "Failed to report session state, queuing for retry"
            );
            self.pending_reports.push(PendingReport {
                session_id,
                state: api_state,
                metadata,
                failed_at: Utc::now(),
            });
        }
    }

    /// Retry all pending reports that previously failed.
    ///
    /// Successfully sent reports are removed from the queue. Reports
    /// that fail again remain queued for the next retry cycle.
    pub async fn retry_pending(&mut self) {
        if self.pending_reports.is_empty() {
            return;
        }

        tracing::info!(
            count = self.pending_reports.len(),
            "Retrying pending state reports"
        );

        let mut still_pending = Vec::new();

        for report in self.pending_reports.drain(..) {
            match self
                .client
                .report_session_state(&report.session_id, &report.state, report.metadata.clone())
                .await
            {
                Ok(()) => {
                    tracing::info!(
                        session_id = %report.session_id,
                        state = %report.state,
                        "Successfully retried pending state report"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        session_id = %report.session_id,
                        state = %report.state,
                        error = %e,
                        "Retry still failing for state report"
                    );
                    still_pending.push(report);
                }
            }
        }

        self.pending_reports = still_pending;
    }

    /// Returns the number of reports waiting to be retried.
    pub fn pending_count(&self) -> usize {
        self.pending_reports.len()
    }
}

/// Map a supervisor event's state string to the API session state and metadata.
///
/// The supervisor emits states like "running" and "stopped" with optional metadata.
/// This function translates those into the API's session state vocabulary:
/// - "running" -> "running"
/// - "stopped" with exit_code 0 (or no metadata) -> "completed"
/// - "stopped" with non-zero exit_code -> "failed"
/// - "paused" -> "paused"
/// - anything else -> passed through as-is
fn map_supervisor_state(
    session_id: &str,
    new_state: &str,
    metadata: Option<serde_json::Value>,
) -> (String, String, Option<serde_json::Value>) {
    match new_state {
        "running" => (session_id.to_string(), "running".to_string(), metadata),
        "stopped" => {
            // Determine if the process completed successfully or failed
            // based on the exit code in metadata.
            let exit_code = metadata
                .as_ref()
                .and_then(|m| m.get("exit_code"))
                .and_then(serde_json::Value::as_i64);

            match exit_code {
                Some(0) => (
                    session_id.to_string(),
                    "completed".to_string(),
                    Some(serde_json::json!({ "exit_code": 0 })),
                ),
                Some(code) => (
                    session_id.to_string(),
                    "failed".to_string(),
                    Some(serde_json::json!({
                        "exit_code": code,
                        "error_message": format!("Process exited with code {code}")
                    })),
                ),
                // No exit code (e.g., process was killed) -> treat as stopped
                None => (
                    session_id.to_string(),
                    "stopped".to_string(),
                    metadata,
                ),
            }
        }
        "paused" => (session_id.to_string(), "paused".to_string(), metadata),
        other => (session_id.to_string(), other.to_string(), metadata),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pending_count_starts_at_zero() {
        let client = ControlClient::new("http://localhost:8080", "test-token");
        let reporter = StateReporter::new(client);
        assert_eq!(reporter.pending_count(), 0);
    }

    #[test]
    fn test_map_running_state() {
        let (session_id, state, metadata) =
            map_supervisor_state("sess-1", "running", Some(serde_json::json!({"pid": 1234})));
        assert_eq!(session_id, "sess-1");
        assert_eq!(state, "running");
        assert_eq!(metadata.unwrap()["pid"], 1234);
    }

    #[test]
    fn test_map_stopped_with_zero_exit_code_becomes_completed() {
        let (session_id, state, metadata) = map_supervisor_state(
            "sess-2",
            "stopped",
            Some(serde_json::json!({"exit_code": 0})),
        );
        assert_eq!(session_id, "sess-2");
        assert_eq!(state, "completed");
        assert_eq!(metadata.unwrap()["exit_code"], 0);
    }

    #[test]
    fn test_map_stopped_with_nonzero_exit_code_becomes_failed() {
        let (session_id, state, metadata) = map_supervisor_state(
            "sess-3",
            "stopped",
            Some(serde_json::json!({"exit_code": 1})),
        );
        assert_eq!(session_id, "sess-3");
        assert_eq!(state, "failed");
        let meta = metadata.unwrap();
        assert_eq!(meta["exit_code"], 1);
        assert!(meta["error_message"].as_str().unwrap().contains("code 1"));
    }

    #[test]
    fn test_map_stopped_without_exit_code_becomes_stopped() {
        let (session_id, state, _metadata) = map_supervisor_state("sess-4", "stopped", None);
        assert_eq!(session_id, "sess-4");
        assert_eq!(state, "stopped");
    }

    #[test]
    fn test_map_paused_state() {
        let (session_id, state, _metadata) = map_supervisor_state("sess-5", "paused", None);
        assert_eq!(session_id, "sess-5");
        assert_eq!(state, "paused");
    }

    #[test]
    fn test_map_unknown_state_passed_through() {
        let (session_id, state, _metadata) =
            map_supervisor_state("sess-6", "waiting_for_input", None);
        assert_eq!(session_id, "sess-6");
        assert_eq!(state, "waiting_for_input");
    }

    #[tokio::test]
    async fn test_handle_event_queues_pending_on_failure() {
        // Use a client pointing to a non-existent server so the report will fail.
        let client = ControlClient::new("http://127.0.0.1:1", "test-token");
        let mut reporter = StateReporter::new(client);

        assert_eq!(reporter.pending_count(), 0);

        let event = SupervisorEvent::StateChanged {
            session_id: "sess-fail".to_string(),
            new_state: "running".to_string(),
            metadata: None,
        };

        reporter.handle_event(event).await;

        assert_eq!(reporter.pending_count(), 1);
    }

    #[tokio::test]
    async fn test_retry_pending_clears_on_continued_failure() {
        // With a non-existent server, retries will also fail and stay queued.
        let client = ControlClient::new("http://127.0.0.1:1", "test-token");
        let mut reporter = StateReporter::new(client);

        let event = SupervisorEvent::StateChanged {
            session_id: "sess-retry".to_string(),
            new_state: "running".to_string(),
            metadata: None,
        };

        reporter.handle_event(event).await;
        assert_eq!(reporter.pending_count(), 1);

        // Retry — will still fail since server doesn't exist
        reporter.retry_pending().await;
        assert_eq!(reporter.pending_count(), 1);
    }

    #[tokio::test]
    async fn test_retry_pending_noop_when_empty() {
        let client = ControlClient::new("http://127.0.0.1:1", "test-token");
        let mut reporter = StateReporter::new(client);

        // Should not panic or do anything when there are no pending reports
        reporter.retry_pending().await;
        assert_eq!(reporter.pending_count(), 0);
    }
}

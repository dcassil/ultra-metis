use crate::discovery::RepoInfo;
use crate::output_capture::OutputEvent;
use crate::policy::MachinePolicy;
use serde::{Deserialize, Serialize};

/// A single log entry to forward to the control service.
#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub level: String,
    pub target: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<serde_json::Value>,
    pub timestamp: String,
}

/// Response from the control service after machine registration.
#[derive(Debug, Deserialize)]
pub struct RegisterResponse {
    pub id: String,
    pub status: String,
}

/// Request body for registering a machine with the control service.
#[derive(Debug, Serialize)]
pub struct RegisterRequest {
    pub name: String,
    pub platform: String,
    pub capabilities: Option<String>,
    pub repos: Vec<RepoInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machine_id: Option<String>,
}

/// Response from the control service after a heartbeat.
#[derive(Debug, Deserialize)]
pub struct HeartbeatResponse {
    pub status: String,
}

/// A command received from the control service.
#[derive(Debug, Clone, Deserialize)]
pub struct CommandResponse {
    pub command_id: String,
    pub command_type: String,
    pub payload: Option<serde_json::Value>,
}

/// Request body for sending a heartbeat to the control service.
#[derive(Debug, Serialize)]
pub struct HeartbeatRequest {
    pub repos: Vec<RepoInfo>,
}

/// Request body for reporting session state to the control service.
#[derive(Debug, Clone, Serialize)]
pub struct ReportStateRequest {
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Error types specific to control service communication.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Machine has been revoked (401 Unauthorized)")]
    MachineRevoked,

    #[error("Machine is pending approval (403 Forbidden)")]
    MachinePending,

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Unexpected status {status}: {body}")]
    UnexpectedStatus { status: u16, body: String },
}

/// HTTP client for communicating with the Cadre Control Service API.
pub struct ControlClient {
    http: reqwest::Client,
    base_url: String,
    token: String,
}

impl ControlClient {
    /// Create a new control client with the given base URL and API token.
    pub fn new(base_url: &str, token: &str) -> Self {
        let http = reqwest::Client::new();
        Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
        }
    }

    /// Register this machine with the control service.
    ///
    /// # Errors
    ///
    /// Returns `ClientError` on HTTP failures or unexpected status codes.
    pub async fn register(
        &self,
        request: &RegisterRequest,
    ) -> Result<RegisterResponse, ClientError> {
        let url = format!("{}/api/machines/register", self.base_url);

        let response = self
            .http
            .post(&url)
            .bearer_auth(&self.token)
            .json(request)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let body = response.json::<RegisterResponse>().await?;
            Ok(body)
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(ClientError::UnexpectedStatus {
                status: status.as_u16(),
                body,
            })
        }
    }

    /// Send a heartbeat to the control service for the given machine.
    ///
    /// # Errors
    ///
    /// Returns `ClientError::MachineRevoked` on 401, `ClientError::MachinePending`
    /// on 403, or other errors for unexpected responses.
    pub async fn heartbeat(
        &self,
        machine_id: &str,
        request: &HeartbeatRequest,
    ) -> Result<HeartbeatResponse, ClientError> {
        let url = format!("{}/api/machines/{machine_id}/heartbeat", self.base_url);

        let response = self
            .http
            .post(&url)
            .bearer_auth(&self.token)
            .json(request)
            .send()
            .await?;

        handle_heartbeat_response(response).await
    }

    /// Fetch pending commands for the given machine.
    ///
    /// # Errors
    ///
    /// Returns `ClientError` on HTTP failures or unexpected status codes.
    pub async fn fetch_commands(
        &self,
        machine_id: &str,
    ) -> Result<Vec<CommandResponse>, ClientError> {
        let url = format!("{}/api/machines/{machine_id}/commands", self.base_url);

        let response = self
            .http
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let commands = response.json::<Vec<CommandResponse>>().await?;
            Ok(commands)
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(ClientError::UnexpectedStatus {
                status: status.as_u16(),
                body,
            })
        }
    }

    /// Acknowledge receipt of a command, marking it as delivered.
    ///
    /// # Errors
    ///
    /// Returns `ClientError` on HTTP failures or unexpected status codes.
    pub async fn ack_command(
        &self,
        machine_id: &str,
        command_id: &str,
    ) -> Result<(), ClientError> {
        let url = format!(
            "{}/api/machines/{machine_id}/commands/{command_id}/ack",
            self.base_url
        );

        let response = self
            .http
            .post(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(ClientError::UnexpectedStatus {
                status: status.as_u16(),
                body,
            })
        }
    }

    /// Report the state of a session to the control service.
    ///
    /// # Errors
    ///
    /// Returns `ClientError` on HTTP failures or unexpected status codes.
    pub async fn report_session_state(
        &self,
        session_id: &str,
        new_state: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), ClientError> {
        let url = format!("{}/api/sessions/{session_id}/state", self.base_url);

        let request = ReportStateRequest {
            state: new_state.to_string(),
            metadata,
        };

        let response = self
            .http
            .post(&url)
            .bearer_auth(&self.token)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(ClientError::UnexpectedStatus {
                status: status.as_u16(),
                body,
            })
        }
    }

    /// Post a batch of output events for a session to the control service.
    ///
    /// # Errors
    ///
    /// Returns `ClientError` on HTTP failures or unexpected status codes.
    pub async fn post_session_events(
        &self,
        session_id: &str,
        events: &[OutputEvent],
    ) -> Result<(), ClientError> {
        let url = format!("{}/api/sessions/{session_id}/events", self.base_url);

        let body = serde_json::json!({ "events": events });

        let response = self
            .http
            .post(&url)
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(ClientError::UnexpectedStatus {
                status: status.as_u16(),
                body,
            })
        }
    }

    /// Post a batch of log entries for a machine to the control service.
    ///
    /// # Errors
    ///
    /// Returns `ClientError` on HTTP failures or unexpected status codes.
    pub async fn post_machine_logs(
        &self,
        machine_id: &str,
        logs: &[LogEntry],
    ) -> Result<(), ClientError> {
        let url = format!("{}/api/machines/{machine_id}/logs", self.base_url);

        let body = serde_json::json!({ "logs": logs });

        let response = self
            .http
            .post(&url)
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(ClientError::UnexpectedStatus {
                status: status.as_u16(),
                body,
            })
        }
    }

    /// Fetch the machine policy from the control service.
    ///
    /// # Errors
    ///
    /// Returns `ClientError` on HTTP failures or unexpected status codes.
    pub async fn fetch_policy(&self, machine_id: &str) -> Result<MachinePolicy, ClientError> {
        let url = format!("{}/api/machines/{machine_id}/policy", self.base_url);

        let response = self
            .http
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let policy = response.json::<MachinePolicy>().await?;
            Ok(policy)
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(ClientError::UnexpectedStatus {
                status: status.as_u16(),
                body,
            })
        }
    }
}

async fn handle_heartbeat_response(
    response: reqwest::Response,
) -> Result<HeartbeatResponse, ClientError> {
    let status = response.status();

    match status.as_u16() {
        200..=299 => {
            let body = response.json::<HeartbeatResponse>().await?;
            Ok(body)
        }
        401 => Err(ClientError::MachineRevoked),
        403 => Err(ClientError::MachinePending),
        _ => {
            let body = response.text().await.unwrap_or_default();
            Err(ClientError::UnexpectedStatus {
                status: status.as_u16(),
                body,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_request_serialization() {
        let request = RegisterRequest {
            name: "my-machine".to_string(),
            platform: "linux/x86_64".to_string(),
            capabilities: Some("claude_code".to_string()),
            repos: vec![RepoInfo {
                repo_name: "test-repo".to_string(),
                repo_path: "/home/user/test-repo".to_string(),
                cadre_managed: true,
            }],
            machine_id: None,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["name"], "my-machine");
        assert_eq!(json["platform"], "linux/x86_64");
        assert_eq!(json["capabilities"], "claude_code");
        assert_eq!(json["repos"][0]["name"], "test-repo");
        assert_eq!(json["repos"][0]["cadre_managed"], true);
    }

    #[test]
    fn test_register_response_deserialization() {
        let json = r#"{"id": "machine-uuid-123", "status": "pending"}"#;
        let response: RegisterResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, "machine-uuid-123");
        assert_eq!(response.status, "pending");
    }

    #[test]
    fn test_heartbeat_request_serialization() {
        let request = HeartbeatRequest {
            repos: vec![
                RepoInfo {
                    repo_name: "repo-a".to_string(),
                    repo_path: "/path/a".to_string(),
                    cadre_managed: false,
                },
                RepoInfo {
                    repo_name: "repo-b".to_string(),
                    repo_path: "/path/b".to_string(),
                    cadre_managed: true,
                },
            ],
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["repos"].as_array().unwrap().len(), 2);
        assert_eq!(json["repos"][0]["name"], "repo-a");
        assert_eq!(json["repos"][1]["cadre_managed"], true);
    }

    #[test]
    fn test_heartbeat_response_deserialization() {
        let json = r#"{"status": "ok"}"#;
        let response: HeartbeatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, "ok");
    }

    #[test]
    fn test_control_client_trims_trailing_slash() {
        let client = ControlClient::new("https://api.example.com/", "token");
        assert_eq!(client.base_url, "https://api.example.com");
    }

    #[test]
    fn test_command_response_deserialization() {
        let json = r#"[
            {"command_id": "cmd-1", "command_type": "start_session", "payload": {"session_id": "s-1"}},
            {"command_id": "cmd-2", "command_type": "stop", "payload": null}
        ]"#;
        let commands: Vec<CommandResponse> = serde_json::from_str(json).unwrap();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].command_id, "cmd-1");
        assert_eq!(commands[0].command_type, "start_session");
        assert!(commands[0].payload.is_some());
        assert_eq!(commands[0].payload.as_ref().unwrap()["session_id"], "s-1");
        assert_eq!(commands[1].command_id, "cmd-2");
        assert_eq!(commands[1].command_type, "stop");
        assert!(commands[1].payload.is_none());
    }

    #[test]
    fn test_command_response_empty_array() {
        let json = "[]";
        let commands: Vec<CommandResponse> = serde_json::from_str(json).unwrap();
        assert!(commands.is_empty());
    }

    #[test]
    fn test_log_entry_serialization() {
        let entry = LogEntry {
            level: "info".to_string(),
            target: "cadre_machine_runner::runner".to_string(),
            message: "Heartbeat sent".to_string(),
            fields: Some(serde_json::json!({ "machine_id": "m-1" })),
            timestamp: "2026-03-27T12:00:00Z".to_string(),
        };

        let json = serde_json::to_value(&entry).unwrap();
        assert_eq!(json["level"], "info");
        assert_eq!(json["target"], "cadre_machine_runner::runner");
        assert_eq!(json["message"], "Heartbeat sent");
        assert_eq!(json["fields"]["machine_id"], "m-1");
        assert_eq!(json["timestamp"], "2026-03-27T12:00:00Z");
    }

    #[test]
    fn test_log_entry_serialization_without_fields() {
        let entry = LogEntry {
            level: "warn".to_string(),
            target: "test".to_string(),
            message: "something".to_string(),
            fields: None,
            timestamp: "2026-03-27T12:00:00Z".to_string(),
        };

        let json = serde_json::to_value(&entry).unwrap();
        // fields should be absent (not null) due to skip_serializing_if
        assert!(json.get("fields").is_none());
    }

    #[test]
    fn test_report_state_request_serialization_with_metadata() {
        let request = ReportStateRequest {
            state: "completed".to_string(),
            metadata: Some(serde_json::json!({ "exit_code": 0 })),
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["state"], "completed");
        assert_eq!(json["metadata"]["exit_code"], 0);
    }

    #[test]
    fn test_report_state_request_serialization_without_metadata() {
        let request = ReportStateRequest {
            state: "running".to_string(),
            metadata: None,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["state"], "running");
        // metadata should be absent (not null) due to skip_serializing_if
        assert!(json.get("metadata").is_none());
    }
}

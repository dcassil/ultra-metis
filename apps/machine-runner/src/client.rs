use crate::discovery::RepoInfo;
use serde::{Deserialize, Serialize};

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
}

/// Response from the control service after a heartbeat.
#[derive(Debug, Deserialize)]
pub struct HeartbeatResponse {
    pub status: String,
}

/// Request body for sending a heartbeat to the control service.
#[derive(Debug, Serialize)]
pub struct HeartbeatRequest {
    pub repos: Vec<RepoInfo>,
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
}

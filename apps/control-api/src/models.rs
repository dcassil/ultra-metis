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
}

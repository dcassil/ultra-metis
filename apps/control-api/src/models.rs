use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrustTier {
    Trusted,
    Restricted,
}

impl TrustTier {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Trusted => "trusted",
            Self::Restricted => "restricted",
        }
    }
}

impl std::str::FromStr for TrustTier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "trusted" => Ok(Self::Trusted),
            "restricted" => Ok(Self::Restricted),
            other => Err(format!("unknown trust tier: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectivityStatus {
    Online,
    Stale,
    Offline,
}

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Machine {
    pub id: String,
    pub name: String,
    pub platform: Option<String>,
    pub status: MachineStatus,
    pub trust_tier: TrustTier,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub user_id: String,
    pub team_id: String,
    pub org_id: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Machine {
    /// Compute the connectivity status based on the last heartbeat timestamp.
    ///
    /// - `None`            -> `Offline`
    /// - < 30 s ago        -> `Online`
    /// - 30 s .. 5 min ago -> `Stale`
    /// - > 5 min ago       -> `Offline`
    #[must_use]
    pub fn connectivity_status(&self) -> ConnectivityStatus {
        let Some(hb) = self.last_heartbeat else {
            return ConnectivityStatus::Offline;
        };

        let elapsed = Utc::now().signed_duration_since(hb);
        let secs = elapsed.num_seconds();

        if secs < 30 {
            ConnectivityStatus::Online
        } else if secs < 300 {
            ConnectivityStatus::Stale
        } else {
            ConnectivityStatus::Offline
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineRepo {
    pub id: String,
    pub machine_id: String,
    pub repo_name: String,
    pub repo_path: String,
    pub cadre_managed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// API request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineResponse {
    pub id: String,
    pub name: String,
    pub platform: Option<String>,
    pub status: MachineStatus,
    pub trust_tier: TrustTier,
    pub connectivity_status: ConnectivityStatus,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub user_id: String,
    pub team_id: String,
    pub org_id: String,
    pub metadata: serde_json::Value,
    pub repos: Vec<RepoInfo>,
    pub active_session_count: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MachineResponse {
    /// Build a response from a [`Machine`] and its associated repos.
    #[must_use]
    pub fn from_machine(machine: &Machine, repos: &[MachineRepo]) -> Self {
        let connectivity_status = machine.connectivity_status();
        Self {
            id: machine.id.clone(),
            name: machine.name.clone(),
            platform: machine.platform.clone(),
            status: machine.status.clone(),
            trust_tier: machine.trust_tier.clone(),
            connectivity_status,
            last_heartbeat: machine.last_heartbeat,
            user_id: machine.user_id.clone(),
            team_id: machine.team_id.clone(),
            org_id: machine.org_id.clone(),
            metadata: machine.metadata.clone(),
            repos: repos
                .iter()
                .map(|r| RepoInfo {
                    repo_name: r.repo_name.clone(),
                    repo_path: r.repo_path.clone(),
                    cadre_managed: Some(r.cadre_managed),
                })
                .collect(),
            active_session_count: 0,
            created_at: machine.created_at,
            updated_at: machine.updated_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub platform: Option<String>,
    pub capabilities: Option<serde_json::Value>,
    pub repos: Vec<RepoInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    pub repo_name: String,
    pub repo_path: String,
    pub cadre_managed: Option<bool>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn make_machine(last_heartbeat: Option<DateTime<Utc>>) -> Machine {
        Machine {
            id: "m-1".to_string(),
            name: "test-machine".to_string(),
            platform: Some("linux".to_string()),
            status: MachineStatus::Trusted,
            trust_tier: TrustTier::Trusted,
            last_heartbeat,
            user_id: "u-1".to_string(),
            team_id: "t-1".to_string(),
            org_id: "o-1".to_string(),
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn connectivity_no_heartbeat_is_offline() {
        let m = make_machine(None);
        assert_eq!(m.connectivity_status(), ConnectivityStatus::Offline);
    }

    #[test]
    fn connectivity_recent_heartbeat_is_online() {
        let hb = Utc::now() - Duration::seconds(10);
        let m = make_machine(Some(hb));
        assert_eq!(m.connectivity_status(), ConnectivityStatus::Online);
    }

    #[test]
    fn connectivity_stale_heartbeat() {
        let hb = Utc::now() - Duration::seconds(60);
        let m = make_machine(Some(hb));
        assert_eq!(m.connectivity_status(), ConnectivityStatus::Stale);
    }

    #[test]
    fn connectivity_old_heartbeat_is_offline() {
        let hb = Utc::now() - Duration::seconds(600);
        let m = make_machine(Some(hb));
        assert_eq!(m.connectivity_status(), ConnectivityStatus::Offline);
    }

    #[test]
    fn serde_machine_status_roundtrip() {
        let statuses = vec![
            MachineStatus::Pending,
            MachineStatus::Trusted,
            MachineStatus::Revoked,
        ];
        for s in statuses {
            let json = serde_json::to_string(&s).unwrap();
            let back: MachineStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    #[test]
    fn serde_trust_tier_roundtrip() {
        let tiers = vec![TrustTier::Trusted, TrustTier::Restricted];
        for t in tiers {
            let json = serde_json::to_string(&t).unwrap();
            let back: TrustTier = serde_json::from_str(&json).unwrap();
            assert_eq!(t, back);
        }
    }

    #[test]
    fn serde_connectivity_status_roundtrip() {
        let statuses = vec![
            ConnectivityStatus::Online,
            ConnectivityStatus::Stale,
            ConnectivityStatus::Offline,
        ];
        for s in statuses {
            let json = serde_json::to_string(&s).unwrap();
            let back: ConnectivityStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    #[test]
    fn machine_status_snake_case_serialization() {
        assert_eq!(
            serde_json::to_string(&MachineStatus::Pending).unwrap(),
            "\"pending\""
        );
        assert_eq!(
            serde_json::to_string(&MachineStatus::Trusted).unwrap(),
            "\"trusted\""
        );
        assert_eq!(
            serde_json::to_string(&MachineStatus::Revoked).unwrap(),
            "\"revoked\""
        );
    }

    #[test]
    fn trust_tier_snake_case_serialization() {
        assert_eq!(
            serde_json::to_string(&TrustTier::Trusted).unwrap(),
            "\"trusted\""
        );
        assert_eq!(
            serde_json::to_string(&TrustTier::Restricted).unwrap(),
            "\"restricted\""
        );
    }

    #[test]
    fn machine_response_from_machine() {
        let m = make_machine(Some(Utc::now()));
        let repos = vec![MachineRepo {
            id: "r-1".to_string(),
            machine_id: "m-1".to_string(),
            repo_name: "my-project".to_string(),
            repo_path: "/home/user/my-project".to_string(),
            cadre_managed: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }];

        let resp = MachineResponse::from_machine(&m, &repos);
        assert_eq!(resp.id, "m-1");
        assert_eq!(resp.repos.len(), 1);
        assert_eq!(resp.repos[0].repo_name, "my-project");
        assert_eq!(resp.active_session_count, 0);
        assert_eq!(resp.connectivity_status, ConnectivityStatus::Online);
    }
}

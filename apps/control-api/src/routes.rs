//! HTTP route handlers for the Machine Registry API.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use rusqlite::params;

use crate::auth::{DashboardAuth, MachineTokenAuth};
use crate::models::{
    ErrorBody, HeartbeatRequest, Machine, MachineDetailResponse, MachineRepo, MachineResponse,
    MachineStatus, RegisterRequest, RegisterResponse, RepoInfo,
};
use crate::AppState;

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

// ---------------------------------------------------------------------------
// POST /api/machines/register
// ---------------------------------------------------------------------------

pub async fn register_machine(
    State(state): State<AppState>,
    MachineTokenAuth(auth): MachineTokenAuth,
    Json(body): Json<RegisterRequest>,
) -> impl IntoResponse {
    let machine_id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let result = insert_machine(&state, &machine_id, &auth.user_id, &body, &now);
    match result {
        Ok(()) => {}
        Err(e) => {
            tracing::error!("failed to register machine: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::to_value(ErrorBody {
                    error: "failed to register machine".into(),
                })
                .unwrap_or_default()),
            )
                .into_response();
        }
    }

    if let Some(repos) = &body.repos {
        if let Err(e) = insert_repos(&state, &machine_id, repos, &now) {
            tracing::error!("failed to insert repos: {e}");
        }
    }

    (
        StatusCode::CREATED,
        Json(
            serde_json::to_value(RegisterResponse {
                id: machine_id,
                status: MachineStatus::Pending,
            })
            .unwrap_or_default(),
        ),
    )
        .into_response()
}

fn insert_machine(
    state: &AppState,
    machine_id: &str,
    user_id: &str,
    body: &RegisterRequest,
    now: &str,
) -> Result<(), rusqlite::Error> {
    state
        .db
        .lock()
        .expect("db lock poisoned")
        .execute(
            "INSERT INTO machines (id, user_id, name, platform, status, trust_tier, capabilities, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'pending', 'untrusted', ?5, ?6, ?7)",
            params![machine_id, user_id, body.name, body.platform, body.capabilities, now, now],
        )?;
    Ok(())
}

fn insert_repos(
    state: &AppState,
    machine_id: &str,
    repos: &[RepoInfo],
    now: &str,
) -> Result<(), rusqlite::Error> {
    let db = state.db.lock().expect("db lock poisoned");
    for repo in repos {
        let repo_id = uuid::Uuid::new_v4().to_string();
        db.execute(
            "INSERT INTO machine_repos (id, machine_id, repo_path, repo_name, last_seen)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![repo_id, machine_id, repo.path, repo.name, now],
        )?;
    }
    drop(db);
    Ok(())
}

// ---------------------------------------------------------------------------
// POST /api/machines/{id}/heartbeat
// ---------------------------------------------------------------------------

pub async fn heartbeat(
    State(state): State<AppState>,
    MachineTokenAuth(auth): MachineTokenAuth,
    Path(machine_id): Path<String>,
    Json(body): Json<HeartbeatRequest>,
) -> impl IntoResponse {
    let machine = match load_machine(&state, &machine_id, &auth.user_id) {
        Ok(Some(m)) => m,
        Ok(None) => return not_found("machine not found"),
        Err(e) => return internal_error(&format!("db error: {e}")),
    };

    match machine.status {
        MachineStatus::Revoked => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::to_value(ErrorBody {
                    error: "machine is revoked".into(),
                })
                .unwrap_or_default()),
            )
                .into_response();
        }
        MachineStatus::Pending => {
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::to_value(ErrorBody {
                    error: "machine is pending approval".into(),
                })
                .unwrap_or_default()),
            )
                .into_response();
        }
        MachineStatus::Trusted => {}
    }

    let now = Utc::now().to_rfc3339();
    if let Err(e) = update_heartbeat(&state, &machine_id, &now) {
        return internal_error(&format!("failed to update heartbeat: {e}"));
    }

    if let Some(repos) = &body.repos {
        if let Err(e) = replace_repos(&state, &machine_id, repos, &now) {
            tracing::error!("failed to update repos: {e}");
        }
    }

    Json(serde_json::json!({"status": "ok"})).into_response()
}

fn update_heartbeat(
    state: &AppState,
    machine_id: &str,
    now: &str,
) -> Result<(), rusqlite::Error> {
    state
        .db
        .lock()
        .expect("db lock poisoned")
        .execute(
            "UPDATE machines SET last_heartbeat = ?1, updated_at = ?2 WHERE id = ?3",
            params![now, now, machine_id],
        )?;
    Ok(())
}

fn replace_repos(
    state: &AppState,
    machine_id: &str,
    repos: &[RepoInfo],
    now: &str,
) -> Result<(), rusqlite::Error> {
    let db = state.db.lock().expect("db lock poisoned");
    db.execute(
        "DELETE FROM machine_repos WHERE machine_id = ?1",
        [machine_id],
    )?;
    for repo in repos {
        let repo_id = uuid::Uuid::new_v4().to_string();
        db.execute(
            "INSERT INTO machine_repos (id, machine_id, repo_path, repo_name, last_seen)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![repo_id, machine_id, repo.path, repo.name, now],
        )?;
    }
    drop(db);
    Ok(())
}

// ---------------------------------------------------------------------------
// GET /api/machines
// ---------------------------------------------------------------------------

pub async fn list_machines(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
) -> impl IntoResponse {
    match query_machines_for_user(&state, &auth.user_id) {
        Ok(machines) => Json(serde_json::to_value(machines).unwrap_or_default()).into_response(),
        Err(e) => internal_error(&format!("db error: {e}")),
    }
}

#[allow(clippy::significant_drop_tightening)]
fn query_machines_for_user(
    state: &AppState,
    user_id: &str,
) -> Result<Vec<MachineResponse>, rusqlite::Error> {
    let db = state.db.lock().expect("db lock poisoned");
    let mut stmt = db.prepare(
        "SELECT m.id, m.name, m.platform, m.status, m.trust_tier, m.capabilities,
                m.last_heartbeat, m.created_at, m.updated_at,
                (SELECT COUNT(*) FROM machine_repos mr WHERE mr.machine_id = m.id) as repos_count
         FROM machines m
         WHERE m.user_id = ?1
         ORDER BY m.status, m.name",
    )?;

    let rows = stmt.query_map([user_id], |row| {
        Ok(MachineRowData {
            id: row.get(0)?,
            name: row.get(1)?,
            platform: row.get(2)?,
            status: row.get::<_, String>(3)?,
            trust_tier: row.get::<_, String>(4)?,
            capabilities: row.get(5)?,
            last_heartbeat: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
            repos_count: row.get(9)?,
        })
    })?;

    let mut machines = Vec::new();
    for row in rows {
        let data = row?;
        machines.push(machine_row_to_response(data));
    }
    Ok(machines)
}

/// Intermediate struct for query results to keep function sizes manageable.
struct MachineRowData {
    id: String,
    name: String,
    platform: String,
    status: String,
    trust_tier: String,
    capabilities: Option<String>,
    last_heartbeat: Option<String>,
    created_at: String,
    updated_at: String,
    repos_count: i64,
}

fn machine_row_to_response(data: MachineRowData) -> MachineResponse {
    let status: MachineStatus = data.status.parse().unwrap_or(MachineStatus::Pending);
    let trust_tier = data.trust_tier.parse().unwrap_or(crate::models::TrustTier::Untrusted);

    let machine = Machine {
        id: data.id.clone(),
        user_id: String::new(),
        name: data.name.clone(),
        platform: data.platform.clone(),
        status: status.clone(),
        trust_tier: trust_tier.clone(),
        capabilities: data.capabilities.clone(),
        last_heartbeat: data.last_heartbeat.clone(),
        created_at: data.created_at.clone(),
        updated_at: data.updated_at.clone(),
    };

    MachineResponse {
        id: data.id,
        name: data.name,
        platform: data.platform,
        status,
        trust_tier,
        connectivity_status: machine.connectivity_status(),
        capabilities: data.capabilities,
        last_heartbeat: data.last_heartbeat,
        repos_count: data.repos_count,
        created_at: data.created_at,
        updated_at: data.updated_at,
    }
}

// ---------------------------------------------------------------------------
// GET /api/machines/{id}
// ---------------------------------------------------------------------------

pub async fn get_machine(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Path(machine_id): Path<String>,
) -> impl IntoResponse {
    let machine = match load_machine(&state, &machine_id, &auth.user_id) {
        Ok(Some(m)) => m,
        Ok(None) => return not_found("machine not found"),
        Err(e) => return internal_error(&format!("db error: {e}")),
    };

    let repos = match load_machine_repos(&state, &machine_id) {
        Ok(r) => r,
        Err(e) => return internal_error(&format!("db error: {e}")),
    };

    let detail = build_detail_response(machine, repos);
    Json(serde_json::to_value(detail).unwrap_or_default()).into_response()
}

fn build_detail_response(machine: Machine, repos: Vec<MachineRepo>) -> MachineDetailResponse {
    let connectivity = machine.connectivity_status();
    MachineDetailResponse {
        id: machine.id,
        name: machine.name,
        platform: machine.platform,
        status: machine.status,
        trust_tier: machine.trust_tier,
        connectivity_status: connectivity,
        capabilities: machine.capabilities,
        last_heartbeat: machine.last_heartbeat,
        repos,
        created_at: machine.created_at,
        updated_at: machine.updated_at,
    }
}

// ---------------------------------------------------------------------------
// POST /api/machines/{id}/approve
// ---------------------------------------------------------------------------

pub async fn approve_machine(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Path(machine_id): Path<String>,
) -> impl IntoResponse {
    let machine = match load_machine(&state, &machine_id, &auth.user_id) {
        Ok(Some(m)) => m,
        Ok(None) => return not_found("machine not found"),
        Err(e) => return internal_error(&format!("db error: {e}")),
    };

    if machine.status != MachineStatus::Pending {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::to_value(ErrorBody {
                error: format!("machine is {}, expected pending", machine.status),
            })
            .unwrap_or_default()),
        )
            .into_response();
    }

    let now = Utc::now().to_rfc3339();
    if let Err(e) = set_machine_status(&state, &machine_id, "trusted", "basic", &now) {
        return internal_error(&format!("failed to approve: {e}"));
    }

    Json(serde_json::json!({"status": "trusted", "id": machine_id})).into_response()
}

// ---------------------------------------------------------------------------
// POST /api/machines/{id}/revoke
// ---------------------------------------------------------------------------

pub async fn revoke_machine(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Path(machine_id): Path<String>,
) -> impl IntoResponse {
    let machine = match load_machine(&state, &machine_id, &auth.user_id) {
        Ok(Some(m)) => m,
        Ok(None) => return not_found("machine not found"),
        Err(e) => return internal_error(&format!("db error: {e}")),
    };

    // Allow revoking from any non-revoked state
    if machine.status == MachineStatus::Revoked {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::to_value(ErrorBody {
                error: "machine is already revoked".into(),
            })
            .unwrap_or_default()),
        )
            .into_response();
    }

    let now = Utc::now().to_rfc3339();
    if let Err(e) = set_machine_status(&state, &machine_id, "revoked", "untrusted", &now) {
        return internal_error(&format!("failed to revoke: {e}"));
    }

    Json(serde_json::json!({"status": "revoked", "id": machine_id})).into_response()
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn load_machine(
    state: &AppState,
    machine_id: &str,
    user_id: &str,
) -> Result<Option<Machine>, rusqlite::Error> {
    let result = state.db.lock().expect("db lock poisoned").query_row(
        "SELECT id, user_id, name, platform, status, trust_tier, capabilities,
                last_heartbeat, created_at, updated_at
         FROM machines WHERE id = ?1 AND user_id = ?2",
        params![machine_id, user_id],
        |row| {
            Ok(Machine {
                id: row.get(0)?,
                user_id: row.get(1)?,
                name: row.get(2)?,
                platform: row.get(3)?,
                status: row.get::<_, String>(4)?
                    .parse()
                    .unwrap_or(MachineStatus::Pending),
                trust_tier: row.get::<_, String>(5)?
                    .parse()
                    .unwrap_or(crate::models::TrustTier::Untrusted),
                capabilities: row.get(6)?,
                last_heartbeat: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        },
    );

    match result {
        Ok(machine) => Ok(Some(machine)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

#[allow(clippy::significant_drop_tightening)]
fn load_machine_repos(
    state: &AppState,
    machine_id: &str,
) -> Result<Vec<MachineRepo>, rusqlite::Error> {
    let db = state.db.lock().expect("db lock poisoned");
    let mut stmt = db.prepare(
        "SELECT id, machine_id, repo_path, repo_name, last_seen
         FROM machine_repos WHERE machine_id = ?1",
    )?;
    let rows = stmt.query_map([machine_id], |row| {
        Ok(MachineRepo {
            id: row.get(0)?,
            machine_id: row.get(1)?,
            repo_path: row.get(2)?,
            repo_name: row.get(3)?,
            last_seen: row.get(4)?,
        })
    })?;

    let mut repos = Vec::new();
    for row in rows {
        repos.push(row?);
    }
    Ok(repos)
}

fn set_machine_status(
    state: &AppState,
    machine_id: &str,
    status: &str,
    trust_tier: &str,
    now: &str,
) -> Result<(), rusqlite::Error> {
    state
        .db
        .lock()
        .expect("db lock poisoned")
        .execute(
            "UPDATE machines SET status = ?1, trust_tier = ?2, updated_at = ?3 WHERE id = ?4",
            params![status, trust_tier, now, machine_id],
        )?;
    Ok(())
}

fn not_found(msg: &str) -> axum::response::Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::to_value(ErrorBody {
            error: msg.to_string(),
        })
        .unwrap_or_default()),
    )
        .into_response()
}

fn internal_error(msg: &str) -> axum::response::Response {
    tracing::error!("{msg}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::to_value(ErrorBody {
            error: "internal server error".to_string(),
        })
        .unwrap_or_default()),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use std::sync::{Arc, Mutex};

    fn test_state() -> AppState {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        db::init_db(&conn).unwrap();
        AppState {
            db: Arc::new(Mutex::new(conn)),
        }
    }

    #[test]
    fn test_insert_and_load_machine() {
        let state = test_state();
        let now = Utc::now().to_rfc3339();
        let req = RegisterRequest {
            name: "test-machine".into(),
            platform: "linux".into(),
            capabilities: Some("docker".into()),
            repos: None,
        };

        insert_machine(&state, "m-1", "user-default", &req, &now).unwrap();
        let machine = load_machine(&state, "m-1", "user-default").unwrap();
        assert!(machine.is_some());
        let m = machine.unwrap();
        assert_eq!(m.name, "test-machine");
        assert_eq!(m.status, MachineStatus::Pending);
    }

    #[test]
    fn test_load_machine_wrong_user() {
        let state = test_state();
        let now = Utc::now().to_rfc3339();
        let req = RegisterRequest {
            name: "test".into(),
            platform: "linux".into(),
            capabilities: None,
            repos: None,
        };

        insert_machine(&state, "m-1", "user-default", &req, &now).unwrap();
        let machine = load_machine(&state, "m-1", "other-user").unwrap();
        assert!(machine.is_none());
    }

    #[test]
    fn test_approve_and_revoke_flow() {
        let state = test_state();
        let now = Utc::now().to_rfc3339();
        let req = RegisterRequest {
            name: "test".into(),
            platform: "linux".into(),
            capabilities: None,
            repos: None,
        };

        insert_machine(&state, "m-1", "user-default", &req, &now).unwrap();

        // Approve
        set_machine_status(&state, "m-1", "trusted", "basic", &now).unwrap();
        let m = load_machine(&state, "m-1", "user-default").unwrap().unwrap();
        assert_eq!(m.status, MachineStatus::Trusted);

        // Revoke
        set_machine_status(&state, "m-1", "revoked", "untrusted", &now).unwrap();
        let m = load_machine(&state, "m-1", "user-default").unwrap().unwrap();
        assert_eq!(m.status, MachineStatus::Revoked);
    }

    #[test]
    fn test_insert_and_load_repos() {
        let state = test_state();
        let now = Utc::now().to_rfc3339();
        let req = RegisterRequest {
            name: "test".into(),
            platform: "linux".into(),
            capabilities: None,
            repos: None,
        };

        insert_machine(&state, "m-1", "user-default", &req, &now).unwrap();
        let repos = vec![
            RepoInfo {
                path: "/home/user/project".into(),
                name: Some("project".into()),
            },
            RepoInfo {
                path: "/home/user/other".into(),
                name: None,
            },
        ];
        insert_repos(&state, "m-1", &repos, &now).unwrap();

        let loaded = load_machine_repos(&state, "m-1").unwrap();
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn test_replace_repos() {
        let state = test_state();
        let now = Utc::now().to_rfc3339();
        let req = RegisterRequest {
            name: "test".into(),
            platform: "linux".into(),
            capabilities: None,
            repos: None,
        };

        insert_machine(&state, "m-1", "user-default", &req, &now).unwrap();

        let repos_v1 = vec![RepoInfo {
            path: "/old".into(),
            name: None,
        }];
        insert_repos(&state, "m-1", &repos_v1, &now).unwrap();
        assert_eq!(load_machine_repos(&state, "m-1").unwrap().len(), 1);

        let repos_v2 = vec![
            RepoInfo {
                path: "/new1".into(),
                name: None,
            },
            RepoInfo {
                path: "/new2".into(),
                name: None,
            },
        ];
        replace_repos(&state, "m-1", &repos_v2, &now).unwrap();
        assert_eq!(load_machine_repos(&state, "m-1").unwrap().len(), 2);
    }

    #[test]
    fn test_query_machines_for_user() {
        let state = test_state();
        let now = Utc::now().to_rfc3339();

        for i in 0..3 {
            let req = RegisterRequest {
                name: format!("machine-{i}"),
                platform: "linux".into(),
                capabilities: None,
                repos: None,
            };
            insert_machine(&state, &format!("m-{i}"), "user-default", &req, &now).unwrap();
        }

        let machines = query_machines_for_user(&state, "user-default").unwrap();
        assert_eq!(machines.len(), 3);
    }

    #[test]
    fn test_heartbeat_updates_timestamp() {
        let state = test_state();
        let now = Utc::now().to_rfc3339();
        let req = RegisterRequest {
            name: "test".into(),
            platform: "linux".into(),
            capabilities: None,
            repos: None,
        };

        insert_machine(&state, "m-1", "user-default", &req, &now).unwrap();
        let m = load_machine(&state, "m-1", "user-default").unwrap().unwrap();
        assert!(m.last_heartbeat.is_none());

        let hb_time = Utc::now().to_rfc3339();
        update_heartbeat(&state, "m-1", &hb_time).unwrap();

        let m = load_machine(&state, "m-1", "user-default").unwrap().unwrap();
        assert!(m.last_heartbeat.is_some());
    }
}

//! HTTP route handlers for the Machine Registry API.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use rusqlite::params;

use crate::auth::{DashboardAuth, MachineTokenAuth};
use axum::extract::Query;

use crate::models::{
    ActionCategory, AutonomyLevel, CommandResponse, CreateSessionRequest, CreateSessionResponse,
    EffectivePolicy, EffectivePolicyQuery, ErrorBody, HeartbeatRequest, ListSessionsQuery,
    ListViolationsQuery, Machine, MachineDetailResponse, MachinePolicy, MachineRepo,
    MachineResponse, MachineStatus, PolicyViolationRecord, RegisterRequest, RegisterResponse,
    RepoInfo, RepoPolicy, RepoPolicyQuery, ReportStateRequest, Session, SessionListResponse,
    SessionMode, SessionResponse, SessionState, UpdateMachinePolicyRequest,
    UpdateRepoPolicyRequest, ViolationsListResponse,
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

    // Insert a default permissive policy for the newly trusted machine
    if let Err(e) = insert_default_machine_policy(&state, &machine_id) {
        tracing::error!("failed to insert default policy: {e}");
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
// POST /api/sessions
// ---------------------------------------------------------------------------

pub async fn create_session(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Json(body): Json<CreateSessionRequest>,
) -> impl IntoResponse {
    // Validate machine exists, belongs to user, and is trusted
    let machine = match load_machine(&state, &body.machine_id, &auth.user_id) {
        Ok(Some(m)) => m,
        Ok(None) => return not_found("machine not found"),
        Err(e) => return internal_error(&format!("db error: {e}")),
    };

    if machine.status != MachineStatus::Trusted {
        return bad_request(&format!(
            "machine is {}, must be trusted to start a session",
            machine.status
        ));
    }

    // Policy enforcement: validate requested autonomy level
    let requested_autonomy = body
        .autonomy_level
        .clone()
        .unwrap_or(AutonomyLevel::Normal);
    if let Some(machine_policy) = match load_machine_policy(&state, &body.machine_id) {
        Ok(p) => p,
        Err(e) => return internal_error(&format!("failed to load machine policy: {e}")),
    } {
        let repo_policy = match load_repo_policy(&state, &body.machine_id, &body.repo_path) {
            Ok(p) => p,
            Err(e) => return internal_error(&format!("failed to load repo policy: {e}")),
        };
        if let Err(violation) =
            crate::models::is_autonomy_allowed(&requested_autonomy, &machine_policy, repo_policy.as_ref())
        {
            // Log the policy violation before returning the error
            if let Err(e) = insert_policy_violation(
                &state,
                None,
                &body.machine_id,
                &auth.user_id,
                &violation.blocked_action,
                &violation.policy_scope,
                &violation.reason,
                Some(&body.repo_path),
            ) {
                tracing::error!("failed to log policy violation: {e}");
            }
            return bad_request(&violation.reason);
        }
    }

    let session_id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let autonomy = body
        .autonomy_level
        .as_ref()
        .map_or("normal", |a| a.as_str());

    if let Err(e) = insert_session(
        &state,
        &session_id,
        &auth.user_id,
        &body,
        autonomy,
        &now,
    ) {
        return internal_error(&format!("failed to create session: {e}"));
    }

    // Insert initial session event
    if let Err(e) = insert_session_event(&state, &session_id, None, "starting", &now, None) {
        tracing::error!("failed to insert initial session event: {e}");
    }

    // Queue a start_session command for the runner to pick up
    let payload = serde_json::json!({
        "session_id": session_id,
        "repo_path": body.repo_path,
        "title": body.title,
        "instructions": body.instructions,
        "autonomy_level": autonomy,
        "context": body.context,
    });
    if let Err(e) = insert_session_command(
        &state,
        &session_id,
        &body.machine_id,
        "start_session",
        Some(&payload.to_string()),
    ) {
        tracing::error!("failed to queue start_session command: {e}");
    }

    (
        StatusCode::CREATED,
        Json(
            serde_json::to_value(CreateSessionResponse {
                id: session_id,
                state: SessionState::Starting,
            })
            .unwrap_or_default(),
        ),
    )
        .into_response()
}

fn insert_session(
    state: &AppState,
    session_id: &str,
    user_id: &str,
    body: &CreateSessionRequest,
    autonomy: &str,
    now: &str,
) -> Result<(), rusqlite::Error> {
    state
        .db
        .lock()
        .expect("db lock poisoned")
        .execute(
            "INSERT INTO sessions (id, user_id, machine_id, repo_path, title, instructions,
             autonomy_level, work_item_id, context, state, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'starting', ?10, ?11)",
            params![
                session_id,
                user_id,
                body.machine_id,
                body.repo_path,
                body.title,
                body.instructions,
                autonomy,
                body.work_item_id,
                body.context,
                now,
                now
            ],
        )?;
    Ok(())
}

fn insert_session_event(
    state: &AppState,
    session_id: &str,
    from_state: Option<&str>,
    to_state: &str,
    now: &str,
    metadata: Option<&str>,
) -> Result<(), rusqlite::Error> {
    let event_id = uuid::Uuid::new_v4().to_string();
    state
        .db
        .lock()
        .expect("db lock poisoned")
        .execute(
            "INSERT INTO session_events (id, session_id, from_state, to_state, timestamp, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![event_id, session_id, from_state, to_state, now, metadata],
        )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// GET /api/sessions/{id}
// ---------------------------------------------------------------------------

pub async fn get_session(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    match load_session(&state, &session_id, &auth.user_id) {
        Ok(Some(session)) => {
            let resp = session_to_response(session);
            Json(serde_json::to_value(resp).unwrap_or_default()).into_response()
        }
        Ok(None) => not_found("session not found"),
        Err(e) => internal_error(&format!("db error: {e}")),
    }
}

// ---------------------------------------------------------------------------
// GET /api/sessions
// ---------------------------------------------------------------------------

pub async fn list_sessions(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Query(query): Query<ListSessionsQuery>,
) -> impl IntoResponse {
    match query_sessions(&state, &auth.user_id, &query) {
        Ok(result) => Json(serde_json::to_value(result).unwrap_or_default()).into_response(),
        Err(e) => internal_error(&format!("db error: {e}")),
    }
}

fn load_session(
    state: &AppState,
    session_id: &str,
    user_id: &str,
) -> Result<Option<Session>, rusqlite::Error> {
    let result = state.db.lock().expect("db lock poisoned").query_row(
        "SELECT id, user_id, machine_id, repo_path, title, instructions, autonomy_level,
                work_item_id, context, state, created_at, updated_at, started_at, completed_at
         FROM sessions WHERE id = ?1 AND user_id = ?2",
        params![session_id, user_id],
        |row| {
            Ok(Session {
                id: row.get(0)?,
                user_id: row.get(1)?,
                machine_id: row.get(2)?,
                repo_path: row.get(3)?,
                title: row.get(4)?,
                instructions: row.get(5)?,
                autonomy_level: row.get::<_, String>(6)?
                    .parse()
                    .unwrap_or(crate::models::AutonomyLevel::Normal),
                work_item_id: row.get(7)?,
                context: row.get(8)?,
                state: row.get::<_, String>(9)?
                    .parse()
                    .unwrap_or(SessionState::Starting),
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
                started_at: row.get(12)?,
                completed_at: row.get(13)?,
            })
        },
    );

    match result {
        Ok(session) => Ok(Some(session)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

#[allow(clippy::significant_drop_tightening)]
fn query_sessions(
    state: &AppState,
    user_id: &str,
    query: &ListSessionsQuery,
) -> Result<SessionListResponse, rusqlite::Error> {
    let db = state.db.lock().expect("db lock poisoned");

    // Build WHERE clause dynamically
    let mut conditions = vec!["user_id = ?1".to_string()];
    let mut param_idx = 2;

    if query.machine_id.is_some() {
        conditions.push(format!("machine_id = ?{param_idx}"));
        param_idx += 1;
    }
    if query.repo_path.is_some() {
        conditions.push(format!("repo_path = ?{param_idx}"));
        param_idx += 1;
    }
    if query.state.is_some() {
        conditions.push(format!("state = ?{param_idx}"));
        param_idx += 1;
    }

    let where_clause = conditions.join(" AND ");

    // Count query
    let count_sql = format!("SELECT COUNT(*) FROM sessions WHERE {where_clause}");
    let list_sql = format!(
        "SELECT id, user_id, machine_id, repo_path, title, instructions, autonomy_level,
                work_item_id, context, state, created_at, updated_at, started_at, completed_at
         FROM sessions WHERE {where_clause}
         ORDER BY created_at DESC
         LIMIT ?{} OFFSET ?{}",
        param_idx,
        param_idx + 1
    );

    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    // Execute count
    let total: i64 = match (&query.machine_id, &query.repo_path, &query.state) {
        (None, None, None) => {
            db.query_row(&count_sql, [user_id], |row| row.get(0))?
        }
        (Some(mid), None, None) => {
            db.query_row(&count_sql, params![user_id, mid], |row| row.get(0))?
        }
        (None, Some(rp), None) => {
            db.query_row(&count_sql, params![user_id, rp], |row| row.get(0))?
        }
        (None, None, Some(st)) => {
            db.query_row(&count_sql, params![user_id, st], |row| row.get(0))?
        }
        (Some(mid), Some(rp), None) => {
            db.query_row(&count_sql, params![user_id, mid, rp], |row| row.get(0))?
        }
        (Some(mid), None, Some(st)) => {
            db.query_row(&count_sql, params![user_id, mid, st], |row| row.get(0))?
        }
        (None, Some(rp), Some(st)) => {
            db.query_row(&count_sql, params![user_id, rp, st], |row| row.get(0))?
        }
        (Some(mid), Some(rp), Some(st)) => {
            db.query_row(&count_sql, params![user_id, mid, rp, st], |row| row.get(0))?
        }
    };

    // Execute list
    let sessions: Vec<Session> = match (&query.machine_id, &query.repo_path, &query.state) {
        (None, None, None) => {
            query_sessions_rows(&db, &list_sql, params![user_id, limit, offset])?
        }
        (Some(mid), None, None) => {
            query_sessions_rows(&db, &list_sql, params![user_id, mid, limit, offset])?
        }
        (None, Some(rp), None) => {
            query_sessions_rows(&db, &list_sql, params![user_id, rp, limit, offset])?
        }
        (None, None, Some(st)) => {
            query_sessions_rows(&db, &list_sql, params![user_id, st, limit, offset])?
        }
        (Some(mid), Some(rp), None) => {
            query_sessions_rows(&db, &list_sql, params![user_id, mid, rp, limit, offset])?
        }
        (Some(mid), None, Some(st)) => {
            query_sessions_rows(&db, &list_sql, params![user_id, mid, st, limit, offset])?
        }
        (None, Some(rp), Some(st)) => {
            query_sessions_rows(&db, &list_sql, params![user_id, rp, st, limit, offset])?
        }
        (Some(mid), Some(rp), Some(st)) => {
            query_sessions_rows(&db, &list_sql, params![user_id, mid, rp, st, limit, offset])?
        }
    };

    Ok(SessionListResponse {
        sessions: sessions.into_iter().map(session_to_response).collect(),
        total,
    })
}

#[allow(clippy::significant_drop_tightening)]
fn query_sessions_rows(
    db: &rusqlite::Connection,
    sql: &str,
    params: impl rusqlite::Params,
) -> Result<Vec<Session>, rusqlite::Error> {
    let mut stmt = db.prepare(sql)?;
    let rows = stmt.query_map(params, |row| {
        Ok(Session {
            id: row.get(0)?,
            user_id: row.get(1)?,
            machine_id: row.get(2)?,
            repo_path: row.get(3)?,
            title: row.get(4)?,
            instructions: row.get(5)?,
            autonomy_level: row.get::<_, String>(6)?
                .parse()
                .unwrap_or(crate::models::AutonomyLevel::Normal),
            work_item_id: row.get(7)?,
            context: row.get(8)?,
            state: row.get::<_, String>(9)?
                .parse()
                .unwrap_or(SessionState::Starting),
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
            started_at: row.get(12)?,
            completed_at: row.get(13)?,
        })
    })?;

    let mut sessions = Vec::new();
    for row in rows {
        sessions.push(row?);
    }
    Ok(sessions)
}

// ---------------------------------------------------------------------------
// POST /api/sessions/{id}/stop
// ---------------------------------------------------------------------------

pub async fn stop_session(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let session = match load_session(&state, &session_id, &auth.user_id) {
        Ok(Some(s)) => s,
        Ok(None) => return not_found("session not found"),
        Err(e) => return internal_error(&format!("db error: {e}")),
    };

    if !matches!(session.state, SessionState::Running | SessionState::WaitingForInput) {
        return conflict(&format!(
            "cannot stop session in {} state, must be running or waiting_for_input",
            session.state
        ));
    }

    if let Err(e) = insert_session_command(&state, &session_id, &session.machine_id, "stop", None) {
        return internal_error(&format!("failed to queue stop command: {e}"));
    }

    Json(serde_json::json!({"status": "stop_requested", "session_id": session_id})).into_response()
}

// ---------------------------------------------------------------------------
// POST /api/sessions/{id}/force-stop
// ---------------------------------------------------------------------------

pub async fn force_stop_session(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let session = match load_session(&state, &session_id, &auth.user_id) {
        Ok(Some(s)) => s,
        Ok(None) => return not_found("session not found"),
        Err(e) => return internal_error(&format!("db error: {e}")),
    };

    if session.state.is_terminal() {
        return conflict(&format!(
            "cannot force-stop session in {} state",
            session.state
        ));
    }

    // If still in starting state (no process yet), transition directly
    if session.state == SessionState::Starting {
        let now = Utc::now().to_rfc3339();
        if let Err(e) = update_session_state(&state, &session_id, "stopped", &now, Some(&now)) {
            return internal_error(&format!("failed to update state: {e}"));
        }
        if let Err(e) = insert_session_event(&state, &session_id, Some("starting"), "stopped", &now, None) {
            tracing::error!("failed to insert event: {e}");
        }
        return Json(serde_json::json!({"status": "stopped", "session_id": session_id})).into_response();
    }

    if let Err(e) = insert_session_command(&state, &session_id, &session.machine_id, "force_stop", None) {
        return internal_error(&format!("failed to queue force_stop command: {e}"));
    }

    Json(serde_json::json!({"status": "force_stop_requested", "session_id": session_id})).into_response()
}

// ---------------------------------------------------------------------------
// POST /api/sessions/{id}/pause
// ---------------------------------------------------------------------------

pub async fn pause_session(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let session = match load_session(&state, &session_id, &auth.user_id) {
        Ok(Some(s)) => s,
        Ok(None) => return not_found("session not found"),
        Err(e) => return internal_error(&format!("db error: {e}")),
    };

    if session.state != SessionState::Running {
        return conflict(&format!(
            "cannot pause session in {} state, must be running",
            session.state
        ));
    }

    if let Err(e) = insert_session_command(&state, &session_id, &session.machine_id, "pause", None) {
        return internal_error(&format!("failed to queue pause command: {e}"));
    }

    Json(serde_json::json!({"status": "pause_requested", "session_id": session_id})).into_response()
}

// ---------------------------------------------------------------------------
// POST /api/sessions/{id}/resume
// ---------------------------------------------------------------------------

pub async fn resume_session(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let session = match load_session(&state, &session_id, &auth.user_id) {
        Ok(Some(s)) => s,
        Ok(None) => return not_found("session not found"),
        Err(e) => return internal_error(&format!("db error: {e}")),
    };

    if !matches!(session.state, SessionState::Paused | SessionState::WaitingForInput) {
        return conflict(&format!(
            "cannot resume session in {} state, must be paused or waiting_for_input",
            session.state
        ));
    }

    if let Err(e) = insert_session_command(&state, &session_id, &session.machine_id, "resume", None) {
        return internal_error(&format!("failed to queue resume command: {e}"));
    }

    Json(serde_json::json!({"status": "resume_requested", "session_id": session_id})).into_response()
}

// ---------------------------------------------------------------------------
// POST /api/sessions/{id}/state  (runner reports state transitions)
// ---------------------------------------------------------------------------

pub async fn report_session_state(
    State(state): State<AppState>,
    MachineTokenAuth(auth): MachineTokenAuth,
    Path(session_id): Path<String>,
    Json(body): Json<ReportStateRequest>,
) -> impl IntoResponse {
    // Load session scoped to the runner's user (via machine token)
    let session = match load_session(&state, &session_id, &auth.user_id) {
        Ok(Some(s)) => s,
        Ok(None) => return not_found("session not found"),
        Err(e) => return internal_error(&format!("db error: {e}")),
    };

    if !session.state.is_valid_transition(&body.state) {
        return conflict(&format!(
            "invalid state transition: {} -> {}",
            session.state, body.state
        ));
    }

    let now = Utc::now().to_rfc3339();
    let completed_at = if body.state.is_terminal() { Some(now.as_str()) } else { None };

    // Update started_at when transitioning to running
    if body.state == SessionState::Running && session.started_at.is_none() {
        if let Err(e) = set_session_started_at(&state, &session_id, &now) {
            tracing::error!("failed to set started_at: {e}");
        }
    }

    if let Err(e) = update_session_state(&state, &session_id, body.state.as_str(), &now, completed_at) {
        return internal_error(&format!("failed to update state: {e}"));
    }

    let metadata_str = body.metadata.as_ref().map(|m| m.to_string());
    if let Err(e) = insert_session_event(
        &state,
        &session_id,
        Some(session.state.as_str()),
        body.state.as_str(),
        &now,
        metadata_str.as_deref(),
    ) {
        tracing::error!("failed to insert event: {e}");
    }

    Json(serde_json::json!({"status": "ok", "state": body.state})).into_response()
}

// ---------------------------------------------------------------------------
// GET /api/machines/{id}/commands  (runner polls for pending commands)
// ---------------------------------------------------------------------------

pub async fn get_machine_commands(
    State(state): State<AppState>,
    MachineTokenAuth(_auth): MachineTokenAuth,
    Path(machine_id): Path<String>,
) -> impl IntoResponse {
    match query_pending_commands(&state, &machine_id) {
        Ok(commands) => Json(serde_json::to_value(commands).unwrap_or_default()).into_response(),
        Err(e) => internal_error(&format!("failed to fetch commands: {e}")),
    }
}

#[allow(clippy::significant_drop_tightening)]
fn query_pending_commands(
    state: &AppState,
    machine_id: &str,
) -> Result<Vec<CommandResponse>, rusqlite::Error> {
    let db = state.db.lock().expect("db lock poisoned");
    let mut stmt = db.prepare(
        "SELECT id, command_type, payload FROM session_commands
         WHERE machine_id = ?1 AND status = 'pending'
         ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map([machine_id], |row| {
        let payload_str: Option<String> = row.get(2)?;
        let payload_json = payload_str
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok());
        Ok(CommandResponse {
            command_id: row.get(0)?,
            command_type: row.get(1)?,
            payload: payload_json,
        })
    })?;

    let mut commands = Vec::new();
    for row in rows {
        commands.push(row?);
    }
    Ok(commands)
}

// ---------------------------------------------------------------------------
// POST /api/machines/{id}/commands/{cmd_id}/ack  (runner acknowledges receipt)
// ---------------------------------------------------------------------------

pub async fn ack_command(
    State(state): State<AppState>,
    MachineTokenAuth(_auth): MachineTokenAuth,
    Path((machine_id, cmd_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    match set_command_delivered(&state, &cmd_id, &machine_id, &now) {
        Ok(updated) if updated > 0 => {
            Json(serde_json::json!({"status": "ok"})).into_response()
        }
        Ok(_) => not_found("command not found or not pending"),
        Err(e) => internal_error(&format!("failed to ack command: {e}")),
    }
}

fn set_command_delivered(
    state: &AppState,
    cmd_id: &str,
    machine_id: &str,
    now: &str,
) -> Result<usize, rusqlite::Error> {
    let updated = state
        .db
        .lock()
        .expect("db lock poisoned")
        .execute(
            "UPDATE session_commands SET status = 'delivered', delivered_at = ?1
             WHERE id = ?2 AND machine_id = ?3 AND status = 'pending'",
            params![now, cmd_id, machine_id],
        )?;
    Ok(updated)
}

fn insert_session_command(
    state: &AppState,
    session_id: &str,
    machine_id: &str,
    command_type: &str,
    payload: Option<&str>,
) -> Result<(), rusqlite::Error> {
    let cmd_id = uuid::Uuid::new_v4().to_string();
    state
        .db
        .lock()
        .expect("db lock poisoned")
        .execute(
            "INSERT INTO session_commands (id, session_id, machine_id, command_type, payload, status)
             VALUES (?1, ?2, ?3, ?4, ?5, 'pending')",
            params![cmd_id, session_id, machine_id, command_type, payload],
        )?;
    Ok(())
}

fn update_session_state(
    state: &AppState,
    session_id: &str,
    new_state: &str,
    now: &str,
    completed_at: Option<&str>,
) -> Result<(), rusqlite::Error> {
    if let Some(cat) = completed_at {
        state
            .db
            .lock()
            .expect("db lock poisoned")
            .execute(
                "UPDATE sessions SET state = ?1, updated_at = ?2, completed_at = ?3 WHERE id = ?4",
                params![new_state, now, cat, session_id],
            )?;
    } else {
        state
            .db
            .lock()
            .expect("db lock poisoned")
            .execute(
                "UPDATE sessions SET state = ?1, updated_at = ?2 WHERE id = ?3",
                params![new_state, now, session_id],
            )?;
    }
    Ok(())
}

fn set_session_started_at(
    state: &AppState,
    session_id: &str,
    now: &str,
) -> Result<(), rusqlite::Error> {
    state
        .db
        .lock()
        .expect("db lock poisoned")
        .execute(
            "UPDATE sessions SET started_at = ?1 WHERE id = ?2 AND started_at IS NULL",
            params![now, session_id],
        )?;
    Ok(())
}

fn session_to_response(session: Session) -> SessionResponse {
    SessionResponse {
        id: session.id,
        machine_id: session.machine_id,
        repo_path: session.repo_path,
        title: session.title,
        instructions: session.instructions,
        autonomy_level: session.autonomy_level,
        work_item_id: session.work_item_id,
        context: session.context,
        state: session.state,
        created_at: session.created_at,
        updated_at: session.updated_at,
        started_at: session.started_at,
        completed_at: session.completed_at,
    }
}

// ---------------------------------------------------------------------------
// GET /api/policy-violations
// ---------------------------------------------------------------------------

pub async fn list_policy_violations(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Query(query): Query<ListViolationsQuery>,
) -> impl IntoResponse {
    match query_violations(&state, &auth.user_id, &query) {
        Ok(result) => Json(serde_json::to_value(result).unwrap_or_default()).into_response(),
        Err(e) => internal_error(&format!("db error: {e}")),
    }
}

// ---------------------------------------------------------------------------
// GET /api/sessions/{id}/violations
// ---------------------------------------------------------------------------

pub async fn list_session_violations(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    match query_violations_by_session(&state, &session_id, &auth.user_id) {
        Ok(violations) => Json(serde_json::to_value(violations).unwrap_or_default()).into_response(),
        Err(e) => internal_error(&format!("db error: {e}")),
    }
}

fn insert_policy_violation(
    state: &AppState,
    session_id: Option<&str>,
    machine_id: &str,
    user_id: &str,
    action: &str,
    policy_scope: &str,
    reason: &str,
    repo_path: Option<&str>,
) -> Result<(), rusqlite::Error> {
    let violation_id = uuid::Uuid::new_v4().to_string();
    state
        .db
        .lock()
        .expect("db lock poisoned")
        .execute(
            "INSERT INTO policy_violations (id, session_id, machine_id, user_id, action, policy_scope, reason, repo_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![violation_id, session_id, machine_id, user_id, action, policy_scope, reason, repo_path],
        )?;
    Ok(())
}

#[allow(clippy::significant_drop_tightening)]
fn query_violations(
    state: &AppState,
    user_id: &str,
    query: &ListViolationsQuery,
) -> Result<ViolationsListResponse, rusqlite::Error> {
    let db = state.db.lock().expect("db lock poisoned");

    // Build WHERE clause dynamically
    let mut conditions = vec!["user_id = ?1".to_string()];
    let mut param_idx = 2;

    if query.machine_id.is_some() {
        conditions.push(format!("machine_id = ?{param_idx}"));
        param_idx += 1;
    }
    if query.session_id.is_some() {
        conditions.push(format!("session_id = ?{param_idx}"));
        param_idx += 1;
    }

    let where_clause = conditions.join(" AND ");

    let count_sql = format!("SELECT COUNT(*) FROM policy_violations WHERE {where_clause}");
    let list_sql = format!(
        "SELECT id, session_id, machine_id, user_id, action, policy_scope, reason, repo_path, timestamp
         FROM policy_violations WHERE {where_clause}
         ORDER BY timestamp DESC
         LIMIT ?{} OFFSET ?{}",
        param_idx,
        param_idx + 1
    );

    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    // Execute count
    let total: i64 = match (&query.machine_id, &query.session_id) {
        (None, None) => {
            db.query_row(&count_sql, [user_id], |row| row.get(0))?
        }
        (Some(mid), None) => {
            db.query_row(&count_sql, params![user_id, mid], |row| row.get(0))?
        }
        (None, Some(sid)) => {
            db.query_row(&count_sql, params![user_id, sid], |row| row.get(0))?
        }
        (Some(mid), Some(sid)) => {
            db.query_row(&count_sql, params![user_id, mid, sid], |row| row.get(0))?
        }
    };

    // Execute list
    let violations: Vec<PolicyViolationRecord> = match (&query.machine_id, &query.session_id) {
        (None, None) => {
            query_violation_rows(&db, &list_sql, params![user_id, limit, offset])?
        }
        (Some(mid), None) => {
            query_violation_rows(&db, &list_sql, params![user_id, mid, limit, offset])?
        }
        (None, Some(sid)) => {
            query_violation_rows(&db, &list_sql, params![user_id, sid, limit, offset])?
        }
        (Some(mid), Some(sid)) => {
            query_violation_rows(&db, &list_sql, params![user_id, mid, sid, limit, offset])?
        }
    };

    Ok(ViolationsListResponse { violations, total })
}

#[allow(clippy::significant_drop_tightening)]
fn query_violation_rows(
    db: &rusqlite::Connection,
    sql: &str,
    params: impl rusqlite::Params,
) -> Result<Vec<PolicyViolationRecord>, rusqlite::Error> {
    let mut stmt = db.prepare(sql)?;
    let rows = stmt.query_map(params, |row| {
        Ok(PolicyViolationRecord {
            id: row.get(0)?,
            session_id: row.get(1)?,
            machine_id: row.get(2)?,
            user_id: row.get(3)?,
            action: row.get(4)?,
            policy_scope: row.get(5)?,
            reason: row.get(6)?,
            repo_path: row.get(7)?,
            timestamp: row.get(8)?,
        })
    })?;

    let mut violations = Vec::new();
    for row in rows {
        violations.push(row?);
    }
    Ok(violations)
}

#[allow(clippy::significant_drop_tightening)]
fn query_violations_by_session(
    state: &AppState,
    session_id: &str,
    user_id: &str,
) -> Result<Vec<PolicyViolationRecord>, rusqlite::Error> {
    let db = state.db.lock().expect("db lock poisoned");
    let mut stmt = db.prepare(
        "SELECT id, session_id, machine_id, user_id, action, policy_scope, reason, repo_path, timestamp
         FROM policy_violations WHERE session_id = ?1 AND user_id = ?2
         ORDER BY timestamp DESC",
    )?;
    let rows = stmt.query_map(params![session_id, user_id], |row| {
        Ok(PolicyViolationRecord {
            id: row.get(0)?,
            session_id: row.get(1)?,
            machine_id: row.get(2)?,
            user_id: row.get(3)?,
            action: row.get(4)?,
            policy_scope: row.get(5)?,
            reason: row.get(6)?,
            repo_path: row.get(7)?,
            timestamp: row.get(8)?,
        })
    })?;

    let mut violations = Vec::new();
    for row in rows {
        violations.push(row?);
    }
    Ok(violations)
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

fn bad_request(msg: &str) -> axum::response::Response {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::to_value(ErrorBody {
            error: msg.to_string(),
        })
        .unwrap_or_default()),
    )
        .into_response()
}

pub(crate) fn conflict(msg: &str) -> axum::response::Response {
    (
        StatusCode::CONFLICT,
        Json(serde_json::to_value(ErrorBody {
            error: msg.to_string(),
        })
        .unwrap_or_default()),
    )
        .into_response()
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

// ---------------------------------------------------------------------------
// Policy helpers
// ---------------------------------------------------------------------------

fn default_machine_policy(machine_id: &str) -> MachinePolicy {
    MachinePolicy {
        id: uuid::Uuid::new_v4().to_string(),
        machine_id: machine_id.to_string(),
        allowed_categories: ActionCategory::all(),
        blocked_categories: vec![],
        max_autonomy_level: AutonomyLevel::Autonomous,
        session_mode: SessionMode::Normal,
        require_approval_for: vec![],
        created_at: String::new(),
        updated_at: String::new(),
    }
}

fn insert_default_machine_policy(
    state: &AppState,
    machine_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let policy = default_machine_policy(machine_id);
    upsert_machine_policy(state, machine_id, &policy)
}

fn load_machine_policy(
    state: &AppState,
    machine_id: &str,
) -> Result<Option<MachinePolicy>, rusqlite::Error> {
    let result = state.db.lock().expect("db lock poisoned").query_row(
        "SELECT id, machine_id, allowed_categories, blocked_categories, max_autonomy_level,
                session_mode, require_approval_for, created_at, updated_at
         FROM machine_policies WHERE machine_id = ?1",
        params![machine_id],
        |row| {
            let allowed_str: String = row.get(2)?;
            let blocked_str: String = row.get(3)?;
            let max_autonomy_str: String = row.get(4)?;
            let session_mode_str: String = row.get(5)?;
            let approval_str: String = row.get(6)?;

            Ok(MachinePolicyRow {
                id: row.get(0)?,
                machine_id: row.get(1)?,
                allowed_categories: allowed_str,
                blocked_categories: blocked_str,
                max_autonomy_level: max_autonomy_str,
                session_mode: session_mode_str,
                require_approval_for: approval_str,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        },
    );

    match result {
        Ok(row) => Ok(Some(parse_machine_policy_row(row))),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

struct MachinePolicyRow {
    id: String,
    machine_id: String,
    allowed_categories: String,
    blocked_categories: String,
    max_autonomy_level: String,
    session_mode: String,
    require_approval_for: String,
    created_at: String,
    updated_at: String,
}

fn parse_machine_policy_row(row: MachinePolicyRow) -> MachinePolicy {
    MachinePolicy {
        id: row.id,
        machine_id: row.machine_id,
        allowed_categories: serde_json::from_str(&row.allowed_categories).unwrap_or_default(),
        blocked_categories: serde_json::from_str(&row.blocked_categories).unwrap_or_default(),
        max_autonomy_level: row
            .max_autonomy_level
            .parse()
            .unwrap_or(AutonomyLevel::Normal),
        session_mode: row.session_mode.parse().unwrap_or(SessionMode::Normal),
        require_approval_for: serde_json::from_str(&row.require_approval_for).unwrap_or_default(),
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn upsert_machine_policy(
    state: &AppState,
    machine_id: &str,
    policy: &MachinePolicy,
) -> Result<(), Box<dyn std::error::Error>> {
    let now = Utc::now().to_rfc3339();
    let allowed = serde_json::to_string(&policy.allowed_categories)?;
    let blocked = serde_json::to_string(&policy.blocked_categories)?;
    let approval = serde_json::to_string(&policy.require_approval_for)?;
    let policy_id = uuid::Uuid::new_v4().to_string();

    state
        .db
        .lock()
        .expect("db lock poisoned")
        .execute(
            "INSERT INTO machine_policies (id, machine_id, allowed_categories, blocked_categories,
             max_autonomy_level, session_mode, require_approval_for, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(machine_id) DO UPDATE SET
                allowed_categories = excluded.allowed_categories,
                blocked_categories = excluded.blocked_categories,
                max_autonomy_level = excluded.max_autonomy_level,
                session_mode = excluded.session_mode,
                require_approval_for = excluded.require_approval_for,
                updated_at = excluded.updated_at",
            params![
                policy_id,
                machine_id,
                allowed,
                blocked,
                policy.max_autonomy_level.as_str(),
                policy.session_mode.as_str(),
                approval,
                now,
                now
            ],
        )?;
    Ok(())
}

fn load_repo_policy(
    state: &AppState,
    machine_id: &str,
    repo_path: &str,
) -> Result<Option<RepoPolicy>, rusqlite::Error> {
    let result = state.db.lock().expect("db lock poisoned").query_row(
        "SELECT id, machine_id, repo_path, allowed_categories, blocked_categories,
                max_autonomy_level, require_approval_for, created_at, updated_at
         FROM repo_policies WHERE machine_id = ?1 AND repo_path = ?2",
        params![machine_id, repo_path],
        |row| {
            let allowed_str: String = row.get(3)?;
            let blocked_str: String = row.get(4)?;
            let max_autonomy_str: Option<String> = row.get(5)?;
            let approval_str: String = row.get(6)?;

            Ok(RepoPolicyRow {
                id: row.get(0)?,
                machine_id: row.get(1)?,
                repo_path: row.get(2)?,
                allowed_categories: allowed_str,
                blocked_categories: blocked_str,
                max_autonomy_level: max_autonomy_str,
                require_approval_for: approval_str,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        },
    );

    match result {
        Ok(row) => Ok(Some(parse_repo_policy_row(row))),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

struct RepoPolicyRow {
    id: String,
    machine_id: String,
    repo_path: String,
    allowed_categories: String,
    blocked_categories: String,
    max_autonomy_level: Option<String>,
    require_approval_for: String,
    created_at: String,
    updated_at: String,
}

fn parse_repo_policy_row(row: RepoPolicyRow) -> RepoPolicy {
    RepoPolicy {
        id: row.id,
        machine_id: row.machine_id,
        repo_path: row.repo_path,
        allowed_categories: serde_json::from_str(&row.allowed_categories).unwrap_or_default(),
        blocked_categories: serde_json::from_str(&row.blocked_categories).unwrap_or_default(),
        max_autonomy_level: row
            .max_autonomy_level
            .and_then(|s| s.parse().ok()),
        require_approval_for: serde_json::from_str(&row.require_approval_for).unwrap_or_default(),
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn upsert_repo_policy(
    state: &AppState,
    machine_id: &str,
    repo_path: &str,
    policy: &RepoPolicy,
) -> Result<(), Box<dyn std::error::Error>> {
    let now = Utc::now().to_rfc3339();
    let allowed = serde_json::to_string(&policy.allowed_categories)?;
    let blocked = serde_json::to_string(&policy.blocked_categories)?;
    let approval = serde_json::to_string(&policy.require_approval_for)?;
    let max_autonomy = policy.max_autonomy_level.as_ref().map(|a| a.as_str().to_string());
    let policy_id = uuid::Uuid::new_v4().to_string();

    state
        .db
        .lock()
        .expect("db lock poisoned")
        .execute(
            "INSERT INTO repo_policies (id, machine_id, repo_path, allowed_categories, blocked_categories,
             max_autonomy_level, require_approval_for, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(machine_id, repo_path) DO UPDATE SET
                allowed_categories = excluded.allowed_categories,
                blocked_categories = excluded.blocked_categories,
                max_autonomy_level = excluded.max_autonomy_level,
                require_approval_for = excluded.require_approval_for,
                updated_at = excluded.updated_at",
            params![
                policy_id,
                machine_id,
                repo_path,
                allowed,
                blocked,
                max_autonomy,
                approval,
                now,
                now
            ],
        )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// GET /api/machines/{id}/policy
// ---------------------------------------------------------------------------

pub async fn get_machine_policy(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Path(machine_id): Path<String>,
) -> impl IntoResponse {
    // Validate machine belongs to user
    match load_machine(&state, &machine_id, &auth.user_id) {
        Ok(Some(_)) => {}
        Ok(None) => return not_found("machine not found"),
        Err(e) => return internal_error(&format!("db error: {e}")),
    }

    match load_machine_policy(&state, &machine_id) {
        Ok(Some(policy)) => Json(serde_json::to_value(policy).unwrap_or_default()).into_response(),
        Ok(None) => {
            // Return a default policy (not persisted)
            let default = default_machine_policy(&machine_id);
            Json(serde_json::to_value(default).unwrap_or_default()).into_response()
        }
        Err(e) => internal_error(&format!("db error: {e}")),
    }
}

// ---------------------------------------------------------------------------
// PUT /api/machines/{id}/policy
// ---------------------------------------------------------------------------

pub async fn update_machine_policy(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Path(machine_id): Path<String>,
    Json(body): Json<UpdateMachinePolicyRequest>,
) -> impl IntoResponse {
    // Validate machine belongs to user
    match load_machine(&state, &machine_id, &auth.user_id) {
        Ok(Some(_)) => {}
        Ok(None) => return not_found("machine not found"),
        Err(e) => return internal_error(&format!("db error: {e}")),
    }

    // Load existing or create default, then apply updates
    let mut policy = match load_machine_policy(&state, &machine_id) {
        Ok(Some(p)) => p,
        Ok(None) => default_machine_policy(&machine_id),
        Err(e) => return internal_error(&format!("db error: {e}")),
    };

    if let Some(allowed) = body.allowed_categories {
        policy.allowed_categories = allowed;
    }
    if let Some(blocked) = body.blocked_categories {
        policy.blocked_categories = blocked;
    }
    if let Some(max) = body.max_autonomy_level {
        policy.max_autonomy_level = max;
    }
    if let Some(mode) = body.session_mode {
        policy.session_mode = mode;
    }
    if let Some(approval) = body.require_approval_for {
        policy.require_approval_for = approval;
    }

    if let Err(e) = upsert_machine_policy(&state, &machine_id, &policy) {
        return internal_error(&format!("failed to upsert machine policy: {e}"));
    }

    // Reload to get updated timestamps
    match load_machine_policy(&state, &machine_id) {
        Ok(Some(p)) => Json(serde_json::to_value(p).unwrap_or_default()).into_response(),
        Ok(None) => internal_error("policy not found after upsert"),
        Err(e) => internal_error(&format!("db error: {e}")),
    }
}

// ---------------------------------------------------------------------------
// GET /api/machines/{id}/repo-policy?repo_path=X
// ---------------------------------------------------------------------------

pub async fn get_repo_policy(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Path(machine_id): Path<String>,
    Query(query): Query<RepoPolicyQuery>,
) -> impl IntoResponse {
    match load_machine(&state, &machine_id, &auth.user_id) {
        Ok(Some(_)) => {}
        Ok(None) => return not_found("machine not found"),
        Err(e) => return internal_error(&format!("db error: {e}")),
    }

    match load_repo_policy(&state, &machine_id, &query.repo_path) {
        Ok(Some(policy)) => Json(serde_json::to_value(policy).unwrap_or_default()).into_response(),
        Ok(None) => {
            // Return empty default repo policy
            let default = RepoPolicy {
                id: String::new(),
                machine_id: machine_id.clone(),
                repo_path: query.repo_path,
                allowed_categories: vec![],
                blocked_categories: vec![],
                max_autonomy_level: None,
                require_approval_for: vec![],
                created_at: String::new(),
                updated_at: String::new(),
            };
            Json(serde_json::to_value(default).unwrap_or_default()).into_response()
        }
        Err(e) => internal_error(&format!("db error: {e}")),
    }
}

// ---------------------------------------------------------------------------
// PUT /api/machines/{id}/repo-policy?repo_path=X
// ---------------------------------------------------------------------------

pub async fn update_repo_policy(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Path(machine_id): Path<String>,
    Query(query): Query<RepoPolicyQuery>,
    Json(body): Json<UpdateRepoPolicyRequest>,
) -> impl IntoResponse {
    match load_machine(&state, &machine_id, &auth.user_id) {
        Ok(Some(_)) => {}
        Ok(None) => return not_found("machine not found"),
        Err(e) => return internal_error(&format!("db error: {e}")),
    }

    // Load existing or create default
    let mut policy = match load_repo_policy(&state, &machine_id, &query.repo_path) {
        Ok(Some(p)) => p,
        Ok(None) => RepoPolicy {
            id: uuid::Uuid::new_v4().to_string(),
            machine_id: machine_id.clone(),
            repo_path: query.repo_path.clone(),
            allowed_categories: vec![],
            blocked_categories: vec![],
            max_autonomy_level: None,
            require_approval_for: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        },
        Err(e) => return internal_error(&format!("db error: {e}")),
    };

    if let Some(allowed) = body.allowed_categories {
        policy.allowed_categories = allowed;
    }
    if let Some(blocked) = body.blocked_categories {
        policy.blocked_categories = blocked;
    }
    if let Some(max) = body.max_autonomy_level {
        policy.max_autonomy_level = Some(max);
    }
    if let Some(approval) = body.require_approval_for {
        policy.require_approval_for = approval;
    }

    if let Err(e) = upsert_repo_policy(&state, &machine_id, &query.repo_path, &policy) {
        return internal_error(&format!("failed to upsert repo policy: {e}"));
    }

    match load_repo_policy(&state, &machine_id, &query.repo_path) {
        Ok(Some(p)) => Json(serde_json::to_value(p).unwrap_or_default()).into_response(),
        Ok(None) => internal_error("repo policy not found after upsert"),
        Err(e) => internal_error(&format!("db error: {e}")),
    }
}

// ---------------------------------------------------------------------------
// GET /api/machines/{id}/policy/effective?repo_path=X
// ---------------------------------------------------------------------------

pub async fn get_effective_policy(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Path(machine_id): Path<String>,
    Query(query): Query<EffectivePolicyQuery>,
) -> impl IntoResponse {
    match load_machine(&state, &machine_id, &auth.user_id) {
        Ok(Some(_)) => {}
        Ok(None) => return not_found("machine not found"),
        Err(e) => return internal_error(&format!("db error: {e}")),
    }

    let machine_policy = match load_machine_policy(&state, &machine_id) {
        Ok(Some(p)) => p,
        Ok(None) => default_machine_policy(&machine_id),
        Err(e) => return internal_error(&format!("db error: {e}")),
    };

    let repo_policy = if let Some(ref repo_path) = query.repo_path {
        match load_repo_policy(&state, &machine_id, repo_path) {
            Ok(p) => p,
            Err(e) => return internal_error(&format!("db error: {e}")),
        }
    } else {
        None
    };

    let effective = merge_policies(&machine_policy, repo_policy.as_ref());
    Json(serde_json::to_value(effective).unwrap_or_default()).into_response()
}

fn merge_policies(
    machine: &MachinePolicy,
    repo: Option<&RepoPolicy>,
) -> EffectivePolicy {
    let mut allowed = machine.allowed_categories.clone();
    let mut blocked = machine.blocked_categories.clone();
    let mut max_autonomy = machine.max_autonomy_level.clone();
    let mut approval = machine.require_approval_for.clone();

    if let Some(rp) = repo {
        // Repo blocked categories are additive (union)
        for cat in &rp.blocked_categories {
            if !blocked.contains(cat) {
                blocked.push(cat.clone());
            }
        }

        // If repo has allowed categories, narrow the machine allowed list (intersection)
        if !rp.allowed_categories.is_empty() {
            allowed.retain(|c| rp.allowed_categories.contains(c));
        }

        // Repo max autonomy can only narrow (pick the more restrictive)
        if let Some(ref repo_max) = rp.max_autonomy_level {
            let rank = |level: &AutonomyLevel| -> u8 {
                match level {
                    AutonomyLevel::Normal | AutonomyLevel::Stricter => 0,
                    AutonomyLevel::Autonomous => 2,
                }
            };
            if rank(repo_max) < rank(&max_autonomy) {
                max_autonomy = repo_max.clone();
            }
        }

        // Merge approval lists (union)
        for cat in &rp.require_approval_for {
            if !approval.contains(cat) {
                approval.push(cat.clone());
            }
        }
    }

    // Remove blocked from allowed
    allowed.retain(|c| !blocked.contains(c));

    EffectivePolicy {
        allowed_categories: allowed,
        blocked_categories: blocked,
        max_autonomy_level: max_autonomy,
        session_mode: machine.session_mode.clone(),
        require_approval_for: approval,
    }
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

    fn create_trusted_machine(state: &AppState, machine_id: &str) {
        let now = Utc::now().to_rfc3339();
        let req = RegisterRequest {
            name: "trusted-machine".into(),
            platform: "linux".into(),
            capabilities: None,
            repos: None,
        };
        insert_machine(state, machine_id, "user-default", &req, &now).unwrap();
        set_machine_status(state, machine_id, "trusted", "basic", &now).unwrap();
    }

    #[test]
    fn test_create_session() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        let now = Utc::now().to_rfc3339();

        let body = CreateSessionRequest {
            machine_id: "m-1".into(),
            repo_path: "/home/user/project".into(),
            title: "Test session".into(),
            instructions: "Fix the bug".into(),
            autonomy_level: None,
            work_item_id: Some("SMET-T-0100".into()),
            context: None,
        };

        insert_session(&state, "s-1", "user-default", &body, "normal", &now).unwrap();
        insert_session_event(&state, "s-1", None, "starting", &now, None).unwrap();

        let session = load_session(&state, "s-1", "user-default").unwrap();
        assert!(session.is_some());
        let s = session.unwrap();
        assert_eq!(s.title, "Test session");
        assert_eq!(s.state, SessionState::Starting);
        assert_eq!(s.work_item_id, Some("SMET-T-0100".into()));
    }

    #[test]
    fn test_load_session_wrong_user() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        let now = Utc::now().to_rfc3339();

        let body = CreateSessionRequest {
            machine_id: "m-1".into(),
            repo_path: "/project".into(),
            title: "Test".into(),
            instructions: "Do stuff".into(),
            autonomy_level: None,
            work_item_id: None,
            context: None,
        };
        insert_session(&state, "s-1", "user-default", &body, "normal", &now).unwrap();

        let session = load_session(&state, "s-1", "other-user").unwrap();
        assert!(session.is_none());
    }

    #[test]
    fn test_list_sessions_with_filter() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_trusted_machine(&state, "m-2");
        let now = Utc::now().to_rfc3339();

        for i in 0..3 {
            let body = CreateSessionRequest {
                machine_id: if i < 2 { "m-1" } else { "m-2" }.into(),
                repo_path: "/project".into(),
                title: format!("Session {i}"),
                instructions: "Work".into(),
                autonomy_level: None,
                work_item_id: None,
                context: None,
            };
            insert_session(&state, &format!("s-{i}"), "user-default", &body, "normal", &now).unwrap();
        }

        // All sessions
        let query = ListSessionsQuery {
            machine_id: None,
            repo_path: None,
            state: None,
            limit: None,
            offset: None,
        };
        let result = query_sessions(&state, "user-default", &query).unwrap();
        assert_eq!(result.total, 3);
        assert_eq!(result.sessions.len(), 3);

        // Filter by machine
        let query = ListSessionsQuery {
            machine_id: Some("m-1".into()),
            repo_path: None,
            state: None,
            limit: None,
            offset: None,
        };
        let result = query_sessions(&state, "user-default", &query).unwrap();
        assert_eq!(result.total, 2);

        // Pagination
        let query = ListSessionsQuery {
            machine_id: None,
            repo_path: None,
            state: None,
            limit: Some(1),
            offset: Some(0),
        };
        let result = query_sessions(&state, "user-default", &query).unwrap();
        assert_eq!(result.total, 3);
        assert_eq!(result.sessions.len(), 1);
    }

    fn create_running_session(state: &AppState, session_id: &str, machine_id: &str) {
        let now = Utc::now().to_rfc3339();
        let body = CreateSessionRequest {
            machine_id: machine_id.into(),
            repo_path: "/project".into(),
            title: "Test".into(),
            instructions: "Work".into(),
            autonomy_level: None,
            work_item_id: None,
            context: None,
        };
        insert_session(state, session_id, "user-default", &body, "normal", &now).unwrap();
        // Transition to running
        update_session_state(state, session_id, "running", &now, None).unwrap();
        set_session_started_at(state, session_id, &now).unwrap();
    }

    #[test]
    fn test_insert_session_command() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_running_session(&state, "s-1", "m-1");

        insert_session_command(&state, "s-1", "m-1", "stop", None).unwrap();

        let count: i64 = state
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM session_commands WHERE session_id = ?1 AND command_type = 'stop'",
                ["s-1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_update_session_state() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        let now = Utc::now().to_rfc3339();
        let body = CreateSessionRequest {
            machine_id: "m-1".into(),
            repo_path: "/project".into(),
            title: "Test".into(),
            instructions: "Work".into(),
            autonomy_level: None,
            work_item_id: None,
            context: None,
        };
        insert_session(&state, "s-1", "user-default", &body, "normal", &now).unwrap();

        // Transition to running
        update_session_state(&state, "s-1", "running", &now, None).unwrap();
        let s = load_session(&state, "s-1", "user-default").unwrap().unwrap();
        assert_eq!(s.state, SessionState::Running);

        // Transition to completed with completed_at
        update_session_state(&state, "s-1", "completed", &now, Some(&now)).unwrap();
        let s = load_session(&state, "s-1", "user-default").unwrap().unwrap();
        assert_eq!(s.state, SessionState::Completed);
        assert!(s.completed_at.is_some());
    }

    #[test]
    fn test_set_session_started_at() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        let now = Utc::now().to_rfc3339();
        let body = CreateSessionRequest {
            machine_id: "m-1".into(),
            repo_path: "/project".into(),
            title: "Test".into(),
            instructions: "Work".into(),
            autonomy_level: None,
            work_item_id: None,
            context: None,
        };
        insert_session(&state, "s-1", "user-default", &body, "normal", &now).unwrap();

        let s = load_session(&state, "s-1", "user-default").unwrap().unwrap();
        assert!(s.started_at.is_none());

        set_session_started_at(&state, "s-1", &now).unwrap();
        let s = load_session(&state, "s-1", "user-default").unwrap().unwrap();
        assert!(s.started_at.is_some());

        // Second call should not overwrite
        let later = "2099-01-01T00:00:00Z";
        set_session_started_at(&state, "s-1", later).unwrap();
        let s = load_session(&state, "s-1", "user-default").unwrap().unwrap();
        assert_ne!(s.started_at.unwrap(), later);
    }

    #[test]
    fn test_session_event_inserted() {
        let state = test_state();
        let now = Utc::now().to_rfc3339();

        // Create the session table entry first
        create_trusted_machine(&state, "m-1");
        let body = CreateSessionRequest {
            machine_id: "m-1".into(),
            repo_path: "/project".into(),
            title: "Test".into(),
            instructions: "Work".into(),
            autonomy_level: None,
            work_item_id: None,
            context: None,
        };
        insert_session(&state, "s-1", "user-default", &body, "normal", &now).unwrap();
        insert_session_event(&state, "s-1", None, "starting", &now, None).unwrap();

        let count: i64 = state
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM session_events WHERE session_id = ?1",
                ["s-1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_create_session_inserts_start_session_command() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        let now = Utc::now().to_rfc3339();

        let body = CreateSessionRequest {
            machine_id: "m-1".into(),
            repo_path: "/home/user/project".into(),
            title: "Test session".into(),
            instructions: "Fix the bug".into(),
            autonomy_level: None,
            work_item_id: None,
            context: Some("some context".into()),
        };

        insert_session(&state, "s-1", "user-default", &body, "normal", &now).unwrap();
        insert_session_event(&state, "s-1", None, "starting", &now, None).unwrap();

        // Queue the start_session command (mirrors what create_session handler does)
        let payload = serde_json::json!({
            "session_id": "s-1",
            "repo_path": body.repo_path,
            "title": body.title,
            "instructions": body.instructions,
            "autonomy_level": "normal",
            "context": body.context,
        });
        insert_session_command(&state, "s-1", "m-1", "start_session", Some(&payload.to_string()))
            .unwrap();

        // Verify command was inserted
        let db = state.db.lock().unwrap();
        let (cmd_type, cmd_status, cmd_payload): (String, String, Option<String>) = db
            .query_row(
                "SELECT command_type, status, payload FROM session_commands
                 WHERE session_id = ?1 AND command_type = 'start_session'",
                ["s-1"],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(cmd_type, "start_session");
        assert_eq!(cmd_status, "pending");

        let parsed: serde_json::Value = serde_json::from_str(&cmd_payload.unwrap()).unwrap();
        assert_eq!(parsed["session_id"], "s-1");
        assert_eq!(parsed["repo_path"], "/home/user/project");
        assert_eq!(parsed["title"], "Test session");
        assert_eq!(parsed["instructions"], "Fix the bug");
        assert_eq!(parsed["context"], "some context");
    }

    #[test]
    fn test_get_pending_commands() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_running_session(&state, "s-1", "m-1");

        // Insert two pending commands
        insert_session_command(&state, "s-1", "m-1", "stop", None).unwrap();
        insert_session_command(&state, "s-1", "m-1", "pause", None).unwrap();

        let commands = query_pending_commands(&state, "m-1").unwrap();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].command_type, "stop");
        assert_eq!(commands[1].command_type, "pause");
    }

    #[test]
    fn test_get_pending_commands_excludes_delivered() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_running_session(&state, "s-1", "m-1");

        insert_session_command(&state, "s-1", "m-1", "stop", None).unwrap();

        // Mark the command as delivered
        let now = Utc::now().to_rfc3339();
        let cmd_id: String = state
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT id FROM session_commands WHERE session_id = ?1",
                ["s-1"],
                |row| row.get(0),
            )
            .unwrap();
        set_command_delivered(&state, &cmd_id, "m-1", &now).unwrap();

        let commands = query_pending_commands(&state, "m-1").unwrap();
        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_ack_sets_delivered_status() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_running_session(&state, "s-1", "m-1");

        insert_session_command(&state, "s-1", "m-1", "stop", None).unwrap();

        let cmd_id: String = state
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT id FROM session_commands WHERE session_id = ?1",
                ["s-1"],
                |row| row.get(0),
            )
            .unwrap();

        let now = Utc::now().to_rfc3339();
        let updated = set_command_delivered(&state, &cmd_id, "m-1", &now).unwrap();
        assert_eq!(updated, 1);

        // Verify status changed
        let (status, delivered_at): (String, Option<String>) = state
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT status, delivered_at FROM session_commands WHERE id = ?1",
                [&cmd_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(status, "delivered");
        assert!(delivered_at.is_some());
    }

    #[test]
    fn test_ack_wrong_machine_returns_zero() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_trusted_machine(&state, "m-2");
        create_running_session(&state, "s-1", "m-1");

        insert_session_command(&state, "s-1", "m-1", "stop", None).unwrap();

        let cmd_id: String = state
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT id FROM session_commands WHERE session_id = ?1",
                ["s-1"],
                |row| row.get(0),
            )
            .unwrap();

        let now = Utc::now().to_rfc3339();
        // Try to ack with wrong machine
        let updated = set_command_delivered(&state, &cmd_id, "m-2", &now).unwrap();
        assert_eq!(updated, 0);
    }

    #[test]
    fn test_ack_already_delivered_returns_zero() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_running_session(&state, "s-1", "m-1");

        insert_session_command(&state, "s-1", "m-1", "stop", None).unwrap();

        let cmd_id: String = state
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT id FROM session_commands WHERE session_id = ?1",
                ["s-1"],
                |row| row.get(0),
            )
            .unwrap();

        let now = Utc::now().to_rfc3339();
        let updated = set_command_delivered(&state, &cmd_id, "m-1", &now).unwrap();
        assert_eq!(updated, 1);

        // Second ack should return 0
        let updated = set_command_delivered(&state, &cmd_id, "m-1", &now).unwrap();
        assert_eq!(updated, 0);
    }

    #[test]
    fn test_pending_commands_with_payload() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_running_session(&state, "s-1", "m-1");

        let payload = serde_json::json!({"session_id": "s-1", "repo_path": "/project"});
        insert_session_command(
            &state,
            "s-1",
            "m-1",
            "start_session",
            Some(&payload.to_string()),
        )
        .unwrap();

        let commands = query_pending_commands(&state, "m-1").unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].command_type, "start_session");
        let p = commands[0].payload.as_ref().unwrap();
        assert_eq!(p["session_id"], "s-1");
        assert_eq!(p["repo_path"], "/project");
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

    // -- Policy tests --

    #[test]
    fn test_load_machine_policy_returns_none_when_absent() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");

        let policy = load_machine_policy(&state, "m-1").unwrap();
        assert!(policy.is_none());
    }

    #[test]
    fn test_upsert_and_load_machine_policy() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");

        let policy = MachinePolicy {
            id: "p-1".into(),
            machine_id: "m-1".into(),
            allowed_categories: vec![ActionCategory::ReadFiles, ActionCategory::WriteFiles],
            blocked_categories: vec![ActionCategory::InstallPackages],
            max_autonomy_level: AutonomyLevel::Normal,
            session_mode: SessionMode::Restricted,
            require_approval_for: vec![ActionCategory::GitOperations],
            created_at: String::new(),
            updated_at: String::new(),
        };
        upsert_machine_policy(&state, "m-1", &policy).unwrap();

        let loaded = load_machine_policy(&state, "m-1").unwrap().unwrap();
        assert_eq!(loaded.machine_id, "m-1");
        assert_eq!(loaded.allowed_categories.len(), 2);
        assert!(loaded.allowed_categories.contains(&ActionCategory::ReadFiles));
        assert!(loaded.allowed_categories.contains(&ActionCategory::WriteFiles));
        assert_eq!(loaded.blocked_categories, vec![ActionCategory::InstallPackages]);
        assert_eq!(loaded.max_autonomy_level, AutonomyLevel::Normal);
        assert_eq!(loaded.session_mode, SessionMode::Restricted);
        assert_eq!(loaded.require_approval_for, vec![ActionCategory::GitOperations]);
    }

    #[test]
    fn test_upsert_machine_policy_overwrites() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");

        let policy1 = MachinePolicy {
            id: "p-1".into(),
            machine_id: "m-1".into(),
            allowed_categories: vec![ActionCategory::ReadFiles],
            blocked_categories: vec![],
            max_autonomy_level: AutonomyLevel::Normal,
            session_mode: SessionMode::Normal,
            require_approval_for: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        upsert_machine_policy(&state, "m-1", &policy1).unwrap();

        let policy2 = MachinePolicy {
            id: "p-2".into(),
            machine_id: "m-1".into(),
            allowed_categories: ActionCategory::all(),
            blocked_categories: vec![ActionCategory::ShellExecution],
            max_autonomy_level: AutonomyLevel::Autonomous,
            session_mode: SessionMode::Elevated,
            require_approval_for: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        upsert_machine_policy(&state, "m-1", &policy2).unwrap();

        let loaded = load_machine_policy(&state, "m-1").unwrap().unwrap();
        assert_eq!(loaded.allowed_categories.len(), ActionCategory::all().len());
        assert_eq!(loaded.blocked_categories, vec![ActionCategory::ShellExecution]);
        assert_eq!(loaded.max_autonomy_level, AutonomyLevel::Autonomous);
        assert_eq!(loaded.session_mode, SessionMode::Elevated);
    }

    #[test]
    fn test_load_repo_policy_returns_none_when_absent() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");

        let policy = load_repo_policy(&state, "m-1", "/project").unwrap();
        assert!(policy.is_none());
    }

    #[test]
    fn test_upsert_and_load_repo_policy() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");

        let policy = RepoPolicy {
            id: "rp-1".into(),
            machine_id: "m-1".into(),
            repo_path: "/home/user/project".into(),
            allowed_categories: vec![ActionCategory::ReadFiles],
            blocked_categories: vec![ActionCategory::NetworkAccess],
            max_autonomy_level: Some(AutonomyLevel::Normal),
            require_approval_for: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        upsert_repo_policy(&state, "m-1", "/home/user/project", &policy).unwrap();

        let loaded = load_repo_policy(&state, "m-1", "/home/user/project")
            .unwrap()
            .unwrap();
        assert_eq!(loaded.machine_id, "m-1");
        assert_eq!(loaded.repo_path, "/home/user/project");
        assert_eq!(loaded.allowed_categories, vec![ActionCategory::ReadFiles]);
        assert_eq!(loaded.blocked_categories, vec![ActionCategory::NetworkAccess]);
        assert_eq!(loaded.max_autonomy_level, Some(AutonomyLevel::Normal));
    }

    #[test]
    fn test_repo_policy_different_paths_independent() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");

        let policy_a = RepoPolicy {
            id: "rp-a".into(),
            machine_id: "m-1".into(),
            repo_path: "/project-a".into(),
            allowed_categories: vec![ActionCategory::ReadFiles],
            blocked_categories: vec![],
            max_autonomy_level: None,
            require_approval_for: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        upsert_repo_policy(&state, "m-1", "/project-a", &policy_a).unwrap();

        let policy_b = RepoPolicy {
            id: "rp-b".into(),
            machine_id: "m-1".into(),
            repo_path: "/project-b".into(),
            allowed_categories: vec![ActionCategory::WriteFiles],
            blocked_categories: vec![],
            max_autonomy_level: None,
            require_approval_for: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        upsert_repo_policy(&state, "m-1", "/project-b", &policy_b).unwrap();

        let loaded_a = load_repo_policy(&state, "m-1", "/project-a").unwrap().unwrap();
        assert_eq!(loaded_a.allowed_categories, vec![ActionCategory::ReadFiles]);

        let loaded_b = load_repo_policy(&state, "m-1", "/project-b").unwrap().unwrap();
        assert_eq!(loaded_b.allowed_categories, vec![ActionCategory::WriteFiles]);
    }

    #[test]
    fn test_default_policy_created_on_approval() {
        let state = test_state();
        let now = Utc::now().to_rfc3339();
        let req = RegisterRequest {
            name: "test".into(),
            platform: "linux".into(),
            capabilities: None,
            repos: None,
        };
        insert_machine(&state, "m-1", "user-default", &req, &now).unwrap();

        // No policy before approval
        assert!(load_machine_policy(&state, "m-1").unwrap().is_none());

        // Approve and insert default policy (mirrors approve_machine handler)
        set_machine_status(&state, "m-1", "trusted", "basic", &now).unwrap();
        insert_default_machine_policy(&state, "m-1").unwrap();

        // Policy should now exist with default values
        let policy = load_machine_policy(&state, "m-1").unwrap().unwrap();
        assert_eq!(policy.machine_id, "m-1");
        assert_eq!(policy.allowed_categories, ActionCategory::all());
        assert!(policy.blocked_categories.is_empty());
        assert_eq!(policy.max_autonomy_level, AutonomyLevel::Autonomous);
        assert_eq!(policy.session_mode, SessionMode::Normal);
        assert!(policy.require_approval_for.is_empty());
    }

    #[test]
    fn test_policy_enforcement_blocks_autonomous_session() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");

        // Set machine policy to only allow normal autonomy
        let policy = MachinePolicy {
            id: "p-1".into(),
            machine_id: "m-1".into(),
            allowed_categories: ActionCategory::all(),
            blocked_categories: vec![],
            max_autonomy_level: AutonomyLevel::Normal,
            session_mode: SessionMode::Normal,
            require_approval_for: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        upsert_machine_policy(&state, "m-1", &policy).unwrap();

        // Verify that autonomous is blocked
        let loaded = load_machine_policy(&state, "m-1").unwrap().unwrap();
        let result = crate::models::is_autonomy_allowed(
            &AutonomyLevel::Autonomous,
            &loaded,
            None,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().reason.contains("exceeds machine max"));
    }

    #[test]
    fn test_policy_enforcement_allows_normal_session() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");

        let policy = MachinePolicy {
            id: "p-1".into(),
            machine_id: "m-1".into(),
            allowed_categories: ActionCategory::all(),
            blocked_categories: vec![],
            max_autonomy_level: AutonomyLevel::Normal,
            session_mode: SessionMode::Normal,
            require_approval_for: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        upsert_machine_policy(&state, "m-1", &policy).unwrap();

        let loaded = load_machine_policy(&state, "m-1").unwrap().unwrap();
        let result = crate::models::is_autonomy_allowed(
            &AutonomyLevel::Normal,
            &loaded,
            None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_policy_enforcement_repo_narrows_autonomy() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");

        // Machine allows autonomous
        let mp = MachinePolicy {
            id: "p-1".into(),
            machine_id: "m-1".into(),
            allowed_categories: ActionCategory::all(),
            blocked_categories: vec![],
            max_autonomy_level: AutonomyLevel::Autonomous,
            session_mode: SessionMode::Normal,
            require_approval_for: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        upsert_machine_policy(&state, "m-1", &mp).unwrap();

        // Repo restricts to normal
        let rp = RepoPolicy {
            id: "rp-1".into(),
            machine_id: "m-1".into(),
            repo_path: "/project".into(),
            allowed_categories: vec![],
            blocked_categories: vec![],
            max_autonomy_level: Some(AutonomyLevel::Normal),
            require_approval_for: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        upsert_repo_policy(&state, "m-1", "/project", &rp).unwrap();

        let loaded_mp = load_machine_policy(&state, "m-1").unwrap().unwrap();
        let loaded_rp = load_repo_policy(&state, "m-1", "/project").unwrap();
        let result = crate::models::is_autonomy_allowed(
            &AutonomyLevel::Autonomous,
            &loaded_mp,
            loaded_rp.as_ref(),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().reason.contains("repo max"));
    }

    #[test]
    fn test_effective_policy_merge_no_repo() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");

        let mp = MachinePolicy {
            id: "p-1".into(),
            machine_id: "m-1".into(),
            allowed_categories: ActionCategory::all(),
            blocked_categories: vec![ActionCategory::InstallPackages],
            max_autonomy_level: AutonomyLevel::Autonomous,
            session_mode: SessionMode::Normal,
            require_approval_for: vec![ActionCategory::GitOperations],
            created_at: String::new(),
            updated_at: String::new(),
        };

        let effective = merge_policies(&mp, None);
        // InstallPackages should be removed from allowed since it's blocked
        assert!(!effective.allowed_categories.contains(&ActionCategory::InstallPackages));
        assert!(effective.blocked_categories.contains(&ActionCategory::InstallPackages));
        assert_eq!(effective.max_autonomy_level, AutonomyLevel::Autonomous);
        assert_eq!(effective.require_approval_for, vec![ActionCategory::GitOperations]);
    }

    #[test]
    fn test_effective_policy_merge_with_repo() {
        let mp = MachinePolicy {
            id: "p-1".into(),
            machine_id: "m-1".into(),
            allowed_categories: ActionCategory::all(),
            blocked_categories: vec![],
            max_autonomy_level: AutonomyLevel::Autonomous,
            session_mode: SessionMode::Normal,
            require_approval_for: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };

        let rp = RepoPolicy {
            id: "rp-1".into(),
            machine_id: "m-1".into(),
            repo_path: "/project".into(),
            allowed_categories: vec![ActionCategory::ReadFiles, ActionCategory::WriteFiles],
            blocked_categories: vec![ActionCategory::WriteFiles],
            max_autonomy_level: Some(AutonomyLevel::Normal),
            require_approval_for: vec![ActionCategory::ReadFiles],
            created_at: String::new(),
            updated_at: String::new(),
        };

        let effective = merge_policies(&mp, Some(&rp));
        // Allowed narrowed to ReadFiles, WriteFiles — then WriteFiles removed because blocked
        assert!(effective.allowed_categories.contains(&ActionCategory::ReadFiles));
        assert!(!effective.allowed_categories.contains(&ActionCategory::WriteFiles));
        assert!(!effective.allowed_categories.contains(&ActionCategory::GitOperations));
        assert!(effective.blocked_categories.contains(&ActionCategory::WriteFiles));
        assert_eq!(effective.max_autonomy_level, AutonomyLevel::Normal);
        assert_eq!(effective.require_approval_for, vec![ActionCategory::ReadFiles]);
    }

    // -- Policy violation tests --

    #[test]
    fn test_insert_and_query_policy_violation() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");

        insert_policy_violation(
            &state,
            Some("s-1"),
            "m-1",
            "user-default",
            "autonomy:autonomous",
            "machine",
            "requested autonomy 'autonomous' exceeds machine max 'normal'",
            Some("/project"),
        )
        .unwrap();

        let query = ListViolationsQuery {
            machine_id: None,
            session_id: None,
            limit: None,
            offset: None,
        };
        let result = query_violations(&state, "user-default", &query).unwrap();
        assert_eq!(result.total, 1);
        assert_eq!(result.violations.len(), 1);

        let v = &result.violations[0];
        assert_eq!(v.machine_id, "m-1");
        assert_eq!(v.user_id, "user-default");
        assert_eq!(v.action, "autonomy:autonomous");
        assert_eq!(v.policy_scope, "machine");
        assert_eq!(v.session_id, Some("s-1".into()));
        assert_eq!(v.repo_path, Some("/project".into()));
    }

    #[test]
    fn test_query_violations_filter_by_machine_id() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_trusted_machine(&state, "m-2");

        insert_policy_violation(
            &state, None, "m-1", "user-default", "autonomy:autonomous", "machine",
            "exceeds max", Some("/project"),
        ).unwrap();
        insert_policy_violation(
            &state, None, "m-2", "user-default", "autonomy:autonomous", "machine",
            "exceeds max", Some("/other"),
        ).unwrap();

        let query = ListViolationsQuery {
            machine_id: Some("m-1".into()),
            session_id: None,
            limit: None,
            offset: None,
        };
        let result = query_violations(&state, "user-default", &query).unwrap();
        assert_eq!(result.total, 1);
        assert_eq!(result.violations[0].machine_id, "m-1");
    }

    #[test]
    fn test_query_violations_filter_by_session_id() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");

        // Create a session so FK is satisfied
        let now = Utc::now().to_rfc3339();
        let body = CreateSessionRequest {
            machine_id: "m-1".into(),
            repo_path: "/project".into(),
            title: "Test".into(),
            instructions: "Work".into(),
            autonomy_level: None,
            work_item_id: None,
            context: None,
        };
        insert_session(&state, "s-1", "user-default", &body, "normal", &now).unwrap();

        insert_policy_violation(
            &state, Some("s-1"), "m-1", "user-default", "autonomy:autonomous", "machine",
            "exceeds max", Some("/project"),
        ).unwrap();
        insert_policy_violation(
            &state, None, "m-1", "user-default", "autonomy:autonomous", "machine",
            "exceeds max", Some("/other"),
        ).unwrap();

        let query = ListViolationsQuery {
            machine_id: None,
            session_id: Some("s-1".into()),
            limit: None,
            offset: None,
        };
        let result = query_violations(&state, "user-default", &query).unwrap();
        assert_eq!(result.total, 1);
        assert_eq!(result.violations[0].session_id, Some("s-1".into()));
    }

    #[test]
    fn test_query_violations_by_session() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");

        let now = Utc::now().to_rfc3339();
        let body = CreateSessionRequest {
            machine_id: "m-1".into(),
            repo_path: "/project".into(),
            title: "Test".into(),
            instructions: "Work".into(),
            autonomy_level: None,
            work_item_id: None,
            context: None,
        };
        insert_session(&state, "s-1", "user-default", &body, "normal", &now).unwrap();

        insert_policy_violation(
            &state, Some("s-1"), "m-1", "user-default", "autonomy:autonomous", "machine",
            "exceeds max", Some("/project"),
        ).unwrap();
        insert_policy_violation(
            &state, None, "m-1", "user-default", "autonomy:autonomous", "machine",
            "other violation", Some("/other"),
        ).unwrap();

        let violations = query_violations_by_session(&state, "s-1", "user-default").unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].session_id, Some("s-1".into()));
    }

    #[test]
    fn test_session_creation_violation_logged() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");

        // Set machine policy to only allow normal autonomy
        let policy = MachinePolicy {
            id: "p-1".into(),
            machine_id: "m-1".into(),
            allowed_categories: ActionCategory::all(),
            blocked_categories: vec![],
            max_autonomy_level: AutonomyLevel::Normal,
            session_mode: SessionMode::Normal,
            require_approval_for: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        upsert_machine_policy(&state, "m-1", &policy).unwrap();

        // Simulate what create_session does when autonomy is blocked:
        // Load the machine policy and check autonomy
        let loaded_policy = load_machine_policy(&state, "m-1").unwrap().unwrap();
        let repo_policy = load_repo_policy(&state, "m-1", "/project").unwrap();
        let result = crate::models::is_autonomy_allowed(
            &AutonomyLevel::Autonomous,
            &loaded_policy,
            repo_policy.as_ref(),
        );
        assert!(result.is_err());

        let violation = result.unwrap_err();
        // Log the violation (mirrors what the handler now does)
        insert_policy_violation(
            &state,
            None,
            "m-1",
            "user-default",
            &violation.blocked_action,
            &violation.policy_scope,
            &violation.reason,
            Some("/project"),
        )
        .unwrap();

        // Verify violation record exists
        let query = ListViolationsQuery {
            machine_id: None,
            session_id: None,
            limit: None,
            offset: None,
        };
        let result = query_violations(&state, "user-default", &query).unwrap();
        assert_eq!(result.total, 1);
        let v = &result.violations[0];
        assert_eq!(v.machine_id, "m-1");
        assert_eq!(v.action, "autonomy:autonomous");
        assert_eq!(v.policy_scope, "machine");
        assert!(v.reason.contains("exceeds machine max"));
        assert_eq!(v.session_id, None);
        assert_eq!(v.repo_path, Some("/project".into()));
    }
}

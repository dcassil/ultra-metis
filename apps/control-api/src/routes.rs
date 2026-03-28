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
    CommandResponse, CreateSessionRequest, CreateSessionResponse, ErrorBody, HeartbeatRequest,
    ListSessionsQuery, Machine, MachineDetailResponse, MachineRepo, MachineResponse, MachineStatus,
    RegisterRequest, RegisterResponse, RepoInfo, ReportStateRequest, Session, SessionListResponse,
    SessionResponse, SessionState,
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
}

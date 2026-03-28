//! HTTP route handlers for the Machine Registry API.

use std::time::Duration;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use rusqlite::params;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::auth::{DashboardAuth, MachineTokenAuth};
use axum::extract::Query;

use crate::models::{
    ActionCategory, ApprovalStatus, AutonomyLevel, CommandResponse, CreateSessionRequest,
    CreateSessionResponse, EffectivePolicy, EffectivePolicyQuery, ErrorBody, HeartbeatRequest,
    IngestEventsRequest, InjectGuidanceRequest, ListNotificationsQuery, ListSessionsQuery,
    ListViolationsQuery, Machine, MachineDetailResponse, MachinePolicy, MachineRepo,
    MachineResponse, MachineStatus, Notification, PendingApproval, PolicyViolationRecord,
    QueryEventsParams, RegisterDeviceRequest, RegisterRequest, RegisterResponse, RepoInfo,
    RepoPolicy, RepoPolicyQuery, ReportStateRequest, RespondToApprovalRequest, Session,
    SessionListResponse, SessionMode, SessionOutcome, SessionOutputEvent, SessionOutputEventType,
    SessionResponse, SessionState, UnreadCountResponse, UpdateMachinePolicyRequest,
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

#[allow(clippy::significant_drop_tightening, clippy::too_many_lines)]
fn query_sessions(
    state: &AppState,
    user_id: &str,
    query: &ListSessionsQuery,
) -> Result<SessionListResponse, rusqlite::Error> {
    let db = state.db.lock().expect("db lock poisoned");

    // Determine if we need a JOIN on session_outcomes
    let needs_outcome_join = query.outcome.is_some();

    // Build WHERE clause dynamically with a params vec
    let mut conditions: Vec<String> = vec!["s.user_id = ?1".to_string()];
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> =
        vec![Box::new(user_id.to_string())];
    let mut param_idx: usize = 2;

    if let Some(ref mid) = query.machine_id {
        conditions.push(format!("s.machine_id = ?{param_idx}"));
        param_values.push(Box::new(mid.clone()));
        param_idx += 1;
    }
    if let Some(ref rp) = query.repo_path {
        conditions.push(format!("s.repo_path = ?{param_idx}"));
        param_values.push(Box::new(rp.clone()));
        param_idx += 1;
    }
    if let Some(ref st) = query.state {
        conditions.push(format!("s.state = ?{param_idx}"));
        param_values.push(Box::new(st.clone()));
        param_idx += 1;
    }
    if let Some(ref outcome) = query.outcome {
        conditions.push(format!("so.status = ?{param_idx}"));
        param_values.push(Box::new(outcome.clone()));
        param_idx += 1;
    }
    if let Some(ref search) = query.search {
        let pattern = format!("%{search}%");
        conditions.push(format!(
            "(s.title LIKE ?{pi1} OR s.instructions LIKE ?{pi2})",
            pi1 = param_idx,
            pi2 = param_idx + 1
        ));
        param_values.push(Box::new(pattern.clone()));
        param_values.push(Box::new(pattern));
        param_idx += 2;
    }
    if let Some(ref from_date) = query.from_date {
        conditions.push(format!("s.created_at >= ?{param_idx}"));
        param_values.push(Box::new(from_date.clone()));
        param_idx += 1;
    }
    if let Some(ref to_date) = query.to_date {
        conditions.push(format!("s.created_at <= ?{param_idx}"));
        param_values.push(Box::new(to_date.clone()));
        param_idx += 1;
    }

    let where_clause = conditions.join(" AND ");

    // Validate and build ORDER BY
    let sort_col = match query.sort_by.as_deref() {
        Some("updated_at") => "s.updated_at",
        Some("title") => "s.title",
        Some("created_at") | None => "s.created_at",
        Some(_) => "s.created_at", // ignore invalid values, fall back to default
    };
    let sort_dir = match query.sort_order.as_deref() {
        Some("asc") | Some("ASC") => "ASC",
        Some("desc") | Some("DESC") | None => "DESC",
        Some(_) => "DESC",
    };

    let join_clause = if needs_outcome_join {
        "INNER JOIN session_outcomes so ON so.session_id = s.id"
    } else {
        "LEFT JOIN session_outcomes so ON so.session_id = s.id"
    };

    // Count query
    let count_sql = format!(
        "SELECT COUNT(*) FROM sessions s {join_clause} WHERE {where_clause}"
    );

    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    // Build list query
    let list_sql = format!(
        "SELECT s.id, s.user_id, s.machine_id, s.repo_path, s.title, s.instructions,
                s.autonomy_level, s.work_item_id, s.context, s.state,
                s.created_at, s.updated_at, s.started_at, s.completed_at,
                so.status AS outcome_status
         FROM sessions s
         {join_clause}
         WHERE {where_clause}
         ORDER BY {sort_col} {sort_dir}
         LIMIT ?{param_idx} OFFSET ?{}",
        param_idx + 1
    );

    // Build params refs for the count query
    let count_params: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|b| b.as_ref()).collect();
    let total: i64 = db.query_row(&count_sql, count_params.as_slice(), |row| row.get(0))?;

    // Build params for the list query (add limit and offset)
    param_values.push(Box::new(limit));
    param_values.push(Box::new(offset));
    let list_params: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|b| b.as_ref()).collect();

    let sessions = query_sessions_rows(&db, &list_sql, list_params.as_slice())?;

    Ok(SessionListResponse {
        sessions: sessions.into_iter().map(Into::into).collect(),
        total,
    })
}

/// A row from the sessions list query including the optional outcome_status.
struct SessionWithOutcome {
    session: Session,
    outcome_status: Option<String>,
}

impl From<SessionWithOutcome> for SessionResponse {
    fn from(s: SessionWithOutcome) -> Self {
        SessionResponse {
            id: s.session.id,
            machine_id: s.session.machine_id,
            repo_path: s.session.repo_path,
            title: s.session.title,
            instructions: s.session.instructions,
            autonomy_level: s.session.autonomy_level,
            work_item_id: s.session.work_item_id,
            context: s.session.context,
            state: s.session.state,
            created_at: s.session.created_at,
            updated_at: s.session.updated_at,
            started_at: s.session.started_at,
            completed_at: s.session.completed_at,
            outcome_status: s.outcome_status,
        }
    }
}

#[allow(clippy::significant_drop_tightening)]
fn query_sessions_rows(
    db: &rusqlite::Connection,
    sql: &str,
    params: &[&dyn rusqlite::types::ToSql],
) -> Result<Vec<SessionWithOutcome>, rusqlite::Error> {
    let mut stmt = db.prepare(sql)?;
    let rows = stmt.query_map(params, |row| {
        Ok(SessionWithOutcome {
            session: Session {
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
            },
            outcome_status: row.get(14)?,
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
        write_session_outcome(&state, &session_id, &SessionState::Stopped);
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

    // Write outcome record when session reaches a terminal state
    if body.state.is_terminal() {
        write_session_outcome(&state, &session_id, &body.state);

        // Generate notification for terminal session states
        let (notification_type, priority, title) = match body.state {
            SessionState::Completed => (
                "session_completed",
                "normal",
                format!("Session completed: {}", session.title),
            ),
            SessionState::Failed => (
                "session_failed",
                "high",
                format!("Session failed: {}", session.title),
            ),
            SessionState::Stopped => (
                "session_stopped",
                "normal",
                format!("Session stopped: {}", session.title),
            ),
            _ => unreachable!("is_terminal() already checked"),
        };

        create_notification(
            &state,
            &session.user_id,
            &session_id,
            notification_type,
            priority,
            &title,
            "",
            &format!("/sessions/{session_id}"),
        );
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
        outcome_status: None,
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

// ---------------------------------------------------------------------------
// POST /api/sessions/{id}/events  (runner ingests output events)
// ---------------------------------------------------------------------------

pub async fn ingest_events(
    State(state): State<AppState>,
    MachineTokenAuth(auth): MachineTokenAuth,
    Path(session_id): Path<String>,
    Json(body): Json<IngestEventsRequest>,
) -> impl IntoResponse {
    // Validate session exists and belongs to the authenticated user
    let session = match load_session(&state, &session_id, &auth.user_id) {
        Ok(Some(s)) => s,
        Ok(None) => return not_found("session not found"),
        Err(e) => return internal_error(&format!("db error: {e}")),
    };

    let now = Utc::now().to_rfc3339();

    // Get current max sequence_num for this session
    let base_seq = match get_max_sequence_num(&state, &session_id) {
        Ok(seq) => seq,
        Err(e) => return internal_error(&format!("failed to get sequence_num: {e}")),
    };

    let mut inserted = 0i64;
    for (i, item) in body.events.iter().enumerate() {
        let event_id = uuid::Uuid::new_v4().to_string();
        let seq = base_seq + (i as i64) + 1;
        let metadata_str = item.metadata.as_ref().map(std::string::ToString::to_string);
        let category_str = item.category.as_ref().map(|c| c.as_str().to_string());

        if let Err(e) = insert_output_event(
            &state,
            &event_id,
            &session_id,
            item.event_type.as_str(),
            category_str.as_deref(),
            &item.content,
            metadata_str.as_deref(),
            seq,
            &now,
        ) {
            return internal_error(&format!("failed to insert event: {e}"));
        }

        // If this is an approval_request, also insert into pending_approvals
        if item.event_type == SessionOutputEventType::ApprovalRequest {
            let question = item.content.clone();
            let options = item
                .metadata
                .as_ref()
                .and_then(|m| m.get("options"))
                .map(std::string::ToString::to_string)
                .unwrap_or_else(|| "[]".to_string());
            let context = item
                .metadata
                .as_ref()
                .and_then(|m| m.get("context"))
                .and_then(|c| c.as_str())
                .map(String::from);

            if let Err(e) =
                insert_pending_approval(&state, &session_id, &question, &options, context.as_deref(), &now)
            {
                tracing::error!("failed to insert pending approval: {e}");
            }

            // Generate notification for approval request
            let truncated_body: String = item.content.chars().take(200).collect();
            create_notification(
                &state,
                &session.user_id,
                &session_id,
                "approval_request",
                "urgent",
                "Session needs your input",
                &truncated_body,
                &format!("/sessions/{session_id}"),
            );
        }

        // Broadcast to SSE subscribers
        let output_event = SessionOutputEvent {
            id: event_id,
            session_id: session_id.clone(),
            event_type: item.event_type.clone(),
            category: item.category.clone(),
            content: item.content.clone(),
            metadata: metadata_str.clone(),
            sequence_num: seq,
            timestamp: now.clone(),
        };
        if let Ok(json) = serde_json::to_string(&output_event) {
            broadcast_event(&state, &session_id, &json);
        }

        inserted += 1;
    }

    Json(serde_json::json!({"ingested": inserted})).into_response()
}

fn get_max_sequence_num(state: &AppState, session_id: &str) -> Result<i64, rusqlite::Error> {
    let seq: Option<i64> = state.db.lock().expect("db lock poisoned").query_row(
        "SELECT MAX(sequence_num) FROM session_output_events WHERE session_id = ?1",
        [session_id],
        |row| row.get(0),
    )?;
    Ok(seq.unwrap_or(0))
}

#[allow(clippy::too_many_arguments)]
fn insert_output_event(
    state: &AppState,
    event_id: &str,
    session_id: &str,
    event_type: &str,
    category: Option<&str>,
    content: &str,
    metadata: Option<&str>,
    sequence_num: i64,
    timestamp: &str,
) -> Result<(), rusqlite::Error> {
    state
        .db
        .lock()
        .expect("db lock poisoned")
        .execute(
            "INSERT INTO session_output_events (id, session_id, event_type, category, content, metadata, sequence_num, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![event_id, session_id, event_type, category, content, metadata, sequence_num, timestamp],
        )?;
    Ok(())
}

fn insert_pending_approval(
    state: &AppState,
    session_id: &str,
    question: &str,
    options: &str,
    context: Option<&str>,
    now: &str,
) -> Result<(), rusqlite::Error> {
    let approval_id = uuid::Uuid::new_v4().to_string();
    state
        .db
        .lock()
        .expect("db lock poisoned")
        .execute(
            "INSERT INTO pending_approvals (id, session_id, question, options, context, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6)",
            params![approval_id, session_id, question, options, context, now],
        )?;
    Ok(())
}

fn broadcast_event(state: &AppState, session_id: &str, json: &str) {
    let channels = state.event_channels.lock().expect("event_channels lock poisoned");
    if let Some(tx) = channels.get(session_id) {
        // Ignore send errors (no active receivers is fine)
        let _ = tx.send(json.to_string());
    }
}

// ---------------------------------------------------------------------------
// GET /api/sessions/{id}/events  (dashboard queries event history)
// ---------------------------------------------------------------------------

pub async fn query_events(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Path(session_id): Path<String>,
    Query(query): Query<QueryEventsParams>,
) -> impl IntoResponse {
    // Validate session ownership
    match load_session(&state, &session_id, &auth.user_id) {
        Ok(Some(_)) => {}
        Ok(None) => return not_found("session not found"),
        Err(e) => return internal_error(&format!("db error: {e}")),
    }

    match query_output_events(&state, &session_id, &query) {
        Ok(events) => Json(serde_json::to_value(events).unwrap_or_default()).into_response(),
        Err(e) => internal_error(&format!("db error: {e}")),
    }
}

#[allow(clippy::significant_drop_tightening)]
fn query_output_events(
    state: &AppState,
    session_id: &str,
    query: &QueryEventsParams,
) -> Result<Vec<SessionOutputEvent>, rusqlite::Error> {
    let db = state.db.lock().expect("db lock poisoned");
    let since = query.since_sequence.unwrap_or(0);
    let limit = query.limit.unwrap_or(100);

    let (sql, events) = if let Some(ref event_type) = query.event_type {
        let sql = "SELECT id, session_id, event_type, category, content, metadata, sequence_num, timestamp
             FROM session_output_events
             WHERE session_id = ?1 AND sequence_num > ?2 AND event_type = ?3
             ORDER BY sequence_num ASC
             LIMIT ?4";
        let mut stmt = db.prepare(sql)?;
        let rows = stmt.query_map(params![session_id, since, event_type, limit], parse_output_event_row)?;
        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }
        (sql, events)
    } else {
        let sql = "SELECT id, session_id, event_type, category, content, metadata, sequence_num, timestamp
             FROM session_output_events
             WHERE session_id = ?1 AND sequence_num > ?2
             ORDER BY sequence_num ASC
             LIMIT ?3";
        let mut stmt = db.prepare(sql)?;
        let rows = stmt.query_map(params![session_id, since, limit], parse_output_event_row)?;
        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }
        (sql, events)
    };

    // Suppress unused variable warning for sql
    let _ = sql;
    Ok(events)
}

fn parse_output_event_row(row: &rusqlite::Row<'_>) -> Result<SessionOutputEvent, rusqlite::Error> {
    Ok(SessionOutputEvent {
        id: row.get(0)?,
        session_id: row.get(1)?,
        event_type: row
            .get::<_, String>(2)?
            .parse()
            .unwrap_or(SessionOutputEventType::OutputLine),
        category: row
            .get::<_, Option<String>>(3)?
            .and_then(|s| s.parse().ok()),
        content: row.get(4)?,
        metadata: row.get(5)?,
        sequence_num: row.get(6)?,
        timestamp: row.get(7)?,
    })
}

// ---------------------------------------------------------------------------
// GET /api/sessions/{id}/events/stream  (SSE live event stream)
// ---------------------------------------------------------------------------

pub async fn event_stream(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    // Validate session ownership
    match load_session(&state, &session_id, &auth.user_id) {
        Ok(Some(_)) => {}
        Ok(None) => return Err(not_found("session not found")),
        Err(e) => return Err(internal_error(&format!("db error: {e}"))),
    }

    // Get or create the broadcast channel for this session
    let rx = {
        let mut channels = state.event_channels.lock().expect("event_channels lock poisoned");
        let tx = channels
            .entry(session_id)
            .or_insert_with(|| {
                let (tx, _rx) = tokio::sync::broadcast::channel(256);
                tx
            });
        let rx = tx.subscribe();
        drop(channels);
        rx
    };

    let event_stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(json) => Some(Ok::<_, std::convert::Infallible>(
            Event::default().event("session_event").data(json),
        )),
        Err(_) => None,
    });

    Ok(Sse::new(event_stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
}

// ---------------------------------------------------------------------------
// GET /api/sessions/{id}/approvals  (dashboard queries pending approvals)
// ---------------------------------------------------------------------------

pub async fn list_pending_approvals(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    // Validate session ownership
    match load_session(&state, &session_id, &auth.user_id) {
        Ok(Some(_)) => {}
        Ok(None) => return not_found("session not found"),
        Err(e) => return internal_error(&format!("db error: {e}")),
    }

    match query_pending_approvals(&state, &session_id) {
        Ok(approvals) => Json(serde_json::to_value(approvals).unwrap_or_default()).into_response(),
        Err(e) => internal_error(&format!("db error: {e}")),
    }
}

#[allow(clippy::significant_drop_tightening)]
fn query_pending_approvals(
    state: &AppState,
    session_id: &str,
) -> Result<Vec<PendingApproval>, rusqlite::Error> {
    let db = state.db.lock().expect("db lock poisoned");
    let mut stmt = db.prepare(
        "SELECT id, session_id, question, options, context, status, response_choice, response_note, created_at, responded_at
         FROM pending_approvals
         WHERE session_id = ?1 AND status = 'pending'
         ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map([session_id], |row| {
        Ok(PendingApproval {
            id: row.get(0)?,
            session_id: row.get(1)?,
            question: row.get(2)?,
            options: row.get(3)?,
            context: row.get(4)?,
            status: row
                .get::<_, String>(5)?
                .parse()
                .unwrap_or(ApprovalStatus::Pending),
            response_choice: row.get(6)?,
            response_note: row.get(7)?,
            created_at: row.get(8)?,
            responded_at: row.get(9)?,
        })
    })?;

    let mut approvals = Vec::new();
    for row in rows {
        approvals.push(row?);
    }
    Ok(approvals)
}

// ---------------------------------------------------------------------------
// POST /api/sessions/{id}/respond  (dashboard responds to approval)
// ---------------------------------------------------------------------------

pub async fn respond_to_approval(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Path(session_id): Path<String>,
    Json(body): Json<RespondToApprovalRequest>,
) -> impl IntoResponse {
    let session = match load_session(&state, &session_id, &auth.user_id) {
        Ok(Some(s)) => s,
        Ok(None) => return not_found("session not found"),
        Err(e) => return internal_error(&format!("db error: {e}")),
    };

    // Load the pending approval
    let approval = match load_pending_approval(&state, &body.approval_id, &session_id) {
        Ok(Some(a)) => a,
        Ok(None) => return bad_request("approval not found or already responded"),
        Err(e) => return internal_error(&format!("db error: {e}")),
    };

    if approval.status != ApprovalStatus::Pending {
        return bad_request("approval not found or already responded");
    }

    let now = Utc::now().to_rfc3339();

    // Update pending_approvals: set status = 'responded', response fields
    if let Err(e) = update_approval_response(&state, &body.approval_id, &body.choice, body.note.as_deref(), &now) {
        return internal_error(&format!("failed to update approval: {e}"));
    }

    // Queue a respond command
    let payload = serde_json::json!({
        "approval_id": body.approval_id,
        "choice": body.choice,
        "note": body.note,
    });
    if let Err(e) = insert_session_command(
        &state,
        &session_id,
        &session.machine_id,
        "respond",
        Some(&payload.to_string()),
    ) {
        return internal_error(&format!("failed to queue respond command: {e}"));
    }

    // Insert an approval_response event
    let base_seq = match get_max_sequence_num(&state, &session_id) {
        Ok(seq) => seq,
        Err(e) => return internal_error(&format!("failed to get sequence_num: {e}")),
    };
    let seq = base_seq + 1;
    let event_id = uuid::Uuid::new_v4().to_string();
    let content = format!("Approval response: {}", body.choice);
    let metadata = serde_json::json!({
        "approval_id": body.approval_id,
        "choice": body.choice,
        "note": body.note,
    })
    .to_string();

    if let Err(e) = insert_output_event(
        &state,
        &event_id,
        &session_id,
        SessionOutputEventType::ApprovalResponse.as_str(),
        None,
        &content,
        Some(&metadata),
        seq,
        &now,
    ) {
        return internal_error(&format!("failed to insert event: {e}"));
    }

    // If session state is waiting_for_input, transition back to running
    if session.state == SessionState::WaitingForInput {
        if let Err(e) = update_session_state(&state, &session_id, "running", &now, None) {
            return internal_error(&format!("failed to update session state: {e}"));
        }
        if let Err(e) = insert_session_event(
            &state,
            &session_id,
            Some("waiting_for_input"),
            "running",
            &now,
            None,
        ) {
            tracing::error!("failed to insert state change event: {e}");
        }
    }

    // Broadcast the event to SSE subscribers
    let output_event = SessionOutputEvent {
        id: event_id,
        session_id: session_id.clone(),
        event_type: SessionOutputEventType::ApprovalResponse,
        category: None,
        content,
        metadata: Some(metadata),
        sequence_num: seq,
        timestamp: now,
    };
    if let Ok(json) = serde_json::to_string(&output_event) {
        broadcast_event(&state, &session_id, &json);
    }

    Json(serde_json::json!({"status": "responded", "session_id": session_id})).into_response()
}

// ---------------------------------------------------------------------------
// POST /api/sessions/{id}/inject  (dashboard injects guidance)
// ---------------------------------------------------------------------------

pub async fn inject_guidance(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Path(session_id): Path<String>,
    Json(body): Json<InjectGuidanceRequest>,
) -> impl IntoResponse {
    let session = match load_session(&state, &session_id, &auth.user_id) {
        Ok(Some(s)) => s,
        Ok(None) => return not_found("session not found"),
        Err(e) => return internal_error(&format!("db error: {e}")),
    };

    // Validate session is in running or waiting_for_input state
    if !matches!(
        session.state,
        SessionState::Running | SessionState::WaitingForInput
    ) {
        return conflict(&format!(
            "cannot inject guidance into session in {} state, must be running or waiting_for_input",
            session.state
        ));
    }

    let now = Utc::now().to_rfc3339();

    // Queue an inject command
    let payload = serde_json::json!({
        "message": body.message,
        "injection_type": body.injection_type.as_str(),
    });
    if let Err(e) = insert_session_command(
        &state,
        &session_id,
        &session.machine_id,
        "inject",
        Some(&payload.to_string()),
    ) {
        return internal_error(&format!("failed to queue inject command: {e}"));
    }

    // Insert a guidance_injected event
    let base_seq = match get_max_sequence_num(&state, &session_id) {
        Ok(seq) => seq,
        Err(e) => return internal_error(&format!("failed to get sequence_num: {e}")),
    };
    let seq = base_seq + 1;
    let event_id = uuid::Uuid::new_v4().to_string();
    let metadata = serde_json::json!({
        "injection_type": body.injection_type.as_str(),
    })
    .to_string();

    if let Err(e) = insert_output_event(
        &state,
        &event_id,
        &session_id,
        SessionOutputEventType::GuidanceInjected.as_str(),
        None,
        &body.message,
        Some(&metadata),
        seq,
        &now,
    ) {
        return internal_error(&format!("failed to insert event: {e}"));
    }

    // Broadcast the event to SSE subscribers
    let output_event = SessionOutputEvent {
        id: event_id,
        session_id: session_id.clone(),
        event_type: SessionOutputEventType::GuidanceInjected,
        category: None,
        content: body.message,
        metadata: Some(metadata),
        sequence_num: seq,
        timestamp: now,
    };
    if let Ok(json) = serde_json::to_string(&output_event) {
        broadcast_event(&state, &session_id, &json);
    }

    Json(serde_json::json!({"status": "injected", "session_id": session_id})).into_response()
}

// ---------------------------------------------------------------------------
// Approval / injection helpers
// ---------------------------------------------------------------------------

fn load_pending_approval(
    state: &AppState,
    approval_id: &str,
    session_id: &str,
) -> Result<Option<PendingApproval>, rusqlite::Error> {
    let result = state.db.lock().expect("db lock poisoned").query_row(
        "SELECT id, session_id, question, options, context, status, response_choice, response_note, created_at, responded_at
         FROM pending_approvals WHERE id = ?1 AND session_id = ?2",
        params![approval_id, session_id],
        |row| {
            Ok(PendingApproval {
                id: row.get(0)?,
                session_id: row.get(1)?,
                question: row.get(2)?,
                options: row.get(3)?,
                context: row.get(4)?,
                status: row
                    .get::<_, String>(5)?
                    .parse()
                    .unwrap_or(ApprovalStatus::Pending),
                response_choice: row.get(6)?,
                response_note: row.get(7)?,
                created_at: row.get(8)?,
                responded_at: row.get(9)?,
            })
        },
    );

    match result {
        Ok(approval) => Ok(Some(approval)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

fn update_approval_response(
    state: &AppState,
    approval_id: &str,
    choice: &str,
    note: Option<&str>,
    now: &str,
) -> Result<(), rusqlite::Error> {
    state
        .db
        .lock()
        .expect("db lock poisoned")
        .execute(
            "UPDATE pending_approvals SET status = 'responded', response_choice = ?1, response_note = ?2, responded_at = ?3
             WHERE id = ?4",
            params![choice, note, now, approval_id],
        )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Session outcome helpers
// ---------------------------------------------------------------------------

/// Map a terminal session state to an outcome status string.
fn outcome_status_for_state(state: &SessionState) -> &'static str {
    match state {
        SessionState::Completed => "success",
        SessionState::Failed => "failure",
        SessionState::Stopped => "partial",
        _ => "partial",
    }
}

/// Write a session outcome record after the session reaches a terminal state.
///
/// Collects event_count, intervention_count, a summary from the last output events,
/// and the duration from started_at to now.
fn write_session_outcome(
    state: &AppState,
    session_id: &str,
    terminal_state: &SessionState,
) {
    let db = state.db.lock().expect("db lock poisoned");

    // Check if outcome already exists (idempotent)
    let existing: Result<i64, _> = db.query_row(
        "SELECT COUNT(*) FROM session_outcomes WHERE session_id = ?1",
        params![session_id],
        |row| row.get(0),
    );
    if existing.unwrap_or(0) > 0 {
        return;
    }

    let outcome_id = uuid::Uuid::new_v4().to_string();
    let status = outcome_status_for_state(terminal_state);

    // Count total output events
    let event_count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM session_output_events WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Count intervention events (approval_response, guidance_injected)
    let intervention_count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM session_output_events
             WHERE session_id = ?1 AND event_type IN ('approval_response', 'guidance_injected')",
            params![session_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Build summary from the last few output events
    let summary: String = {
        let mut stmt = db
            .prepare(
                "SELECT content FROM session_output_events
                 WHERE session_id = ?1 AND content != ''
                 ORDER BY sequence_num DESC LIMIT 5",
            )
            .unwrap_or_else(|e| {
                tracing::error!("failed to prepare summary query: {e}");
                // Return a dummy statement that will produce no rows
                db.prepare("SELECT '' WHERE 0").unwrap()
            });
        let rows: Vec<String> = stmt
            .query_map(params![session_id], |row| row.get::<_, String>(0))
            .ok()
            .map(|iter| iter.filter_map(Result::ok).collect())
            .unwrap_or_default();
        // Reverse to chronological order and join
        let mut reversed = rows;
        reversed.reverse();
        reversed.join("\n")
    };

    // Calculate duration_seconds from session.started_at to now
    let duration_seconds: i64 = db
        .query_row(
            "SELECT started_at FROM sessions WHERE id = ?1",
            params![session_id],
            |row| row.get::<_, Option<String>>(0),
        )
        .ok()
        .flatten()
        .and_then(|started_at| {
            started_at.parse::<chrono::DateTime<Utc>>().ok().map(|start| {
                Utc::now().signed_duration_since(start).num_seconds().max(0)
            })
        })
        .unwrap_or(0);

    if let Err(e) = db.execute(
        "INSERT INTO session_outcomes
         (id, session_id, status, summary, artifacts, next_steps, event_count, intervention_count, duration_seconds)
         VALUES (?1, ?2, ?3, ?4, '[]', '', ?5, ?6, ?7)",
        params![
            outcome_id,
            session_id,
            status,
            summary,
            event_count,
            intervention_count,
            duration_seconds,
        ],
    ) {
        tracing::error!("failed to write session outcome: {e}");
    }
}

fn load_session_outcome(
    state: &AppState,
    session_id: &str,
) -> Result<Option<SessionOutcome>, rusqlite::Error> {
    let result = state.db.lock().expect("db lock poisoned").query_row(
        "SELECT id, session_id, status, summary, artifacts, next_steps,
                event_count, intervention_count, duration_seconds, created_at
         FROM session_outcomes WHERE session_id = ?1",
        params![session_id],
        |row| {
            Ok(SessionOutcome {
                id: row.get(0)?,
                session_id: row.get(1)?,
                status: row.get(2)?,
                summary: row.get(3)?,
                artifacts: row.get(4)?,
                next_steps: row.get(5)?,
                event_count: row.get(6)?,
                intervention_count: row.get(7)?,
                duration_seconds: row.get(8)?,
                created_at: row.get(9)?,
            })
        },
    );

    match result {
        Ok(outcome) => Ok(Some(outcome)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

// ---------------------------------------------------------------------------
// GET /api/sessions/{id}/outcome
// ---------------------------------------------------------------------------

pub async fn get_session_outcome(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    // Verify session exists and belongs to user
    match load_session(&state, &session_id, &auth.user_id) {
        Ok(Some(_)) => {}
        Ok(None) => return not_found("session not found"),
        Err(e) => return internal_error(&format!("db error: {e}")),
    };

    match load_session_outcome(&state, &session_id) {
        Ok(Some(outcome)) => Json(serde_json::to_value(outcome).unwrap_or_default()).into_response(),
        Ok(None) => not_found("no outcome recorded for this session"),
        Err(e) => internal_error(&format!("db error: {e}")),
    }
}

// ---------------------------------------------------------------------------
// Notifications
// ---------------------------------------------------------------------------

/// Insert a notification into the database, deduplicating by session_id + notification_type.
///
/// If an unread notification with the same session_id and notification_type already exists,
/// this is a no-op.
#[allow(clippy::too_many_arguments)]
fn create_notification(
    state: &AppState,
    user_id: &str,
    session_id: &str,
    notification_type: &str,
    priority: &str,
    title: &str,
    body: &str,
    deep_link: &str,
) {
    let db = state.db.lock().expect("db lock poisoned");

    // Check for duplicate: don't insert if an unread notification with the same
    // session_id + notification_type already exists.
    let existing: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM notifications
             WHERE session_id = ?1 AND notification_type = ?2 AND read_at IS NULL",
            params![session_id, notification_type],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if existing > 0 {
        return;
    }

    let id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    if let Err(e) = db.execute(
        "INSERT INTO notifications (id, user_id, session_id, notification_type, priority, title, body, deep_link, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![id, user_id, session_id, notification_type, priority, title, body, deep_link, now],
    ) {
        tracing::error!("failed to create notification: {e}");
        return;
    }

    tracing::info!(
        notification_id = %id,
        notification_type = %notification_type,
        session_id = %session_id,
        user_id = %user_id,
        "notification created (push delivery stub)"
    );
}

/// GET /api/notifications — list notifications for the current user.
pub async fn list_notifications(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Query(params): Query<ListNotificationsQuery>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    match query_notifications(&state, &auth.user_id, limit, offset) {
        Ok(notifications) => Json(serde_json::to_value(notifications).unwrap_or_default()).into_response(),
        Err(e) => internal_error(&format!("db error: {e}")),
    }
}

#[allow(clippy::significant_drop_tightening)]
fn query_notifications(
    state: &AppState,
    user_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<Notification>, rusqlite::Error> {
    let db = state.db.lock().expect("db lock poisoned");
    let mut stmt = db.prepare(
        "SELECT id, user_id, session_id, notification_type, priority, title, body,
                deep_link, read_at, dismissed_at, created_at
         FROM notifications
         WHERE user_id = ?1
         ORDER BY read_at IS NULL DESC, created_at DESC
         LIMIT ?2 OFFSET ?3",
    )?;

    let rows = stmt.query_map(params![user_id, limit, offset], |row| {
        Ok(Notification {
            id: row.get(0)?,
            user_id: row.get(1)?,
            session_id: row.get(2)?,
            notification_type: row.get(3)?,
            priority: row.get(4)?,
            title: row.get(5)?,
            body: row.get(6)?,
            deep_link: row.get(7)?,
            read_at: row.get(8)?,
            dismissed_at: row.get(9)?,
            created_at: row.get(10)?,
        })
    })?;

    let mut notifications = Vec::new();
    for row in rows {
        notifications.push(row?);
    }
    Ok(notifications)
}

/// POST /api/notifications/{id}/read — mark a notification as read.
pub async fn mark_notification_read(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Path(notification_id): Path<String>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let result = state.db.lock().expect("db lock poisoned").execute(
        "UPDATE notifications SET read_at = ?1 WHERE id = ?2 AND user_id = ?3 AND read_at IS NULL",
        params![now, notification_id, auth.user_id],
    );

    match result {
        Ok(0) => not_found("notification not found or already read"),
        Ok(_) => Json(serde_json::json!({"status": "ok"})).into_response(),
        Err(e) => internal_error(&format!("db error: {e}")),
    }
}

/// POST /api/notifications/{id}/dismiss — dismiss a notification.
pub async fn dismiss_notification(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Path(notification_id): Path<String>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let result = state.db.lock().expect("db lock poisoned").execute(
        "UPDATE notifications SET dismissed_at = ?1 WHERE id = ?2 AND user_id = ?3 AND dismissed_at IS NULL",
        params![now, notification_id, auth.user_id],
    );

    match result {
        Ok(0) => not_found("notification not found or already dismissed"),
        Ok(_) => Json(serde_json::json!({"status": "ok"})).into_response(),
        Err(e) => internal_error(&format!("db error: {e}")),
    }
}

/// GET /api/notifications/unread-count — count unread, non-dismissed notifications.
pub async fn unread_count(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
) -> impl IntoResponse {
    let result = state.db.lock().expect("db lock poisoned").query_row(
        "SELECT COUNT(*) FROM notifications WHERE user_id = ?1 AND read_at IS NULL AND dismissed_at IS NULL",
        params![auth.user_id],
        |row| row.get::<_, i64>(0),
    );

    match result {
        Ok(count) => Json(serde_json::to_value(UnreadCountResponse { count }).unwrap_or_default()).into_response(),
        Err(e) => internal_error(&format!("db error: {e}")),
    }
}

/// POST /api/devices — register a device token for push notifications.
pub async fn register_device(
    State(state): State<AppState>,
    DashboardAuth(auth): DashboardAuth,
    Json(body): Json<RegisterDeviceRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let result = state.db.lock().expect("db lock poisoned").execute(
        "INSERT INTO device_tokens (id, user_id, token, platform, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(token) DO UPDATE SET platform = excluded.platform",
        params![id, auth.user_id, body.token, body.platform, now],
    );

    match result {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"status": "ok"})),
        )
            .into_response(),
        Err(e) => internal_error(&format!("db error: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    fn test_state() -> AppState {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        db::init_db(&conn).unwrap();
        AppState {
            db: Arc::new(Mutex::new(conn)),
            event_channels: Arc::new(Mutex::new(HashMap::new())),
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
            outcome: None,
            search: None,
            from_date: None,
            to_date: None,
            sort_by: None,
            sort_order: None,
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
            outcome: None,
            search: None,
            from_date: None,
            to_date: None,
            sort_by: None,
            sort_order: None,
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
            outcome: None,
            search: None,
            from_date: None,
            to_date: None,
            sort_by: None,
            sort_order: None,
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

    // -----------------------------------------------------------------------
    // Event ingestion and query tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_ingest_and_query_events() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_running_session(&state, "s-1", "m-1");
        let now = Utc::now().to_rfc3339();

        // Insert a batch of events
        insert_output_event(
            &state, "e-1", "s-1", "output_line", Some("info"), "Hello world", None, 1, &now,
        )
        .unwrap();
        insert_output_event(
            &state, "e-2", "s-1", "output_line", Some("warning"), "Watch out", None, 2, &now,
        )
        .unwrap();
        insert_output_event(
            &state, "e-3", "s-1", "state_changed", None, "running", None, 3, &now,
        )
        .unwrap();

        // Query all events
        let query = QueryEventsParams {
            since_sequence: None,
            limit: None,
            event_type: None,
        };
        let events = query_output_events(&state, "s-1", &query).unwrap();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].sequence_num, 1);
        assert_eq!(events[1].sequence_num, 2);
        assert_eq!(events[2].sequence_num, 3);
    }

    #[test]
    fn test_sequence_numbering_is_monotonic() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_running_session(&state, "s-1", "m-1");
        let now = Utc::now().to_rfc3339();

        // Insert some events
        for i in 1..=5 {
            insert_output_event(
                &state,
                &format!("e-{i}"),
                "s-1",
                "output_line",
                Some("info"),
                &format!("Line {i}"),
                None,
                i,
                &now,
            )
            .unwrap();
        }

        // Check max sequence
        let max = get_max_sequence_num(&state, "s-1").unwrap();
        assert_eq!(max, 5);

        // Insert another and verify it would be 6
        let next = max + 1;
        assert_eq!(next, 6);

        insert_output_event(
            &state, "e-6", "s-1", "output_line", Some("info"), "Line 6", None, next, &now,
        )
        .unwrap();
        let max = get_max_sequence_num(&state, "s-1").unwrap();
        assert_eq!(max, 6);
    }

    #[test]
    fn test_approval_created_from_approval_request_event() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_running_session(&state, "s-1", "m-1");
        let now = Utc::now().to_rfc3339();

        let options = r#"["yes","no","skip"]"#;
        insert_pending_approval(
            &state,
            "s-1",
            "Should I proceed with the deployment?",
            options,
            Some("prod environment"),
            &now,
        )
        .unwrap();

        let approvals = query_pending_approvals(&state, "s-1").unwrap();
        assert_eq!(approvals.len(), 1);
        assert_eq!(approvals[0].question, "Should I proceed with the deployment?");
        assert_eq!(approvals[0].options, options);
        assert_eq!(approvals[0].context, Some("prod environment".into()));
        assert_eq!(approvals[0].status, ApprovalStatus::Pending);
    }

    #[test]
    fn test_event_query_with_since_sequence() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_running_session(&state, "s-1", "m-1");
        let now = Utc::now().to_rfc3339();

        for i in 1..=10 {
            insert_output_event(
                &state,
                &format!("e-{i}"),
                "s-1",
                "output_line",
                Some("info"),
                &format!("Line {i}"),
                None,
                i,
                &now,
            )
            .unwrap();
        }

        // Query since sequence 7 (should return events 8, 9, 10)
        let query = QueryEventsParams {
            since_sequence: Some(7),
            limit: None,
            event_type: None,
        };
        let events = query_output_events(&state, "s-1", &query).unwrap();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].sequence_num, 8);
        assert_eq!(events[1].sequence_num, 9);
        assert_eq!(events[2].sequence_num, 10);
    }

    #[test]
    fn test_event_query_with_event_type_filter() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_running_session(&state, "s-1", "m-1");
        let now = Utc::now().to_rfc3339();

        insert_output_event(
            &state, "e-1", "s-1", "output_line", Some("info"), "Hello", None, 1, &now,
        )
        .unwrap();
        insert_output_event(
            &state, "e-2", "s-1", "state_changed", None, "paused", None, 2, &now,
        )
        .unwrap();
        insert_output_event(
            &state, "e-3", "s-1", "output_line", Some("error"), "Error!", None, 3, &now,
        )
        .unwrap();

        // Filter by output_line only
        let query = QueryEventsParams {
            since_sequence: None,
            limit: None,
            event_type: Some("output_line".to_string()),
        };
        let events = query_output_events(&state, "s-1", &query).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].content, "Hello");
        assert_eq!(events[1].content, "Error!");
    }

    #[test]
    fn test_event_query_with_limit() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_running_session(&state, "s-1", "m-1");
        let now = Utc::now().to_rfc3339();

        for i in 1..=10 {
            insert_output_event(
                &state,
                &format!("e-{i}"),
                "s-1",
                "output_line",
                Some("info"),
                &format!("Line {i}"),
                None,
                i,
                &now,
            )
            .unwrap();
        }

        let query = QueryEventsParams {
            since_sequence: None,
            limit: Some(3),
            event_type: None,
        };
        let events = query_output_events(&state, "s-1", &query).unwrap();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].sequence_num, 1);
        assert_eq!(events[2].sequence_num, 3);
    }

    #[test]
    fn test_pending_approvals_returns_only_pending() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_running_session(&state, "s-1", "m-1");
        let now = Utc::now().to_rfc3339();

        // Insert two approvals
        insert_pending_approval(&state, "s-1", "Q1?", "[]", None, &now).unwrap();
        insert_pending_approval(&state, "s-1", "Q2?", "[]", None, &now).unwrap();

        // Mark one as responded
        state
            .db
            .lock()
            .unwrap()
            .execute(
                "UPDATE pending_approvals SET status = 'responded' WHERE question = 'Q1?'",
                [],
            )
            .unwrap();

        let approvals = query_pending_approvals(&state, "s-1").unwrap();
        assert_eq!(approvals.len(), 1);
        assert_eq!(approvals[0].question, "Q2?");
    }

    #[test]
    fn test_get_max_sequence_num_empty_session() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_running_session(&state, "s-1", "m-1");

        let max = get_max_sequence_num(&state, "s-1").unwrap();
        assert_eq!(max, 0);
    }

    #[test]
    fn test_broadcast_event_no_subscribers() {
        let state = test_state();
        // Should not panic even when no channel exists
        broadcast_event(&state, "nonexistent-session", r#"{"test": true}"#);
    }

    #[test]
    fn test_event_metadata_preserved() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_running_session(&state, "s-1", "m-1");
        let now = Utc::now().to_rfc3339();
        let metadata = r#"{"key":"value","nested":{"a":1}}"#;

        insert_output_event(
            &state,
            "e-1",
            "s-1",
            "output_line",
            Some("info"),
            "test",
            Some(metadata),
            1,
            &now,
        )
        .unwrap();

        let query = QueryEventsParams {
            since_sequence: None,
            limit: None,
            event_type: None,
        };
        let events = query_output_events(&state, "s-1", &query).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].metadata, Some(metadata.to_string()));
    }

    // -----------------------------------------------------------------------
    // Approval response and guidance injection tests
    // -----------------------------------------------------------------------

    /// Helper: insert a pending approval and return its id.
    fn create_pending_approval(state: &AppState, session_id: &str) -> String {
        let now = Utc::now().to_rfc3339();
        insert_pending_approval(state, session_id, "Proceed?", r#"["yes","no"]"#, Some("ctx"), &now).unwrap();
        // Fetch the approval id
        let approvals = query_pending_approvals(state, session_id).unwrap();
        approvals.last().unwrap().id.clone()
    }

    #[test]
    fn test_respond_updates_approval_and_queues_command() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_running_session(&state, "s-1", "m-1");
        let approval_id = create_pending_approval(&state, "s-1");

        let now = Utc::now().to_rfc3339();

        // Update approval response
        update_approval_response(&state, &approval_id, "yes", Some("looks good"), &now).unwrap();

        // Verify status changed
        let approval = load_pending_approval(&state, &approval_id, "s-1").unwrap().unwrap();
        assert_eq!(approval.status, ApprovalStatus::Responded);
        assert_eq!(approval.response_choice, Some("yes".into()));
        assert_eq!(approval.response_note, Some("looks good".into()));
        assert!(approval.responded_at.is_some());

        // Queue respond command
        let payload = serde_json::json!({
            "approval_id": approval_id,
            "choice": "yes",
            "note": "looks good",
        });
        insert_session_command(&state, "s-1", "m-1", "respond", Some(&payload.to_string())).unwrap();

        // Verify command was queued
        let count: i64 = state
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM session_commands WHERE session_id = ?1 AND command_type = 'respond'",
                ["s-1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_respond_to_already_responded_approval() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_running_session(&state, "s-1", "m-1");
        let approval_id = create_pending_approval(&state, "s-1");

        let now = Utc::now().to_rfc3339();

        // First response
        update_approval_response(&state, &approval_id, "yes", None, &now).unwrap();

        // Loading it should show responded status
        let approval = load_pending_approval(&state, &approval_id, "s-1").unwrap().unwrap();
        assert_eq!(approval.status, ApprovalStatus::Responded);
        // Attempting to use it again should be caught by the status != Pending check
        assert_ne!(approval.status, ApprovalStatus::Pending);
    }

    #[test]
    fn test_respond_to_nonexistent_approval() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_running_session(&state, "s-1", "m-1");

        // Load a nonexistent approval
        let approval = load_pending_approval(&state, "nonexistent-id", "s-1").unwrap();
        assert!(approval.is_none());
    }

    #[test]
    fn test_respond_transitions_session_from_waiting_to_running() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_running_session(&state, "s-1", "m-1");
        let now = Utc::now().to_rfc3339();

        // Set session to waiting_for_input
        update_session_state(&state, "s-1", "waiting_for_input", &now, None).unwrap();
        let session = load_session(&state, "s-1", "user-default").unwrap().unwrap();
        assert_eq!(session.state, SessionState::WaitingForInput);

        // Transition back to running (as the handler would)
        update_session_state(&state, "s-1", "running", &now, None).unwrap();
        insert_session_event(&state, "s-1", Some("waiting_for_input"), "running", &now, None).unwrap();

        let session = load_session(&state, "s-1", "user-default").unwrap().unwrap();
        assert_eq!(session.state, SessionState::Running);

        // Verify state change event was inserted
        let count: i64 = state
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM session_events WHERE session_id = ?1 AND to_state = 'running' AND from_state = 'waiting_for_input'",
                ["s-1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_inject_queues_command_and_persists_event() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_running_session(&state, "s-1", "m-1");
        let now = Utc::now().to_rfc3339();

        // Queue inject command
        let payload = serde_json::json!({
            "message": "Focus on the login bug",
            "injection_type": "normal",
        });
        insert_session_command(&state, "s-1", "m-1", "inject", Some(&payload.to_string())).unwrap();

        // Insert guidance_injected event
        let metadata = serde_json::json!({
            "injection_type": "normal",
        })
        .to_string();
        insert_output_event(
            &state,
            "e-inject-1",
            "s-1",
            SessionOutputEventType::GuidanceInjected.as_str(),
            None,
            "Focus on the login bug",
            Some(&metadata),
            1,
            &now,
        )
        .unwrap();

        // Verify command queued
        let count: i64 = state
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM session_commands WHERE session_id = ?1 AND command_type = 'inject'",
                ["s-1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // Verify event persisted
        let query = QueryEventsParams {
            since_sequence: None,
            limit: None,
            event_type: Some("guidance_injected".to_string()),
        };
        let events = query_output_events(&state, "s-1", &query).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].content, "Focus on the login bug");
        assert_eq!(events[0].event_type, SessionOutputEventType::GuidanceInjected);
    }

    #[test]
    fn test_inject_on_terminal_session_is_rejected() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_running_session(&state, "s-1", "m-1");
        let now = Utc::now().to_rfc3339();

        // Transition to completed (terminal)
        update_session_state(&state, "s-1", "completed", &now, Some(&now)).unwrap();
        let session = load_session(&state, "s-1", "user-default").unwrap().unwrap();
        assert_eq!(session.state, SessionState::Completed);
        assert!(session.state.is_terminal());

        // Verify that a running/waiting_for_input check would reject this
        assert!(!matches!(
            session.state,
            SessionState::Running | SessionState::WaitingForInput
        ));
    }

    #[test]
    fn test_inject_on_running_session_succeeds() {
        let state = test_state();
        create_trusted_machine(&state, "m-1");
        create_running_session(&state, "s-1", "m-1");

        let session = load_session(&state, "s-1", "user-default").unwrap().unwrap();
        assert_eq!(session.state, SessionState::Running);

        // Running session passes the state check
        assert!(matches!(
            session.state,
            SessionState::Running | SessionState::WaitingForInput
        ));

        // Can queue inject command without error
        let payload = serde_json::json!({
            "message": "Update approach",
            "injection_type": "side_note",
        });
        insert_session_command(&state, "s-1", "m-1", "inject", Some(&payload.to_string())).unwrap();

        let count: i64 = state
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM session_commands WHERE session_id = ?1 AND command_type = 'inject'",
                ["s-1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}

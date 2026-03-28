//! Integration tests for session history and replay (outcome, filtering, search).
//!
//! Each test spins up an in-process Axum server on a random port with its own
//! temporary SQLite database for full isolation.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use cadre_control_api::db;
use cadre_control_api::{build_app, AppState};
use reqwest::Client;
use tempfile::NamedTempFile;

// ---------------------------------------------------------------------------
// Test helper
// ---------------------------------------------------------------------------

const VALID_TOKEN: &str = "cadre-mvp-static-token";

/// Spin up a test server on a random port and return the base URL + task handle.
async fn start_test_server() -> (String, tokio::task::JoinHandle<()>) {
    let tmp = NamedTempFile::new().expect("failed to create temp file");
    let db_path = tmp.path().to_str().expect("non-utf8 path").to_string();

    // Keep the temp file alive by leaking it; the OS will clean up.
    std::mem::forget(tmp);

    let conn = rusqlite::Connection::open(&db_path).expect("failed to open temp db");
    db::init_db(&conn).expect("failed to init db");

    let state = AppState {
        db: Arc::new(Mutex::new(conn)),
        event_channels: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = build_app(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind");
    let addr = listener.local_addr().expect("no local addr");
    let base_url = format!("http://{addr}");

    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server error");
    });

    (base_url, handle)
}

/// Register a machine via the API. Returns the machine id.
async fn register_machine(client: &Client, base: &str, name: &str) -> String {
    let body = serde_json::json!({
        "name": name,
        "platform": "linux",
        "repos": [{"path": "/home/user/repo", "name": "repo"}],
    });

    let resp = client
        .post(format!("{base}/api/machines/register"))
        .bearer_auth(VALID_TOKEN)
        .json(&body)
        .send()
        .await
        .expect("register request failed");

    assert_eq!(resp.status(), 201);
    let json: serde_json::Value = resp.json().await.expect("register response not json");
    json["id"]
        .as_str()
        .expect("no id in register response")
        .to_string()
}

/// Approve a machine via the API.
async fn approve_machine(client: &Client, base: &str, id: &str) {
    let resp = client
        .post(format!("{base}/api/machines/{id}/approve"))
        .send()
        .await
        .expect("approve request failed");
    assert_eq!(resp.status(), 200);
}

/// Register and approve a machine. Returns the machine id.
async fn register_and_approve(client: &Client, base: &str, name: &str) -> String {
    let id = register_machine(client, base, name).await;
    approve_machine(client, base, &id).await;
    id
}

/// Create a session with a custom title. Returns the session id.
async fn create_session_with_title(
    client: &Client,
    base: &str,
    machine_id: &str,
    title: &str,
) -> String {
    let resp = client
        .post(format!("{base}/api/sessions"))
        .json(&serde_json::json!({
            "machine_id": machine_id,
            "repo_path": "/home/user/repo",
            "title": title,
            "instructions": "Do the thing",
            "autonomy_level": "normal",
        }))
        .send()
        .await
        .expect("create session request failed");

    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.expect("create session body not json");
    assert_eq!(body["state"], "starting");
    body["id"]
        .as_str()
        .expect("no id in create session response")
        .to_string()
}

/// Runner reports a state transition for a session.
async fn report_state(
    client: &Client,
    base: &str,
    session_id: &str,
    state: &str,
) -> reqwest::StatusCode {
    client
        .post(format!("{base}/api/sessions/{session_id}/state"))
        .bearer_auth(VALID_TOKEN)
        .json(&serde_json::json!({ "state": state }))
        .send()
        .await
        .expect("report state request failed")
        .status()
}

/// Ingest output events for a session.
async fn ingest_events(
    client: &Client,
    base: &str,
    session_id: &str,
    events: serde_json::Value,
) -> reqwest::StatusCode {
    client
        .post(format!("{base}/api/sessions/{session_id}/events"))
        .bearer_auth(VALID_TOKEN)
        .json(&serde_json::json!({ "events": events }))
        .send()
        .await
        .expect("ingest events request failed")
        .status()
}

/// Acknowledge start command so the session can proceed.
async fn ack_start_command(client: &Client, base: &str, machine_id: &str) {
    let resp = client
        .get(format!("{base}/api/machines/{machine_id}/commands"))
        .bearer_auth(VALID_TOKEN)
        .send()
        .await
        .expect("get commands request failed");
    assert_eq!(resp.status(), 200);
    let cmds: Vec<serde_json::Value> = resp.json().await.expect("commands body not json");
    assert!(!cmds.is_empty(), "expected start command to be queued");
    let cmd_id = cmds[0]["command_id"].as_str().unwrap();
    let ack_resp = client
        .post(format!(
            "{base}/api/machines/{machine_id}/commands/{cmd_id}/ack"
        ))
        .bearer_auth(VALID_TOKEN)
        .send()
        .await
        .expect("ack command request failed");
    assert_eq!(ack_resp.status(), 200);
}

// ===========================================================================
// Test 1: Session outcome written on completion
// ===========================================================================

#[tokio::test]
async fn session_outcome_written_on_completion() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    // Register + approve machine
    let machine_id = register_and_approve(&client, &base, "history-m1").await;

    // Create session
    let session_id =
        create_session_with_title(&client, &base, &machine_id, "Outcome test session").await;

    // Ack the start command
    ack_start_command(&client, &base, &machine_id).await;

    // Report state as running
    let s = report_state(&client, &base, &session_id, "running").await;
    assert_eq!(s, 200);

    // Ingest some events
    let s = ingest_events(
        &client,
        &base,
        &session_id,
        serde_json::json!([
            {"event_type": "output_line", "content": "Hello", "category": "info"},
            {"event_type": "output_line", "content": "World", "category": "info"},
        ]),
    )
    .await;
    assert_eq!(s, 200);

    // Report state as completed
    let s = report_state(&client, &base, &session_id, "completed").await;
    assert_eq!(s, 200);

    // GET /api/sessions/{id}/outcome -> verify 200, status="success", event_count > 0
    let resp = client
        .get(format!("{base}/api/sessions/{session_id}/outcome"))
        .send()
        .await
        .expect("get outcome request failed");

    assert_eq!(resp.status(), 200);
    let outcome: serde_json::Value = resp.json().await.expect("outcome body not json");
    assert_eq!(outcome["status"], "success");
    assert!(
        outcome["event_count"].as_i64().unwrap_or(0) > 0,
        "event_count should be > 0, got {:?}",
        outcome["event_count"]
    );
    assert!(
        outcome["session_id"].as_str() == Some(session_id.as_str()),
        "outcome should reference the correct session_id"
    );

    handle.abort();
}

// ===========================================================================
// Test 2: Outcome not found for running session
// ===========================================================================

#[tokio::test]
async fn outcome_not_found_for_running_session() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    // Register + approve machine
    let machine_id = register_and_approve(&client, &base, "history-m2").await;

    // Create session
    let session_id =
        create_session_with_title(&client, &base, &machine_id, "Running session test").await;

    // Ack the start command
    ack_start_command(&client, &base, &machine_id).await;

    // Report state as running (but do NOT complete it)
    let s = report_state(&client, &base, &session_id, "running").await;
    assert_eq!(s, 200);

    // GET /api/sessions/{id}/outcome -> verify 404
    let resp = client
        .get(format!("{base}/api/sessions/{session_id}/outcome"))
        .send()
        .await
        .expect("get outcome request failed");

    assert_eq!(
        resp.status(),
        404,
        "outcome should not exist for a running session"
    );

    handle.abort();
}

// ===========================================================================
// Test 3: Filter sessions by outcome
// ===========================================================================

#[tokio::test]
async fn filter_sessions_by_outcome() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    // Register + approve machine
    let machine_id = register_and_approve(&client, &base, "history-m3").await;

    // --- Session A: completed (outcome = "success") ---
    let session_a =
        create_session_with_title(&client, &base, &machine_id, "Session A completed").await;
    ack_start_command(&client, &base, &machine_id).await;
    let s = report_state(&client, &base, &session_a, "running").await;
    assert_eq!(s, 200);
    let s = report_state(&client, &base, &session_a, "completed").await;
    assert_eq!(s, 200);

    // --- Session B: stopped (outcome = "partial") ---
    let session_b =
        create_session_with_title(&client, &base, &machine_id, "Session B stopped").await;
    ack_start_command(&client, &base, &machine_id).await;
    let s = report_state(&client, &base, &session_b, "running").await;
    assert_eq!(s, 200);
    let s = report_state(&client, &base, &session_b, "stopped").await;
    assert_eq!(s, 200);

    // GET /api/sessions?outcome=success -> should only contain session A
    let resp = client
        .get(format!("{base}/api/sessions?outcome=success"))
        .send()
        .await
        .expect("list sessions request failed");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("list body not json");
    let sessions = body["sessions"].as_array().expect("sessions should be array");
    assert_eq!(
        sessions.len(),
        1,
        "expected 1 session with outcome=success, got {}",
        sessions.len()
    );
    assert_eq!(sessions[0]["id"], session_a);

    // GET /api/sessions?outcome=partial -> should only contain session B
    let resp = client
        .get(format!("{base}/api/sessions?outcome=partial"))
        .send()
        .await
        .expect("list sessions request failed");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("list body not json");
    let sessions = body["sessions"].as_array().expect("sessions should be array");
    assert_eq!(
        sessions.len(),
        1,
        "expected 1 session with outcome=partial, got {}",
        sessions.len()
    );
    assert_eq!(sessions[0]["id"], session_b);

    handle.abort();
}

// ===========================================================================
// Test 4: Search sessions by title
// ===========================================================================

#[tokio::test]
async fn search_sessions_by_title() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    // Register + approve machine
    let machine_id = register_and_approve(&client, &base, "history-m4").await;

    // Create two sessions with distinct titles
    let session_frontend =
        create_session_with_title(&client, &base, &machine_id, "Deploy frontend v2").await;
    let _session_backend =
        create_session_with_title(&client, &base, &machine_id, "Backend refactor").await;

    // GET /api/sessions?search=frontend -> should only find "Deploy frontend v2"
    let resp = client
        .get(format!("{base}/api/sessions?search=frontend"))
        .send()
        .await
        .expect("search sessions request failed");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("search body not json");
    let sessions = body["sessions"].as_array().expect("sessions should be array");
    assert_eq!(
        sessions.len(),
        1,
        "expected 1 session matching 'frontend', got {}",
        sessions.len()
    );
    assert_eq!(sessions[0]["id"], session_frontend);
    assert_eq!(sessions[0]["title"], "Deploy frontend v2");

    // Also verify searching for "Backend" finds the other one
    let resp = client
        .get(format!("{base}/api/sessions?search=Backend"))
        .send()
        .await
        .expect("search sessions request failed");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("search body not json");
    let sessions = body["sessions"].as_array().expect("sessions should be array");
    assert_eq!(
        sessions.len(),
        1,
        "expected 1 session matching 'Backend', got {}",
        sessions.len()
    );
    assert_eq!(sessions[0]["title"], "Backend refactor");

    handle.abort();
}

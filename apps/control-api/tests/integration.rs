//! Integration tests for the Cadre Control API.
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
    // Without this the file is deleted when `tmp` drops.
    std::mem::forget(tmp);

    let conn =
        rusqlite::Connection::open(&db_path).expect("failed to open temp db");
    db::init_db(&conn).expect("failed to init db");

    let state = AppState {
        db: Arc::new(Mutex::new(conn)),
        event_channels: Arc::new(Mutex::new(HashMap::new())),
        log_channels: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = build_app(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind");
    let addr = listener.local_addr().expect("no local addr");
    let base_url = format!("http://{addr}");

    let handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("server error");
    });

    (base_url, handle)
}

/// Register a machine via the API. Returns the machine id.
async fn register_machine(
    client: &Client,
    base: &str,
    name: &str,
    repos: Option<serde_json::Value>,
) -> (String, reqwest::StatusCode) {
    let mut body = serde_json::json!({
        "name": name,
        "platform": "linux",
    });

    if let Some(r) = repos {
        body["repos"] = r;
    }

    let resp = client
        .post(format!("{base}/api/machines/register"))
        .bearer_auth(VALID_TOKEN)
        .json(&body)
        .send()
        .await
        .expect("register request failed");

    let status = resp.status();
    let json: serde_json::Value =
        resp.json().await.expect("register response not json");
    let id = json["id"]
        .as_str()
        .expect("no id in register response")
        .to_string();

    (id, status)
}

/// Approve a machine via the API.
async fn approve_machine(
    client: &Client,
    base: &str,
    id: &str,
) -> reqwest::StatusCode {
    client
        .post(format!("{base}/api/machines/{id}/approve"))
        .send()
        .await
        .expect("approve request failed")
        .status()
}

// ---------------------------------------------------------------------------
// 1. Health check
// ---------------------------------------------------------------------------

#[tokio::test]
async fn health_check_returns_ok() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let resp = client
        .get(format!("{base}/health"))
        .send()
        .await
        .expect("health request failed");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value =
        resp.json().await.expect("health body not json");
    assert_eq!(body["status"], "ok");

    handle.abort();
}

// ---------------------------------------------------------------------------
// 2. TC-001: Registration to approval lifecycle
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tc001_registration_to_approval_lifecycle() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    // Register
    let (id, status) = register_machine(
        &client,
        &base,
        "lifecycle-machine",
        Some(serde_json::json!([{"path": "/home/user/repo", "name": "repo"}])),
    )
    .await;
    assert_eq!(status, 201);
    assert!(!id.is_empty());

    // List — should appear as pending
    let machines = list_machines(&client, &base).await;
    let m = find_machine_in_list(&machines, &id);
    assert_eq!(m["status"], "pending");

    // Approve
    let approve_status = approve_machine(&client, &base, &id).await;
    assert_eq!(approve_status, 200);

    // List — should now be trusted
    let machines = list_machines(&client, &base).await;
    let m = find_machine_in_list(&machines, &id);
    assert_eq!(m["status"], "trusted");

    handle.abort();
}

/// GET /api/machines and return the JSON array.
async fn list_machines(client: &Client, base: &str) -> Vec<serde_json::Value> {
    let resp = client
        .get(format!("{base}/api/machines"))
        .send()
        .await
        .expect("list request failed");
    assert_eq!(resp.status(), 200);
    resp.json().await.expect("list body not json array")
}

/// Find a machine by id in a list response.
fn find_machine_in_list<'a>(
    machines: &'a [serde_json::Value],
    id: &str,
) -> &'a serde_json::Value {
    machines
        .iter()
        .find(|m| m["id"].as_str() == Some(id))
        .expect("machine not found in list")
}

// ---------------------------------------------------------------------------
// 3. TC-002: Revocation flow
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tc002_revocation_flow() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    // Register + approve
    let (id, _) =
        register_machine(&client, &base, "revoke-machine", None).await;
    approve_machine(&client, &base, &id).await;

    // Revoke
    let revoke_resp = client
        .post(format!("{base}/api/machines/{id}/revoke"))
        .send()
        .await
        .expect("revoke request failed");
    assert_eq!(revoke_resp.status(), 200);

    // Heartbeat should fail with 401 (revoked)
    let hb_resp = send_heartbeat(&client, &base, &id).await;
    assert_eq!(hb_resp.status(), 401);

    // GET detail — status is revoked
    let detail = get_machine_detail(&client, &base, &id).await;
    assert_eq!(detail["status"], "revoked");

    handle.abort();
}

/// Send a heartbeat for a machine.
async fn send_heartbeat(
    client: &Client,
    base: &str,
    id: &str,
) -> reqwest::Response {
    client
        .post(format!("{base}/api/machines/{id}/heartbeat"))
        .bearer_auth(VALID_TOKEN)
        .json(&serde_json::json!({}))
        .send()
        .await
        .expect("heartbeat request failed")
}

/// GET /api/machines/{id} and return the JSON body.
async fn get_machine_detail(
    client: &Client,
    base: &str,
    id: &str,
) -> serde_json::Value {
    let resp = client
        .get(format!("{base}/api/machines/{id}"))
        .send()
        .await
        .expect("get machine request failed");
    assert_eq!(resp.status(), 200);
    resp.json().await.expect("machine detail not json")
}

// ---------------------------------------------------------------------------
// 4. Pending blocks heartbeat
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pending_machine_blocks_heartbeat() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let (id, _) =
        register_machine(&client, &base, "pending-hb", None).await;

    let resp = send_heartbeat(&client, &base, &id).await;
    assert_eq!(resp.status(), 403);

    handle.abort();
}

// ---------------------------------------------------------------------------
// 5. Invalid token
// ---------------------------------------------------------------------------

#[tokio::test]
async fn invalid_token_returns_401() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let resp = client
        .post(format!("{base}/api/machines/register"))
        .bearer_auth("invalid-token")
        .json(&serde_json::json!({
            "name": "bad",
            "platform": "linux",
        }))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 401);

    handle.abort();
}

// ---------------------------------------------------------------------------
// 6. Approve non-pending returns 409
// ---------------------------------------------------------------------------

#[tokio::test]
async fn approve_non_pending_returns_409() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let (id, _) =
        register_machine(&client, &base, "double-approve", None).await;

    // First approve succeeds
    let s = approve_machine(&client, &base, &id).await;
    assert_eq!(s, 200);

    // Second approve should be 409 Conflict
    let s = approve_machine(&client, &base, &id).await;
    assert_eq!(s, 409);

    handle.abort();
}

// ---------------------------------------------------------------------------
// 7. Heartbeat updates timestamp
// ---------------------------------------------------------------------------

#[tokio::test]
async fn heartbeat_updates_last_heartbeat() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let (id, _) =
        register_machine(&client, &base, "hb-timestamp", None).await;
    approve_machine(&client, &base, &id).await;

    // Before heartbeat — last_heartbeat should be null
    let detail = get_machine_detail(&client, &base, &id).await;
    assert!(
        detail["last_heartbeat"].is_null(),
        "expected null before heartbeat"
    );

    // Send heartbeat
    let resp = send_heartbeat(&client, &base, &id).await;
    assert_eq!(resp.status(), 200);

    // After heartbeat — last_heartbeat should be set
    let detail = get_machine_detail(&client, &base, &id).await;
    assert!(
        detail["last_heartbeat"].is_string(),
        "expected last_heartbeat to be set after heartbeat"
    );

    handle.abort();
}

// ---------------------------------------------------------------------------
// 8. Machine detail includes repos
// ---------------------------------------------------------------------------

#[tokio::test]
async fn machine_detail_includes_repos() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let repos = serde_json::json!([
        {"path": "/home/user/project-a", "name": "project-a"},
        {"path": "/home/user/project-b", "name": "project-b"},
    ]);
    let (id, _) =
        register_machine(&client, &base, "with-repos", Some(repos)).await;
    approve_machine(&client, &base, &id).await;

    let detail = get_machine_detail(&client, &base, &id).await;

    let repos_arr = detail["repos"]
        .as_array()
        .expect("repos should be an array");
    assert_eq!(repos_arr.len(), 2);

    let paths: Vec<&str> = repos_arr
        .iter()
        .filter_map(|r| r["repo_path"].as_str())
        .collect();
    assert!(paths.contains(&"/home/user/project-a"));
    assert!(paths.contains(&"/home/user/project-b"));

    handle.abort();
}

// ===========================================================================
// Session lifecycle helpers
// ===========================================================================

/// Register a machine and approve it. Returns the machine id.
async fn register_and_approve_machine(
    client: &Client,
    base: &str,
    name: &str,
) -> String {
    let (id, status) = register_machine(
        client,
        base,
        name,
        Some(serde_json::json!([{"path": "/home/user/repo", "name": "repo"}])),
    )
    .await;
    assert_eq!(status, 201);
    let s = approve_machine(client, base, &id).await;
    assert_eq!(s, 200);
    id
}

/// Create a session on a machine. Returns the session id.
async fn create_session(
    client: &Client,
    base: &str,
    machine_id: &str,
) -> String {
    let resp = client
        .post(format!("{base}/api/sessions"))
        .json(&serde_json::json!({
            "machine_id": machine_id,
            "repo_path": "/home/user/repo",
            "title": "Test session",
            "instructions": "Do the thing",
        }))
        .send()
        .await
        .expect("create session request failed");
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.expect("create session body not json");
    assert_eq!(body["state"], "starting");
    body["id"].as_str().expect("no id in create session response").to_string()
}

/// Get session detail and return the JSON body.
async fn get_session(
    client: &Client,
    base: &str,
    session_id: &str,
) -> serde_json::Value {
    let resp = client
        .get(format!("{base}/api/sessions/{session_id}"))
        .send()
        .await
        .expect("get session request failed");
    assert_eq!(resp.status(), 200);
    resp.json().await.expect("session body not json")
}

/// Runner reports a state transition for a session.
async fn report_state(
    client: &Client,
    base: &str,
    session_id: &str,
    state: &str,
    metadata: Option<serde_json::Value>,
) -> reqwest::StatusCode {
    let mut body = serde_json::json!({ "state": state });
    if let Some(m) = metadata {
        body["metadata"] = m;
    }
    client
        .post(format!("{base}/api/sessions/{session_id}/state"))
        .bearer_auth(VALID_TOKEN)
        .json(&body)
        .send()
        .await
        .expect("report state request failed")
        .status()
}

/// Fetch pending commands for a machine. Returns the JSON array.
async fn get_commands(
    client: &Client,
    base: &str,
    machine_id: &str,
) -> Vec<serde_json::Value> {
    let resp = client
        .get(format!("{base}/api/machines/{machine_id}/commands"))
        .bearer_auth(VALID_TOKEN)
        .send()
        .await
        .expect("get commands request failed");
    assert_eq!(resp.status(), 200);
    resp.json().await.expect("commands body not json")
}

/// Acknowledge a command. Returns the status code.
async fn ack_command(
    client: &Client,
    base: &str,
    machine_id: &str,
    command_id: &str,
) -> reqwest::StatusCode {
    client
        .post(format!(
            "{base}/api/machines/{machine_id}/commands/{command_id}/ack"
        ))
        .bearer_auth(VALID_TOKEN)
        .send()
        .await
        .expect("ack command request failed")
        .status()
}

// ===========================================================================
// 9. Full session lifecycle
// ===========================================================================

#[tokio::test]
async fn session_full_lifecycle() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    // Register + approve machine
    let machine_id = register_and_approve_machine(&client, &base, "lifecycle-m").await;

    // Create session — should be 201 with state "starting"
    let session_id = create_session(&client, &base, &machine_id).await;

    // Verify start_session command was queued
    let cmds = get_commands(&client, &base, &machine_id).await;
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0]["command_type"], "start_session");
    let start_cmd_id = cmds[0]["command_id"].as_str().unwrap().to_string();

    // Ack the start command
    let ack_status = ack_command(&client, &base, &machine_id, &start_cmd_id).await;
    assert_eq!(ack_status, 200);

    // Verify no more pending commands (it was acked)
    let cmds = get_commands(&client, &base, &machine_id).await;
    assert_eq!(cmds.len(), 0);

    // Runner reports "running"
    let s = report_state(&client, &base, &session_id, "running", None).await;
    assert_eq!(s, 200);

    // Verify session state is running
    let detail = get_session(&client, &base, &session_id).await;
    assert_eq!(detail["state"], "running");
    assert!(detail["started_at"].is_string(), "started_at should be set");

    // Runner reports "completed" with metadata
    let s = report_state(
        &client,
        &base,
        &session_id,
        "completed",
        Some(serde_json::json!({"exit_code": 0})),
    )
    .await;
    assert_eq!(s, 200);

    // Verify session is completed
    let detail = get_session(&client, &base, &session_id).await;
    assert_eq!(detail["state"], "completed");
    assert!(
        detail["completed_at"].is_string(),
        "completed_at should be set"
    );

    handle.abort();
}

// ===========================================================================
// 10. Stop flow
// ===========================================================================

#[tokio::test]
async fn session_stop_flow() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let machine_id = register_and_approve_machine(&client, &base, "stop-m").await;
    let session_id = create_session(&client, &base, &machine_id).await;

    // Ack the start command so we can move forward
    let cmds = get_commands(&client, &base, &machine_id).await;
    let start_cmd_id = cmds[0]["command_id"].as_str().unwrap().to_string();
    ack_command(&client, &base, &machine_id, &start_cmd_id).await;

    // Runner reports running
    let s = report_state(&client, &base, &session_id, "running", None).await;
    assert_eq!(s, 200);

    // Dashboard requests stop
    let resp = client
        .post(format!("{base}/api/sessions/{session_id}/stop"))
        .send()
        .await
        .expect("stop request failed");
    assert_eq!(resp.status(), 200);

    // Verify stop command appears
    let cmds = get_commands(&client, &base, &machine_id).await;
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0]["command_type"], "stop");
    let stop_cmd_id = cmds[0]["command_id"].as_str().unwrap().to_string();

    // Ack the stop command
    let ack_status = ack_command(&client, &base, &machine_id, &stop_cmd_id).await;
    assert_eq!(ack_status, 200);

    // Runner reports stopped
    let s = report_state(&client, &base, &session_id, "stopped", None).await;
    assert_eq!(s, 200);

    // Verify session state
    let detail = get_session(&client, &base, &session_id).await;
    assert_eq!(detail["state"], "stopped");
    assert!(
        detail["completed_at"].is_string(),
        "completed_at should be set for terminal state"
    );

    handle.abort();
}

// ===========================================================================
// 11. Force stop on starting session
// ===========================================================================

#[tokio::test]
async fn session_force_stop_while_starting() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let machine_id = register_and_approve_machine(&client, &base, "fstop-m").await;
    let session_id = create_session(&client, &base, &machine_id).await;

    // Force-stop while still in "starting" — should transition directly
    let resp = client
        .post(format!("{base}/api/sessions/{session_id}/force-stop"))
        .send()
        .await
        .expect("force-stop request failed");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "stopped");

    // Verify session is stopped
    let detail = get_session(&client, &base, &session_id).await;
    assert_eq!(detail["state"], "stopped");

    handle.abort();
}

// ===========================================================================
// 12. Pause and resume
// ===========================================================================

#[tokio::test]
async fn session_pause_and_resume() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let machine_id = register_and_approve_machine(&client, &base, "pause-m").await;
    let session_id = create_session(&client, &base, &machine_id).await;

    // Ack start command, report running
    let cmds = get_commands(&client, &base, &machine_id).await;
    let start_cmd_id = cmds[0]["command_id"].as_str().unwrap().to_string();
    ack_command(&client, &base, &machine_id, &start_cmd_id).await;
    report_state(&client, &base, &session_id, "running", None).await;

    // --- Pause ---
    let resp = client
        .post(format!("{base}/api/sessions/{session_id}/pause"))
        .send()
        .await
        .expect("pause request failed");
    assert_eq!(resp.status(), 200);

    // Verify pause command queued
    let cmds = get_commands(&client, &base, &machine_id).await;
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0]["command_type"], "pause");
    let pause_cmd_id = cmds[0]["command_id"].as_str().unwrap().to_string();

    // Ack pause command
    ack_command(&client, &base, &machine_id, &pause_cmd_id).await;

    // Runner reports paused
    let s = report_state(&client, &base, &session_id, "paused", None).await;
    assert_eq!(s, 200);

    let detail = get_session(&client, &base, &session_id).await;
    assert_eq!(detail["state"], "paused");

    // --- Resume ---
    let resp = client
        .post(format!("{base}/api/sessions/{session_id}/resume"))
        .send()
        .await
        .expect("resume request failed");
    assert_eq!(resp.status(), 200);

    // Verify resume command queued
    let cmds = get_commands(&client, &base, &machine_id).await;
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0]["command_type"], "resume");
    let resume_cmd_id = cmds[0]["command_id"].as_str().unwrap().to_string();

    // Ack resume command
    ack_command(&client, &base, &machine_id, &resume_cmd_id).await;

    // Runner reports running again
    let s = report_state(&client, &base, &session_id, "running", None).await;
    assert_eq!(s, 200);

    let detail = get_session(&client, &base, &session_id).await;
    assert_eq!(detail["state"], "running");

    handle.abort();
}

// ===========================================================================
// 13. Waiting-for-input flow
// ===========================================================================

#[tokio::test]
async fn session_waiting_for_input_flow() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let machine_id = register_and_approve_machine(&client, &base, "wfi-m").await;
    let session_id = create_session(&client, &base, &machine_id).await;

    // Get to running state
    let cmds = get_commands(&client, &base, &machine_id).await;
    let start_cmd_id = cmds[0]["command_id"].as_str().unwrap().to_string();
    ack_command(&client, &base, &machine_id, &start_cmd_id).await;
    report_state(&client, &base, &session_id, "running", None).await;

    // Runner reports waiting_for_input
    let s = report_state(&client, &base, &session_id, "waiting_for_input", None).await;
    assert_eq!(s, 200);

    let detail = get_session(&client, &base, &session_id).await;
    assert_eq!(detail["state"], "waiting_for_input");

    // Dashboard sends resume
    let resp = client
        .post(format!("{base}/api/sessions/{session_id}/resume"))
        .send()
        .await
        .expect("resume request failed");
    assert_eq!(resp.status(), 200);

    // Ack resume command
    let cmds = get_commands(&client, &base, &machine_id).await;
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0]["command_type"], "resume");
    let resume_cmd_id = cmds[0]["command_id"].as_str().unwrap().to_string();
    ack_command(&client, &base, &machine_id, &resume_cmd_id).await;

    // Runner reports running again
    let s = report_state(&client, &base, &session_id, "running", None).await;
    assert_eq!(s, 200);

    let detail = get_session(&client, &base, &session_id).await;
    assert_eq!(detail["state"], "running");

    handle.abort();
}

// ===========================================================================
// 14. Invalid state transitions (409)
// ===========================================================================

#[tokio::test]
async fn session_invalid_state_transitions() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let machine_id = register_and_approve_machine(&client, &base, "invalid-t-m").await;
    let session_id = create_session(&client, &base, &machine_id).await;

    // Get to completed state
    let cmds = get_commands(&client, &base, &machine_id).await;
    let start_cmd_id = cmds[0]["command_id"].as_str().unwrap().to_string();
    ack_command(&client, &base, &machine_id, &start_cmd_id).await;
    report_state(&client, &base, &session_id, "running", None).await;
    report_state(&client, &base, &session_id, "completed", None).await;

    let detail = get_session(&client, &base, &session_id).await;
    assert_eq!(detail["state"], "completed");

    // Try to pause completed session — expect 409
    let resp = client
        .post(format!("{base}/api/sessions/{session_id}/pause"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);

    // Try to stop completed session — expect 409
    let resp = client
        .post(format!("{base}/api/sessions/{session_id}/stop"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);

    // Try to resume a running session (create new session to test)
    let session_id2 = create_session(&client, &base, &machine_id).await;
    let cmds = get_commands(&client, &base, &machine_id).await;
    // Find the start_session command for the new session
    let start_cmd_id2 = cmds
        .iter()
        .find(|c| c["command_type"] == "start_session")
        .unwrap()["command_id"]
        .as_str()
        .unwrap()
        .to_string();
    ack_command(&client, &base, &machine_id, &start_cmd_id2).await;
    report_state(&client, &base, &session_id2, "running", None).await;

    // Resume on running session should fail
    let resp = client
        .post(format!("{base}/api/sessions/{session_id2}/resume"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);

    // Force-stop on completed session should fail
    let resp = client
        .post(format!("{base}/api/sessions/{session_id}/force-stop"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);

    handle.abort();
}

// ===========================================================================
// 15. Machine validation for session creation
// ===========================================================================

#[tokio::test]
async fn session_create_rejects_non_trusted_machines() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    // Pending machine — should get 400
    let (pending_id, _) =
        register_machine(&client, &base, "pending-sess", None).await;

    let resp = client
        .post(format!("{base}/api/sessions"))
        .json(&serde_json::json!({
            "machine_id": pending_id,
            "repo_path": "/home/user/repo",
            "title": "test",
            "instructions": "do stuff",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);

    // Revoked machine — approve then revoke, then try to create session
    let (revoked_id, _) =
        register_machine(&client, &base, "revoked-sess", None).await;
    approve_machine(&client, &base, &revoked_id).await;
    client
        .post(format!("{base}/api/machines/{revoked_id}/revoke"))
        .send()
        .await
        .unwrap();

    let resp = client
        .post(format!("{base}/api/sessions"))
        .json(&serde_json::json!({
            "machine_id": revoked_id,
            "repo_path": "/home/user/repo",
            "title": "test",
            "instructions": "do stuff",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);

    // Nonexistent machine — should get 404
    let resp = client
        .post(format!("{base}/api/sessions"))
        .json(&serde_json::json!({
            "machine_id": "nonexistent-machine-id",
            "repo_path": "/home/user/repo",
            "title": "test",
            "instructions": "do stuff",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);

    handle.abort();
}

// ===========================================================================
// 16. Command queue ordering
// ===========================================================================

#[tokio::test]
async fn session_command_queue_ordering() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let machine_id = register_and_approve_machine(&client, &base, "queue-m").await;

    // Create session — this queues a start_session command
    let session_id = create_session(&client, &base, &machine_id).await;

    // Ack start, report running so we can issue stop
    let cmds = get_commands(&client, &base, &machine_id).await;
    assert_eq!(cmds.len(), 1);
    let start_cmd_id = cmds[0]["command_id"].as_str().unwrap().to_string();
    ack_command(&client, &base, &machine_id, &start_cmd_id).await;
    report_state(&client, &base, &session_id, "running", None).await;

    // Now stop the session — queues a stop command
    let resp = client
        .post(format!("{base}/api/sessions/{session_id}/stop"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Create another session — queues another start_session command
    let _session_id2 = create_session(&client, &base, &machine_id).await;

    // Both pending commands should appear in creation order
    let cmds = get_commands(&client, &base, &machine_id).await;
    assert_eq!(cmds.len(), 2);
    assert_eq!(cmds[0]["command_type"], "stop");
    assert_eq!(cmds[1]["command_type"], "start_session");

    handle.abort();
}

// ===========================================================================
// 17. Session list endpoint
// ===========================================================================

#[tokio::test]
async fn session_list_and_filter() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let machine_id = register_and_approve_machine(&client, &base, "list-m").await;

    // Create two sessions
    let sid1 = create_session(&client, &base, &machine_id).await;
    let sid2 = create_session(&client, &base, &machine_id).await;

    // List all sessions
    let resp = client
        .get(format!("{base}/api/sessions"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["total"], 2);
    let sessions = body["sessions"].as_array().unwrap();
    assert_eq!(sessions.len(), 2);

    // Move sid1 to running
    let cmds = get_commands(&client, &base, &machine_id).await;
    // Ack start for sid1
    let cmd_for_sid1 = cmds.iter().find(|c| {
        c["payload"].as_object().map_or(false, |p| p["session_id"].as_str() == Some(&sid1))
    });
    if let Some(cmd) = cmd_for_sid1 {
        let cid = cmd["command_id"].as_str().unwrap();
        ack_command(&client, &base, &machine_id, cid).await;
    }
    report_state(&client, &base, &sid1, "running", None).await;

    // Filter by state=starting (should return only sid2)
    let resp = client
        .get(format!("{base}/api/sessions?state=starting"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["total"], 1);
    let sessions = body["sessions"].as_array().unwrap();
    assert_eq!(sessions[0]["id"], sid2);

    // Filter by state=running (should return only sid1)
    let resp = client
        .get(format!("{base}/api/sessions?state=running"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["total"], 1);
    assert_eq!(body["sessions"][0]["id"], sid1);

    handle.abort();
}

// ===========================================================================
// 18. Stop on waiting_for_input session
// ===========================================================================

#[tokio::test]
async fn session_stop_while_waiting_for_input() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let machine_id = register_and_approve_machine(&client, &base, "stop-wfi-m").await;
    let session_id = create_session(&client, &base, &machine_id).await;

    // Get to waiting_for_input
    let cmds = get_commands(&client, &base, &machine_id).await;
    let start_cmd_id = cmds[0]["command_id"].as_str().unwrap().to_string();
    ack_command(&client, &base, &machine_id, &start_cmd_id).await;
    report_state(&client, &base, &session_id, "running", None).await;
    report_state(&client, &base, &session_id, "waiting_for_input", None).await;

    // Stop should work from waiting_for_input
    let resp = client
        .post(format!("{base}/api/sessions/{session_id}/stop"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Verify stop command queued
    let cmds = get_commands(&client, &base, &machine_id).await;
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0]["command_type"], "stop");

    handle.abort();
}

// ===========================================================================
// 19. Runner state report auth requirement
// ===========================================================================

#[tokio::test]
async fn session_state_report_requires_bearer_token() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let machine_id = register_and_approve_machine(&client, &base, "auth-m").await;
    let session_id = create_session(&client, &base, &machine_id).await;

    // Report state without bearer token — should be 401
    let resp = client
        .post(format!("{base}/api/sessions/{session_id}/state"))
        .json(&serde_json::json!({ "state": "running" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    // Report state with invalid token — should be 401
    let resp = client
        .post(format!("{base}/api/sessions/{session_id}/state"))
        .bearer_auth("bad-token")
        .json(&serde_json::json!({ "state": "running" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    handle.abort();
}

// ===========================================================================
// 20. Force stop on running session queues command
// ===========================================================================

#[tokio::test]
async fn session_force_stop_while_running() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let machine_id = register_and_approve_machine(&client, &base, "fstop-run-m").await;
    let session_id = create_session(&client, &base, &machine_id).await;

    // Get to running
    let cmds = get_commands(&client, &base, &machine_id).await;
    let start_cmd_id = cmds[0]["command_id"].as_str().unwrap().to_string();
    ack_command(&client, &base, &machine_id, &start_cmd_id).await;
    report_state(&client, &base, &session_id, "running", None).await;

    // Force stop on running session — should queue a force_stop command
    let resp = client
        .post(format!("{base}/api/sessions/{session_id}/force-stop"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "force_stop_requested");

    // Verify force_stop command queued
    let cmds = get_commands(&client, &base, &machine_id).await;
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0]["command_type"], "force_stop");

    handle.abort();
}

// ===========================================================================
// Policy helpers
// ===========================================================================

/// GET /api/machines/{id}/policy — returns the JSON body.
async fn get_machine_policy_api(
    client: &Client,
    base: &str,
    machine_id: &str,
) -> serde_json::Value {
    let resp = client
        .get(format!("{base}/api/machines/{machine_id}/policy"))
        .send()
        .await
        .expect("get machine policy request failed");
    assert_eq!(resp.status(), 200);
    resp.json().await.expect("machine policy not json")
}

/// PUT /api/machines/{id}/policy — returns the raw response.
async fn update_machine_policy_api(
    client: &Client,
    base: &str,
    machine_id: &str,
    body: serde_json::Value,
) -> reqwest::Response {
    client
        .put(format!("{base}/api/machines/{machine_id}/policy"))
        .json(&body)
        .send()
        .await
        .expect("update machine policy request failed")
}

/// GET /api/policy-violations with optional query params — returns JSON body.
async fn get_violations(
    client: &Client,
    base: &str,
    params: &[(&str, &str)],
) -> serde_json::Value {
    let resp = client
        .get(format!("{base}/api/policy-violations"))
        .query(params)
        .send()
        .await
        .expect("get violations request failed");
    assert_eq!(resp.status(), 200);
    resp.json().await.expect("violations not json")
}

/// Create a session with a specific autonomy level — returns the raw response.
async fn create_session_with_autonomy(
    client: &Client,
    base: &str,
    machine_id: &str,
    autonomy_level: &str,
) -> reqwest::Response {
    client
        .post(format!("{base}/api/sessions"))
        .json(&serde_json::json!({
            "machine_id": machine_id,
            "repo_path": "/home/user/repo",
            "title": "Test session",
            "instructions": "Do the thing",
            "autonomy_level": autonomy_level,
        }))
        .send()
        .await
        .expect("create session request failed")
}

// ===========================================================================
// 21. Default policy created on approval
// ===========================================================================

#[tokio::test]
async fn policy_default_created_on_approval() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let machine_id = register_and_approve_machine(&client, &base, "policy-default").await;

    let policy = get_machine_policy_api(&client, &base, &machine_id).await;

    // All action categories should be in allowed list
    let allowed = policy["allowed_categories"]
        .as_array()
        .expect("allowed_categories should be an array");
    let allowed_strs: Vec<&str> = allowed.iter().filter_map(|v| v.as_str()).collect();
    assert!(allowed_strs.contains(&"read_files"));
    assert!(allowed_strs.contains(&"write_files"));
    assert!(allowed_strs.contains(&"run_tests"));
    assert!(allowed_strs.contains(&"run_builds"));
    assert!(allowed_strs.contains(&"git_operations"));
    assert!(allowed_strs.contains(&"install_packages"));
    assert!(allowed_strs.contains(&"network_access"));
    assert!(allowed_strs.contains(&"worktree_operations"));
    assert!(allowed_strs.contains(&"shell_execution"));

    // Blocked should be empty
    let blocked = policy["blocked_categories"]
        .as_array()
        .expect("blocked_categories should be an array");
    assert!(blocked.is_empty());

    // Max autonomy = autonomous, session_mode = normal
    assert_eq!(policy["max_autonomy_level"], "autonomous");
    assert_eq!(policy["session_mode"], "normal");

    handle.abort();
}

// ===========================================================================
// 22. Policy blocks high autonomy session
// ===========================================================================

#[tokio::test]
async fn policy_blocks_high_autonomy_session() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let machine_id = register_and_approve_machine(&client, &base, "policy-block").await;

    // Set max autonomy to normal
    let resp = update_machine_policy_api(
        &client,
        &base,
        &machine_id,
        serde_json::json!({ "max_autonomy_level": "normal" }),
    )
    .await;
    assert_eq!(resp.status(), 200);

    // Try to create session with autonomy_level=autonomous — should fail
    let resp = create_session_with_autonomy(&client, &base, &machine_id, "autonomous").await;
    assert_eq!(resp.status(), 400);

    let body: serde_json::Value = resp.json().await.unwrap();
    let error_msg = body["error"].as_str().expect("should have error field");
    assert!(
        error_msg.contains("autonomy") || error_msg.contains("policy"),
        "error message should mention policy or autonomy, got: {error_msg}"
    );

    handle.abort();
}

// ===========================================================================
// 23. Policy allows matching autonomy
// ===========================================================================

#[tokio::test]
async fn policy_allows_matching_autonomy() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let machine_id = register_and_approve_machine(&client, &base, "policy-allow").await;

    // Set max autonomy to normal
    let resp = update_machine_policy_api(
        &client,
        &base,
        &machine_id,
        serde_json::json!({ "max_autonomy_level": "normal" }),
    )
    .await;
    assert_eq!(resp.status(), 200);

    // Create session with autonomy_level=normal — should succeed
    let resp = create_session_with_autonomy(&client, &base, &machine_id, "normal").await;
    assert_eq!(resp.status(), 201);

    handle.abort();
}

// ===========================================================================
// 24. Policy update roundtrip
// ===========================================================================

#[tokio::test]
async fn policy_update_roundtrip() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let machine_id = register_and_approve_machine(&client, &base, "policy-roundtrip").await;

    // Update with blocked categories and session mode
    let resp = update_machine_policy_api(
        &client,
        &base,
        &machine_id,
        serde_json::json!({
            "blocked_categories": ["install_packages", "network_access"],
            "session_mode": "restricted",
        }),
    )
    .await;
    assert_eq!(resp.status(), 200);

    // Read back
    let policy = get_machine_policy_api(&client, &base, &machine_id).await;

    let blocked = policy["blocked_categories"]
        .as_array()
        .expect("blocked_categories should be an array");
    let blocked_strs: Vec<&str> = blocked.iter().filter_map(|v| v.as_str()).collect();
    assert!(blocked_strs.contains(&"install_packages"));
    assert!(blocked_strs.contains(&"network_access"));
    assert_eq!(blocked.len(), 2);

    assert_eq!(policy["session_mode"], "restricted");

    handle.abort();
}

// ===========================================================================
// 25. Effective policy merge
// ===========================================================================

#[tokio::test]
async fn policy_effective_merge() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let machine_id = register_and_approve_machine(&client, &base, "policy-effective").await;

    // Machine policy: allowed=all, blocked=[] (default after approval)
    // No changes needed — default is already permissive

    // Set repo policy for "/project": blocked=["git_operations"]
    let resp = client
        .put(format!(
            "{base}/api/machines/{machine_id}/repo-policy?repo_path=/project"
        ))
        .json(&serde_json::json!({
            "blocked_categories": ["git_operations"],
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Get effective policy for that repo
    let resp = client
        .get(format!(
            "{base}/api/machines/{machine_id}/policy/effective?repo_path=/project"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let effective: serde_json::Value = resp.json().await.unwrap();

    let blocked = effective["blocked_categories"]
        .as_array()
        .expect("blocked_categories should be an array");
    let blocked_strs: Vec<&str> = blocked.iter().filter_map(|v| v.as_str()).collect();
    assert!(
        blocked_strs.contains(&"git_operations"),
        "effective policy should include git_operations in blocked list"
    );

    handle.abort();
}

// ===========================================================================
// 26. Policy violation logged on rejection
// ===========================================================================

#[tokio::test]
async fn policy_violation_logged_on_rejection() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let machine_id =
        register_and_approve_machine(&client, &base, "policy-violation").await;

    // Set max autonomy to normal
    let resp = update_machine_policy_api(
        &client,
        &base,
        &machine_id,
        serde_json::json!({ "max_autonomy_level": "normal" }),
    )
    .await;
    assert_eq!(resp.status(), 200);

    // Attempt to create autonomous session — should fail
    let resp = create_session_with_autonomy(&client, &base, &machine_id, "autonomous").await;
    assert_eq!(resp.status(), 400);

    // Check that a violation was logged
    let violations = get_violations(&client, &base, &[]).await;
    let total = violations["total"].as_i64().expect("total should be a number");
    assert!(total >= 1, "expected at least 1 violation, got {total}");

    let records = violations["violations"]
        .as_array()
        .expect("violations should be an array");
    let v = &records[0];
    assert_eq!(v["machine_id"], machine_id);
    let reason = v["reason"].as_str().unwrap_or("");
    assert!(
        reason.contains("autonomy"),
        "violation reason should mention autonomy, got: {reason}"
    );

    handle.abort();
}

// ===========================================================================
// 27. Policy violations filter by machine
// ===========================================================================

#[tokio::test]
async fn policy_violations_filter_by_machine() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    // Set up two machines, each with restricted autonomy
    let machine_a =
        register_and_approve_machine(&client, &base, "viol-filter-a").await;
    let machine_b =
        register_and_approve_machine(&client, &base, "viol-filter-b").await;

    // Restrict both
    for mid in [&machine_a, &machine_b] {
        let resp = update_machine_policy_api(
            &client,
            &base,
            mid,
            serde_json::json!({ "max_autonomy_level": "normal" }),
        )
        .await;
        assert_eq!(resp.status(), 200);
    }

    // Trigger violations on both
    let resp_a =
        create_session_with_autonomy(&client, &base, &machine_a, "autonomous").await;
    assert_eq!(resp_a.status(), 400);
    let resp_b =
        create_session_with_autonomy(&client, &base, &machine_b, "autonomous").await;
    assert_eq!(resp_b.status(), 400);

    // Filter by machine_a
    let violations =
        get_violations(&client, &base, &[("machine_id", &machine_a)]).await;
    let records = violations["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(
        violations["total"].as_i64().unwrap(),
        1,
        "expected exactly 1 violation for machine_a"
    );
    assert_eq!(records[0]["machine_id"], machine_a);

    // Filter by machine_b
    let violations =
        get_violations(&client, &base, &[("machine_id", &machine_b)]).await;
    let records = violations["violations"]
        .as_array()
        .expect("violations should be an array");
    assert_eq!(
        violations["total"].as_i64().unwrap(),
        1,
        "expected exactly 1 violation for machine_b"
    );
    assert_eq!(records[0]["machine_id"], machine_b);

    handle.abort();
}

// ===========================================================================
// 28. Runner can fetch policy with bearer token
// ===========================================================================

#[tokio::test]
async fn policy_runner_can_fetch_policy() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let machine_id =
        register_and_approve_machine(&client, &base, "policy-runner").await;

    // GET policy with Bearer token (simulating a runner request)
    let resp = client
        .get(format!("{base}/api/machines/{machine_id}/policy"))
        .bearer_auth(VALID_TOKEN)
        .send()
        .await
        .expect("get policy with bearer token failed");
    assert_eq!(resp.status(), 200);

    let policy: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(policy["machine_id"], machine_id);
    assert_eq!(policy["max_autonomy_level"], "autonomous");

    handle.abort();
}

// ===========================================================================
// 29. Repo policy CRUD
// ===========================================================================

#[tokio::test]
async fn repo_policy_crud() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let machine_id = register_and_approve_machine(&client, &base, "repo-crud").await;

    // PUT repo policy for /project — blocked=["shell_execution"]
    let resp = client
        .put(format!(
            "{base}/api/machines/{machine_id}/repo-policy?repo_path=/project"
        ))
        .json(&serde_json::json!({
            "blocked_categories": ["shell_execution"],
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // GET repo policy for /project — verify blocked includes shell_execution
    let resp = client
        .get(format!(
            "{base}/api/machines/{machine_id}/repo-policy?repo_path=/project"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let policy: serde_json::Value = resp.json().await.unwrap();
    let blocked = policy["blocked_categories"]
        .as_array()
        .expect("blocked_categories should be an array");
    let blocked_strs: Vec<&str> = blocked.iter().filter_map(|v| v.as_str()).collect();
    assert!(
        blocked_strs.contains(&"shell_execution"),
        "repo policy should block shell_execution"
    );

    // GET different repo_path — should return empty/default
    let resp = client
        .get(format!(
            "{base}/api/machines/{machine_id}/repo-policy?repo_path=/other-project"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let other_policy: serde_json::Value = resp.json().await.unwrap();
    let other_blocked = other_policy["blocked_categories"]
        .as_array()
        .expect("blocked_categories should be an array");
    assert!(
        other_blocked.is_empty(),
        "different repo path should have empty blocked list"
    );

    handle.abort();
}

// ===========================================================================
// Monitoring & intervention helpers
// ===========================================================================

/// Helper to get a session into "running" state. Returns (machine_id, session_id).
async fn setup_running_session(
    client: &Client,
    base: &str,
    name: &str,
) -> (String, String) {
    let machine_id = register_and_approve_machine(client, base, name).await;
    let session_id = create_session(client, base, &machine_id).await;

    // Ack the start_session command
    let cmds = get_commands(client, base, &machine_id).await;
    let start_cmd_id = cmds[0]["command_id"].as_str().unwrap().to_string();
    ack_command(client, base, &machine_id, &start_cmd_id).await;

    // Report running
    let s = report_state(client, base, &session_id, "running", None).await;
    assert_eq!(s, 200);

    (machine_id, session_id)
}

/// POST /api/sessions/{id}/events — batch event ingestion (needs MachineTokenAuth).
async fn post_events(
    client: &Client,
    base: &str,
    session_id: &str,
    events: serde_json::Value,
) -> reqwest::Response {
    client
        .post(format!("{base}/api/sessions/{session_id}/events"))
        .bearer_auth(VALID_TOKEN)
        .json(&serde_json::json!({ "events": events }))
        .send()
        .await
        .expect("post_events request failed")
}

/// GET /api/sessions/{id}/events — query events (DashboardAuth, no special auth in MVP).
async fn get_events(
    client: &Client,
    base: &str,
    session_id: &str,
    since_sequence: Option<i64>,
) -> serde_json::Value {
    let mut url = format!("{base}/api/sessions/{session_id}/events");
    if let Some(seq) = since_sequence {
        url = format!("{url}?since_sequence={seq}");
    }
    let resp = client.get(&url).send().await.expect("get_events request failed");
    assert_eq!(resp.status(), 200);
    resp.json().await.expect("get_events body not json")
}

/// GET /api/sessions/{id}/events with query params as a raw URL suffix.
async fn get_events_with_params(
    client: &Client,
    base: &str,
    session_id: &str,
    params: &str,
) -> serde_json::Value {
    let url = format!("{base}/api/sessions/{session_id}/events?{params}");
    let resp = client.get(&url).send().await.expect("get_events request failed");
    assert_eq!(resp.status(), 200);
    resp.json().await.expect("get_events body not json")
}

/// GET /api/sessions/{id}/approvals — list pending approvals (DashboardAuth).
async fn get_approvals(
    client: &Client,
    base: &str,
    session_id: &str,
) -> serde_json::Value {
    let resp = client
        .get(format!("{base}/api/sessions/{session_id}/approvals"))
        .send()
        .await
        .expect("get_approvals request failed");
    assert_eq!(resp.status(), 200);
    resp.json().await.expect("get_approvals body not json")
}

/// POST /api/sessions/{id}/respond — respond to an approval (DashboardAuth).
async fn respond_to_approval(
    client: &Client,
    base: &str,
    session_id: &str,
    approval_id: &str,
    choice: &str,
    note: Option<&str>,
) -> reqwest::Response {
    let mut body = serde_json::json!({
        "approval_id": approval_id,
        "choice": choice,
    });
    if let Some(n) = note {
        body["note"] = serde_json::json!(n);
    }
    client
        .post(format!("{base}/api/sessions/{session_id}/respond"))
        .json(&body)
        .send()
        .await
        .expect("respond_to_approval request failed")
}

/// POST /api/sessions/{id}/inject — inject guidance (DashboardAuth).
async fn inject_guidance(
    client: &Client,
    base: &str,
    session_id: &str,
    message: &str,
    injection_type: &str,
) -> reqwest::Response {
    client
        .post(format!("{base}/api/sessions/{session_id}/inject"))
        .json(&serde_json::json!({
            "message": message,
            "injection_type": injection_type,
        }))
        .send()
        .await
        .expect("inject_guidance request failed")
}

// ===========================================================================
// Monitoring integration tests
// ===========================================================================

// ---------------------------------------------------------------------------
// M1. Event ingestion and query
// ---------------------------------------------------------------------------

#[tokio::test]
async fn monitoring_event_ingestion_and_query() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let (_machine_id, session_id) =
        setup_running_session(&client, &base, "mon-ingest-m").await;

    // POST 3 output_line events with different categories
    let events = serde_json::json!([
        { "event_type": "output_line", "category": "info", "content": "Starting build..." },
        { "event_type": "output_line", "category": "warning", "content": "Deprecated API used" },
        { "event_type": "output_line", "category": "error", "content": "Build failed" },
    ]);
    let resp = post_events(&client, &base, &session_id, events).await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["ingested"], 3);

    // GET events — verify 3 events returned in order with correct sequence numbers
    let events = get_events(&client, &base, &session_id, None).await;
    let events_arr = events.as_array().expect("events should be an array");
    assert_eq!(events_arr.len(), 3);

    assert_eq!(events_arr[0]["content"], "Starting build...");
    assert_eq!(events_arr[0]["event_type"], "output_line");
    assert_eq!(events_arr[0]["category"], "info");
    assert_eq!(events_arr[0]["sequence_num"], 1);

    assert_eq!(events_arr[1]["content"], "Deprecated API used");
    assert_eq!(events_arr[1]["category"], "warning");
    assert_eq!(events_arr[1]["sequence_num"], 2);

    assert_eq!(events_arr[2]["content"], "Build failed");
    assert_eq!(events_arr[2]["category"], "error");
    assert_eq!(events_arr[2]["sequence_num"], 3);

    handle.abort();
}

// ---------------------------------------------------------------------------
// M2. Approval auto-created from approval_request event
// ---------------------------------------------------------------------------

#[tokio::test]
async fn monitoring_approval_auto_created() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let (_machine_id, session_id) =
        setup_running_session(&client, &base, "mon-approval-m").await;

    // POST an approval_request event
    let events = serde_json::json!([
        {
            "event_type": "approval_request",
            "content": "May I delete the database?",
            "metadata": {
                "options": ["Allow", "Deny"],
                "context": "rm -rf /var/db"
            }
        }
    ]);
    let resp = post_events(&client, &base, &session_id, events).await;
    assert_eq!(resp.status(), 200);

    // GET approvals — verify one pending approval exists with correct question
    let approvals = get_approvals(&client, &base, &session_id).await;
    let approvals_arr = approvals.as_array().expect("approvals should be an array");
    assert_eq!(approvals_arr.len(), 1);

    assert_eq!(approvals_arr[0]["question"], "May I delete the database?");
    assert_eq!(approvals_arr[0]["status"], "pending");
    assert!(approvals_arr[0]["id"].is_string(), "approval should have an id");

    handle.abort();
}

// ---------------------------------------------------------------------------
// M3. Respond to approval
// ---------------------------------------------------------------------------

#[tokio::test]
async fn monitoring_respond_to_approval() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let (machine_id, session_id) =
        setup_running_session(&client, &base, "mon-respond-m").await;

    // Post approval_request event
    let events = serde_json::json!([
        {
            "event_type": "approval_request",
            "content": "Deploy to production?",
            "metadata": { "options": ["Allow", "Deny"] }
        }
    ]);
    post_events(&client, &base, &session_id, events).await;

    // Get pending approvals to find the approval_id
    let approvals = get_approvals(&client, &base, &session_id).await;
    let approval_id = approvals[0]["id"].as_str().unwrap().to_string();

    // POST respond with choice="Allow", note="Looks good"
    let resp = respond_to_approval(
        &client,
        &base,
        &session_id,
        &approval_id,
        "Allow",
        Some("Looks good"),
    )
    .await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "responded");

    // GET approvals — the endpoint only returns pending approvals,
    // so after responding the list should be empty (status is now 'responded')
    let approvals = get_approvals(&client, &base, &session_id).await;
    let approvals_arr = approvals.as_array().unwrap();
    assert_eq!(
        approvals_arr.len(),
        0,
        "responded approval should no longer appear in pending list"
    );

    // Verify the approval_response event was persisted
    let events = get_events(&client, &base, &session_id, None).await;
    let events_arr = events.as_array().unwrap();
    let response_event = events_arr
        .iter()
        .find(|e| e["event_type"] == "approval_response")
        .expect("approval_response event should exist");
    assert!(
        response_event["content"]
            .as_str()
            .unwrap()
            .contains("Allow"),
        "response event content should mention the choice"
    );

    // GET commands — verify a "respond" command was queued
    let cmds = get_commands(&client, &base, &machine_id).await;
    let respond_cmd = cmds
        .iter()
        .find(|c| c["command_type"] == "respond")
        .expect("respond command should be queued");
    // payload is already deserialized as JSON by the API
    let payload = &respond_cmd["payload"];
    assert_eq!(payload["approval_id"], approval_id);
    assert_eq!(payload["choice"], "Allow");

    handle.abort();
}

// ---------------------------------------------------------------------------
// M4. Respond to already-responded approval
// ---------------------------------------------------------------------------

#[tokio::test]
async fn monitoring_respond_already_responded() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let (_machine_id, session_id) =
        setup_running_session(&client, &base, "mon-double-resp-m").await;

    // Post approval_request event and respond to it
    let events = serde_json::json!([
        {
            "event_type": "approval_request",
            "content": "Run tests?",
            "metadata": { "options": ["Allow", "Deny"] }
        }
    ]);
    post_events(&client, &base, &session_id, events).await;

    let approvals = get_approvals(&client, &base, &session_id).await;
    let approval_id = approvals[0]["id"].as_str().unwrap().to_string();

    // First response — should succeed
    let resp = respond_to_approval(
        &client,
        &base,
        &session_id,
        &approval_id,
        "Allow",
        None,
    )
    .await;
    assert_eq!(resp.status(), 200);

    // Second response to same approval — should get 400
    let resp = respond_to_approval(
        &client,
        &base,
        &session_id,
        &approval_id,
        "Deny",
        Some("Changed my mind"),
    )
    .await;
    assert_eq!(resp.status(), 400);

    handle.abort();
}

// ---------------------------------------------------------------------------
// M5. Inject guidance
// ---------------------------------------------------------------------------

#[tokio::test]
async fn monitoring_inject_guidance() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let (machine_id, session_id) =
        setup_running_session(&client, &base, "mon-inject-m").await;

    // POST inject with message and injection_type
    let resp =
        inject_guidance(&client, &base, &session_id, "Focus on tests", "normal").await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "injected");

    // GET commands — verify "inject" command queued with correct payload
    let cmds = get_commands(&client, &base, &machine_id).await;
    let inject_cmd = cmds
        .iter()
        .find(|c| c["command_type"] == "inject")
        .expect("inject command should be queued");
    // payload is already deserialized as JSON by the API
    let payload = &inject_cmd["payload"];
    assert_eq!(payload["message"], "Focus on tests");
    assert_eq!(payload["injection_type"], "normal");

    // GET events — verify guidance_injected event persisted
    let events = get_events(&client, &base, &session_id, None).await;
    let events_arr = events.as_array().unwrap();
    let guidance_event = events_arr
        .iter()
        .find(|e| e["event_type"] == "guidance_injected")
        .expect("guidance_injected event should exist");
    assert_eq!(guidance_event["content"], "Focus on tests");

    handle.abort();
}

// ---------------------------------------------------------------------------
// M6. Inject guidance on terminal session rejected
// ---------------------------------------------------------------------------

#[tokio::test]
async fn monitoring_inject_terminal_session_rejected() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let (_machine_id, session_id) =
        setup_running_session(&client, &base, "mon-inject-term-m").await;

    // Move session to completed
    let s = report_state(&client, &base, &session_id, "completed", None).await;
    assert_eq!(s, 200);

    // POST inject — expect 409 Conflict
    let resp =
        inject_guidance(&client, &base, &session_id, "Too late", "normal").await;
    assert_eq!(resp.status(), 409);

    handle.abort();
}

// ---------------------------------------------------------------------------
// M7. Event query with since_sequence filter
// ---------------------------------------------------------------------------

#[tokio::test]
async fn monitoring_event_query_since_sequence() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let (_machine_id, session_id) =
        setup_running_session(&client, &base, "mon-since-m").await;

    // Post 5 events
    let events = serde_json::json!([
        { "event_type": "output_line", "content": "line 1" },
        { "event_type": "output_line", "content": "line 2" },
        { "event_type": "output_line", "content": "line 3" },
        { "event_type": "output_line", "content": "line 4" },
        { "event_type": "output_line", "content": "line 5" },
    ]);
    let resp = post_events(&client, &base, &session_id, events).await;
    assert_eq!(resp.status(), 200);

    // GET events?since_sequence=3 — should return only events 4 and 5
    let events = get_events(&client, &base, &session_id, Some(3)).await;
    let events_arr = events.as_array().expect("events should be an array");
    assert_eq!(events_arr.len(), 2);
    assert_eq!(events_arr[0]["content"], "line 4");
    assert_eq!(events_arr[0]["sequence_num"], 4);
    assert_eq!(events_arr[1]["content"], "line 5");
    assert_eq!(events_arr[1]["sequence_num"], 5);

    handle.abort();
}

// ---------------------------------------------------------------------------
// M8. Event query with type filter
// ---------------------------------------------------------------------------

#[tokio::test]
async fn monitoring_event_query_with_type_filter() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let (_machine_id, session_id) =
        setup_running_session(&client, &base, "mon-filter-m").await;

    // Post a mix of output_line and approval_request events
    let events = serde_json::json!([
        { "event_type": "output_line", "content": "line A" },
        {
            "event_type": "approval_request",
            "content": "May I proceed?",
            "metadata": { "options": ["Yes", "No"] }
        },
        { "event_type": "output_line", "content": "line B" },
        {
            "event_type": "approval_request",
            "content": "Delete file?",
            "metadata": { "options": ["Allow", "Deny"] }
        },
    ]);
    let resp = post_events(&client, &base, &session_id, events).await;
    assert_eq!(resp.status(), 200);

    // GET events?event_type=approval_request — verify only approval events returned
    let events = get_events_with_params(
        &client,
        &base,
        &session_id,
        "event_type=approval_request",
    )
    .await;
    let events_arr = events.as_array().expect("events should be an array");
    assert_eq!(events_arr.len(), 2);
    assert_eq!(events_arr[0]["event_type"], "approval_request");
    assert_eq!(events_arr[0]["content"], "May I proceed?");
    assert_eq!(events_arr[1]["event_type"], "approval_request");
    assert_eq!(events_arr[1]["content"], "Delete file?");

    handle.abort();
}

// ---------------------------------------------------------------------------
// M9. Session ownership validation (documents scoping behavior)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn monitoring_session_ownership_validation() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    let (_machine_id, session_id) =
        setup_running_session(&client, &base, "mon-owner-m").await;

    // Post events — should work since MVP auth always returns "user-default"
    // which matches the session creator
    let events = serde_json::json!([
        { "event_type": "output_line", "content": "owned event" },
    ]);
    let resp = post_events(&client, &base, &session_id, events).await;
    assert_eq!(resp.status(), 200);

    // Query events — should also work for the same reason
    let events = get_events(&client, &base, &session_id, None).await;
    let events_arr = events.as_array().unwrap();
    assert_eq!(events_arr.len(), 1);
    assert_eq!(events_arr[0]["content"], "owned event");

    // Get approvals — should work (empty list, no approvals posted)
    let approvals = get_approvals(&client, &base, &session_id).await;
    let approvals_arr = approvals.as_array().unwrap();
    assert_eq!(approvals_arr.len(), 0);

    // Inject guidance — should work on running session
    let resp =
        inject_guidance(&client, &base, &session_id, "Some guidance", "normal").await;
    assert_eq!(resp.status(), 200);

    // NOTE: In a real multi-user system, querying events for a session owned
    // by a different user should return 404 (session not found for this user).
    // The MVP DashboardAuth stub always returns "user-default", so all
    // dashboard requests succeed for sessions created by "user-default".

    handle.abort();
}

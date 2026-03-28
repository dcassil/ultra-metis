//! Integration tests for the Cadre Control API.
//!
//! Each test spins up an in-process Axum server on a random port with its own
//! temporary SQLite database for full isolation.

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

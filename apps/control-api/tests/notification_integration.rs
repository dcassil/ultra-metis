//! Integration tests for notification generation and management.
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
// Test helpers
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
        log_channels: Arc::new(Mutex::new(HashMap::new())),
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
// Test 1: Notification created on approval request
// ===========================================================================

#[tokio::test]
async fn notification_created_on_approval_request() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    // Register + approve machine
    let machine_id = register_and_approve(&client, &base, "notif-m1").await;

    // Create session
    let session_id =
        create_session_with_title(&client, &base, &machine_id, "Approval notification test")
            .await;

    // Ack the start command
    ack_start_command(&client, &base, &machine_id).await;

    // Report state as running
    let s = report_state(&client, &base, &session_id, "running").await;
    assert_eq!(s, 200);

    // Ingest an approval_request event
    let s = ingest_events(
        &client,
        &base,
        &session_id,
        serde_json::json!([
            {"event_type": "approval_request", "content": "Allow file write?", "category": "info"}
        ]),
    )
    .await;
    assert_eq!(s, 200);

    // GET /api/notifications -> verify at least 1 notification with
    // notification_type="approval_request" and priority="urgent"
    let resp = client
        .get(format!("{base}/api/notifications"))
        .send()
        .await
        .expect("list notifications request failed");
    assert_eq!(resp.status(), 200);

    let notifications: Vec<serde_json::Value> = resp.json().await.expect("notifications not json");
    assert!(
        !notifications.is_empty(),
        "expected at least 1 notification after approval_request event"
    );

    let approval_notif = notifications
        .iter()
        .find(|n| n["notification_type"] == "approval_request")
        .expect("no notification with notification_type=approval_request found");

    assert_eq!(
        approval_notif["priority"], "urgent",
        "approval_request notification should have priority=urgent"
    );
    assert_eq!(
        approval_notif["session_id"],
        session_id,
        "notification should reference the correct session_id"
    );

    handle.abort();
}

// ===========================================================================
// Test 2: Notification created on session completion
// ===========================================================================

#[tokio::test]
async fn notification_created_on_session_completion() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    // Register + approve machine
    let machine_id = register_and_approve(&client, &base, "notif-m2").await;

    // Create session
    let session_id =
        create_session_with_title(&client, &base, &machine_id, "Completion notification test")
            .await;

    // Ack the start command
    ack_start_command(&client, &base, &machine_id).await;

    // Report state as running
    let s = report_state(&client, &base, &session_id, "running").await;
    assert_eq!(s, 200);

    // Report state as completed
    let s = report_state(&client, &base, &session_id, "completed").await;
    assert_eq!(s, 200);

    // GET /api/notifications -> verify notification with
    // notification_type="session_completed" and priority="normal"
    let resp = client
        .get(format!("{base}/api/notifications"))
        .send()
        .await
        .expect("list notifications request failed");
    assert_eq!(resp.status(), 200);

    let notifications: Vec<serde_json::Value> = resp.json().await.expect("notifications not json");
    assert!(
        !notifications.is_empty(),
        "expected at least 1 notification after session completion"
    );

    let completion_notif = notifications
        .iter()
        .find(|n| n["notification_type"] == "session_completed")
        .expect("no notification with notification_type=session_completed found");

    assert_eq!(
        completion_notif["priority"], "normal",
        "session_completed notification should have priority=normal"
    );
    assert_eq!(
        completion_notif["session_id"],
        session_id,
        "notification should reference the correct session_id"
    );

    handle.abort();
}

// ===========================================================================
// Test 3: Unread count tracks reads correctly
// ===========================================================================

#[tokio::test]
async fn unread_count_tracks_reads() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    // Register + approve machine
    let machine_id = register_and_approve(&client, &base, "notif-m3").await;

    // Create session and generate an approval_request notification
    let session_id =
        create_session_with_title(&client, &base, &machine_id, "Unread count test").await;
    ack_start_command(&client, &base, &machine_id).await;
    let s = report_state(&client, &base, &session_id, "running").await;
    assert_eq!(s, 200);

    let s = ingest_events(
        &client,
        &base,
        &session_id,
        serde_json::json!([
            {"event_type": "approval_request", "content": "Allow network access?", "category": "info"}
        ]),
    )
    .await;
    assert_eq!(s, 200);

    // GET /api/notifications/unread-count -> verify count > 0
    let resp = client
        .get(format!("{base}/api/notifications/unread-count"))
        .send()
        .await
        .expect("unread count request failed");
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("unread count not json");
    let initial_count = body["count"].as_i64().expect("count should be an integer");
    assert!(
        initial_count > 0,
        "expected unread count > 0, got {initial_count}"
    );

    // Get the notification id
    let resp = client
        .get(format!("{base}/api/notifications"))
        .send()
        .await
        .expect("list notifications request failed");
    assert_eq!(resp.status(), 200);

    let notifications: Vec<serde_json::Value> = resp.json().await.expect("notifications not json");
    assert!(
        !notifications.is_empty(),
        "expected at least 1 notification"
    );
    let notif_id = notifications[0]["id"]
        .as_str()
        .expect("notification should have an id");

    // POST /api/notifications/{id}/read -> mark it read
    let resp = client
        .post(format!("{base}/api/notifications/{notif_id}/read"))
        .send()
        .await
        .expect("mark read request failed");
    assert_eq!(resp.status(), 200);

    // GET /api/notifications/unread-count -> verify count decreased by 1
    let resp = client
        .get(format!("{base}/api/notifications/unread-count"))
        .send()
        .await
        .expect("unread count request failed");
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("unread count not json");
    let new_count = body["count"].as_i64().expect("count should be an integer");
    assert_eq!(
        new_count,
        initial_count - 1,
        "unread count should decrease by 1 after marking read"
    );

    handle.abort();
}

// ===========================================================================
// Test 4: Mark read and dismiss
// ===========================================================================

#[tokio::test]
async fn mark_read_and_dismiss() {
    let (base, handle) = start_test_server().await;
    let client = Client::new();

    // Register + approve machine
    let machine_id = register_and_approve(&client, &base, "notif-m4").await;

    // Create session and generate a notification via approval_request
    let session_id =
        create_session_with_title(&client, &base, &machine_id, "Read and dismiss test").await;
    ack_start_command(&client, &base, &machine_id).await;
    let s = report_state(&client, &base, &session_id, "running").await;
    assert_eq!(s, 200);

    let s = ingest_events(
        &client,
        &base,
        &session_id,
        serde_json::json!([
            {"event_type": "approval_request", "content": "Allow disk write?", "category": "info"}
        ]),
    )
    .await;
    assert_eq!(s, 200);

    // GET /api/notifications -> get the notification id
    let resp = client
        .get(format!("{base}/api/notifications"))
        .send()
        .await
        .expect("list notifications request failed");
    assert_eq!(resp.status(), 200);

    let notifications: Vec<serde_json::Value> = resp.json().await.expect("notifications not json");
    assert!(
        !notifications.is_empty(),
        "expected at least 1 notification"
    );
    let notif_id = notifications[0]["id"]
        .as_str()
        .expect("notification should have an id")
        .to_string();

    // Verify read_at is initially null
    assert!(
        notifications[0]["read_at"].is_null(),
        "read_at should be null before marking read"
    );

    // POST /api/notifications/{id}/read -> verify 200
    let resp = client
        .post(format!("{base}/api/notifications/{notif_id}/read"))
        .send()
        .await
        .expect("mark read request failed");
    assert_eq!(resp.status(), 200);

    // GET /api/notifications -> verify read_at is no longer null
    let resp = client
        .get(format!("{base}/api/notifications"))
        .send()
        .await
        .expect("list notifications request failed");
    assert_eq!(resp.status(), 200);

    let notifications: Vec<serde_json::Value> = resp.json().await.expect("notifications not json");
    let notif = notifications
        .iter()
        .find(|n| n["id"] == notif_id)
        .expect("notification should still exist after marking read");
    assert!(
        !notif["read_at"].is_null(),
        "read_at should not be null after marking read"
    );

    // POST /api/notifications/{id}/dismiss -> verify 200
    let resp = client
        .post(format!("{base}/api/notifications/{notif_id}/dismiss"))
        .send()
        .await
        .expect("dismiss request failed");
    assert_eq!(resp.status(), 200);

    // Verify dismissed_at is set
    let resp = client
        .get(format!("{base}/api/notifications"))
        .send()
        .await
        .expect("list notifications request failed");
    assert_eq!(resp.status(), 200);

    let notifications: Vec<serde_json::Value> = resp.json().await.expect("notifications not json");
    let notif = notifications
        .iter()
        .find(|n| n["id"] == notif_id)
        .expect("notification should still exist after dismissing");
    assert!(
        !notif["dismissed_at"].is_null(),
        "dismissed_at should not be null after dismissing"
    );

    handle.abort();
}

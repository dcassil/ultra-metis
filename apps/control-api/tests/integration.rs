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

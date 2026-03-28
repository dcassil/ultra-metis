//! Integration tests for the planning data endpoints.
//!
//! Creates a temporary .metis project with sample documents and verifies
//! all `/api/planning/*` endpoints return correct data.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use cadre_control_api::db;
use cadre_control_api::{build_app_with_planning, init_planning_state, AppState};
use cadre_store::DocumentStore;
use reqwest::Client;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Test helper
// ---------------------------------------------------------------------------

/// Spin up a test server with a real .metis project for planning tests.
async fn start_planning_test_server(
    project_dir: &Path,
) -> (String, tokio::task::JoinHandle<()>) {
    let tmp_db = tempfile::NamedTempFile::new().expect("failed to create temp db file");
    let db_path = tmp_db.path().to_str().expect("non-utf8 path").to_string();
    std::mem::forget(tmp_db);

    let conn = rusqlite::Connection::open(&db_path).expect("failed to open temp db");
    db::init_db(&conn).expect("failed to init db");

    let state = AppState {
        db: Arc::new(Mutex::new(conn)),
        event_channels: Arc::new(Mutex::new(HashMap::new())),
        log_channels: Arc::new(Mutex::new(HashMap::new())),
    };

    let planning_state = init_planning_state(Some(project_dir.to_path_buf()));

    let app = build_app_with_planning(state, planning_state);
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

/// Create a temporary .metis project with sample documents.
fn create_test_project() -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    let store = DocumentStore::new(dir.path());
    store
        .initialize("TEST")
        .expect("failed to initialize project");

    // Create a vision document
    store
        .create_document("vision", "Test Vision", None)
        .expect("failed to create vision");

    // Create an initiative under the vision
    // First we need to find the vision short code
    let docs = store.list_documents(false).expect("failed to list docs");
    let vision = docs
        .iter()
        .find(|d| d.document_type == "vision")
        .expect("no vision found");

    // Transition vision to published so it can be a parent
    store
        .transition_phase(&vision.short_code, Some("review"))
        .expect("failed to transition vision to review");
    store
        .transition_phase(&vision.short_code, Some("published"))
        .expect("failed to transition vision to published");

    // Create an initiative
    store
        .create_document(
            "initiative",
            "Test Initiative",
            Some(&vision.short_code),
        )
        .expect("failed to create initiative");

    // Create a task under the initiative
    let docs = store.list_documents(false).expect("failed to list docs");
    let initiative = docs
        .iter()
        .find(|d| d.document_type == "initiative")
        .expect("no initiative found");

    // Transition initiative through phases to decompose
    store
        .transition_phase(&initiative.short_code, Some("design"))
        .expect("transition to design");
    store
        .transition_phase(&initiative.short_code, Some("ready"))
        .expect("transition to ready");
    store
        .transition_phase(&initiative.short_code, Some("decompose"))
        .expect("transition to decompose");

    store
        .create_document(
            "task",
            "Test Task One",
            Some(&initiative.short_code),
        )
        .expect("failed to create task");

    dir
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_documents() {
    let project_dir = create_test_project();
    let (base, _handle) = start_planning_test_server(project_dir.path()).await;
    let client = Client::new();

    let resp = client
        .get(format!("{base}/api/planning/documents"))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 200);
    let docs: Vec<serde_json::Value> = resp.json().await.expect("not json");
    assert!(
        docs.len() >= 3,
        "expected at least 3 docs (vision, initiative, task), got {}",
        docs.len()
    );
}

#[tokio::test]
async fn test_list_documents_with_type_filter() {
    let project_dir = create_test_project();
    let (base, _handle) = start_planning_test_server(project_dir.path()).await;
    let client = Client::new();

    let resp = client
        .get(format!("{base}/api/planning/documents?document_type=task"))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 200);
    let docs: Vec<serde_json::Value> = resp.json().await.expect("not json");
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0]["document_type"], "task");
}

#[tokio::test]
async fn test_get_document_detail() {
    let project_dir = create_test_project();
    let (base, _handle) = start_planning_test_server(project_dir.path()).await;
    let client = Client::new();

    // First list to find a short code
    let resp = client
        .get(format!(
            "{base}/api/planning/documents?document_type=vision"
        ))
        .send()
        .await
        .expect("request failed");
    let docs: Vec<serde_json::Value> = resp.json().await.expect("not json");
    let short_code = docs[0]["short_code"].as_str().expect("no short_code");

    // Get detail
    let resp = client
        .get(format!("{base}/api/planning/documents/{short_code}"))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 200);
    let detail: serde_json::Value = resp.json().await.expect("not json");
    assert_eq!(detail["short_code"], short_code);
    assert!(detail["content"].as_str().is_some());
    assert_eq!(detail["document_type"], "vision");
}

#[tokio::test]
async fn test_get_document_not_found() {
    let project_dir = create_test_project();
    let (base, _handle) = start_planning_test_server(project_dir.path()).await;
    let client = Client::new();

    let resp = client
        .get(format!("{base}/api/planning/documents/NONEXISTENT-V-9999"))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_search_documents() {
    let project_dir = create_test_project();
    let (base, _handle) = start_planning_test_server(project_dir.path()).await;
    let client = Client::new();

    let resp = client
        .get(format!("{base}/api/planning/documents/search?q=Test"))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 200);
    let docs: Vec<serde_json::Value> = resp.json().await.expect("not json");
    assert!(!docs.is_empty(), "search should find documents containing 'Test'");
}

#[tokio::test]
async fn test_hierarchy() {
    let project_dir = create_test_project();
    let (base, _handle) = start_planning_test_server(project_dir.path()).await;
    let client = Client::new();

    let resp = client
        .get(format!("{base}/api/planning/hierarchy"))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 200);
    let tree: Vec<serde_json::Value> = resp.json().await.expect("not json");
    // Should have at least one root node (the vision)
    assert!(!tree.is_empty(), "hierarchy should have root nodes");

    // Find the vision node and check it has children
    let vision_node = tree
        .iter()
        .find(|n| n["document_type"] == "vision")
        .expect("no vision in tree");
    let children = vision_node["children"].as_array().expect("no children array");
    assert!(
        !children.is_empty(),
        "vision should have initiative children"
    );
}

#[tokio::test]
async fn test_rules_endpoint() {
    let project_dir = create_test_project();
    let (base, _handle) = start_planning_test_server(project_dir.path()).await;
    let client = Client::new();

    let resp = client
        .get(format!("{base}/api/planning/rules"))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 200);
    // May be empty if no rules_config docs exist, but should return 200
    let _rules: Vec<serde_json::Value> = resp.json().await.expect("not json");
}

#[tokio::test]
async fn test_quality_endpoint() {
    let project_dir = create_test_project();
    let (base, _handle) = start_planning_test_server(project_dir.path()).await;
    let client = Client::new();

    // First list to find a short code
    let resp = client
        .get(format!(
            "{base}/api/planning/documents?document_type=vision"
        ))
        .send()
        .await
        .expect("request failed");
    let docs: Vec<serde_json::Value> = resp.json().await.expect("not json");
    let short_code = docs[0]["short_code"].as_str().expect("no short_code");

    let resp = client
        .get(format!("{base}/api/planning/quality/{short_code}"))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 200);
    // May be empty if no quality records exist
    let _quality: Vec<serde_json::Value> = resp.json().await.expect("not json");
}

#[tokio::test]
async fn test_quality_not_found() {
    let project_dir = create_test_project();
    let (base, _handle) = start_planning_test_server(project_dir.path()).await;
    let client = Client::new();

    let resp = client
        .get(format!(
            "{base}/api/planning/quality/NONEXISTENT-V-9999"
        ))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_document_children_in_detail() {
    let project_dir = create_test_project();
    let (base, _handle) = start_planning_test_server(project_dir.path()).await;
    let client = Client::new();

    // Find the initiative
    let resp = client
        .get(format!(
            "{base}/api/planning/documents?document_type=initiative"
        ))
        .send()
        .await
        .expect("request failed");
    let docs: Vec<serde_json::Value> = resp.json().await.expect("not json");
    let short_code = docs[0]["short_code"].as_str().expect("no short_code");

    // Get detail — should have children (the task)
    let resp = client
        .get(format!("{base}/api/planning/documents/{short_code}"))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 200);
    let detail: serde_json::Value = resp.json().await.expect("not json");
    let children = detail["children"].as_array().expect("no children");
    assert_eq!(children.len(), 1, "initiative should have 1 task child");
    assert_eq!(children[0]["document_type"], "task");
}

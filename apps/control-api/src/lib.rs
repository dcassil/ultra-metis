//! Cadre Control API — multi-tenancy, machine registry, and fleet management.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::routing::{get, post};
use axum::Router;
use cadre_store::DocumentStore;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

pub mod auth;
pub mod db;
pub mod models;
pub mod planning;
pub mod routes;

/// Shared application state passed to all route handlers.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<rusqlite::Connection>>,
    /// Per-session broadcast channels for live event streaming (SSE).
    /// Key is session_id, value is broadcast sender.
    pub event_channels: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
}

/// State for planning data endpoints (backed by cadre-store).
#[derive(Clone)]
pub struct PlanningState {
    pub store: Option<Arc<DocumentStore>>,
}

/// Initialize planning state from an optional project path.
///
/// If `project_path` is `None`, reads from `CADRE_PROJECT_PATH` env var.
/// If neither is set, planning endpoints return 503.
pub fn init_planning_state(project_path: Option<PathBuf>) -> PlanningState {
    let path = project_path.or_else(|| std::env::var("CADRE_PROJECT_PATH").ok().map(PathBuf::from));

    let store = path.map(|p| Arc::new(DocumentStore::new(&p)));

    PlanningState { store }
}

/// Build the Axum router with all routes, CORS, and shared state.
///
/// Used by both `main.rs` and integration tests.
pub fn build_app(state: AppState) -> Router {
    build_app_with_planning(state, init_planning_state(None))
}

/// Build the Axum router with explicit planning state (for tests).
pub fn build_app_with_planning(state: AppState, planning_state: PlanningState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let machine_routes = Router::new()
        .route("/register", post(routes::register_machine))
        .route("/{id}", get(routes::get_machine))
        .route("/{id}/heartbeat", post(routes::heartbeat))
        .route("/{id}/approve", post(routes::approve_machine))
        .route("/{id}/revoke", post(routes::revoke_machine))
        .route("/{id}/commands", get(routes::get_machine_commands))
        .route(
            "/{id}/commands/{cmd_id}/ack",
            post(routes::ack_command),
        )
        .route(
            "/{id}/policy",
            get(routes::get_machine_policy).put(routes::update_machine_policy),
        )
        .route("/{id}/policy/effective", get(routes::get_effective_policy))
        .route(
            "/{id}/repo-policy",
            get(routes::get_repo_policy).put(routes::update_repo_policy),
        );

    let session_routes = Router::new()
        .route("/{id}", get(routes::get_session))
        .route("/{id}/stop", post(routes::stop_session))
        .route("/{id}/force-stop", post(routes::force_stop_session))
        .route("/{id}/pause", post(routes::pause_session))
        .route("/{id}/resume", post(routes::resume_session))
        .route("/{id}/state", post(routes::report_session_state))
        .route("/{id}/violations", get(routes::list_session_violations))
        .route(
            "/{id}/events",
            post(routes::ingest_events).get(routes::query_events),
        )
        .route("/{id}/events/stream", get(routes::event_stream))
        .route("/{id}/approvals", get(routes::list_pending_approvals))
        .route("/{id}/respond", post(routes::respond_to_approval))
        .route("/{id}/inject", post(routes::inject_guidance))
        .route("/{id}/outcome", get(routes::get_session_outcome));

    let notification_routes = Router::new()
        .route("/{id}/read", post(routes::mark_notification_read))
        .route("/{id}/dismiss", post(routes::dismiss_notification));

    let planning_routes = Router::new()
        .route("/documents", get(planning::list_documents))
        .route("/documents/search", get(planning::search_documents))
        .route("/documents/{short_code}", get(planning::get_document))
        .route("/hierarchy", get(planning::get_hierarchy))
        .route("/rules", get(planning::list_rules))
        .route("/quality/{short_code}", get(planning::get_quality))
        .with_state(planning_state);

    Router::new()
        .route("/health", get(routes::health))
        .route("/api/machines", get(routes::list_machines))
        .nest("/api/machines", machine_routes)
        .route("/api/sessions", get(routes::list_sessions).post(routes::create_session))
        .nest("/api/sessions", session_routes)
        .route("/api/notifications", get(routes::list_notifications))
        .route("/api/notifications/unread-count", get(routes::unread_count))
        .nest("/api/notifications", notification_routes)
        .route("/api/devices", post(routes::register_device))
        .route("/api/policy-violations", get(routes::list_policy_violations))
        .nest("/api/planning", planning_routes)
        .layer(cors)
        .with_state(state)
}

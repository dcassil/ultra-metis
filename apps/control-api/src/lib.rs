//! Cadre Control API — multi-tenancy, machine registry, and fleet management.

use std::sync::{Arc, Mutex};

use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};

pub mod auth;
pub mod db;
pub mod models;
pub mod routes;

/// Shared application state passed to all route handlers.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<rusqlite::Connection>>,
}

/// Build the Axum router with all routes, CORS, and shared state.
///
/// Used by both `main.rs` and integration tests.
pub fn build_app(state: AppState) -> Router {
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
        );

    let session_routes = Router::new()
        .route("/{id}", get(routes::get_session))
        .route("/{id}/stop", post(routes::stop_session))
        .route("/{id}/force-stop", post(routes::force_stop_session))
        .route("/{id}/pause", post(routes::pause_session))
        .route("/{id}/resume", post(routes::resume_session))
        .route("/{id}/state", post(routes::report_session_state));

    Router::new()
        .route("/health", get(routes::health))
        .route("/api/machines", get(routes::list_machines))
        .nest("/api/machines", machine_routes)
        .route("/api/sessions", get(routes::list_sessions).post(routes::create_session))
        .nest("/api/sessions", session_routes)
        .layer(cors)
        .with_state(state)
}

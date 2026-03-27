//! Cadre Control API server entry point.

use std::sync::{Arc, Mutex};

use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

use cadre_control_api::db;
use cadre_control_api::routes;
use cadre_control_api::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("initializing database");
    let conn = rusqlite::Connection::open("cadre-control.db")?;
    db::init_db(&conn)?;

    let state = AppState {
        db: Arc::new(Mutex::new(conn)),
    };

    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let machine_routes = Router::new()
        .route("/register", post(routes::register_machine))
        .route("/{id}", get(routes::get_machine))
        .route("/{id}/heartbeat", post(routes::heartbeat))
        .route("/{id}/approve", post(routes::approve_machine))
        .route("/{id}/revoke", post(routes::revoke_machine));

    Router::new()
        .route("/health", get(routes::health))
        .route("/api/machines", get(routes::list_machines))
        .nest("/api/machines", machine_routes)
        .layer(cors)
        .with_state(state)
}

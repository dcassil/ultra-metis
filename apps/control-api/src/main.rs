//! Cadre Control API server entry point.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tracing_subscriber::EnvFilter;

use cadre_control_api::db;
use cadre_control_api::{build_app, AppState};

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
        event_channels: Arc::new(Mutex::new(HashMap::new())),
        log_channels: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = build_app(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

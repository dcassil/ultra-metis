//! Cadre Control API — multi-tenancy, machine registry, and fleet management.

use std::sync::{Arc, Mutex};

pub mod auth;
pub mod db;
pub mod models;
pub mod routes;

/// Shared application state passed to all route handlers.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<rusqlite::Connection>>,
}

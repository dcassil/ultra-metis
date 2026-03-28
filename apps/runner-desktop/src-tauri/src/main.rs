#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod tray;

use std::sync::Arc;

use cadre_machine_runner::{RunnerHandle, Settings};
use tokio::sync::RwLock;

pub struct AppState {
    pub runner: Arc<RwLock<Option<RunnerHandle>>>,
    pub settings: Arc<RwLock<Settings>>,
    pub token: Arc<RwLock<String>>,
}

fn main() {
    let settings = Settings::default();
    let app_state = AppState {
        runner: Arc::new(RwLock::new(None)),
        settings: Arc::new(RwLock::new(settings)),
        token: Arc::new(RwLock::new(String::new())),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .setup(|app| {
            tray::create_tray(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::start_runner,
            commands::stop_runner,
            commands::toggle_enabled,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

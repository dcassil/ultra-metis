#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod tray;

use std::sync::Arc;

use cadre_machine_runner::{RunnerHandle, RunnerStatus, Settings};
use tauri::Manager;
use tauri_plugin_notification::NotificationExt;
use tokio::sync::RwLock;

pub struct AppState {
    pub runner: Arc<RwLock<Option<RunnerHandle>>>,
    pub settings: Arc<RwLock<Settings>>,
    pub token: Arc<RwLock<String>>,
}

fn main() {
    let minimized = std::env::args().any(|a| a == "--minimized");

    let settings = Settings::default();
    let app_state = AppState {
        runner: Arc::new(RwLock::new(None)),
        settings: Arc::new(RwLock::new(settings)),
        token: Arc::new(RwLock::new(String::new())),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(app_state)
        .setup(move |app| {
            tray::create_tray(app)?;

            // If not launched with --minimized, open the settings window.
            if !minimized {
                let _window = tauri::WebviewWindowBuilder::new(
                    app,
                    "main",
                    tauri::WebviewUrl::App("index.html".into()),
                )
                .title("Cadre Machine Runner — Settings")
                .inner_size(600.0, 500.0)
                .build()?;
            }

            // Spawn a background watcher that sends desktop notifications on
            // runner status changes.
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                status_notification_watcher(app_handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::start_runner,
            commands::stop_runner,
            commands::toggle_enabled,
            commands::get_settings,
            commands::save_settings,
            commands::get_token,
            commands::set_token,
            commands::test_connection,
            commands::is_first_run,
            commands::set_auto_start,
            commands::get_auto_start,
            commands::send_notification,
            commands::check_for_updates,
            commands::uninstall,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Watches for runner status changes and fires desktop notifications.
///
/// Polls the runner handle every 2 seconds (there is no global subscription
/// channel since the runner may be started/stopped dynamically). When the
/// runner is present and has a `subscribe_status()` watch, we listen on it
/// directly; otherwise we fall back to polling.
async fn status_notification_watcher(app_handle: tauri::AppHandle) {
    use std::mem::discriminant;

    let mut prev_status: Option<RunnerStatus> = None;

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let state = app_handle.state::<AppState>();
        let runner = state.runner.read().await;

        let current_status = match runner.as_ref() {
            Some(handle) => handle.status(),
            None => RunnerStatus::Stopped,
        };

        // Only notify when the status *kind* changes (ignore inner value changes
        // like active_sessions ticking).
        let changed = match &prev_status {
            None => true,
            Some(prev) => discriminant(prev) != discriminant(&current_status),
        };

        if changed {
            if let Some(ref prev) = prev_status {
                let notification = match &current_status {
                    RunnerStatus::Error(_) => Some((
                        "Connection Lost",
                        "The runner lost its connection to the control service.",
                    )),
                    RunnerStatus::Active { .. } => {
                        // Only notify when transitioning from a non-Active state.
                        if !matches!(prev, RunnerStatus::Active { .. }) {
                            Some(("Connected", "Runner is connected to the control service."))
                        } else {
                            None
                        }
                    }
                    RunnerStatus::PendingApproval => Some((
                        "Pending Approval",
                        "This machine is waiting to be approved by an administrator.",
                    )),
                    _ => None,
                };

                if let Some((title, body)) = notification {
                    let _ = app_handle
                        .notification()
                        .builder()
                        .title(title)
                        .body(body)
                        .show();
                }
            }

            prev_status = Some(current_status);
        }
    }
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod tray;

use std::sync::Arc;

use cadre_machine_runner::{RunnerHandle, RunnerStatus, Settings};
use tauri::Manager;
use tauri_plugin_notification::NotificationExt;
use tokio::sync::RwLock;
use tracing_appender::rolling;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub struct AppState {
    pub runner: Arc<RwLock<Option<RunnerHandle>>>,
    pub settings: Arc<RwLock<Settings>>,
    pub token: Arc<RwLock<String>>,
    /// Shared machine ID for the log forwarding layer.
    pub machine_id: Arc<RwLock<Option<String>>>,
}

fn main() {
    let minimized = std::env::args().any(|a| a == "--minimized");

    let settings = Settings::load().unwrap_or_default();
    let machine_id: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));
    let shared_settings = Arc::new(RwLock::new(settings.clone()));

    // Set up file-based logging to ~/.config/cadre/runner.YYYY-MM-DD.log
    let log_dir = dirs::config_dir()
        .unwrap_or_default()
        .join("cadre");
    let file_appender = rolling::RollingFileAppender::builder()
        .rotation(rolling::Rotation::DAILY)
        .max_log_files(3)
        .filename_prefix("runner")
        .filename_suffix("log")
        .build(&log_dir)
        .expect("failed to create log appender");

    // Create the log forwarding layer that batches events to the control API.
    // We use an empty token here; once the user authenticates and the runner
    // starts, the forwarding layer will pick up the machine_id and begin sending.
    let (forwarding_layer, _forwarder_handle) =
        cadre_machine_runner::log_forwarder::create_layer(
            &settings.control_service_url,
            "", // token is not yet available at init; logs are dropped until machine_id is set
            Arc::clone(&machine_id),
            Arc::clone(&shared_settings),
        );

    // Compose: EnvFilter + file appender + API forwarding
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(file_appender)
                .with_target(true)
                .with_ansi(false),
        )
        .with(forwarding_layer)
        .init();

    let app_state = AppState {
        runner: Arc::new(RwLock::new(None)),
        settings: shared_settings,
        token: Arc::new(RwLock::new(String::new())),
        machine_id,
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

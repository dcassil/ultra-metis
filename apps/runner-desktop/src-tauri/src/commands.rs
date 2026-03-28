use cadre_machine_runner::{RunnerHandle, RunnerStatus, Settings};
use tauri::State;
use tauri_plugin_autostart::ManagerExt as AutostartManagerExt;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::UpdaterExt;

use crate::AppState;

/// Return the current runner status as a JSON string.
#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<String, String> {
    let runner = state.runner.read().await;
    let status = match runner.as_ref() {
        Some(handle) => handle.status(),
        None => RunnerStatus::Stopped,
    };
    serde_json::to_string(&status).map_err(|e| e.to_string())
}

/// Create a `RunnerHandle` from the current settings and token, then start it.
///
/// Uses the shared `machine_id` Arc from `AppState` so that the log forwarding
/// layer receives the machine ID as soon as registration succeeds.
#[tauri::command]
pub async fn start_runner(state: State<'_, AppState>) -> Result<(), String> {
    let settings = state.settings.read().await.clone();
    let token = state.token.read().await.clone();
    let machine_id = std::sync::Arc::clone(&state.machine_id);

    let mut handle = RunnerHandle::with_shared_machine_id(settings, token, machine_id);
    handle.start().await.map_err(|e| e.to_string())?;

    let mut runner = state.runner.write().await;
    *runner = Some(handle);
    Ok(())
}

/// Stop the running runner, if any.
#[tauri::command]
pub async fn stop_runner(state: State<'_, AppState>) -> Result<(), String> {
    let mut runner = state.runner.write().await;
    if let Some(ref mut handle) = *runner {
        handle.stop().await.map_err(|e| e.to_string())?;
    }
    *runner = None;
    Ok(())
}

/// Toggle the `enabled` flag in settings and update the runner if it exists.
/// Returns the new value of `enabled`.
#[tauri::command]
pub async fn toggle_enabled(state: State<'_, AppState>) -> Result<bool, String> {
    let new_enabled = {
        let mut settings = state.settings.write().await;
        settings.enabled = !settings.enabled;
        settings.enabled
    };

    // Push the updated settings into the runner if it exists.
    let runner = state.runner.read().await;
    if let Some(ref handle) = *runner {
        let updated = state.settings.read().await.clone();
        handle.update_settings(updated).await;
    }

    Ok(new_enabled)
}

/// Return the current settings as a JSON string.
#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<String, String> {
    let settings = state.settings.read().await;
    serde_json::to_string(&*settings).map_err(|e| e.to_string())
}

/// Parse settings JSON, update shared state, persist to disk, and push to runner.
#[tauri::command]
pub async fn save_settings(
    state: State<'_, AppState>,
    settings_json: String,
) -> Result<(), String> {
    let new_settings: Settings =
        serde_json::from_str(&settings_json).map_err(|e| format!("Invalid settings JSON: {e}"))?;

    // Persist to disk first so we don't lose changes if the app crashes.
    new_settings.save().map_err(|e| e.to_string())?;

    // Update the shared state.
    {
        let mut guard = state.settings.write().await;
        *guard = new_settings.clone();
    }

    // Push to the runner if it is running.
    let runner = state.runner.read().await;
    if let Some(ref handle) = *runner {
        handle.update_settings(new_settings).await;
    }

    Ok(())
}

/// Return the current API token.
#[tauri::command]
pub async fn get_token(state: State<'_, AppState>) -> Result<String, String> {
    let token = state.token.read().await;
    Ok(token.clone())
}

/// Store a new API token and push it to the runner if running.
#[tauri::command]
pub async fn set_token(state: State<'_, AppState>, token: String) -> Result<(), String> {
    {
        let mut guard = state.token.write().await;
        *guard = token.clone();
    }

    // Push to the runner if it is running.
    let runner = state.runner.read().await;
    if let Some(ref handle) = *runner {
        handle.update_token(token).await;
    }

    Ok(())
}

/// Test connectivity to a control service by hitting its health endpoint.
#[tauri::command]
pub async fn test_connection(url: String) -> Result<String, String> {
    let health_url = format!("{}/health", url.trim_end_matches('/'));

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let response = client
        .get(&health_url)
        .send()
        .await
        .map_err(|e| format!("Connection failed: {e}"))?;

    let status = response.status();
    if status.is_success() {
        Ok("Connection successful".to_string())
    } else {
        Err(format!("Server returned status {status}"))
    }
}

/// Check whether this is a first run (settings file does not exist).
#[tauri::command]
pub async fn is_first_run() -> Result<bool, String> {
    let path = Settings::settings_path();
    Ok(!path.exists())
}

/// Enable or disable auto-start on login.
#[tauri::command]
pub async fn set_auto_start(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(|e| e.to_string())?;
    } else {
        manager.disable().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Check whether auto-start on login is currently enabled.
#[tauri::command]
pub async fn get_auto_start(app: tauri::AppHandle) -> Result<bool, String> {
    app.autolaunch()
        .is_enabled()
        .map_err(|e| e.to_string())
}

/// Send a desktop notification with the given title and body.
#[tauri::command]
pub async fn send_notification(
    app: tauri::AppHandle,
    title: String,
    body: String,
) -> Result<(), String> {
    app.notification()
        .builder()
        .title(&title)
        .body(&body)
        .show()
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Uninstall the runner: stop it, disable auto-start, optionally deregister
/// from the server, clean up keychain and settings, then exit the app.
///
/// Every cleanup step is best-effort — we never fail the uninstall because
/// one step couldn't complete.
#[tauri::command]
pub async fn uninstall(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    deregister: bool,
) -> Result<(), String> {
    // 1. Stop the runner if running.
    let mut runner_guard = state.runner.write().await;
    if let Some(ref mut handle) = *runner_guard {
        let _ = handle.stop().await;
    }
    *runner_guard = None;
    drop(runner_guard);

    // 2. Disable auto-start.
    let _ = app.autolaunch().disable();

    // 3. Optionally deregister from server (best-effort).
    if deregister {
        let settings = state.settings.read().await;
        let token = state.token.read().await;
        if !token.is_empty() && !settings.control_service_url.is_empty() {
            let client = reqwest::Client::new();
            let _ = client
                .post(format!(
                    "{}/api/machines/{}/revoke",
                    settings.control_service_url, settings.machine_name
                ))
                .bearer_auth(&*token)
                .send()
                .await;
        }
    }

    // 4. Delete keychain token.
    {
        let settings = state.settings.read().await;
        let _ = cadre_machine_runner::delete_token(&settings.machine_name);
    }

    // 5. Delete settings file.
    let settings_path = Settings::settings_path();
    let _ = std::fs::remove_file(&settings_path);

    // 6. Delete log file if it exists.
    let log_path = settings_path.with_file_name("runner.log");
    let _ = std::fs::remove_file(&log_path);

    // 7. Exit the app.
    app.exit(0);

    Ok(())
}

/// Check for application updates using the Tauri updater plugin.
#[tauri::command]
pub async fn check_for_updates(app: tauri::AppHandle) -> Result<String, String> {
    let current_version = app.package_info().version.to_string();

    let updater = app.updater().map_err(|e| e.to_string())?;

    match updater.check().await {
        Ok(Some(update)) => {
            let version = update.version.clone();
            let body = update.body.clone().unwrap_or_default();
            Ok(serde_json::json!({
                "available": true,
                "current_version": current_version,
                "latest_version": version,
                "notes": body,
                "message": format!("Update available: v{version}")
            })
            .to_string())
        }
        Ok(None) => Ok(serde_json::json!({
            "available": false,
            "current_version": current_version,
            "message": "You're up to date"
        })
        .to_string()),
        Err(e) => Ok(serde_json::json!({
            "available": false,
            "current_version": current_version,
            "message": format!("Could not check for updates: {e}")
        })
        .to_string()),
    }
}

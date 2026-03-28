use cadre_machine_runner::{RunnerHandle, RunnerStatus, Settings};
use tauri::State;
use tauri_plugin_autostart::ManagerExt as AutostartManagerExt;
use tauri_plugin_notification::NotificationExt;

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
#[tauri::command]
pub async fn start_runner(state: State<'_, AppState>) -> Result<(), String> {
    let settings = state.settings.read().await.clone();
    let token = state.token.read().await.clone();

    let mut handle = RunnerHandle::new(settings, token);
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

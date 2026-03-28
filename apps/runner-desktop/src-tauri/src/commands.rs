use cadre_machine_runner::{RunnerHandle, RunnerStatus};
use tauri::State;

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

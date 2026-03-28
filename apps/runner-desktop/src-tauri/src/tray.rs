use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;

/// Create the system tray icon and its context menu.
///
/// Menu items:
/// - Enable/Disable — toggles the runner on/off
/// - Settings — opens the settings window
/// - View Sessions — opens the dashboard URL in the default browser
/// - Quit — exits the application
pub fn create_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let toggle = MenuItemBuilder::with_id("toggle", "Disable")
        .build(app)?;
    let settings = MenuItemBuilder::with_id("settings", "Settings...")
        .build(app)?;
    let sessions = MenuItemBuilder::with_id("sessions", "View Sessions")
        .build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit")
        .build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&toggle)
        .separator()
        .item(&settings)
        .item(&sessions)
        .separator()
        .item(&quit)
        .build()?;

    TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app_handle, event| {
            match event.id().as_ref() {
                "toggle" => {
                    // Fire the toggle_enabled command via the app handle.
                    // In a full implementation this would update the menu text too.
                    let handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        let state = handle.state::<crate::AppState>();
                        let mut settings = state.settings.write().await;
                        settings.enabled = !settings.enabled;
                        tracing::info!(enabled = settings.enabled, "Toggled runner enabled state");
                    });
                }
                "settings" => {
                    // Show or create the main settings window.
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    } else {
                        let _window = tauri::WebviewWindowBuilder::new(
                            app_handle,
                            "main",
                            tauri::WebviewUrl::App("index.html".into()),
                        )
                        .title("Cadre Machine Runner — Settings")
                        .inner_size(600.0, 500.0)
                        .build();
                    }
                }
                "sessions" => {
                    // Open the dashboard URL in the default browser.
                    let handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        let state = handle.state::<crate::AppState>();
                        let settings = state.settings.read().await;
                        let url = format!("{}/dashboard", settings.control_service_url);
                        if let Err(e) = open::that(&url) {
                            tracing::error!(error = %e, "Failed to open dashboard URL");
                        }
                    });
                }
                "quit" => {
                    // Gracefully stop the runner and exit.
                    let handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        let state = handle.state::<crate::AppState>();
                        let mut runner = state.runner.write().await;
                        if let Some(ref mut r) = *runner {
                            let _ = r.stop().await;
                        }
                    });
                    app_handle.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}

use std::sync::Arc;

use tokio::sync::RwLock;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (settings, token) = load_settings_and_token()?;

    // Shared state for the log forwarding layer. The machine_id starts as None
    // and is populated by RunnerHandle after successful registration.
    let shared_settings = Arc::new(RwLock::new(settings.clone()));
    let machine_id: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));

    // Create the log forwarding layer that batches events to the control API.
    let (forwarding_layer, _forwarder_handle) = cadre_machine_runner::log_forwarder::create_layer(
        &settings.control_service_url,
        &token,
        Arc::clone(&machine_id),
        Arc::clone(&shared_settings),
    );

    // Compose: EnvFilter (RUST_LOG) + fmt (stderr) + forwarding (API)
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .with(forwarding_layer)
        .init();

    tracing::info!(
        name = %settings.machine_name,
        url = %settings.control_service_url,
        "Starting machine runner"
    );

    let mut handle = cadre_machine_runner::RunnerHandle::with_shared_machine_id(
        settings,
        token,
        machine_id,
    );
    handle.start().await?;

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;
    tracing::info!("Received Ctrl+C, shutting down");

    handle.stop().await?;
    Ok(())
}

/// Resolve settings and API token through the following priority:
///
/// 1. Load `settings.json` — if it exists and contains data, use it.
/// 2. If `settings.json` is missing but legacy `machine-runner.toml` exists, migrate.
/// 3. Otherwise fall back to defaults (caller will need to configure).
///
/// The API token is loaded from the OS keychain when possible. During migration
/// the token is extracted from the TOML config and stored in the keychain.
fn load_settings_and_token() -> anyhow::Result<(cadre_machine_runner::Settings, String)> {
    use cadre_machine_runner::Settings;

    let settings_path = Settings::settings_path();

    if settings_path.exists() {
        // Happy path: settings.json already exists.
        let settings = Settings::load()?;
        let token = load_token_from_keychain(&settings.machine_name);
        return Ok((settings, token));
    }

    // settings.json doesn't exist yet — try legacy migration.
    if Settings::has_legacy_config() {
        tracing::info!("Legacy TOML config found, migrating to settings.json");
        let (settings, token) = Settings::migrate_from_legacy()?;

        // Attempt to store the migrated token in the OS keychain.
        if let Err(e) = cadre_machine_runner::store_token(&settings.machine_name, &token) {
            tracing::warn!(
                error = %e,
                "Could not store token in OS keychain; using in-memory token"
            );
        }

        return Ok((settings, token));
    }

    // No config at all — return defaults with an empty token.
    tracing::warn!("No configuration found; using default settings with empty token");
    let settings = Settings::default();
    Ok((settings, String::new()))
}

/// Try to retrieve the API token from the OS keychain.
///
/// Returns an empty string if the keychain is unavailable or has no entry,
/// logging a warning so the user knows they need to authenticate.
fn load_token_from_keychain(machine_name: &str) -> String {
    match cadre_machine_runner::get_token(machine_name) {
        Ok(Some(token)) => {
            tracing::debug!("Loaded API token from OS keychain");
            token
        }
        Ok(None) => {
            tracing::warn!("No API token found in OS keychain; runner will need authentication");
            String::new()
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to access OS keychain; running without token");
            String::new()
        }
    }
}

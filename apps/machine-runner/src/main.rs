#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = cadre_machine_runner::config::RunnerConfig::load(None)?;
    let (settings, token) = cadre_machine_runner::Settings::from_runner_config(&config);

    tracing::info!(
        name = %settings.machine_name,
        url = %settings.control_service_url,
        "Starting machine runner"
    );

    let mut handle = cadre_machine_runner::RunnerHandle::new(settings, token);
    handle.start().await?;

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;
    tracing::info!("Received Ctrl+C, shutting down");

    handle.stop().await?;
    Ok(())
}

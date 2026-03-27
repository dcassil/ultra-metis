#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = cadre_machine_runner::config::RunnerConfig::load(None)?;

    tracing::info!(
        name = %config.machine_name,
        url = %config.control_service_url,
        "Starting machine runner"
    );

    let mut runner = cadre_machine_runner::runner::Runner::new(config);
    runner.run().await
}

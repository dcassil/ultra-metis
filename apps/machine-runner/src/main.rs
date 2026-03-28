#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = cadre_machine_runner::config::RunnerConfig::load(None)?;

    tracing::info!(
        name = %config.machine_name,
        url = %config.control_service_url,
        "Starting machine runner"
    );

    let (mut runner, mut event_rx) = cadre_machine_runner::runner::Runner::new(config.clone());

    // Create a StateReporter with its own client to report session state changes
    let client = cadre_machine_runner::client::ControlClient::new(
        &config.control_service_url,
        &config.api_token,
    );
    let mut reporter = cadre_machine_runner::state_reporter::StateReporter::new(client);

    // Spawn a task to consume supervisor events and report state changes
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            tracing::info!(?event, "Supervisor event");
            reporter.handle_event(event).await;
        }
    });

    runner.run().await
}

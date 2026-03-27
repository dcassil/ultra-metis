use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = cadre_machine_runner::config::RunnerConfig::load(None)?;
    let repos = cadre_machine_runner::discovery::discover_repos(&config.repo_paths());

    tracing::info!(count = repos.len(), "Discovered repos");
    for repo in &repos {
        tracing::info!(
            name = %repo.repo_name,
            path = %repo.repo_path,
            cadre = repo.cadre_managed,
            "Found repo"
        );
    }

    Ok(())
}

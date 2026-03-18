use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let mut results_dir = PathBuf::from("benchmarks/practical/results");
    let mut scenario_dir = PathBuf::from("benchmarks/practical/scenario");
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--results-dir" => {
                results_dir = PathBuf::from(args.next().expect("--results-dir requires a value"));
            }
            "--scenario" => {
                scenario_dir = PathBuf::from(args.next().expect("--scenario requires a value"));
            }
            _ => {}
        }
    }

    let scenario = practical_benchmark::scenario_pack::LoadedScenarioPack::load(&scenario_dir)?;
    std::fs::create_dir_all(&results_dir)?;

    println!("=== MCP Benchmark Suite ===");
    println!("Scenario : {}", scenario.manifest.title);
    println!("Results  : {}", results_dir.display());
    println!();

    let result = practical_benchmark::mcp_suite::run_suite(&scenario).await?;
    let ts = result.timestamp.format("%Y%m%dT%H%M%SZ");
    let json_path = results_dir.join(format!("mcp_suite_{}.json", ts));
    let report_path = results_dir.join(format!("mcp_suite_{}.md", ts));
    let json = serde_json::to_string_pretty(&result)?;
    std::fs::write(&json_path, &json)?;
    std::fs::write(results_dir.join("latest_mcp_suite.json"), &json)?;
    let report = practical_benchmark::mcp_suite::format_suite_report(&result);
    std::fs::write(&report_path, &report)?;

    println!("  JSON   : {}", json_path.display());
    println!("  Report : {}", report_path.display());
    println!();
    println!("=== Benchmark Complete ===");

    Ok(())
}

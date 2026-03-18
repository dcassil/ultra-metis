use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
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

    std::fs::create_dir_all(&results_dir)?;
    let scenario = practical_benchmark::scenario_pack::LoadedScenarioPack::load(&scenario_dir)?;

    println!("=== MCP Comparison Benchmark ===");
    println!("Scenario : {}", scenario.manifest.title);
    println!("Results  : {}", results_dir.display());
    println!();

    let result = practical_benchmark::mcp_comparison::run_shared_tool_comparison(
        &scenario.manifest.id,
        &scenario.manifest.title,
    )?;

    let ts = result.timestamp.format("%Y%m%dT%H%M%SZ");
    let json_path = results_dir.join(format!("mcp_comparison_{}.json", ts));
    let report_path = results_dir.join(format!("mcp_comparison_{}.md", ts));

    let json = serde_json::to_string_pretty(&result)?;
    std::fs::write(&json_path, &json)?;
    std::fs::write(results_dir.join("latest_mcp_comparison.json"), &json)?;

    let report = practical_benchmark::mcp_comparison::format_comparison_report(&result);
    std::fs::write(&report_path, &report)?;

    println!("  JSON   : {}", json_path.display());
    println!("  Report : {}", report_path.display());
    println!();
    println!("=== Benchmark Complete ===");

    Ok(())
}

/// Tool Comparison Benchmark Runner
///
/// Compares cadre vs original metis template quality by asking Claude
/// to fill each tool's initiative template, then scoring the results.
///
/// Usage:
///   CADRE_BINARY=./target/release/cadre \
///     cargo run -p practical-benchmark --bin run_tool_comparison -- \
///     --results-dir benchmarks/practical/results
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let mut results_dir = PathBuf::from("benchmarks/practical/results");
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--results-dir" {
            results_dir = PathBuf::from(args.next().expect("--results-dir requires a value"));
        }
    }

    // Check claude CLI is available
    if std::process::Command::new("claude")
        .arg("--version")
        .output()
        .is_err()
    {
        eprintln!("Error: `claude` CLI not found. Install Claude Code to run this benchmark.");
        std::process::exit(1);
    }

    // Check original metis is available
    if practical_benchmark::tool_comparison::find_original_metis_binary().is_none() {
        eprintln!("Warning: original metis binary not found.");
        eprintln!("  Install metis CLI or ensure it's in PATH.");
        eprintln!("  Plugin cache: ~/.claude/plugins/cache/colliery-io-metis/metis/*/bin/metis");
        eprintln!("  Continuing — original-metis results will be empty.");
    }

    std::fs::create_dir_all(&results_dir)?;

    println!("=== Tool Comparison Benchmark ===");
    println!("cadre vs original-metis template quality");
    println!("Results: {}", results_dir.display());
    println!();

    let result = practical_benchmark::tool_comparison::run_comparison()?;

    let u = &result.cadre;
    let o = &result.original_metis;

    println!(
        "  cadre    : {:.1}% completeness | {:.1} placeholders/doc | {} tokens",
        u.avg_completeness_percent, u.avg_placeholder_count, u.tokens_used
    );
    println!(
        "  original-metis : {:.1}% completeness | {:.1} placeholders/doc | {} tokens",
        o.avg_completeness_percent, o.avg_placeholder_count, o.tokens_used
    );
    println!();
    println!(
        "  Completeness delta : {:+.1}% (cadre - original-metis)",
        result.completeness_delta
    );
    println!(
        "  Placeholder delta  : {:+.1}  (original - ultra, positive = original has more)",
        result.placeholder_delta
    );
    println!();

    let ts = result.timestamp.format("%Y%m%dT%H%M%SZ");

    // Save JSON
    let json = serde_json::to_string_pretty(&result)?;
    let json_path = results_dir.join(format!("tool_comparison_{ts}.json"));
    std::fs::write(&json_path, &json)?;
    std::fs::write(results_dir.join("latest_tool_comparison.json"), &json)?;

    // Save markdown report
    let report = practical_benchmark::tool_comparison::format_comparison_report(&result);
    let report_path = results_dir.join(format!("tool_comparison_{ts}.md"));
    std::fs::write(&report_path, &report)?;

    println!("  JSON   : {}", json_path.display());
    println!("  Report : {}", report_path.display());
    println!();
    println!("=== Benchmark Complete ===");

    Ok(())
}

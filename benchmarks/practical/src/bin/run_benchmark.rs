/// Practical Benchmark Runner
///
/// Runs autonomous and validated execution paths against the File Processing
/// Toolkit scenario, then generates a side-by-side comparison report.
///
/// Usage:
///   ANTHROPIC_API_KEY=... ULTRA_METIS_BINARY=./target/release/cadre \
///     cargo run -p practical-benchmark --bin run_benchmark -- --results-dir benchmarks/practical/results
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse args
    let mut results_dir = PathBuf::from("benchmarks/practical/results");
    let mut scenario_path = PathBuf::from("benchmarks/practical/scenario");
    let mut mode = RunMode::Both;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--results-dir" => {
                results_dir = PathBuf::from(args.next().expect("--results-dir requires a value"))
            }
            "--scenario" => {
                scenario_path = PathBuf::from(args.next().expect("--scenario requires a value"))
            }
            "--mode" => {
                mode = match args.next().expect("--mode requires a value").as_str() {
                    "autonomous" => RunMode::Autonomous,
                    "validated" => RunMode::Validated,
                    _ => RunMode::Both,
                }
            }
            _ => {}
        }
    }

    // Verify some form of Claude access is available
    if std::env::var("ANTHROPIC_API_KEY").is_err() {
        // Check claude CLI is available as fallback
        if std::process::Command::new("claude")
            .arg("--version")
            .output()
            .is_err()
        {
            eprintln!("Error: Neither ANTHROPIC_API_KEY nor `claude` CLI found. Set ANTHROPIC_API_KEY or install Claude Code.");
            std::process::exit(1);
        }
        println!("Note: Using `claude` CLI for API calls (no ANTHROPIC_API_KEY set)");
        println!();
    }

    let history_path = results_dir.join("results_history.csv");
    std::fs::create_dir_all(&results_dir)?;

    println!("=== Practical Benchmark Suite ===");
    println!("Scenario : {}", scenario_path.display());
    println!("Results  : {}", results_dir.display());
    println!("Mode     : {:?}", mode);
    println!();

    let harness =
        practical_benchmark::BenchmarkHarness::new(scenario_path.clone(), results_dir.clone());

    let autonomous_run = if matches!(mode, RunMode::Autonomous | RunMode::Both) {
        println!("=== Phase 1: Autonomous Execution (Baseline) ===");
        let run = harness.run_autonomous().await?;
        println!(
            "  Initiatives: {}  |  Tokens: {}  |  Time: {:.1}s",
            run.initiatives.len(),
            run.total_metrics.total_tokens,
            run.total_metrics.total_time.as_secs_f32(),
        );
        let path = practical_benchmark::reports::save_run(&run, &results_dir)?;
        let report_path = practical_benchmark::reports::save_run_report(&run, &results_dir)?;
        practical_benchmark::reports::append_history(&run, &history_path)?;
        println!("  Saved: {}", path.display());
        println!("  Report: {}", report_path.display());
        println!();
        Some(run)
    } else {
        None
    };

    let validated_run = if matches!(mode, RunMode::Validated | RunMode::Both) {
        println!("=== Phase 2: Validated Execution (With Gates) ===");
        let run = harness.run_validated().await?;
        println!(
            "  Initiatives: {}  |  Tokens: {}  |  Gate effectiveness: {:.1}%",
            run.initiatives.len(),
            run.total_metrics.total_tokens,
            run.total_metrics.gate_effectiveness.unwrap_or(0.0),
        );
        let path = practical_benchmark::reports::save_run(&run, &results_dir)?;
        let report_path = practical_benchmark::reports::save_run_report(&run, &results_dir)?;
        practical_benchmark::reports::append_history(&run, &history_path)?;
        println!("  Saved: {}", path.display());
        println!("  Report: {}", report_path.display());
        println!();
        Some(run)
    } else {
        None
    };

    // Generate comparison report when both runs are available
    if let (Some(auto), Some(val)) = (autonomous_run, validated_run) {
        println!("=== Comparison Report ===");
        let ts = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
        let report_path = results_dir.join(format!("comparison_{}.md", ts));
        practical_benchmark::reports::generate_comparison_report(&auto, &val, &report_path)?;
        println!("  Report: {}", report_path.display());

        let analysis = practical_benchmark::analysis::BenchmarkAnalysis::new(auto, val);
        let report = analysis.compare();
        println!();
        println!("  Token overhead  : {:+.1}%", report.token_overhead);
        println!("  Quality delta   : {:+.1} points", report.quality_delta);
        println!("  ROI             : {:.2}", report.roi);
        println!("  Error detection : {:.1}%", report.error_detection_rate);
    }

    println!();
    println!("=== Benchmark Complete ===");
    Ok(())
}

#[derive(Debug)]
enum RunMode {
    Autonomous,
    Validated,
    Both,
}

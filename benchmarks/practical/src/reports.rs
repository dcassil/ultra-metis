use crate::{
    analysis::{BenchmarkAnalysis, ComparisonReport},
    types::{BenchmarkRun, GateDecision},
};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Serialize `run` to JSON and write to `results_dir/run_<timestamp>.json`.
/// Also copies to `results_dir/latest_run.json` for easy access.
/// Returns the path of the timestamped file.
pub fn save_run(run: &BenchmarkRun, results_dir: &Path) -> anyhow::Result<PathBuf> {
    std::fs::create_dir_all(results_dir)?;

    let ts = run.timestamp.format("%Y%m%dT%H%M%SZ");
    let filename = format!("run_{}.json", ts);
    let path = results_dir.join(&filename);

    let json = serde_json::to_string_pretty(run)?;
    std::fs::write(&path, &json)?;

    // Always overwrite latest_run.json so callers can find the most recent run
    let latest = results_dir.join("latest_run.json");
    std::fs::write(latest, &json)?;

    Ok(path)
}

/// Generate a markdown comparison report from two runs (autonomous vs validated).
/// Mirrors the format of `benchmarks/REPORT.md`.
pub fn generate_comparison_report(
    autonomous: &BenchmarkRun,
    validated: &BenchmarkRun,
    output_path: &Path,
) -> anyhow::Result<()> {
    let analysis = BenchmarkAnalysis::new(autonomous.clone(), validated.clone());
    let report = analysis.compare();

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = format_report(autonomous, validated, &report);
    std::fs::write(output_path, content)?;

    Ok(())
}

/// Append a single CSV row for `run` to `history_path`.
/// Creates the file with headers if it does not yet exist.
pub fn append_history(run: &BenchmarkRun, history_path: &Path) -> anyhow::Result<()> {
    let needs_header = !history_path.exists();

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(history_path)?;

    if needs_header {
        writeln!(
            file,
            "timestamp,run_id,scenario_id,mode,total_tokens,total_time_secs,avg_doc_accuracy,avg_instruction_adherence,gate_effectiveness"
        )?;
    }

    let gate_eff = run
        .total_metrics
        .gate_effectiveness
        .map(|v| format!("{:.2}", v))
        .unwrap_or_default();

    writeln!(
        file,
        "{},{},{},{:?},{},{},{:.2},{:.2},{}",
        run.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
        run.run_id,
        run.scenario.id,
        run.execution_mode,
        run.total_metrics.total_tokens,
        run.total_metrics.total_time.as_secs(),
        run.total_metrics.avg_doc_accuracy,
        run.total_metrics.avg_instruction_adherence,
        gate_eff,
    )?;

    Ok(())
}

fn format_report(
    autonomous: &BenchmarkRun,
    validated: &BenchmarkRun,
    report: &ComparisonReport,
) -> String {
    let date = autonomous.timestamp.format("%Y-%m-%d");
    let mut out = String::new();

    // Header
    out.push_str(&format!(
        "# Practical Benchmark Comparison Report\n\n\
         **Date**: {}\n\
         **Scenario**: {} ({})\n\
         **Autonomous run ID**: {}\n\
         **Validated run ID**: {}\n\n",
        date,
        autonomous.scenario.title,
        autonomous.scenario.id,
        autonomous.run_id,
        validated.run_id,
    ));

    // Executive Summary
    out.push_str("## Executive Summary\n\n");
    let roi_verdict = if report.roi > 1.0 {
        "Validation gates delivered **positive ROI**: quality improved more than token overhead."
    } else if report.roi > 0.0 {
        "Validation gates produced marginal improvement relative to token overhead."
    } else {
        "Validation gates did not improve quality relative to token overhead."
    };
    out.push_str(&format!(
        "- Token overhead for validation: **{:+.1}%** ({} → {} tokens)\n\
         - Quality delta (avg score improvement): **{:+.1} points**\n\
         - {}\n\
         - Gate effectiveness (issues found per gate): **{:.1}%**\n\n",
        report.token_overhead,
        autonomous.total_metrics.total_tokens,
        validated.total_metrics.total_tokens,
        report.quality_delta,
        roi_verdict,
        report.gate_effectiveness,
    ));

    // Per-Initiative Results
    out.push_str("## Per-Initiative Results\n\n");
    out.push_str(
        "| Initiative | Autonomous Tokens | Validated Tokens | Token Δ | Gate Decision |\n",
    );
    out.push_str("|-----------|-------------------|-----------------|---------|---------------|\n");

    let auto_by_id: std::collections::HashMap<_, _> = autonomous
        .initiatives
        .iter()
        .map(|i| (i.initiative_id.as_str(), i))
        .collect();

    for v_init in &validated.initiatives {
        let auto_tokens = auto_by_id
            .get(v_init.initiative_id.as_str())
            .map(|i| i.total_tokens)
            .unwrap_or(0);
        let gate_str = v_init
            .tasks
            .iter()
            .filter_map(|t| t.validation_gate.as_ref())
            .map(|g| match g.gate_decision {
                GateDecision::Approved => "✓ Approved",
                GateDecision::RequiresRework => "⚠ Rework",
                GateDecision::Rejected => "✗ Rejected",
            })
            .next()
            .unwrap_or("—");

        let auto_str = if auto_tokens > 0 {
            auto_tokens.to_string()
        } else {
            "—".to_string()
        };
        let delta_str = if auto_tokens > 0 {
            format!("{:+}", v_init.total_tokens as i64 - auto_tokens as i64)
        } else {
            "—".to_string()
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            v_init.initiative_title, auto_str, v_init.total_tokens, delta_str, gate_str,
        ));
    }
    out.push('\n');

    // Aggregate Metrics
    out.push_str("## Aggregate Metrics\n\n");
    out.push_str("| Metric | Autonomous | Validated | Delta |\n");
    out.push_str("|--------|------------|-----------|-------|\n");

    let a = &autonomous.total_metrics;
    let v = &validated.total_metrics;
    out.push_str(&format!(
        "| Total tokens | {} | {} | {:+.1}% |\n\
         | Doc accuracy | {:.1}% | {:.1}% | {:+.1} |\n\
         | Instruction adherence | {:.1}% | {:.1}% | {:+.1} |\n\
         | Quality delta (avg) | — | — | {:+.1} |\n\
         | ROI | — | — | {:.2} |\n\n",
        a.total_tokens,
        v.total_tokens,
        report.token_overhead,
        a.avg_doc_accuracy,
        v.avg_doc_accuracy,
        v.avg_doc_accuracy - a.avg_doc_accuracy,
        a.avg_instruction_adherence,
        v.avg_instruction_adherence,
        v.avg_instruction_adherence - a.avg_instruction_adherence,
        report.quality_delta,
        report.roi,
    ));

    // Gate Effectiveness
    out.push_str("## Gate Effectiveness\n\n");
    let error_rate = report.error_detection_rate;
    out.push_str(&format!(
        "- Error detection rate: **{:.1}%** of gates caught real issues\n\
         - Gate effectiveness (gates that found issues): **{:.1}%**\n\n",
        error_rate, report.gate_effectiveness,
    ));

    // Issues found by gates
    let all_issues: Vec<&str> = validated
        .initiatives
        .iter()
        .flat_map(|i| i.tasks.iter())
        .filter_map(|t| t.validation_gate.as_ref())
        .flat_map(|g| g.issues_found.iter().map(|s| s.as_str()))
        .collect();

    if !all_issues.is_empty() {
        out.push_str("### Issues Found by Gates\n\n");
        for issue in &all_issues {
            out.push_str(&format!("- {}\n", issue));
        }
        out.push('\n');
    }

    // Recommendations
    out.push_str("## Recommendations\n\n");
    if report.roi > 1.0 {
        out.push_str("- **Use validated mode** — gates caught real issues and improved quality\n");
        out.push_str("- Token overhead is justified by quality improvements\n");
    } else {
        out.push_str(
            "- **Autonomous mode** may be sufficient for this scenario — gate overhead exceeds benefit\n",
        );
        out.push_str("- Consider refining gate prompts to reduce false negatives\n");
    }
    if error_rate > 50.0 {
        out.push_str("- High error detection rate suggests AI commonly misses required elements — review scenario prompts\n");
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ExecutionMode, RunMetrics};
    use chrono::Utc;
    use tempfile::TempDir;

    fn make_run(mode: ExecutionMode, tokens: u64) -> BenchmarkRun {
        BenchmarkRun {
            run_id: format!("{:?}-test", mode),
            timestamp: Utc::now(),
            scenario: crate::types::ScenarioSummary {
                id: "test-scenario".to_string(),
                title: "Test Scenario".to_string(),
                root: "scenario".to_string(),
            },
            execution_mode: mode,
            phases: vec![],
            trace: crate::types::RunTrace::default(),
            initiatives: vec![],
            total_metrics: RunMetrics {
                total_tokens: tokens,
                total_time: std::time::Duration::from_secs(60),
                avg_code_quality: 75.0,
                avg_test_coverage: 80.0,
                avg_doc_accuracy: 70.0,
                avg_instruction_adherence: 85.0,
                gate_effectiveness: None,
            },
        }
    }

    #[test]
    fn test_save_run_creates_json_file() {
        let dir = TempDir::new().unwrap();
        let run = make_run(ExecutionMode::Autonomous, 1000);
        let path = save_run(&run, dir.path()).unwrap();

        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["run_id"], run.run_id);
    }

    #[test]
    fn test_save_run_creates_latest_symlink() {
        let dir = TempDir::new().unwrap();
        let run = make_run(ExecutionMode::Autonomous, 1000);
        save_run(&run, dir.path()).unwrap();

        let latest = dir.path().join("latest_run.json");
        assert!(latest.exists());
        let content = std::fs::read_to_string(latest).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["run_id"], run.run_id);
    }

    #[test]
    fn test_json_round_trip() {
        let dir = TempDir::new().unwrap();
        let run = make_run(ExecutionMode::Validated, 5000);
        let path = save_run(&run, dir.path()).unwrap();

        let content = std::fs::read_to_string(path).unwrap();
        let restored: BenchmarkRun = serde_json::from_str(&content).unwrap();
        assert_eq!(restored.run_id, run.run_id);
        assert_eq!(restored.total_metrics.total_tokens, 5000);
    }

    #[test]
    fn test_comparison_report_contains_required_sections() {
        let dir = TempDir::new().unwrap();
        let auto = make_run(ExecutionMode::Autonomous, 1000);
        let mut validated = make_run(ExecutionMode::Validated, 1100);
        validated.total_metrics.avg_doc_accuracy = 80.0;
        validated.total_metrics.avg_instruction_adherence = 90.0;
        validated.total_metrics.gate_effectiveness = Some(60.0);

        let report_path = dir.path().join("comparison.md");
        generate_comparison_report(&auto, &validated, &report_path).unwrap();

        let content = std::fs::read_to_string(&report_path).unwrap();
        assert!(content.contains("Executive Summary"));
        assert!(content.contains("Per-Initiative Results"));
        assert!(content.contains("Aggregate Metrics"));
        assert!(content.contains("Gate Effectiveness"));
        assert!(content.contains("Recommendations"));
    }

    #[test]
    fn test_append_history_creates_csv_with_header() {
        let dir = TempDir::new().unwrap();
        let history = dir.path().join("history.csv");
        let run = make_run(ExecutionMode::Autonomous, 2000);

        append_history(&run, &history).unwrap();
        append_history(&run, &history).unwrap();

        let content = std::fs::read_to_string(&history).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3); // header + 2 rows
        assert!(lines[0].contains("timestamp"));
        assert!(lines[1].contains(&run.run_id));
    }
}

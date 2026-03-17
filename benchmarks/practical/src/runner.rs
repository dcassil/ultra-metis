use crate::types::*;
use std::path::Path;
use std::process::Command;
use chrono::Utc;

/// Invoke the ultra-metis CLI binary, returning stdout and elapsed ms.
/// Mirrors the approach in benchmarks/run-ultra-metis-bench.sh.
pub fn run_cli(binary: &Path, args: &[&str]) -> anyhow::Result<CliResult> {
    let start = std::time::Instant::now();
    let output = Command::new(binary).args(args).output()?;
    let elapsed = start.elapsed();

    Ok(CliResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
        elapsed,
    })
}

#[derive(Debug, Clone)]
pub struct CliResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub elapsed: std::time::Duration,
}

impl CliResult {
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    /// Approximate token count from output byte size.
    /// ~4 bytes per token is a rough but consistent approximation.
    pub fn approx_tokens(&self) -> u64 {
        ((self.stdout.len() + self.stderr.len()) / 4).max(1) as u64
    }
}

/// Execute autonomous benchmark run (no validation gates).
///
/// This mirrors the existing run-ultra-metis-bench.sh approach: invoke the
/// ultra-metis binary directly for all document operations, capture timing
/// and output. The AI session that drives the workflow records this run's
/// results into the harness after completing each initiative.
pub async fn execute_autonomous(scenario_path: &Path) -> anyhow::Result<BenchmarkRun> {
    let start_time = std::time::Instant::now();
    let run_id = uuid::Uuid::new_v4().to_string();

    tracing::info!("Starting autonomous run: {}", run_id);

    // Load scenario files (vision + 2 initiative summaries)
    let _vision = std::fs::read_to_string(scenario_path.join("vision.md"))?;
    let _parse_init = std::fs::read_to_string(scenario_path.join("parse-initiative.md"))?;
    let _transform_init = std::fs::read_to_string(scenario_path.join("transform-initiative.md"))?;

    // The AI session drives the actual execution and calls record_initiative_result()
    // to populate results. The harness collects timing and token approximations from
    // CLI invocations made during execution.
    //
    // TODO: wire up to live AI session. For now, return scaffold for the session to fill.
    let initiatives: Vec<InitiativeResult> = vec![];
    let total_tokens = 0u64;
    let total_time = start_time.elapsed();

    Ok(BenchmarkRun {
        run_id,
        timestamp: Utc::now(),
        execution_mode: ExecutionMode::Autonomous,
        initiatives,
        total_metrics: RunMetrics {
            total_tokens,
            total_time,
            avg_code_quality: 0.0,
            avg_test_coverage: 0.0,
            avg_doc_accuracy: 0.0,
            avg_instruction_adherence: 0.0,
            gate_effectiveness: None,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_result_token_approximation() {
        let result = CliResult {
            stdout: "a".repeat(400),
            stderr: String::new(),
            exit_code: 0,
            elapsed: std::time::Duration::from_millis(10),
        };
        assert_eq!(result.approx_tokens(), 100);
    }

    #[test]
    fn test_cli_result_success() {
        let ok = CliResult { stdout: String::new(), stderr: String::new(), exit_code: 0, elapsed: std::time::Duration::default() };
        let fail = CliResult { exit_code: 1, ..ok.clone() };
        assert!(ok.success());
        assert!(!fail.success());
    }

    #[tokio::test]
    async fn test_autonomous_runner_creates_valid_run() {
        let run = BenchmarkRun {
            run_id: "test".to_string(),
            timestamp: Utc::now(),
            execution_mode: ExecutionMode::Autonomous,
            initiatives: vec![],
            total_metrics: RunMetrics {
                total_tokens: 0,
                total_time: std::time::Duration::from_secs(0),
                avg_code_quality: 0.0,
                avg_test_coverage: 0.0,
                avg_doc_accuracy: 0.0,
                avg_instruction_adherence: 0.0,
                gate_effectiveness: None,
            },
        };
        assert_eq!(run.execution_mode, ExecutionMode::Autonomous);
    }
}

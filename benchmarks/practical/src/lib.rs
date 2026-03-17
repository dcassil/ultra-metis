pub mod types;
pub mod runner;
pub mod gated_runner;
pub mod metrics_collector;
pub mod analysis;
pub use types::*;

#[derive(Debug)]
pub struct BenchmarkHarness {
    scenario_path: std::path::PathBuf,
    #[allow(dead_code)]
    results_dir: std::path::PathBuf,
}

impl BenchmarkHarness {
    pub fn new(scenario_path: std::path::PathBuf, results_dir: std::path::PathBuf) -> Self {
        Self {
            scenario_path,
            results_dir,
        }
    }

    /// Run autonomous execution (baseline)
    pub async fn run_autonomous(&self) -> anyhow::Result<BenchmarkRun> {
        tracing::info!("Starting autonomous benchmark run");
        runner::execute_autonomous(&self.scenario_path).await
    }

    /// Run validated execution (with gates)
    pub async fn run_validated(&self) -> anyhow::Result<BenchmarkRun> {
        tracing::info!("Starting validated benchmark run");
        gated_runner::execute_with_gates(&self.scenario_path).await
    }
}

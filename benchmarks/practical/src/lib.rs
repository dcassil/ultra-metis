pub mod analysis;
pub mod api_client;
pub mod comparison;
pub mod doc_quality;
pub mod gate_scorer;
pub mod gated_runner;
pub mod mcp_adapter;
pub mod mcp_comparison;
pub mod mcp_planning_comparison;
pub mod mcp_suite;
pub mod metrics_collector;
pub mod prompt_builder;
pub mod reports;
pub mod runner;
pub mod scenario_pack;
pub mod scoring;
pub mod tool_comparison;
pub mod types;
pub mod workspace;
pub use types::*;

#[derive(Debug)]
pub struct BenchmarkHarness {
    scenario: crate::scenario_pack::LoadedScenarioPack,
    #[allow(dead_code)]
    results_dir: std::path::PathBuf,
}

impl BenchmarkHarness {
    pub fn new(scenario_path: std::path::PathBuf, results_dir: std::path::PathBuf) -> Self {
        let scenario = crate::scenario_pack::LoadedScenarioPack::load(&scenario_path)
            .unwrap_or_else(|err| {
                panic!(
                    "Failed to load scenario pack {}: {}",
                    scenario_path.display(),
                    err
                )
            });
        Self {
            scenario,
            results_dir,
        }
    }

    /// Run autonomous execution (baseline)
    pub async fn run_autonomous(&self) -> anyhow::Result<BenchmarkRun> {
        tracing::info!("Starting autonomous benchmark run");
        runner::execute_autonomous(&self.scenario).await
    }

    /// Run validated execution (with gates)
    pub async fn run_validated(&self) -> anyhow::Result<BenchmarkRun> {
        tracing::info!("Starting validated benchmark run");
        gated_runner::execute_with_gates(&self.scenario).await
    }
}

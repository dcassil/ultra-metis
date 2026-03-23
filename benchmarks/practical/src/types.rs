use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRun {
    pub run_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub manifest: RunManifest,
    pub scenario: ScenarioSummary,
    pub execution_mode: ExecutionMode,
    #[serde(default)]
    pub phases: Vec<PhaseResult>,
    #[serde(default)]
    pub trace: RunTrace,
    #[serde(default)]
    pub artifacts: RunArtifacts,
    pub initiatives: Vec<InitiativeResult>,
    pub total_metrics: RunMetrics,
}

/// Pre-run configuration that fully describes how a benchmark was executed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunManifest {
    pub system_under_test: SystemUnderTest,
    pub tool_surface: ToolSurface,
    pub model_id: String,
    pub token_budget: u64,
    pub time_budget_secs: u64,
    #[serde(default)]
    pub git_commit: Option<String>,
    #[serde(default)]
    pub environment: RunEnvironment,
}

impl Default for RunManifest {
    fn default() -> Self {
        Self {
            system_under_test: SystemUnderTest::Cadre,
            tool_surface: ToolSurface::Cli,
            model_id: "claude-haiku-4-5-20251001".to_string(),
            token_budget: 500_000,
            time_budget_secs: 600,
            git_commit: None,
            environment: RunEnvironment::default(),
        }
    }
}

/// Which system is being benchmarked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemUnderTest {
    OriginalMetis,
    Cadre,
}

/// Which tool interface the system used.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolSurface {
    Cli,
    Mcp,
    Plugin,
    Mixed,
}

/// Environment metadata for reproducibility.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunEnvironment {
    #[serde(default)]
    pub os: Option<String>,
    #[serde(default)]
    pub binary_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioSummary {
    pub id: String,
    pub title: String,
    pub root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseResult {
    pub phase: BenchmarkPhase,
    pub status: PhaseStatus,
    pub tokens_used: u64,
    pub time_elapsed: Duration,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BenchmarkPhase {
    ScenarioSetup,
    DocumentGeneration,
    Decomposition,
    BuildOutcome,
    Verification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhaseStatus {
    Completed,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunTrace {
    #[serde(default)]
    pub prompt_events: Vec<PromptTraceEvent>,
    #[serde(default)]
    pub cli_events: Vec<CliTraceEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTraceEvent {
    pub label: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub response_excerpt: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub duration: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliTraceEvent {
    pub label: String,
    pub command: String,
    pub exit_code: i32,
    pub duration: Duration,
    pub approx_tokens: u64,
    pub stdout_excerpt: String,
    pub stderr_excerpt: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunArtifacts {
    #[serde(default)]
    pub documents: Vec<DocumentArtifact>,
    #[serde(default)]
    pub code_files: Vec<CodeArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentArtifact {
    pub path: String,
    pub title: String,
    pub short_code: Option<String>,
    pub excerpt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeArtifact {
    pub path: String,
    pub language: String,
    pub line_count: u32,
    pub excerpt: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExecutionMode {
    Autonomous,
    Validated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitiativeResult {
    pub initiative_id: String,
    pub initiative_title: String,
    pub tasks: Vec<TaskResult>,
    pub total_tokens: u64,
    pub total_time: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub task_title: String,
    pub status: TaskStatus,
    pub tokens_used: u64,
    pub time_elapsed: Duration,
    pub code_metrics: CodeMetrics,
    pub validation_gate: Option<ValidationGateResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Completed,
    FailedValidation,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetrics {
    pub lines_of_code: u32,
    pub test_coverage_percent: f32,
    pub cyclomatic_complexity: f32,
    pub doc_accuracy_percent: f32,
    pub instruction_adherence_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationGateResult {
    pub gate_decision: GateDecision,
    pub issues_found: Vec<String>,
    pub rework_tokens: u64,
    pub rework_time: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GateDecision {
    Approved,
    RequiresRework,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetrics {
    pub total_tokens: u64,
    pub total_time: Duration,
    pub avg_code_quality: f32,
    pub avg_test_coverage: f32,
    pub avg_doc_accuracy: f32,
    pub avg_instruction_adherence: f32,
    pub gate_effectiveness: Option<f32>,
}

/// Flattened result for trend tracking and CSV/JSON history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedResult {
    pub run_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub scenario_id: String,
    pub system_under_test: SystemUnderTest,
    pub execution_mode: ExecutionMode,
    pub tool_surface: ToolSurface,
    pub model_id: String,
    pub total_tokens: u64,
    pub total_time_ms: u64,
    pub doc_generation_score: f32,
    pub decomposition_score: f32,
    pub build_outcome_score: f32,
    pub architecture_conformance_score: f32,
    pub gate_effectiveness: Option<f32>,
    pub initiative_count: u32,
    pub task_count: u32,
}

impl NormalizedResult {
    /// Flatten a BenchmarkRun into a NormalizedResult for history tracking.
    pub fn from_run(run: &BenchmarkRun) -> Self {
        let task_count: u32 = run
            .initiatives
            .iter()
            .map(|i| i.tasks.len() as u32)
            .sum();

        Self {
            run_id: run.run_id.clone(),
            timestamp: run.timestamp,
            scenario_id: run.scenario.id.clone(),
            system_under_test: run.manifest.system_under_test.clone(),
            execution_mode: run.execution_mode.clone(),
            tool_surface: run.manifest.tool_surface.clone(),
            model_id: run.manifest.model_id.clone(),
            total_tokens: run.total_metrics.total_tokens,
            total_time_ms: run.total_metrics.total_time.as_millis() as u64,
            doc_generation_score: run.total_metrics.avg_doc_accuracy,
            decomposition_score: run.total_metrics.avg_code_quality,
            build_outcome_score: run.total_metrics.avg_test_coverage,
            architecture_conformance_score: run.total_metrics.avg_instruction_adherence,
            gate_effectiveness: run.total_metrics.gate_effectiveness,
            initiative_count: run.initiatives.len() as u32,
            task_count,
        }
    }
}

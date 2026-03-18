use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRun {
    pub run_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub scenario: ScenarioSummary,
    pub execution_mode: ExecutionMode,
    #[serde(default)]
    pub phases: Vec<PhaseResult>,
    #[serde(default)]
    pub trace: RunTrace,
    pub initiatives: Vec<InitiativeResult>,
    pub total_metrics: RunMetrics,
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
    pub gate_effectiveness: Option<f32>, // Only for validated runs
}

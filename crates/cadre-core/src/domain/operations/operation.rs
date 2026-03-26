//! Core cognitive operation types.
//!
//! Defines the 12 universal cognitive operations that form the kernel of
//! Cadre's workflow composition system.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// ---------------------------------------------------------------------------
// CognitiveOperation enum — the 12 universal operations
// ---------------------------------------------------------------------------

/// The 12 core cognitive operations that compose all workflows.
///
/// Each operation represents a discrete, reusable unit of engineering
/// reasoning.  Operations are combined into loops, and loops into workflows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CognitiveOperation {
    /// Establish or refine the objective for the current work.
    FrameObjective,
    /// Gather and evaluate context needed for the work.
    AcquireContext,
    /// Build or refine a mental model of the relevant system.
    BuildModel,
    /// Narrow down to the specific area of concern.
    LocateFocus,
    /// Analyze structure, boundaries, and interfaces.
    AnalyzeStructure,
    /// Trace flow, causality, and data paths.
    TraceFlow,
    /// Assess impact and risk of proposed changes.
    AssessImpact,
    /// Shape or select a solution approach.
    ShapeSolution,
    /// Decompose work into sequenced, actionable steps.
    DecomposeWork,
    /// Create or modify a concrete artifact (code, config, doc).
    CreateArtifact,
    /// Validate the result against reality (tests, checks, review).
    ValidateReality,
    /// Reassess assumptions and adapt the plan based on new information.
    ReassessAdapt,
}

impl CognitiveOperation {
    /// Returns all 12 operations in canonical order.
    pub fn all() -> &'static [Self] {
        &[
            Self::FrameObjective,
            Self::AcquireContext,
            Self::BuildModel,
            Self::LocateFocus,
            Self::AnalyzeStructure,
            Self::TraceFlow,
            Self::AssessImpact,
            Self::ShapeSolution,
            Self::DecomposeWork,
            Self::CreateArtifact,
            Self::ValidateReality,
            Self::ReassessAdapt,
        ]
    }

    /// Human-readable description of the operation.
    pub fn description(&self) -> &'static str {
        match self {
            Self::FrameObjective => "Establish or refine the objective for the current work",
            Self::AcquireContext => "Gather and evaluate context needed for the work",
            Self::BuildModel => "Build or refine a mental model of the relevant system",
            Self::LocateFocus => "Narrow down to the specific area of concern",
            Self::AnalyzeStructure => "Analyze structure, boundaries, and interfaces",
            Self::TraceFlow => "Trace flow, causality, and data paths",
            Self::AssessImpact => "Assess impact and risk of proposed changes",
            Self::ShapeSolution => "Shape or select a solution approach",
            Self::DecomposeWork => "Decompose work into sequenced, actionable steps",
            Self::CreateArtifact => "Create or modify a concrete artifact",
            Self::ValidateReality => "Validate the result against reality",
            Self::ReassessAdapt => "Reassess assumptions and adapt the plan",
        }
    }

    /// Snake_case identifier for serialization.
    pub fn identifier(&self) -> &'static str {
        match self {
            Self::FrameObjective => "frame_objective",
            Self::AcquireContext => "acquire_context",
            Self::BuildModel => "build_model",
            Self::LocateFocus => "locate_focus",
            Self::AnalyzeStructure => "analyze_structure",
            Self::TraceFlow => "trace_flow",
            Self::AssessImpact => "assess_impact",
            Self::ShapeSolution => "shape_solution",
            Self::DecomposeWork => "decompose_work",
            Self::CreateArtifact => "create_artifact",
            Self::ValidateReality => "validate_reality",
            Self::ReassessAdapt => "reassess_adapt",
        }
    }
}

impl fmt::Display for CognitiveOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.identifier())
    }
}

impl FromStr for CognitiveOperation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "frame_objective" | "frame" => Ok(Self::FrameObjective),
            "acquire_context" | "acquire" | "context" => Ok(Self::AcquireContext),
            "build_model" | "model" => Ok(Self::BuildModel),
            "locate_focus" | "locate" | "focus" => Ok(Self::LocateFocus),
            "analyze_structure" | "analyze" | "structure" => Ok(Self::AnalyzeStructure),
            "trace_flow" | "trace" | "flow" => Ok(Self::TraceFlow),
            "assess_impact" | "assess" | "impact" => Ok(Self::AssessImpact),
            "shape_solution" | "shape" | "solution" => Ok(Self::ShapeSolution),
            "decompose_work" | "decompose" => Ok(Self::DecomposeWork),
            "create_artifact" | "create" | "artifact" => Ok(Self::CreateArtifact),
            "validate_reality" | "validate" | "reality" => Ok(Self::ValidateReality),
            "reassess_adapt" | "reassess" | "adapt" => Ok(Self::ReassessAdapt),
            _ => Err(format!("Unknown cognitive operation: {s}")),
        }
    }
}

// ---------------------------------------------------------------------------
// OutputKind — what an operation produces
// ---------------------------------------------------------------------------

/// The kind of output a cognitive operation produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputKind {
    /// A refined objective statement.
    Objective,
    /// Collected context artifacts.
    ContextSet,
    /// A system model or diagram.
    Model,
    /// A narrowed focus area (file, module, function).
    FocusArea,
    /// Structural analysis results.
    StructuralAnalysis,
    /// Flow/causality trace results.
    FlowTrace,
    /// Impact/risk assessment.
    ImpactAssessment,
    /// A solution design or approach.
    SolutionDesign,
    /// A decomposed work plan.
    WorkPlan,
    /// A created or modified artifact.
    Artifact,
    /// Validation results (pass/fail with evidence).
    ValidationResult,
    /// An adapted plan with rationale.
    AdaptedPlan,
}

// ---------------------------------------------------------------------------
// ToolHint — what static tools back an operation
// ---------------------------------------------------------------------------

/// A hint about what kind of static tool can back a cognitive operation.
///
/// These are categories, not specific tool bindings — actual tools vary per
/// repository.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    /// File search and navigation (grep, find, ast-grep).
    Search,
    /// Code analysis and linting (clippy, eslint, semgrep).
    Analysis,
    /// Testing and validation (cargo test, jest, pytest).
    Testing,
    /// Build and compilation (cargo build, tsc, make).
    Build,
    /// Version control (git log, git diff, git blame).
    VersionControl,
    /// Documentation lookup (docs, man pages, API refs).
    Documentation,
    /// Code generation and modification (patch, sed, refactoring tools).
    CodeModification,
    /// Metrics and coverage (coverage reports, complexity tools).
    Metrics,
}

impl fmt::Display for ToolCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Search => write!(f, "search"),
            Self::Analysis => write!(f, "analysis"),
            Self::Testing => write!(f, "testing"),
            Self::Build => write!(f, "build"),
            Self::VersionControl => write!(f, "version_control"),
            Self::Documentation => write!(f, "documentation"),
            Self::CodeModification => write!(f, "code_modification"),
            Self::Metrics => write!(f, "metrics"),
        }
    }
}

// ---------------------------------------------------------------------------
// EscalationCondition — when an operation should escalate
// ---------------------------------------------------------------------------

/// Conditions under which an operation should escalate rather than proceed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationCondition {
    /// The operation lacks sufficient context to proceed.
    InsufficientContext,
    /// The operation encounters ambiguity requiring human judgment.
    AmbiguityDetected,
    /// The operation detects a conflict with existing design decisions.
    DesignConflict,
    /// The operation detects risk above the acceptable threshold.
    RiskThresholdExceeded,
    /// The operation's output fails validation.
    ValidationFailed,
    /// The operation has exceeded its iteration budget.
    IterationBudgetExhausted,
    /// Custom escalation with a description.
    Custom(String),
}

impl fmt::Display for EscalationCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsufficientContext => write!(f, "insufficient_context"),
            Self::AmbiguityDetected => write!(f, "ambiguity_detected"),
            Self::DesignConflict => write!(f, "design_conflict"),
            Self::RiskThresholdExceeded => write!(f, "risk_threshold_exceeded"),
            Self::ValidationFailed => write!(f, "validation_failed"),
            Self::IterationBudgetExhausted => write!(f, "iteration_budget_exhausted"),
            Self::Custom(desc) => write!(f, "custom: {desc}"),
        }
    }
}

// ---------------------------------------------------------------------------
// OperationSpec — full specification for a cognitive operation
// ---------------------------------------------------------------------------

/// Full specification for a cognitive operation, including its metadata,
/// input requirements, output type, tool hints, and escalation conditions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationSpec {
    /// Which of the 12 operations this spec describes.
    pub operation: CognitiveOperation,
    /// Human-readable description (may be customized beyond the default).
    pub description: String,
    /// What inputs/context the operation requires.
    pub input_requirements: Vec<String>,
    /// What kind of output the operation produces.
    pub output_kind: OutputKind,
    /// Tool categories that can back this operation.
    pub tool_hints: Vec<ToolCategory>,
    /// Conditions under which this operation should escalate.
    pub escalation_conditions: Vec<EscalationCondition>,
}

impl OperationSpec {
    /// Create a new operation spec with the given operation and its defaults.
    pub fn new(operation: CognitiveOperation) -> Self {
        let (inputs, output, tools, escalations) = default_spec(operation);
        Self {
            description: operation.description().to_string(),
            operation,
            input_requirements: inputs,
            output_kind: output,
            tool_hints: tools,
            escalation_conditions: escalations,
        }
    }

    /// Create a spec with a custom description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add an additional input requirement.
    pub fn with_input(mut self, input: impl Into<String>) -> Self {
        self.input_requirements.push(input.into());
        self
    }

    /// Add an additional tool hint.
    pub fn with_tool_hint(mut self, tool: ToolCategory) -> Self {
        if !self.tool_hints.contains(&tool) {
            self.tool_hints.push(tool);
        }
        self
    }

    /// Add an additional escalation condition.
    pub fn with_escalation(mut self, condition: EscalationCondition) -> Self {
        self.escalation_conditions.push(condition);
        self
    }
}

/// Returns default (inputs, output, tools, escalations) for each operation.
fn default_spec(
    op: CognitiveOperation,
) -> (
    Vec<String>,
    OutputKind,
    Vec<ToolCategory>,
    Vec<EscalationCondition>,
) {
    match op {
        CognitiveOperation::FrameObjective => (
            vec!["work item or request".into(), "existing context".into()],
            OutputKind::Objective,
            vec![ToolCategory::Documentation],
            vec![EscalationCondition::AmbiguityDetected],
        ),
        CognitiveOperation::AcquireContext => (
            vec!["objective".into(), "repository access".into()],
            OutputKind::ContextSet,
            vec![
                ToolCategory::Search,
                ToolCategory::VersionControl,
                ToolCategory::Documentation,
            ],
            vec![EscalationCondition::InsufficientContext],
        ),
        CognitiveOperation::BuildModel => (
            vec!["context set".into(), "codebase access".into()],
            OutputKind::Model,
            vec![ToolCategory::Search, ToolCategory::Analysis],
            vec![
                EscalationCondition::InsufficientContext,
                EscalationCondition::AmbiguityDetected,
            ],
        ),
        CognitiveOperation::LocateFocus => (
            vec!["model".into(), "objective".into()],
            OutputKind::FocusArea,
            vec![ToolCategory::Search, ToolCategory::VersionControl],
            vec![EscalationCondition::AmbiguityDetected],
        ),
        CognitiveOperation::AnalyzeStructure => (
            vec!["focus area".into(), "codebase access".into()],
            OutputKind::StructuralAnalysis,
            vec![ToolCategory::Analysis, ToolCategory::Search],
            vec![EscalationCondition::InsufficientContext],
        ),
        CognitiveOperation::TraceFlow => (
            vec!["focus area".into(), "codebase access".into()],
            OutputKind::FlowTrace,
            vec![
                ToolCategory::Search,
                ToolCategory::Analysis,
                ToolCategory::VersionControl,
            ],
            vec![EscalationCondition::InsufficientContext],
        ),
        CognitiveOperation::AssessImpact => (
            vec![
                "structural analysis or flow trace".into(),
                "change description".into(),
            ],
            OutputKind::ImpactAssessment,
            vec![
                ToolCategory::Analysis,
                ToolCategory::VersionControl,
                ToolCategory::Metrics,
            ],
            vec![EscalationCondition::RiskThresholdExceeded],
        ),
        CognitiveOperation::ShapeSolution => (
            vec![
                "objective".into(),
                "model".into(),
                "impact assessment".into(),
            ],
            OutputKind::SolutionDesign,
            vec![ToolCategory::Documentation],
            vec![
                EscalationCondition::DesignConflict,
                EscalationCondition::AmbiguityDetected,
            ],
        ),
        CognitiveOperation::DecomposeWork => (
            vec!["solution design".into(), "codebase model".into()],
            OutputKind::WorkPlan,
            vec![ToolCategory::Documentation],
            vec![EscalationCondition::AmbiguityDetected],
        ),
        CognitiveOperation::CreateArtifact => (
            vec!["work plan or task".into(), "codebase access".into()],
            OutputKind::Artifact,
            vec![ToolCategory::CodeModification, ToolCategory::Build],
            vec![EscalationCondition::DesignConflict],
        ),
        CognitiveOperation::ValidateReality => (
            vec!["artifact".into(), "test infrastructure".into()],
            OutputKind::ValidationResult,
            vec![
                ToolCategory::Testing,
                ToolCategory::Build,
                ToolCategory::Analysis,
            ],
            vec![EscalationCondition::ValidationFailed],
        ),
        CognitiveOperation::ReassessAdapt => (
            vec![
                "validation results or new information".into(),
                "current plan".into(),
            ],
            OutputKind::AdaptedPlan,
            vec![ToolCategory::Documentation],
            vec![EscalationCondition::IterationBudgetExhausted],
        ),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_operations_returns_12() {
        assert_eq!(CognitiveOperation::all().len(), 12);
    }

    #[test]
    fn test_operation_identifiers_are_unique() {
        let ids: Vec<&str> = CognitiveOperation::all()
            .iter()
            .map(super::CognitiveOperation::identifier)
            .collect();
        let mut deduped = ids.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(ids.len(), deduped.len());
    }

    #[test]
    fn test_operation_display_matches_identifier() {
        for op in CognitiveOperation::all() {
            assert_eq!(op.to_string(), op.identifier());
        }
    }

    #[test]
    fn test_operation_roundtrip_from_str() {
        for op in CognitiveOperation::all() {
            let parsed: CognitiveOperation = op.identifier().parse().unwrap();
            assert_eq!(*op, parsed);
        }
    }

    #[test]
    fn test_operation_from_str_aliases() {
        assert_eq!(
            "frame".parse::<CognitiveOperation>().unwrap(),
            CognitiveOperation::FrameObjective
        );
        assert_eq!(
            "acquire".parse::<CognitiveOperation>().unwrap(),
            CognitiveOperation::AcquireContext
        );
        assert_eq!(
            "model".parse::<CognitiveOperation>().unwrap(),
            CognitiveOperation::BuildModel
        );
        assert_eq!(
            "validate".parse::<CognitiveOperation>().unwrap(),
            CognitiveOperation::ValidateReality
        );
    }

    #[test]
    fn test_operation_from_str_invalid() {
        assert!("nonexistent".parse::<CognitiveOperation>().is_err());
    }

    #[test]
    fn test_operation_spec_defaults() {
        for op in CognitiveOperation::all() {
            let spec = OperationSpec::new(*op);
            assert_eq!(spec.operation, *op);
            assert!(
                !spec.input_requirements.is_empty(),
                "op {op:?} has no inputs"
            );
            assert!(!spec.tool_hints.is_empty(), "op {op:?} has no tool hints");
            assert!(
                !spec.escalation_conditions.is_empty(),
                "op {op:?} has no escalations"
            );
            assert!(!spec.description.is_empty());
        }
    }

    #[test]
    fn test_operation_spec_builder_methods() {
        let spec = OperationSpec::new(CognitiveOperation::FrameObjective)
            .with_description("Custom framing")
            .with_input("additional requirement")
            .with_tool_hint(ToolCategory::Search)
            .with_escalation(EscalationCondition::Custom("custom reason".into()));

        assert_eq!(spec.description, "Custom framing");
        assert!(spec
            .input_requirements
            .contains(&"additional requirement".to_string()));
        assert!(spec.tool_hints.contains(&ToolCategory::Search));
        assert!(spec.escalation_conditions.len() >= 2);
    }

    #[test]
    fn test_operation_spec_with_tool_hint_deduplicates() {
        let spec = OperationSpec::new(CognitiveOperation::AcquireContext)
            .with_tool_hint(ToolCategory::Search); // already in defaults
        let search_count = spec
            .tool_hints
            .iter()
            .filter(|t| **t == ToolCategory::Search)
            .count();
        assert_eq!(search_count, 1);
    }

    #[test]
    fn test_operation_serde_roundtrip() {
        let spec = OperationSpec::new(CognitiveOperation::TraceFlow);
        let json = serde_json::to_string(&spec).unwrap();
        let deserialized: OperationSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(spec, deserialized);
    }

    #[test]
    fn test_escalation_condition_display() {
        assert_eq!(
            EscalationCondition::InsufficientContext.to_string(),
            "insufficient_context"
        );
        assert_eq!(
            EscalationCondition::Custom("test".into()).to_string(),
            "custom: test"
        );
    }

    #[test]
    fn test_tool_category_display() {
        assert_eq!(ToolCategory::Search.to_string(), "search");
        assert_eq!(ToolCategory::VersionControl.to_string(), "version_control");
    }
}

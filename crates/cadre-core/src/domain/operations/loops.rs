//! Reusable loop composition types.
//!
//! Each loop is a composition of cognitive operations with entry conditions,
//! exit conditions, max iterations, and escalation rules.  Loops are the
//! mid-level building block between individual operations and full workflows.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use super::operation::{CognitiveOperation, EscalationCondition};

// ---------------------------------------------------------------------------
// LoopKind — the 11 reusable loops
// ---------------------------------------------------------------------------

/// The 11 reusable loops that compose cognitive operations into coherent
/// reasoning cycles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoopKind {
    /// Establish a clear, validated objective.
    ObjectiveFraming,
    /// Gather sufficient context for the work.
    ContextSufficiency,
    /// Build a working model of the relevant system.
    ModelConstruction,
    /// Narrow focus to the specific area of concern.
    FocusNarrowing,
    /// Trace flow and causality through the system.
    Trace,
    /// Assess risk and impact of proposed changes.
    RiskImpact,
    /// Shape and refine a solution approach.
    SolutionShaping,
    /// Decompose work into actionable steps.
    Decomposition,
    /// Produce concrete artifacts (code, config, docs).
    ArtifactProduction,
    /// Validate artifacts against reality.
    Validation,
    /// Adapt the plan based on new information.
    Adaptation,
}

impl LoopKind {
    /// Returns all 11 loop kinds in canonical order.
    pub fn all() -> &'static [LoopKind] {
        &[
            LoopKind::ObjectiveFraming,
            LoopKind::ContextSufficiency,
            LoopKind::ModelConstruction,
            LoopKind::FocusNarrowing,
            LoopKind::Trace,
            LoopKind::RiskImpact,
            LoopKind::SolutionShaping,
            LoopKind::Decomposition,
            LoopKind::ArtifactProduction,
            LoopKind::Validation,
            LoopKind::Adaptation,
        ]
    }

    /// Human-readable description of the loop.
    pub fn description(&self) -> &'static str {
        match self {
            Self::ObjectiveFraming => "Establish a clear, validated objective",
            Self::ContextSufficiency => "Gather sufficient context for the work",
            Self::ModelConstruction => "Build a working model of the relevant system",
            Self::FocusNarrowing => "Narrow focus to the specific area of concern",
            Self::Trace => "Trace flow and causality through the system",
            Self::RiskImpact => "Assess risk and impact of proposed changes",
            Self::SolutionShaping => "Shape and refine a solution approach",
            Self::Decomposition => "Decompose work into actionable steps",
            Self::ArtifactProduction => "Produce concrete artifacts",
            Self::Validation => "Validate artifacts against reality",
            Self::Adaptation => "Adapt the plan based on new information",
        }
    }

    /// Snake_case identifier for serialization.
    pub fn identifier(&self) -> &'static str {
        match self {
            Self::ObjectiveFraming => "objective_framing",
            Self::ContextSufficiency => "context_sufficiency",
            Self::ModelConstruction => "model_construction",
            Self::FocusNarrowing => "focus_narrowing",
            Self::Trace => "trace",
            Self::RiskImpact => "risk_impact",
            Self::SolutionShaping => "solution_shaping",
            Self::Decomposition => "decomposition",
            Self::ArtifactProduction => "artifact_production",
            Self::Validation => "validation",
            Self::Adaptation => "adaptation",
        }
    }

    /// Returns the default operations that compose this loop.
    pub fn default_operations(&self) -> Vec<CognitiveOperation> {
        match self {
            Self::ObjectiveFraming => vec![
                CognitiveOperation::FrameObjective,
                CognitiveOperation::AcquireContext,
                CognitiveOperation::FrameObjective, // re-frame after context
            ],
            Self::ContextSufficiency => vec![
                CognitiveOperation::AcquireContext,
                CognitiveOperation::BuildModel,
            ],
            Self::ModelConstruction => vec![
                CognitiveOperation::AcquireContext,
                CognitiveOperation::BuildModel,
                CognitiveOperation::AnalyzeStructure,
            ],
            Self::FocusNarrowing => vec![
                CognitiveOperation::LocateFocus,
                CognitiveOperation::AnalyzeStructure,
            ],
            Self::Trace => vec![
                CognitiveOperation::LocateFocus,
                CognitiveOperation::TraceFlow,
            ],
            Self::RiskImpact => vec![
                CognitiveOperation::AnalyzeStructure,
                CognitiveOperation::TraceFlow,
                CognitiveOperation::AssessImpact,
            ],
            Self::SolutionShaping => vec![
                CognitiveOperation::ShapeSolution,
                CognitiveOperation::AssessImpact,
                CognitiveOperation::ShapeSolution, // refine after impact
            ],
            Self::Decomposition => vec![
                CognitiveOperation::DecomposeWork,
                CognitiveOperation::AssessImpact,
            ],
            Self::ArtifactProduction => vec![
                CognitiveOperation::CreateArtifact,
                CognitiveOperation::ValidateReality,
            ],
            Self::Validation => vec![
                CognitiveOperation::ValidateReality,
                CognitiveOperation::ReassessAdapt,
            ],
            Self::Adaptation => vec![
                CognitiveOperation::ReassessAdapt,
                CognitiveOperation::FrameObjective,
            ],
        }
    }
}

impl fmt::Display for LoopKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.identifier())
    }
}

impl FromStr for LoopKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "objective_framing" | "objective" | "framing" => Ok(Self::ObjectiveFraming),
            "context_sufficiency" | "context" => Ok(Self::ContextSufficiency),
            "model_construction" | "model" => Ok(Self::ModelConstruction),
            "focus_narrowing" | "focus" => Ok(Self::FocusNarrowing),
            "trace" => Ok(Self::Trace),
            "risk_impact" | "risk" | "impact" => Ok(Self::RiskImpact),
            "solution_shaping" | "solution" => Ok(Self::SolutionShaping),
            "decomposition" | "decompose" => Ok(Self::Decomposition),
            "artifact_production" | "artifact" | "production" => Ok(Self::ArtifactProduction),
            "validation" | "validate" => Ok(Self::Validation),
            "adaptation" | "adapt" => Ok(Self::Adaptation),
            _ => Err(format!("Unknown loop kind: {}", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// Condition — entry and exit conditions for loops
// ---------------------------------------------------------------------------

/// A condition that controls loop entry or exit.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    /// Always true — loop can always enter/exit.
    Always,
    /// True when the specified artifact type exists.
    ArtifactExists(String),
    /// True when the specified artifact passes validation.
    ArtifactValid(String),
    /// True when sufficient context has been gathered (threshold-based).
    ContextSufficient,
    /// True when all specified operations have completed.
    OperationsComplete(Vec<CognitiveOperation>),
    /// True when risk assessment is below threshold.
    RiskAcceptable,
    /// True when all validations pass.
    AllValidationsPass,
    /// True when an objective has been established.
    ObjectiveEstablished,
    /// True when the model is complete enough to proceed.
    ModelSufficient,
    /// Custom condition with a description.
    Custom(String),
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Always => write!(f, "always"),
            Self::ArtifactExists(name) => write!(f, "artifact_exists({})", name),
            Self::ArtifactValid(name) => write!(f, "artifact_valid({})", name),
            Self::ContextSufficient => write!(f, "context_sufficient"),
            Self::OperationsComplete(ops) => {
                let names: Vec<&str> = ops.iter().map(|o| o.identifier()).collect();
                write!(f, "operations_complete({})", names.join(", "))
            }
            Self::RiskAcceptable => write!(f, "risk_acceptable"),
            Self::AllValidationsPass => write!(f, "all_validations_pass"),
            Self::ObjectiveEstablished => write!(f, "objective_established"),
            Self::ModelSufficient => write!(f, "model_sufficient"),
            Self::Custom(desc) => write!(f, "custom: {}", desc),
        }
    }
}

// ---------------------------------------------------------------------------
// LoopDefinition — full specification for a reusable loop
// ---------------------------------------------------------------------------

/// Full definition of a reusable loop, including its operations sequence,
/// entry/exit conditions, iteration limits, and escalation rules.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoopDefinition {
    /// Which of the 11 loops this definition describes.
    pub kind: LoopKind,
    /// Human-readable description.
    pub description: String,
    /// Condition that must be true to enter this loop.
    pub entry_condition: Condition,
    /// Operations executed in sequence within the loop.
    pub operations: Vec<CognitiveOperation>,
    /// Condition that must be true to exit the loop successfully.
    pub exit_condition: Condition,
    /// Maximum iterations before forced escalation.
    pub max_iterations: u32,
    /// Conditions that trigger escalation (breaking out of the loop).
    pub escalation_rules: Vec<EscalationCondition>,
}

impl LoopDefinition {
    /// Create a loop definition with default operations and sensible defaults.
    pub fn new(kind: LoopKind) -> Self {
        let (entry, exit, max_iter) = default_loop_conditions(kind);
        Self {
            description: kind.description().to_string(),
            operations: kind.default_operations(),
            kind,
            entry_condition: entry,
            exit_condition: exit,
            max_iterations: max_iter,
            escalation_rules: vec![EscalationCondition::IterationBudgetExhausted],
        }
    }

    /// Override the entry condition.
    pub fn with_entry_condition(mut self, condition: Condition) -> Self {
        self.entry_condition = condition;
        self
    }

    /// Override the exit condition.
    pub fn with_exit_condition(mut self, condition: Condition) -> Self {
        self.exit_condition = condition;
        self
    }

    /// Override the max iterations.
    pub fn with_max_iterations(mut self, max: u32) -> Self {
        self.max_iterations = max;
        self
    }

    /// Add an escalation rule.
    pub fn with_escalation_rule(mut self, rule: EscalationCondition) -> Self {
        self.escalation_rules.push(rule);
        self
    }

    /// Override the operations sequence.
    pub fn with_operations(mut self, ops: Vec<CognitiveOperation>) -> Self {
        self.operations = ops;
        self
    }
}

/// Returns default (entry_condition, exit_condition, max_iterations) per loop kind.
fn default_loop_conditions(kind: LoopKind) -> (Condition, Condition, u32) {
    match kind {
        LoopKind::ObjectiveFraming => (Condition::Always, Condition::ObjectiveEstablished, 3),
        LoopKind::ContextSufficiency => (
            Condition::ObjectiveEstablished,
            Condition::ContextSufficient,
            5,
        ),
        LoopKind::ModelConstruction => {
            (Condition::ContextSufficient, Condition::ModelSufficient, 3)
        }
        LoopKind::FocusNarrowing => (
            Condition::ModelSufficient,
            Condition::Custom("focus area identified".into()),
            3,
        ),
        LoopKind::Trace => (
            Condition::Custom("focus area identified".into()),
            Condition::Custom("flow traced to satisfaction".into()),
            5,
        ),
        LoopKind::RiskImpact => (
            Condition::Custom("change described".into()),
            Condition::RiskAcceptable,
            3,
        ),
        LoopKind::SolutionShaping => (
            Condition::Custom("problem understood".into()),
            Condition::Custom("solution design accepted".into()),
            5,
        ),
        LoopKind::Decomposition => (
            Condition::Custom("solution design accepted".into()),
            Condition::Custom("work plan complete".into()),
            3,
        ),
        LoopKind::ArtifactProduction => (
            Condition::Custom("work plan complete".into()),
            Condition::AllValidationsPass,
            10,
        ),
        LoopKind::Validation => (
            Condition::ArtifactExists("artifact".into()),
            Condition::AllValidationsPass,
            5,
        ),
        LoopKind::Adaptation => (
            Condition::Custom("new information available".into()),
            Condition::Custom("plan updated".into()),
            3,
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
    fn test_all_loops_returns_11() {
        assert_eq!(LoopKind::all().len(), 11);
    }

    #[test]
    fn test_loop_identifiers_are_unique() {
        let ids: Vec<&str> = LoopKind::all().iter().map(|l| l.identifier()).collect();
        let mut deduped = ids.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(ids.len(), deduped.len());
    }

    #[test]
    fn test_loop_display_matches_identifier() {
        for kind in LoopKind::all() {
            assert_eq!(kind.to_string(), kind.identifier());
        }
    }

    #[test]
    fn test_loop_roundtrip_from_str() {
        for kind in LoopKind::all() {
            let parsed: LoopKind = kind.identifier().parse().unwrap();
            assert_eq!(*kind, parsed);
        }
    }

    #[test]
    fn test_loop_from_str_aliases() {
        assert_eq!(
            "objective".parse::<LoopKind>().unwrap(),
            LoopKind::ObjectiveFraming
        );
        assert_eq!(
            "context".parse::<LoopKind>().unwrap(),
            LoopKind::ContextSufficiency
        );
        assert_eq!("risk".parse::<LoopKind>().unwrap(), LoopKind::RiskImpact);
        assert_eq!(
            "validate".parse::<LoopKind>().unwrap(),
            LoopKind::Validation
        );
    }

    #[test]
    fn test_loop_from_str_invalid() {
        assert!("nonexistent".parse::<LoopKind>().is_err());
    }

    #[test]
    fn test_loop_default_operations_are_nonempty() {
        for kind in LoopKind::all() {
            let ops = kind.default_operations();
            assert!(!ops.is_empty(), "loop {:?} has no default operations", kind);
        }
    }

    #[test]
    fn test_loop_definition_defaults() {
        for kind in LoopKind::all() {
            let def = LoopDefinition::new(*kind);
            assert_eq!(def.kind, *kind);
            assert!(!def.operations.is_empty());
            assert!(def.max_iterations > 0);
            assert!(!def.escalation_rules.is_empty());
            assert!(!def.description.is_empty());
        }
    }

    #[test]
    fn test_loop_definition_builder_methods() {
        let def = LoopDefinition::new(LoopKind::ObjectiveFraming)
            .with_entry_condition(Condition::Custom("custom entry".into()))
            .with_exit_condition(Condition::AllValidationsPass)
            .with_max_iterations(10)
            .with_escalation_rule(EscalationCondition::AmbiguityDetected)
            .with_operations(vec![CognitiveOperation::FrameObjective]);

        assert_eq!(
            def.entry_condition,
            Condition::Custom("custom entry".into())
        );
        assert_eq!(def.exit_condition, Condition::AllValidationsPass);
        assert_eq!(def.max_iterations, 10);
        assert_eq!(def.operations.len(), 1);
        assert!(def.escalation_rules.len() >= 2);
    }

    #[test]
    fn test_loop_definition_serde_roundtrip() {
        let def = LoopDefinition::new(LoopKind::Validation);
        let json = serde_json::to_string(&def).unwrap();
        let deserialized: LoopDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(def, deserialized);
    }

    #[test]
    fn test_condition_display() {
        assert_eq!(Condition::Always.to_string(), "always");
        assert_eq!(
            Condition::ContextSufficient.to_string(),
            "context_sufficient"
        );
        assert_eq!(
            Condition::ArtifactExists("foo".into()).to_string(),
            "artifact_exists(foo)"
        );
        assert_eq!(
            Condition::OperationsComplete(vec![CognitiveOperation::FrameObjective]).to_string(),
            "operations_complete(frame_objective)"
        );
    }
}

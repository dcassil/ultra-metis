//! Gate abstraction types for the autonomy model.
//!
//! Gates are typed checkpoints that control workflow progression.  Each gate
//! defines required evidence, failure behavior, and mode-dependent strictness.
//! Gates are abstract control points -- specific quality checks plug into them.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// ---------------------------------------------------------------------------
// GateType -- the 7 major gate types
// ---------------------------------------------------------------------------

/// The 7 major gate types that control workflow progression.
///
/// Each gate represents a conceptually distinct checkpoint.  Gates are
/// attachable to workflow templates rather than hardcoded to specific flows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateType {
    /// Validates that prerequisites are met before starting work.
    Entry,
    /// Validates that sufficient context has been gathered.
    ContextSufficiency,
    /// Validates that a solution design is sound before implementation.
    Solution,
    /// Validates that all prerequisites for execution are in place.
    ExecutionReadiness,
    /// Validates that implementation meets quality standards.
    Validation,
    /// Validates that all acceptance criteria and evidence requirements are met.
    Completion,
    /// A meta-gate that triggers escalation to a human.
    Escalation,
}

impl GateType {
    /// Returns all 7 gate types in canonical order.
    pub fn all() -> &'static [Self] {
        &[
            Self::Entry,
            Self::ContextSufficiency,
            Self::Solution,
            Self::ExecutionReadiness,
            Self::Validation,
            Self::Completion,
            Self::Escalation,
        ]
    }

    /// Human-readable description of the gate type.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Entry => "Validates prerequisites before starting work",
            Self::ContextSufficiency => "Validates sufficient context has been gathered",
            Self::Solution => "Validates solution design is sound before implementation",
            Self::ExecutionReadiness => "Validates all execution prerequisites are in place",
            Self::Validation => "Validates implementation meets quality standards",
            Self::Completion => "Validates acceptance criteria and evidence requirements",
            Self::Escalation => "Triggers escalation to a human decision-maker",
        }
    }

    /// Snake_case identifier for serialization.
    pub fn identifier(&self) -> &'static str {
        match self {
            Self::Entry => "entry",
            Self::ContextSufficiency => "context_sufficiency",
            Self::Solution => "solution",
            Self::ExecutionReadiness => "execution_readiness",
            Self::Validation => "validation",
            Self::Completion => "completion",
            Self::Escalation => "escalation",
        }
    }
}

impl fmt::Display for GateType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.identifier())
    }
}

impl FromStr for GateType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "entry" => Ok(Self::Entry),
            "context_sufficiency" | "context" => Ok(Self::ContextSufficiency),
            "solution" => Ok(Self::Solution),
            "execution_readiness" | "execution" | "ready" => Ok(Self::ExecutionReadiness),
            "validation" | "validate" => Ok(Self::Validation),
            "completion" | "complete" => Ok(Self::Completion),
            "escalation" | "escalate" => Ok(Self::Escalation),
            _ => Err(format!("Unknown gate type: {s}")),
        }
    }
}

// ---------------------------------------------------------------------------
// GateFailureBehavior -- what happens when a gate check fails
// ---------------------------------------------------------------------------

/// What happens when a gate check fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateFailureBehavior {
    /// Block progression entirely until the gate passes.
    Block,
    /// Warn but allow progression (advisory).
    Warn,
    /// Escalate to a human for a decision.
    EscalateToHuman,
}

impl fmt::Display for GateFailureBehavior {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Block => write!(f, "block"),
            Self::Warn => write!(f, "warn"),
            Self::EscalateToHuman => write!(f, "escalate_to_human"),
        }
    }
}

// ---------------------------------------------------------------------------
// EvidenceRequirement -- what must exist for a gate to pass
// ---------------------------------------------------------------------------

/// A specific piece of evidence required for a gate to pass.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EvidenceRequirement {
    /// Human-readable label for the evidence.
    pub label: String,
    /// The kind of artifact that constitutes this evidence.
    pub artifact_kind: String,
    /// Whether this evidence is strictly required or optional.
    pub required: bool,
}

impl EvidenceRequirement {
    /// Create a required evidence requirement.
    pub fn required(label: impl Into<String>, artifact_kind: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            artifact_kind: artifact_kind.into(),
            required: true,
        }
    }

    /// Create an optional evidence requirement.
    pub fn optional(label: impl Into<String>, artifact_kind: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            artifact_kind: artifact_kind.into(),
            required: false,
        }
    }
}

// ---------------------------------------------------------------------------
// GateDefinition -- full specification for a gate checkpoint
// ---------------------------------------------------------------------------

/// Full specification for a gate checkpoint, including evidence requirements,
/// failure behavior, and mode-dependent configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateDefinition {
    /// Which gate type this definition describes.
    pub gate_type: GateType,
    /// Human-readable name for this specific gate instance.
    pub name: String,
    /// Description of what this gate checks.
    pub description: String,
    /// Evidence that must exist for the gate to pass.
    pub evidence_requirements: Vec<EvidenceRequirement>,
    /// Default behavior on failure (may be overridden by autonomy mode).
    pub default_failure_behavior: GateFailureBehavior,
    /// Whether this gate can be skipped entirely (e.g., for low-risk work).
    pub skippable: bool,
}

impl GateDefinition {
    /// Create a new gate definition with sensible defaults for the given type.
    pub fn new(gate_type: GateType) -> Self {
        let (name, description, evidence, behavior, skippable) = default_gate(gate_type);
        Self {
            gate_type,
            name,
            description,
            evidence_requirements: evidence,
            default_failure_behavior: behavior,
            skippable,
        }
    }

    /// Override the gate name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Override the description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Add an evidence requirement.
    pub fn with_evidence(mut self, evidence: EvidenceRequirement) -> Self {
        self.evidence_requirements.push(evidence);
        self
    }

    /// Set the default failure behavior.
    pub fn with_failure_behavior(mut self, behavior: GateFailureBehavior) -> Self {
        self.default_failure_behavior = behavior;
        self
    }

    /// Set whether this gate is skippable.
    pub fn with_skippable(mut self, skippable: bool) -> Self {
        self.skippable = skippable;
        self
    }

    /// Returns the number of required evidence items.
    pub fn required_evidence_count(&self) -> usize {
        self.evidence_requirements
            .iter()
            .filter(|e| e.required)
            .count()
    }
}

/// Returns default (name, description, evidence, failure_behavior, skippable) for each gate type.
fn default_gate(
    gate_type: GateType,
) -> (
    String,
    String,
    Vec<EvidenceRequirement>,
    GateFailureBehavior,
    bool,
) {
    match gate_type {
        GateType::Entry => (
            "Entry Gate".into(),
            "Validates that prerequisites are met before starting work".into(),
            vec![
                EvidenceRequirement::required("work_item", "document"),
                EvidenceRequirement::required("objective", "text"),
            ],
            GateFailureBehavior::Block,
            false,
        ),
        GateType::ContextSufficiency => (
            "Context Sufficiency Gate".into(),
            "Validates that sufficient context has been gathered to proceed".into(),
            vec![
                EvidenceRequirement::required("context_set", "context_artifacts"),
                EvidenceRequirement::optional("architecture_model", "model"),
            ],
            GateFailureBehavior::EscalateToHuman,
            false,
        ),
        GateType::Solution => (
            "Solution Gate".into(),
            "Validates that the solution design is sound before implementation".into(),
            vec![
                EvidenceRequirement::required("solution_design", "design_document"),
                EvidenceRequirement::required("impact_assessment", "assessment"),
                EvidenceRequirement::optional("alternative_analysis", "analysis"),
            ],
            GateFailureBehavior::EscalateToHuman,
            false,
        ),
        GateType::ExecutionReadiness => (
            "Execution Readiness Gate".into(),
            "Validates that all prerequisites for execution are in place".into(),
            vec![
                EvidenceRequirement::required("work_plan", "plan"),
                EvidenceRequirement::required("approved_design", "design_document"),
            ],
            GateFailureBehavior::Block,
            false,
        ),
        GateType::Validation => (
            "Validation Gate".into(),
            "Validates that implementation meets quality standards".into(),
            vec![
                EvidenceRequirement::required("test_results", "test_output"),
                EvidenceRequirement::required("build_success", "build_output"),
                EvidenceRequirement::optional("lint_results", "lint_output"),
            ],
            GateFailureBehavior::Block,
            false,
        ),
        GateType::Completion => (
            "Completion Gate".into(),
            "Validates that all acceptance criteria and evidence requirements are met".into(),
            vec![
                EvidenceRequirement::required("all_validations_pass", "validation_record"),
                EvidenceRequirement::required("acceptance_criteria_met", "checklist"),
                EvidenceRequirement::optional("documentation_updated", "document"),
            ],
            GateFailureBehavior::Block,
            false,
        ),
        GateType::Escalation => (
            "Escalation Gate".into(),
            "Triggers escalation to a human decision-maker when conditions require it".into(),
            vec![
                EvidenceRequirement::required("escalation_reason", "text"),
                EvidenceRequirement::required("context_summary", "text"),
            ],
            GateFailureBehavior::EscalateToHuman,
            false,
        ),
    }
}

// ---------------------------------------------------------------------------
// GateCheckOutcome -- result of evaluating a gate
// ---------------------------------------------------------------------------

/// The outcome of evaluating a single gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateCheckOutcome {
    /// Which gate was checked.
    pub gate_type: GateType,
    /// Whether the gate passed.
    pub passed: bool,
    /// The behavior that should be applied (based on mode and gate config).
    pub behavior: GateFailureBehavior,
    /// Evidence items that were present.
    pub evidence_present: Vec<String>,
    /// Evidence items that were missing.
    pub evidence_missing: Vec<String>,
    /// Human-readable message explaining the outcome.
    pub message: String,
}

impl GateCheckOutcome {
    /// Create a passing outcome.
    pub fn pass(gate_type: GateType) -> Self {
        Self {
            gate_type,
            passed: true,
            behavior: GateFailureBehavior::Warn, // irrelevant for passing
            evidence_present: Vec::new(),
            evidence_missing: Vec::new(),
            message: format!("{gate_type} gate passed"),
        }
    }

    /// Create a failing outcome.
    pub fn fail(
        gate_type: GateType,
        behavior: GateFailureBehavior,
        missing: Vec<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            gate_type,
            passed: false,
            behavior,
            evidence_present: Vec::new(),
            evidence_missing: missing,
            message: message.into(),
        }
    }

    /// Whether this outcome requires human intervention.
    pub fn requires_human(&self) -> bool {
        !self.passed && self.behavior == GateFailureBehavior::EscalateToHuman
    }

    /// Whether this outcome blocks progression.
    pub fn blocks_progression(&self) -> bool {
        !self.passed
            && (self.behavior == GateFailureBehavior::Block
                || self.behavior == GateFailureBehavior::EscalateToHuman)
    }
}

impl fmt::Display for GateCheckOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.passed { "PASS" } else { "FAIL" };
        write!(f, "[{}] {}: {}", status, self.gate_type, self.message)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_gate_types_returns_7() {
        assert_eq!(GateType::all().len(), 7);
    }

    #[test]
    fn test_gate_type_identifiers_are_unique() {
        let ids: Vec<&str> = GateType::all().iter().map(super::GateType::identifier).collect();
        let mut deduped = ids.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(ids.len(), deduped.len());
    }

    #[test]
    fn test_gate_type_display_matches_identifier() {
        for gt in GateType::all() {
            assert_eq!(gt.to_string(), gt.identifier());
        }
    }

    #[test]
    fn test_gate_type_roundtrip_from_str() {
        for gt in GateType::all() {
            let parsed: GateType = gt.identifier().parse().unwrap();
            assert_eq!(*gt, parsed);
        }
    }

    #[test]
    fn test_gate_type_from_str_aliases() {
        assert_eq!(
            "context".parse::<GateType>().unwrap(),
            GateType::ContextSufficiency
        );
        assert_eq!(
            "ready".parse::<GateType>().unwrap(),
            GateType::ExecutionReadiness
        );
        assert_eq!(
            "validate".parse::<GateType>().unwrap(),
            GateType::Validation
        );
        assert_eq!(
            "complete".parse::<GateType>().unwrap(),
            GateType::Completion
        );
        assert_eq!(
            "escalate".parse::<GateType>().unwrap(),
            GateType::Escalation
        );
    }

    #[test]
    fn test_gate_type_from_str_invalid() {
        assert!("nonexistent".parse::<GateType>().is_err());
    }

    #[test]
    fn test_gate_definition_defaults() {
        for gt in GateType::all() {
            let def = GateDefinition::new(*gt);
            assert_eq!(def.gate_type, *gt);
            assert!(!def.name.is_empty());
            assert!(!def.description.is_empty());
            assert!(!def.evidence_requirements.is_empty());
        }
    }

    #[test]
    fn test_gate_definition_builder_methods() {
        let def = GateDefinition::new(GateType::Entry)
            .with_name("Custom Entry")
            .with_description("Custom description")
            .with_evidence(EvidenceRequirement::required("extra", "document"))
            .with_failure_behavior(GateFailureBehavior::Warn)
            .with_skippable(true);

        assert_eq!(def.name, "Custom Entry");
        assert_eq!(def.description, "Custom description");
        assert_eq!(def.default_failure_behavior, GateFailureBehavior::Warn);
        assert!(def.skippable);
        // 2 defaults + 1 added
        assert_eq!(def.evidence_requirements.len(), 3);
    }

    #[test]
    fn test_required_evidence_count() {
        let def = GateDefinition::new(GateType::Solution);
        // solution has 2 required + 1 optional
        assert_eq!(def.required_evidence_count(), 2);
    }

    #[test]
    fn test_evidence_requirement_constructors() {
        let req = EvidenceRequirement::required("test", "artifact");
        assert!(req.required);

        let opt = EvidenceRequirement::optional("test", "artifact");
        assert!(!opt.required);
    }

    #[test]
    fn test_gate_check_outcome_pass() {
        let outcome = GateCheckOutcome::pass(GateType::Entry);
        assert!(outcome.passed);
        assert!(!outcome.requires_human());
        assert!(!outcome.blocks_progression());
    }

    #[test]
    fn test_gate_check_outcome_fail_block() {
        let outcome = GateCheckOutcome::fail(
            GateType::Validation,
            GateFailureBehavior::Block,
            vec!["test_results".into()],
            "Missing test results",
        );
        assert!(!outcome.passed);
        assert!(!outcome.requires_human());
        assert!(outcome.blocks_progression());
    }

    #[test]
    fn test_gate_check_outcome_fail_escalate() {
        let outcome = GateCheckOutcome::fail(
            GateType::Solution,
            GateFailureBehavior::EscalateToHuman,
            vec!["design".into()],
            "Design needs review",
        );
        assert!(!outcome.passed);
        assert!(outcome.requires_human());
        assert!(outcome.blocks_progression());
    }

    #[test]
    fn test_gate_check_outcome_fail_warn() {
        let outcome = GateCheckOutcome::fail(
            GateType::Validation,
            GateFailureBehavior::Warn,
            vec!["lint_results".into()],
            "Lint results missing but not blocking",
        );
        assert!(!outcome.passed);
        assert!(!outcome.requires_human());
        assert!(!outcome.blocks_progression());
    }

    #[test]
    fn test_gate_check_outcome_display() {
        let pass = GateCheckOutcome::pass(GateType::Entry);
        assert!(pass.to_string().contains("[PASS]"));
        assert!(pass.to_string().contains("entry"));

        let fail = GateCheckOutcome::fail(
            GateType::Validation,
            GateFailureBehavior::Block,
            vec![],
            "Failed",
        );
        assert!(fail.to_string().contains("[FAIL]"));
    }

    #[test]
    fn test_gate_definition_serde_roundtrip() {
        let def = GateDefinition::new(GateType::ContextSufficiency);
        let json = serde_json::to_string(&def).unwrap();
        let deserialized: GateDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(def, deserialized);
    }

    #[test]
    fn test_gate_check_outcome_serde_roundtrip() {
        let outcome = GateCheckOutcome::fail(
            GateType::Solution,
            GateFailureBehavior::EscalateToHuman,
            vec!["design".into()],
            "Needs review",
        );
        let json = serde_json::to_string(&outcome).unwrap();
        let deserialized: GateCheckOutcome = serde_json::from_str(&json).unwrap();
        assert_eq!(outcome, deserialized);
    }

    #[test]
    fn test_gate_failure_behavior_display() {
        assert_eq!(GateFailureBehavior::Block.to_string(), "block");
        assert_eq!(GateFailureBehavior::Warn.to_string(), "warn");
        assert_eq!(
            GateFailureBehavior::EscalateToHuman.to_string(),
            "escalate_to_human"
        );
    }
}

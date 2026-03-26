//! Workflow template types.
//!
//! A workflow template defines a predefined composition of loops with entry
//! conditions, required artifacts, required validations, escalation rules,
//! and completion rules.  This is the highest-level composition abstraction.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use super::loops::{Condition, LoopDefinition, LoopKind};
use super::operation::EscalationCondition;

// ---------------------------------------------------------------------------
// WorkType — the kind of work a workflow template addresses
// ---------------------------------------------------------------------------

/// The kind of engineering work a workflow template is designed for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkType {
    /// Fix a known bug.
    Bugfix,
    /// Implement a new feature slice.
    Feature,
    /// Refactor existing code without changing behavior.
    Refactor,
    /// Investigate an issue or question.
    Investigation,
    /// Migrate to a new version, framework, or pattern.
    Migration,
    /// Make an architecture change.
    ArchitectureChange,
    /// Evaluate a brownfield (existing) codebase.
    BrownfieldEvaluation,
    /// Remediate technical debt or quality issues.
    Remediation,
    /// Bootstrap a greenfield project from scratch.
    GreenfieldBootstrap,
}

impl WorkType {
    /// Returns all work types in canonical order.
    pub fn all() -> &'static [Self] {
        &[
            Self::Bugfix,
            Self::Feature,
            Self::Refactor,
            Self::Investigation,
            Self::Migration,
            Self::ArchitectureChange,
            Self::BrownfieldEvaluation,
            Self::Remediation,
            Self::GreenfieldBootstrap,
        ]
    }

    /// Human-readable description of the work type.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Bugfix => "Fix a known bug",
            Self::Feature => "Implement a new feature slice",
            Self::Refactor => "Refactor existing code without changing behavior",
            Self::Investigation => "Investigate an issue or question",
            Self::Migration => "Migrate to a new version, framework, or pattern",
            Self::ArchitectureChange => "Make an architecture change",
            Self::BrownfieldEvaluation => "Evaluate an existing codebase",
            Self::Remediation => "Remediate technical debt or quality issues",
            Self::GreenfieldBootstrap => "Bootstrap a new project from scratch",
        }
    }

    /// Snake_case identifier for serialization.
    pub fn identifier(&self) -> &'static str {
        match self {
            Self::Bugfix => "bugfix",
            Self::Feature => "feature",
            Self::Refactor => "refactor",
            Self::Investigation => "investigation",
            Self::Migration => "migration",
            Self::ArchitectureChange => "architecture_change",
            Self::BrownfieldEvaluation => "brownfield_evaluation",
            Self::Remediation => "remediation",
            Self::GreenfieldBootstrap => "greenfield_bootstrap",
        }
    }
}

impl fmt::Display for WorkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.identifier())
    }
}

impl FromStr for WorkType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "bugfix" | "bug" | "fix" => Ok(Self::Bugfix),
            "feature" | "feat" => Ok(Self::Feature),
            "refactor" | "refactoring" => Ok(Self::Refactor),
            "investigation" | "investigate" | "research" => Ok(Self::Investigation),
            "migration" | "migrate" => Ok(Self::Migration),
            "architecture_change" | "arch_change" | "architecture" => Ok(Self::ArchitectureChange),
            "brownfield_evaluation" | "brownfield" | "eval" => Ok(Self::BrownfieldEvaluation),
            "remediation" | "remediate" | "tech_debt" => Ok(Self::Remediation),
            "greenfield_bootstrap" | "greenfield" | "bootstrap" => Ok(Self::GreenfieldBootstrap),
            _ => Err(format!("Unknown work type: {s}")),
        }
    }
}

// ---------------------------------------------------------------------------
// LoopStep — a loop within a workflow with optional overrides
// ---------------------------------------------------------------------------

/// A single step in a workflow, wrapping a loop definition with optional
/// workflow-specific overrides.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoopStep {
    /// The loop to execute at this step.
    pub loop_definition: LoopDefinition,
    /// Whether this step is required (vs optional/skippable).
    pub required: bool,
    /// Artifacts that must exist before entering this step.
    pub required_artifacts: Vec<String>,
    /// Artifacts that this step produces.
    pub produced_artifacts: Vec<String>,
}

impl LoopStep {
    /// Create a required loop step from a loop kind with defaults.
    pub fn required(kind: LoopKind) -> Self {
        Self {
            loop_definition: LoopDefinition::new(kind),
            required: true,
            required_artifacts: Vec::new(),
            produced_artifacts: Vec::new(),
        }
    }

    /// Create an optional loop step from a loop kind with defaults.
    pub fn optional(kind: LoopKind) -> Self {
        Self {
            loop_definition: LoopDefinition::new(kind),
            required: false,
            required_artifacts: Vec::new(),
            produced_artifacts: Vec::new(),
        }
    }

    /// Set required artifacts for this step.
    pub fn with_required_artifacts(mut self, artifacts: Vec<String>) -> Self {
        self.required_artifacts = artifacts;
        self
    }

    /// Set produced artifacts for this step.
    pub fn with_produced_artifacts(mut self, artifacts: Vec<String>) -> Self {
        self.produced_artifacts = artifacts;
        self
    }

    /// Override the inner loop definition.
    pub fn with_loop_definition(mut self, def: LoopDefinition) -> Self {
        self.loop_definition = def;
        self
    }
}

// ---------------------------------------------------------------------------
// CompletionRule — what constitutes "done" for a workflow
// ---------------------------------------------------------------------------

/// Rules that define when a workflow is considered complete.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionRule {
    /// All required loops must have exited successfully.
    AllRequiredLoopsComplete,
    /// All specified artifacts must exist.
    ArtifactsExist(Vec<String>),
    /// All validations must pass.
    AllValidationsPass,
    /// A specific gate must be satisfied.
    GateSatisfied(String),
    /// Custom completion condition.
    Custom(String),
}

impl fmt::Display for CompletionRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AllRequiredLoopsComplete => write!(f, "all_required_loops_complete"),
            Self::ArtifactsExist(artifacts) => {
                write!(f, "artifacts_exist({})", artifacts.join(", "))
            }
            Self::AllValidationsPass => write!(f, "all_validations_pass"),
            Self::GateSatisfied(gate) => write!(f, "gate_satisfied({gate})"),
            Self::Custom(desc) => write!(f, "custom: {desc}"),
        }
    }
}

// ---------------------------------------------------------------------------
// WorkflowTemplate — the top-level workflow abstraction
// ---------------------------------------------------------------------------

/// A complete workflow template that defines a structured approach to a
/// specific type of engineering work.
///
/// A workflow is a sequence of loop steps with entry conditions, completion
/// rules, and escalation rules.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    /// What kind of work this workflow addresses.
    pub work_type: WorkType,
    /// Human-readable name.
    pub name: String,
    /// Description of the workflow.
    pub description: String,
    /// Condition that must be true to start this workflow.
    pub entry_condition: Condition,
    /// Ordered sequence of loop steps.
    pub steps: Vec<LoopStep>,
    /// Rules that define when the workflow is complete.
    pub completion_rules: Vec<CompletionRule>,
    /// Global escalation rules for the entire workflow.
    pub escalation_rules: Vec<EscalationCondition>,
}

impl WorkflowTemplate {
    /// Create a new workflow template.
    pub fn new(work_type: WorkType) -> Self {
        Self {
            name: format!("{} workflow", work_type.description()),
            description: work_type.description().to_string(),
            work_type,
            entry_condition: Condition::Always,
            steps: Vec::new(),
            completion_rules: vec![
                CompletionRule::AllRequiredLoopsComplete,
                CompletionRule::AllValidationsPass,
            ],
            escalation_rules: vec![EscalationCondition::IterationBudgetExhausted],
        }
    }

    /// Set the workflow name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set the entry condition.
    pub fn with_entry_condition(mut self, condition: Condition) -> Self {
        self.entry_condition = condition;
        self
    }

    /// Add a loop step to the workflow.
    pub fn with_step(mut self, step: LoopStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Add a completion rule.
    pub fn with_completion_rule(mut self, rule: CompletionRule) -> Self {
        self.completion_rules.push(rule);
        self
    }

    /// Add a global escalation rule.
    pub fn with_escalation_rule(mut self, rule: EscalationCondition) -> Self {
        self.escalation_rules.push(rule);
        self
    }

    /// Returns the loop kinds used in this workflow, in order.
    pub fn loop_sequence(&self) -> Vec<LoopKind> {
        self.steps.iter().map(|s| s.loop_definition.kind).collect()
    }

    /// Returns the total number of steps.
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Returns the number of required steps.
    pub fn required_step_count(&self) -> usize {
        self.steps.iter().filter(|s| s.required).count()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_work_types_returns_9() {
        assert_eq!(WorkType::all().len(), 9);
    }

    #[test]
    fn test_work_type_identifiers_are_unique() {
        let ids: Vec<&str> = WorkType::all().iter().map(super::WorkType::identifier).collect();
        let mut deduped = ids.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(ids.len(), deduped.len());
    }

    #[test]
    fn test_work_type_roundtrip_from_str() {
        for wt in WorkType::all() {
            let parsed: WorkType = wt.identifier().parse().unwrap();
            assert_eq!(*wt, parsed);
        }
    }

    #[test]
    fn test_work_type_from_str_aliases() {
        assert_eq!("bug".parse::<WorkType>().unwrap(), WorkType::Bugfix);
        assert_eq!("feat".parse::<WorkType>().unwrap(), WorkType::Feature);
        assert_eq!(
            "greenfield".parse::<WorkType>().unwrap(),
            WorkType::GreenfieldBootstrap
        );
    }

    #[test]
    fn test_work_type_from_str_invalid() {
        assert!("nonexistent".parse::<WorkType>().is_err());
    }

    #[test]
    fn test_loop_step_required() {
        let step = LoopStep::required(LoopKind::ObjectiveFraming);
        assert!(step.required);
        assert_eq!(step.loop_definition.kind, LoopKind::ObjectiveFraming);
    }

    #[test]
    fn test_loop_step_optional() {
        let step = LoopStep::optional(LoopKind::Adaptation);
        assert!(!step.required);
    }

    #[test]
    fn test_loop_step_with_artifacts() {
        let step = LoopStep::required(LoopKind::Validation)
            .with_required_artifacts(vec!["code_changes".into()])
            .with_produced_artifacts(vec!["test_results".into()]);
        assert_eq!(step.required_artifacts, vec!["code_changes".to_string()]);
        assert_eq!(step.produced_artifacts, vec!["test_results".to_string()]);
    }

    #[test]
    fn test_workflow_template_creation() {
        let wf = WorkflowTemplate::new(WorkType::Bugfix)
            .with_name("Bugfix Workflow")
            .with_step(LoopStep::required(LoopKind::ObjectiveFraming))
            .with_step(LoopStep::required(LoopKind::ContextSufficiency));

        assert_eq!(wf.work_type, WorkType::Bugfix);
        assert_eq!(wf.name, "Bugfix Workflow");
        assert_eq!(wf.step_count(), 2);
        assert_eq!(wf.required_step_count(), 2);
        assert_eq!(
            wf.loop_sequence(),
            vec![LoopKind::ObjectiveFraming, LoopKind::ContextSufficiency]
        );
    }

    #[test]
    fn test_workflow_template_mixed_steps() {
        let wf = WorkflowTemplate::new(WorkType::Feature)
            .with_step(LoopStep::required(LoopKind::ObjectiveFraming))
            .with_step(LoopStep::optional(LoopKind::Adaptation));

        assert_eq!(wf.step_count(), 2);
        assert_eq!(wf.required_step_count(), 1);
    }

    #[test]
    fn test_workflow_template_serde_roundtrip() {
        let wf = WorkflowTemplate::new(WorkType::Refactor)
            .with_step(LoopStep::required(LoopKind::ModelConstruction))
            .with_step(LoopStep::required(LoopKind::ArtifactProduction))
            .with_step(LoopStep::required(LoopKind::Validation));

        let json = serde_json::to_string(&wf).unwrap();
        let deserialized: WorkflowTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(wf, deserialized);
    }

    #[test]
    fn test_completion_rule_display() {
        assert_eq!(
            CompletionRule::AllRequiredLoopsComplete.to_string(),
            "all_required_loops_complete"
        );
        assert_eq!(
            CompletionRule::GateSatisfied("quality".into()).to_string(),
            "gate_satisfied(quality)"
        );
    }
}

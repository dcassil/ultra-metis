//! Built-in workflow templates.
//!
//! Provides pre-defined workflow templates for common engineering work types.
//! These are data definitions — the templates describe the intended loop
//! sequence and conditions, but do not execute anything.

use super::loops::{Condition, LoopKind};
use super::operation::EscalationCondition;
use super::workflow::{CompletionRule, LoopStep, WorkType, WorkflowTemplate};

/// Returns the built-in bugfix workflow template.
///
/// Bugfix: frame the bug -> gather context -> trace the issue -> assess risk
/// -> fix -> validate -> adapt if needed.
pub fn bugfix_template() -> WorkflowTemplate {
    WorkflowTemplate::new(WorkType::Bugfix)
        .with_name("Bugfix")
        .with_description(
            "Structured workflow for diagnosing and fixing a known bug. \
             Emphasizes tracing causality and validating the fix.",
        )
        .with_entry_condition(Condition::ArtifactExists("bug_report".into()))
        .with_step(
            LoopStep::required(LoopKind::ObjectiveFraming)
                .with_produced_artifacts(vec!["objective".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::ContextSufficiency)
                .with_required_artifacts(vec!["objective".into()])
                .with_produced_artifacts(vec!["context_set".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::FocusNarrowing)
                .with_required_artifacts(vec!["context_set".into()])
                .with_produced_artifacts(vec!["focus_area".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::Trace)
                .with_required_artifacts(vec!["focus_area".into()])
                .with_produced_artifacts(vec!["root_cause".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::RiskImpact)
                .with_required_artifacts(vec!["root_cause".into()])
                .with_produced_artifacts(vec!["impact_assessment".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::ArtifactProduction)
                .with_required_artifacts(vec!["root_cause".into()])
                .with_produced_artifacts(vec!["fix".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::Validation)
                .with_required_artifacts(vec!["fix".into()])
                .with_produced_artifacts(vec!["validation_results".into()]),
        )
        .with_step(LoopStep::optional(LoopKind::Adaptation))
        .with_completion_rule(CompletionRule::AllValidationsPass)
        .with_escalation_rule(EscalationCondition::RiskThresholdExceeded)
}

/// Returns the built-in feature workflow template.
///
/// Feature: frame objective -> gather context -> build model -> design ->
/// decompose -> produce -> validate -> adapt.
pub fn feature_template() -> WorkflowTemplate {
    WorkflowTemplate::new(WorkType::Feature)
        .with_name("Feature Slice")
        .with_description(
            "Structured workflow for implementing a new feature slice. \
             Emphasizes understanding existing structure before building.",
        )
        .with_step(
            LoopStep::required(LoopKind::ObjectiveFraming)
                .with_produced_artifacts(vec!["objective".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::ContextSufficiency)
                .with_required_artifacts(vec!["objective".into()])
                .with_produced_artifacts(vec!["context_set".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::ModelConstruction)
                .with_required_artifacts(vec!["context_set".into()])
                .with_produced_artifacts(vec!["system_model".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::SolutionShaping)
                .with_required_artifacts(vec!["system_model".into()])
                .with_produced_artifacts(vec!["solution_design".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::Decomposition)
                .with_required_artifacts(vec!["solution_design".into()])
                .with_produced_artifacts(vec!["work_plan".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::ArtifactProduction)
                .with_required_artifacts(vec!["work_plan".into()])
                .with_produced_artifacts(vec!["implementation".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::Validation)
                .with_required_artifacts(vec!["implementation".into()])
                .with_produced_artifacts(vec!["validation_results".into()]),
        )
        .with_step(LoopStep::optional(LoopKind::Adaptation))
        .with_completion_rule(CompletionRule::AllValidationsPass)
}

/// Returns the built-in refactor workflow template.
///
/// Refactor: frame objective -> build model -> analyze structure -> assess risk
/// -> produce -> validate (behavior preserved).
pub fn refactor_template() -> WorkflowTemplate {
    WorkflowTemplate::new(WorkType::Refactor)
        .with_name("Refactor")
        .with_description(
            "Structured workflow for refactoring existing code without changing \
             behavior. Emphasizes structural analysis and behavior preservation.",
        )
        .with_step(
            LoopStep::required(LoopKind::ObjectiveFraming)
                .with_produced_artifacts(vec!["objective".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::ModelConstruction)
                .with_required_artifacts(vec!["objective".into()])
                .with_produced_artifacts(vec!["system_model".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::FocusNarrowing)
                .with_required_artifacts(vec!["system_model".into()])
                .with_produced_artifacts(vec!["focus_area".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::RiskImpact)
                .with_required_artifacts(vec!["focus_area".into()])
                .with_produced_artifacts(vec!["impact_assessment".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::ArtifactProduction)
                .with_required_artifacts(vec!["impact_assessment".into()])
                .with_produced_artifacts(vec!["refactored_code".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::Validation)
                .with_required_artifacts(vec!["refactored_code".into()])
                .with_produced_artifacts(vec!["validation_results".into()]),
        )
        .with_step(LoopStep::optional(LoopKind::Adaptation))
        .with_completion_rule(CompletionRule::AllValidationsPass)
        .with_completion_rule(CompletionRule::Custom(
            "Behavior is preserved — all existing tests pass".into(),
        ))
}

/// Returns the built-in investigation workflow template.
///
/// Investigation: frame question -> gather context -> build model -> trace ->
/// document findings.
pub fn investigation_template() -> WorkflowTemplate {
    WorkflowTemplate::new(WorkType::Investigation)
        .with_name("Investigation")
        .with_description(
            "Structured workflow for investigating an issue or question. \
             Emphasizes model building and flow tracing; may not produce code changes.",
        )
        .with_step(
            LoopStep::required(LoopKind::ObjectiveFraming)
                .with_produced_artifacts(vec!["question".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::ContextSufficiency)
                .with_required_artifacts(vec!["question".into()])
                .with_produced_artifacts(vec!["context_set".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::ModelConstruction)
                .with_required_artifacts(vec!["context_set".into()])
                .with_produced_artifacts(vec!["system_model".into()]),
        )
        .with_step(
            LoopStep::optional(LoopKind::Trace)
                .with_required_artifacts(vec!["system_model".into()])
                .with_produced_artifacts(vec!["trace_results".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::ArtifactProduction)
                .with_produced_artifacts(vec!["findings_document".into()]),
        )
        .with_completion_rule(CompletionRule::ArtifactsExist(vec![
            "findings_document".into()
        ]))
}

/// Returns the built-in migration workflow template.
///
/// Migration: frame objective -> build model of current state -> assess impact
/// -> decompose migration steps -> produce -> validate.
pub fn migration_template() -> WorkflowTemplate {
    WorkflowTemplate::new(WorkType::Migration)
        .with_name("Migration")
        .with_description(
            "Structured workflow for migrating to a new version, framework, or \
             pattern. Emphasizes impact assessment and incremental production.",
        )
        .with_step(
            LoopStep::required(LoopKind::ObjectiveFraming)
                .with_produced_artifacts(vec!["migration_objective".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::ModelConstruction)
                .with_required_artifacts(vec!["migration_objective".into()])
                .with_produced_artifacts(vec!["current_state_model".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::RiskImpact)
                .with_required_artifacts(vec!["current_state_model".into()])
                .with_produced_artifacts(vec!["migration_impact".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::Decomposition)
                .with_required_artifacts(vec!["migration_impact".into()])
                .with_produced_artifacts(vec!["migration_plan".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::ArtifactProduction)
                .with_required_artifacts(vec!["migration_plan".into()])
                .with_produced_artifacts(vec!["migrated_code".into()]),
        )
        .with_step(
            LoopStep::required(LoopKind::Validation)
                .with_required_artifacts(vec!["migrated_code".into()])
                .with_produced_artifacts(vec!["validation_results".into()]),
        )
        .with_step(LoopStep::optional(LoopKind::Adaptation))
        .with_completion_rule(CompletionRule::AllValidationsPass)
        .with_escalation_rule(EscalationCondition::RiskThresholdExceeded)
}

/// Returns all built-in workflow templates.
pub fn all_templates() -> Vec<WorkflowTemplate> {
    vec![
        bugfix_template(),
        feature_template(),
        refactor_template(),
        investigation_template(),
        migration_template(),
    ]
}

/// Look up a built-in workflow template by work type.
pub fn template_for_work_type(work_type: WorkType) -> Option<WorkflowTemplate> {
    all_templates()
        .into_iter()
        .find(|t| t.work_type == work_type)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_templates_count() {
        assert_eq!(all_templates().len(), 5);
    }

    #[test]
    fn test_all_templates_have_unique_work_types() {
        let types: Vec<WorkType> = all_templates().iter().map(|t| t.work_type).collect();
        let mut deduped = types.clone();
        deduped.sort_by_key(super::super::workflow::WorkType::identifier);
        deduped.dedup();
        assert_eq!(types.len(), deduped.len());
    }

    #[test]
    fn test_all_templates_have_steps() {
        for template in all_templates() {
            assert!(
                !template.steps.is_empty(),
                "template {:?} has no steps",
                template.work_type
            );
        }
    }

    #[test]
    fn test_all_templates_have_completion_rules() {
        for template in all_templates() {
            assert!(
                !template.completion_rules.is_empty(),
                "template {:?} has no completion rules",
                template.work_type
            );
        }
    }

    #[test]
    fn test_bugfix_template_structure() {
        let t = bugfix_template();
        assert_eq!(t.work_type, WorkType::Bugfix);
        assert_eq!(t.name, "Bugfix");
        assert!(t.step_count() >= 7);
        assert!(t.required_step_count() >= 6);
        // Bugfix should include trace and validation loops
        let kinds = t.loop_sequence();
        assert!(kinds.contains(&LoopKind::Trace));
        assert!(kinds.contains(&LoopKind::Validation));
    }

    #[test]
    fn test_feature_template_structure() {
        let t = feature_template();
        assert_eq!(t.work_type, WorkType::Feature);
        let kinds = t.loop_sequence();
        assert!(kinds.contains(&LoopKind::SolutionShaping));
        assert!(kinds.contains(&LoopKind::Decomposition));
        assert!(kinds.contains(&LoopKind::ArtifactProduction));
    }

    #[test]
    fn test_refactor_template_preserves_behavior() {
        let t = refactor_template();
        assert_eq!(t.work_type, WorkType::Refactor);
        // Should have a custom completion rule about behavior preservation
        let has_behavior_rule = t
            .completion_rules
            .iter()
            .any(|r| matches!(r, CompletionRule::Custom(s) if s.contains("Behavior")));
        assert!(has_behavior_rule);
    }

    #[test]
    fn test_investigation_template_produces_findings() {
        let t = investigation_template();
        assert_eq!(t.work_type, WorkType::Investigation);
        let has_findings_rule = t.completion_rules.iter().any(|r| {
            matches!(r, CompletionRule::ArtifactsExist(a) if a.contains(&"findings_document".to_string()))
        });
        assert!(has_findings_rule);
    }

    #[test]
    fn test_migration_template_has_risk_assessment() {
        let t = migration_template();
        assert_eq!(t.work_type, WorkType::Migration);
        let kinds = t.loop_sequence();
        assert!(kinds.contains(&LoopKind::RiskImpact));
    }

    #[test]
    fn test_template_for_work_type_found() {
        for wt in &[
            WorkType::Bugfix,
            WorkType::Feature,
            WorkType::Refactor,
            WorkType::Investigation,
            WorkType::Migration,
        ] {
            let t = template_for_work_type(*wt);
            assert!(t.is_some(), "no template for {wt:?}");
            assert_eq!(t.unwrap().work_type, *wt);
        }
    }

    #[test]
    fn test_template_for_work_type_not_found() {
        // These work types don't have templates yet
        assert!(template_for_work_type(WorkType::GreenfieldBootstrap).is_none());
        assert!(template_for_work_type(WorkType::ArchitectureChange).is_none());
    }

    #[test]
    fn test_all_templates_serde_roundtrip() {
        for template in all_templates() {
            let json = serde_json::to_string(&template).unwrap();
            let deserialized: WorkflowTemplate = serde_json::from_str(&json).unwrap();
            assert_eq!(template, deserialized);
        }
    }
}

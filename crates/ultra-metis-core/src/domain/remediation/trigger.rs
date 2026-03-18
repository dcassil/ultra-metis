//! Investigation trigger engine.
//!
//! Converts quality gate failures into remediation loops with appropriate
//! actions (create investigation, create remediation record, etc.).

use super::types::{FailedMetric, RemediationAction, RemediationLoop, RemediationTrigger};
use crate::domain::documents::quality_gate_config::GateSeverity;
use crate::domain::quality::gate_engine::{GateCheckResult, MetricCheckResult};

/// Engine that creates remediation loops from quality gate failures.
pub struct InvestigationTriggerEngine;

impl InvestigationTriggerEngine {
    /// Evaluate a gate check result and, if it failed, create a remediation loop
    /// with recommended actions.
    ///
    /// Returns `None` if the gate check passed (no remediation needed).
    pub fn evaluate(
        result: &GateCheckResult,
        gate_config_ref: &str,
        from_phase: Option<&str>,
        to_phase: Option<&str>,
        loop_id: &str,
    ) -> Option<RemediationLoop> {
        if result.passed {
            return None;
        }

        let failed_metrics: Vec<FailedMetric> = result
            .blocking_failures
            .iter()
            .chain(result.advisory_failures.iter())
            .map(|m| metric_check_to_failed_metric(m))
            .collect();

        let trigger = RemediationTrigger::GateFailure {
            gate_config_ref: gate_config_ref.to_string(),
            failed_metrics,
            from_phase: from_phase.map(String::from),
            to_phase: to_phase.map(String::from),
        };

        let mut remediation_loop = RemediationLoop::new(loop_id.to_string(), trigger);

        // Determine actions based on failure characteristics
        let actions = Self::determine_actions(result, gate_config_ref);
        for action in actions {
            remediation_loop.add_action(action);
        }

        Some(remediation_loop)
    }

    /// Determine what actions to recommend based on the failure pattern.
    fn determine_actions(
        result: &GateCheckResult,
        gate_config_ref: &str,
    ) -> Vec<RemediationAction> {
        let mut actions = Vec::new();
        let blocking_count = result.blocking_failures.len();

        if blocking_count > 0 {
            // Multiple blocking failures suggest systemic issue — create investigation
            let is_systemic = blocking_count >= 2;

            let suggested_title = if blocking_count == 1 {
                format!(
                    "Investigate {} threshold violation",
                    result.blocking_failures[0].metric
                )
            } else {
                format!(
                    "Investigate {} blocking quality gate failures",
                    blocking_count
                )
            };

            actions.push(RemediationAction::CreateInvestigation {
                suggested_title,
                trigger_refs: vec![gate_config_ref.to_string()],
            });

            // Also create a remediation record for tracking
            let affected_metrics: Vec<String> = result
                .blocking_failures
                .iter()
                .map(|f| f.metric.clone())
                .collect();

            actions.push(RemediationAction::CreateRemediationRecord {
                problem_type: "quality-gate-failure".to_string(),
                affected_scope: affected_metrics.join(", "),
                is_systemic,
            });
        }

        // If there are only advisory failures, suggest a quality re-check
        if blocking_count == 0 && !result.advisory_failures.is_empty() {
            actions.push(RemediationAction::RerunQualityCheck {
                gate_config_ref: gate_config_ref.to_string(),
            });
        }

        actions
    }

    /// Create a remediation loop from a manual trigger.
    pub fn manual_trigger(loop_id: &str, triggered_by: &str, reason: &str) -> RemediationLoop {
        let trigger = RemediationTrigger::Manual {
            triggered_by: triggered_by.to_string(),
            reason: reason.to_string(),
        };
        RemediationLoop::new(loop_id.to_string(), trigger)
    }

    /// Create a remediation loop from trend degradation.
    pub fn trend_trigger(
        loop_id: &str,
        degrading_metrics: Vec<String>,
        consecutive_regressions: u32,
    ) -> RemediationLoop {
        let trigger = RemediationTrigger::TrendDegradation {
            degrading_metrics: degrading_metrics.clone(),
            consecutive_regressions,
        };

        let mut rl = RemediationLoop::new(loop_id.to_string(), trigger);

        // If many consecutive regressions, suggest investigation
        if consecutive_regressions >= 3 {
            rl.add_action(RemediationAction::CreateInvestigation {
                suggested_title: format!(
                    "Investigate persistent degradation in {} metrics",
                    degrading_metrics.len()
                ),
                trigger_refs: vec![],
            });
        }

        rl
    }
}

/// Convert a MetricCheckResult into a FailedMetric.
fn metric_check_to_failed_metric(m: &MetricCheckResult) -> FailedMetric {
    FailedMetric {
        metric_name: m.metric.clone(),
        threshold: match &m.threshold {
            crate::domain::documents::quality_gate_config::ThresholdType::Absolute(v) => *v,
            crate::domain::documents::quality_gate_config::ThresholdType::RelativeRegression(v) => {
                *v
            }
            crate::domain::documents::quality_gate_config::ThresholdType::Trend(_) => 0.0,
        },
        actual_value: m.actual_value,
        delta: m.delta,
        is_blocking: m.severity == GateSeverity::Blocking,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::documents::quality_gate_config::{
        GateSeverity, MetricGateRule, QualityGateConfig, ThresholdType,
    };
    use crate::domain::documents::types::{Phase, Tag};
    use crate::domain::quality::gate_engine::GateCheckEngine;
    use std::collections::HashMap;

    fn metrics(pairs: &[(&str, f64)]) -> HashMap<String, f64> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    fn make_failing_config() -> QualityGateConfig {
        QualityGateConfig::new(
            "Test Gates".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-QGC-0001".to_string(),
            GateSeverity::Blocking,
            vec![
                MetricGateRule::blocking_absolute("lint_errors", 10.0),
                MetricGateRule::advisory_absolute("info_count", 50.0),
            ],
            vec![],
        )
        .unwrap()
    }

    #[test]
    fn test_evaluate_passing_returns_none() {
        let config = make_failing_config();
        let current = metrics(&[("lint_errors", 5.0), ("info_count", 30.0)]);

        let result = GateCheckEngine::check(&current, &config, None, None, None, None);
        let rl = InvestigationTriggerEngine::evaluate(&result, "QGC-0001", None, None, "RL-001");
        assert!(rl.is_none());
    }

    #[test]
    fn test_evaluate_blocking_failure_creates_loop() {
        let config = make_failing_config();
        let current = metrics(&[("lint_errors", 15.0), ("info_count", 30.0)]);

        let result = GateCheckEngine::check(&current, &config, None, None, None, None);
        let rl = InvestigationTriggerEngine::evaluate(
            &result,
            "QGC-0001",
            Some("active"),
            Some("completed"),
            "RL-001",
        );

        assert!(rl.is_some());
        let rl = rl.unwrap();
        assert_eq!(rl.id, "RL-001");
        // Should have both CreateInvestigation and CreateRemediationRecord
        assert!(rl.actions.len() >= 2);
        assert!(rl
            .actions
            .iter()
            .any(|a| matches!(a, RemediationAction::CreateInvestigation { .. })));
        assert!(rl
            .actions
            .iter()
            .any(|a| matches!(a, RemediationAction::CreateRemediationRecord { .. })));
    }

    #[test]
    fn test_evaluate_advisory_only_suggests_recheck() {
        let config = make_failing_config();
        let current = metrics(&[("lint_errors", 5.0), ("info_count", 60.0)]);

        let result = GateCheckEngine::check(&current, &config, None, None, None, None);
        // Advisory failures don't block — result.passed is true
        // So evaluate returns None
        let rl = InvestigationTriggerEngine::evaluate(&result, "QGC-0001", None, None, "RL-001");
        assert!(rl.is_none()); // passed gate, advisory only
    }

    #[test]
    fn test_manual_trigger() {
        let rl =
            InvestigationTriggerEngine::manual_trigger("RL-002", "engineer", "Code smell detected");
        assert_eq!(rl.id, "RL-002");
        match &rl.trigger {
            RemediationTrigger::Manual {
                triggered_by,
                reason,
            } => {
                assert_eq!(triggered_by, "engineer");
                assert_eq!(reason, "Code smell detected");
            }
            _ => panic!("Expected Manual trigger"),
        }
    }

    #[test]
    fn test_trend_trigger_with_enough_regressions() {
        let rl = InvestigationTriggerEngine::trend_trigger(
            "RL-003",
            vec!["errors".to_string(), "warnings".to_string()],
            3,
        );
        // Should have investigation action when >= 3 consecutive regressions
        assert_eq!(rl.actions.len(), 1);
        assert!(matches!(
            &rl.actions[0],
            RemediationAction::CreateInvestigation { .. }
        ));
    }

    #[test]
    fn test_trend_trigger_few_regressions_no_action() {
        let rl = InvestigationTriggerEngine::trend_trigger("RL-004", vec!["errors".to_string()], 2);
        // < 3 consecutive regressions: no automatic investigation
        assert!(rl.actions.is_empty());
    }

    #[test]
    fn test_multiple_blocking_failures_marked_systemic() {
        let config = QualityGateConfig::new(
            "Multi Test".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-QGC-0002".to_string(),
            GateSeverity::Blocking,
            vec![
                MetricGateRule::blocking_absolute("lint_errors", 10.0),
                MetricGateRule::blocking_absolute("total_warnings", 5.0),
            ],
            vec![],
        )
        .unwrap();

        let current = metrics(&[("lint_errors", 15.0), ("total_warnings", 10.0)]);
        let result = GateCheckEngine::check(&current, &config, None, None, None, None);

        let rl = InvestigationTriggerEngine::evaluate(&result, "QGC-0002", None, None, "RL-005");
        let rl = rl.unwrap();

        // Check that remediation record has is_systemic = true
        let rr_action = rl
            .actions
            .iter()
            .find(|a| matches!(a, RemediationAction::CreateRemediationRecord { .. }));
        match rr_action.unwrap() {
            RemediationAction::CreateRemediationRecord { is_systemic, .. } => {
                assert!(
                    is_systemic,
                    "Multiple blocking failures should be flagged as systemic"
                );
            }
            _ => unreachable!(),
        }
    }
}

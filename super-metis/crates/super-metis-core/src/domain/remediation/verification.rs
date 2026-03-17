//! Verification engine for remediation loops.
//!
//! After remediation is complete, re-checks quality metrics to confirm
//! the issue has been resolved. Handles pass/fail/retry logic.

use super::types::{RemediationAction, RemediationLoop, RemediationLoopPhase};
use crate::domain::quality::gate_engine::GateCheckResult;
use serde::{Deserialize, Serialize};

/// Outcome of a verification check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationOutcome {
    /// All quality checks pass — remediation was successful.
    Passed,
    /// Quality checks still fail — remediation was not sufficient.
    Failed {
        /// Which metrics still fail.
        still_failing: Vec<String>,
    },
    /// Verification budget exhausted — escalate to human.
    BudgetExhausted {
        /// Total attempts made.
        attempts: u32,
    },
}

/// Engine that verifies whether remediation resolved the quality issues.
pub struct VerificationEngine;

impl VerificationEngine {
    /// Check if a remediation was successful by evaluating a new gate check result.
    ///
    /// Automatically transitions the loop to the appropriate phase:
    /// - Passed -> Resolved
    /// - Failed (budget remaining) -> Investigating (loop back)
    /// - Failed (budget exhausted) -> Closed with escalation action
    pub fn verify(
        loop_state: &mut RemediationLoop,
        gate_result: &GateCheckResult,
    ) -> VerificationOutcome {
        loop_state.record_verification_attempt();

        if gate_result.passed {
            // Quality restored
            let _ = loop_state.transition(RemediationLoopPhase::Resolved);
            VerificationOutcome::Passed
        } else if loop_state.verification_budget_exhausted() {
            // Too many failed attempts — close and escalate
            loop_state.add_action(RemediationAction::EscalateToHuman {
                reason: format!(
                    "Remediation verification failed after {} attempts",
                    loop_state.verification_attempts
                ),
                context: std::collections::HashMap::new(),
            });
            let _ = loop_state.transition(RemediationLoopPhase::Closed);
            VerificationOutcome::BudgetExhausted {
                attempts: loop_state.verification_attempts,
            }
        } else {
            // Still failing — loop back to investigating
            let still_failing: Vec<String> = gate_result
                .blocking_failures
                .iter()
                .map(|f| f.metric.clone())
                .collect();
            let _ = loop_state.transition(RemediationLoopPhase::Investigating);
            VerificationOutcome::Failed { still_failing }
        }
    }

    /// Check whether a loop is ready for verification.
    /// A loop must be in the Verifying phase.
    pub fn can_verify(loop_state: &RemediationLoop) -> bool {
        loop_state.phase == RemediationLoopPhase::Verifying
    }

    /// Advance a loop through the full remediation path to the Verifying phase
    /// for testing/scripting convenience. Returns an error if any transition fails.
    pub fn advance_to_verifying(loop_state: &mut RemediationLoop) -> Result<(), String> {
        if loop_state.phase == RemediationLoopPhase::Triggered {
            loop_state.transition(RemediationLoopPhase::Investigating)?;
        }
        if loop_state.phase == RemediationLoopPhase::Investigating {
            loop_state.transition(RemediationLoopPhase::ProposalReady)?;
        }
        if loop_state.phase == RemediationLoopPhase::ProposalReady {
            loop_state.transition(RemediationLoopPhase::Remediating)?;
        }
        if loop_state.phase == RemediationLoopPhase::Remediating {
            loop_state.transition(RemediationLoopPhase::Verifying)?;
        }
        Ok(())
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
    use crate::domain::remediation::types::{RemediationTrigger, FailedMetric};
    use std::collections::HashMap;

    fn metrics(pairs: &[(&str, f64)]) -> HashMap<String, f64> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    fn make_config() -> QualityGateConfig {
        QualityGateConfig::new(
            "Test Gates".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-QGC-0001".to_string(),
            GateSeverity::Blocking,
            vec![MetricGateRule::blocking_absolute("lint_errors", 10.0)],
            vec![],
        )
        .unwrap()
    }

    fn make_loop_at_verifying() -> RemediationLoop {
        let trigger = RemediationTrigger::GateFailure {
            gate_config_ref: "QGC-0001".to_string(),
            failed_metrics: vec![FailedMetric::new("lint_errors", 10.0, 15.0, true)],
            from_phase: None,
            to_phase: None,
        };
        let mut rl = RemediationLoop::new("RL-001".to_string(), trigger);
        VerificationEngine::advance_to_verifying(&mut rl).unwrap();
        rl
    }

    #[test]
    fn test_verify_passed() {
        let config = make_config();
        let current = metrics(&[("lint_errors", 5.0)]); // below threshold
        let result = GateCheckEngine::check(&current, &config, None, None, None, None);

        let mut rl = make_loop_at_verifying();
        let outcome = VerificationEngine::verify(&mut rl, &result);

        assert_eq!(outcome, VerificationOutcome::Passed);
        assert_eq!(rl.phase, RemediationLoopPhase::Resolved);
        assert!(rl.is_complete());
    }

    #[test]
    fn test_verify_failed_loops_back() {
        let config = make_config();
        let current = metrics(&[("lint_errors", 15.0)]); // still over threshold
        let result = GateCheckEngine::check(&current, &config, None, None, None, None);

        let mut rl = make_loop_at_verifying();
        let outcome = VerificationEngine::verify(&mut rl, &result);

        match outcome {
            VerificationOutcome::Failed { still_failing } => {
                assert_eq!(still_failing, vec!["lint_errors"]);
            }
            _ => panic!("Expected Failed outcome"),
        }
        assert_eq!(rl.phase, RemediationLoopPhase::Investigating);
        assert_eq!(rl.verification_attempts, 1);
    }

    #[test]
    fn test_verify_budget_exhausted() {
        let config = make_config();
        let current = metrics(&[("lint_errors", 15.0)]); // still failing
        let result = GateCheckEngine::check(&current, &config, None, None, None, None);

        let mut rl = make_loop_at_verifying();
        rl.max_verification_attempts = 1; // exhaust after 1 attempt

        let outcome = VerificationEngine::verify(&mut rl, &result);
        assert!(matches!(outcome, VerificationOutcome::BudgetExhausted { attempts: 1 }));
        assert_eq!(rl.phase, RemediationLoopPhase::Closed);
        // Should have escalation action
        assert!(rl.actions.iter().any(|a| matches!(a, RemediationAction::EscalateToHuman { .. })));
    }

    #[test]
    fn test_can_verify() {
        let trigger = RemediationTrigger::Manual {
            triggered_by: "test".to_string(),
            reason: "test".to_string(),
        };
        let mut rl = RemediationLoop::new("RL-002".to_string(), trigger);
        assert!(!VerificationEngine::can_verify(&rl));

        VerificationEngine::advance_to_verifying(&mut rl).unwrap();
        assert!(VerificationEngine::can_verify(&rl));
    }

    #[test]
    fn test_advance_to_verifying() {
        let trigger = RemediationTrigger::Manual {
            triggered_by: "test".to_string(),
            reason: "test".to_string(),
        };
        let mut rl = RemediationLoop::new("RL-003".to_string(), trigger);
        VerificationEngine::advance_to_verifying(&mut rl).unwrap();
        assert_eq!(rl.phase, RemediationLoopPhase::Verifying);
    }

    #[test]
    fn test_full_remediation_cycle_with_retry() {
        let config = make_config();
        let mut rl = make_loop_at_verifying();

        // First verification fails
        let failing = metrics(&[("lint_errors", 15.0)]);
        let result = GateCheckEngine::check(&failing, &config, None, None, None, None);
        let outcome = VerificationEngine::verify(&mut rl, &result);
        assert!(matches!(outcome, VerificationOutcome::Failed { .. }));
        assert_eq!(rl.phase, RemediationLoopPhase::Investigating);

        // Re-remediate and re-verify
        VerificationEngine::advance_to_verifying(&mut rl).unwrap();

        // Second verification passes
        let passing = metrics(&[("lint_errors", 5.0)]);
        let result = GateCheckEngine::check(&passing, &config, None, None, None, None);
        let outcome = VerificationEngine::verify(&mut rl, &result);
        assert_eq!(outcome, VerificationOutcome::Passed);
        assert_eq!(rl.phase, RemediationLoopPhase::Resolved);
    }
}

//! Core types for the remediation loop engine.
//!
//! Defines triggers (what caused remediation), actions (what should be done),
//! and the overall remediation loop lifecycle.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// RemediationTrigger — what caused the remediation loop to start
// ---------------------------------------------------------------------------

/// The event that triggered a remediation loop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RemediationTrigger {
    /// A quality gate check failed with blocking violations.
    GateFailure {
        /// Short code of the quality gate config that was checked.
        gate_config_ref: String,
        /// Metrics that failed their thresholds.
        failed_metrics: Vec<FailedMetric>,
        /// The transition that was blocked (e.g., "active" -> "completed").
        from_phase: Option<String>,
        to_phase: Option<String>,
    },
    /// Quality trend is degrading even if gates haven't failed yet.
    TrendDegradation {
        /// Metrics showing negative trends.
        degrading_metrics: Vec<String>,
        /// Number of consecutive degradations observed.
        consecutive_regressions: u32,
    },
    /// Manual trigger by a human or agent.
    Manual {
        /// Who triggered the investigation.
        triggered_by: String,
        /// Reason for triggering.
        reason: String,
    },
}

impl fmt::Display for RemediationTrigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GateFailure {
                gate_config_ref,
                failed_metrics,
                ..
            } => {
                write!(
                    f,
                    "gate_failure({}, {} metrics failed)",
                    gate_config_ref,
                    failed_metrics.len()
                )
            }
            Self::TrendDegradation {
                degrading_metrics,
                consecutive_regressions,
            } => {
                write!(
                    f,
                    "trend_degradation({} metrics, {} consecutive)",
                    degrading_metrics.len(),
                    consecutive_regressions
                )
            }
            Self::Manual { triggered_by, .. } => {
                write!(f, "manual({triggered_by})")
            }
        }
    }
}

/// A metric that failed its quality gate threshold.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailedMetric {
    /// Name of the metric.
    pub metric_name: String,
    /// The threshold value that was exceeded.
    pub threshold: f64,
    /// The actual value observed.
    pub actual_value: f64,
    /// How far over the threshold (actual - threshold).
    pub delta: f64,
    /// Whether this was a blocking or advisory failure.
    pub is_blocking: bool,
}

impl FailedMetric {
    pub fn new(metric_name: &str, threshold: f64, actual_value: f64, is_blocking: bool) -> Self {
        Self {
            metric_name: metric_name.to_string(),
            threshold,
            actual_value,
            delta: actual_value - threshold,
            is_blocking,
        }
    }
}

// ---------------------------------------------------------------------------
// RemediationAction — what should be done in response
// ---------------------------------------------------------------------------

/// An action to be taken as part of remediation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RemediationAction {
    /// Create an architecture investigation document.
    CreateInvestigation {
        /// Suggested title for the investigation.
        suggested_title: String,
        /// References to link as triggers.
        trigger_refs: Vec<String>,
    },
    /// Create a remediation record to track the fix.
    CreateRemediationRecord {
        /// Problem type classification.
        problem_type: String,
        /// Scope of affected code.
        affected_scope: String,
        /// Whether this appears to be a systemic issue.
        is_systemic: bool,
    },
    /// Flag for human review — the system cannot determine the right action.
    EscalateToHuman {
        /// Why automated remediation cannot proceed.
        reason: String,
        /// Context to present to the human.
        context: HashMap<String, String>,
    },
    /// Re-run quality checks to verify the issue still exists.
    RerunQualityCheck {
        /// Which gate config to re-check against.
        gate_config_ref: String,
    },
}

impl fmt::Display for RemediationAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateInvestigation {
                suggested_title, ..
            } => {
                write!(f, "create_investigation({suggested_title})")
            }
            Self::CreateRemediationRecord { problem_type, .. } => {
                write!(f, "create_remediation_record({problem_type})")
            }
            Self::EscalateToHuman { reason, .. } => {
                write!(f, "escalate_to_human({reason})")
            }
            Self::RerunQualityCheck { gate_config_ref } => {
                write!(f, "rerun_quality_check({gate_config_ref})")
            }
        }
    }
}

// ---------------------------------------------------------------------------
// RemediationLoopState — lifecycle of a remediation loop
// ---------------------------------------------------------------------------

/// Current state of a remediation loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemediationLoopPhase {
    /// Trigger detected, loop is being initiated.
    Triggered,
    /// Investigation is underway.
    Investigating,
    /// A remediation proposal has been made.
    ProposalReady,
    /// Remediation is being executed.
    Remediating,
    /// Verification is in progress (re-checking quality).
    Verifying,
    /// Remediation was successful — quality restored.
    Resolved,
    /// Remediation was not successful or was abandoned.
    Closed,
}

impl RemediationLoopPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Triggered => "triggered",
            Self::Investigating => "investigating",
            Self::ProposalReady => "proposal_ready",
            Self::Remediating => "remediating",
            Self::Verifying => "verifying",
            Self::Resolved => "resolved",
            Self::Closed => "closed",
        }
    }

    /// Returns valid next phases from the current phase.
    pub fn valid_transitions(&self) -> &'static [Self] {
        match self {
            Self::Triggered => &[Self::Investigating, Self::Closed],
            Self::Investigating => &[Self::ProposalReady, Self::Closed],
            Self::ProposalReady => &[Self::Remediating, Self::Investigating, Self::Closed],
            Self::Remediating => &[Self::Verifying, Self::Closed],
            Self::Verifying => &[Self::Resolved, Self::Investigating, Self::Closed],
            Self::Resolved => &[],
            Self::Closed => &[],
        }
    }

    /// Whether this is a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Resolved | Self::Closed)
    }
}

impl fmt::Display for RemediationLoopPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for RemediationLoopPhase {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "triggered" => Ok(Self::Triggered),
            "investigating" => Ok(Self::Investigating),
            "proposal_ready" => Ok(Self::ProposalReady),
            "remediating" => Ok(Self::Remediating),
            "verifying" => Ok(Self::Verifying),
            "resolved" => Ok(Self::Resolved),
            "closed" => Ok(Self::Closed),
            _ => Err(format!("Unknown remediation loop phase: {s}")),
        }
    }
}

// ---------------------------------------------------------------------------
// RemediationLoop — the full loop state
// ---------------------------------------------------------------------------

/// A remediation loop tracks the full lifecycle from quality degradation
/// detection through investigation, remediation, and verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationLoop {
    /// Unique identifier for this loop instance.
    pub id: String,
    /// What triggered this remediation loop.
    pub trigger: RemediationTrigger,
    /// Current phase of the loop.
    pub phase: RemediationLoopPhase,
    /// Actions recommended or taken.
    pub actions: Vec<RemediationAction>,
    /// Short codes of linked investigation documents.
    pub investigation_refs: Vec<String>,
    /// Short codes of linked remediation record documents.
    pub remediation_refs: Vec<String>,
    /// When the loop was created.
    pub created_at: DateTime<Utc>,
    /// When the loop was last updated.
    pub updated_at: DateTime<Utc>,
    /// Number of verification attempts made.
    pub verification_attempts: u32,
    /// Maximum verification attempts before escalation.
    pub max_verification_attempts: u32,
}

impl RemediationLoop {
    /// Create a new remediation loop from a trigger.
    pub fn new(id: String, trigger: RemediationTrigger) -> Self {
        let now = Utc::now();
        Self {
            id,
            trigger,
            phase: RemediationLoopPhase::Triggered,
            actions: Vec::new(),
            investigation_refs: Vec::new(),
            remediation_refs: Vec::new(),
            created_at: now,
            updated_at: now,
            verification_attempts: 0,
            max_verification_attempts: 3,
        }
    }

    /// Transition to a new phase, returning an error if the transition is invalid.
    pub fn transition(&mut self, to: RemediationLoopPhase) -> Result<(), String> {
        let valid = self.phase.valid_transitions();
        if valid.contains(&to) {
            self.phase = to;
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(format!(
                "Cannot transition from {} to {} (valid: {:?})",
                self.phase,
                to,
                valid.iter().map(RemediationLoopPhase::as_str).collect::<Vec<_>>()
            ))
        }
    }

    /// Add an action to this loop.
    pub fn add_action(&mut self, action: RemediationAction) {
        self.actions.push(action);
        self.updated_at = Utc::now();
    }

    /// Link an investigation document to this loop.
    pub fn link_investigation(&mut self, short_code: &str) {
        if !self.investigation_refs.contains(&short_code.to_string()) {
            self.investigation_refs.push(short_code.to_string());
            self.updated_at = Utc::now();
        }
    }

    /// Link a remediation record document to this loop.
    pub fn link_remediation_record(&mut self, short_code: &str) {
        if !self.remediation_refs.contains(&short_code.to_string()) {
            self.remediation_refs.push(short_code.to_string());
            self.updated_at = Utc::now();
        }
    }

    /// Record a verification attempt.
    pub fn record_verification_attempt(&mut self) {
        self.verification_attempts += 1;
        self.updated_at = Utc::now();
    }

    /// Whether the loop has exhausted its verification budget.
    pub fn verification_budget_exhausted(&self) -> bool {
        self.verification_attempts >= self.max_verification_attempts
    }

    /// Whether the loop is in a terminal state.
    pub fn is_complete(&self) -> bool {
        self.phase.is_terminal()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_gate_failure_trigger() -> RemediationTrigger {
        RemediationTrigger::GateFailure {
            gate_config_ref: "QGC-0001".to_string(),
            failed_metrics: vec![
                FailedMetric::new("lint_errors", 10.0, 15.0, true),
                FailedMetric::new("total_warnings", 20.0, 25.0, false),
            ],
            from_phase: Some("active".to_string()),
            to_phase: Some("completed".to_string()),
        }
    }

    fn make_loop() -> RemediationLoop {
        RemediationLoop::new("RL-001".to_string(), make_gate_failure_trigger())
    }

    #[test]
    fn test_failed_metric_creation() {
        let fm = FailedMetric::new("lint_errors", 10.0, 15.0, true);
        assert_eq!(fm.metric_name, "lint_errors");
        assert_eq!(fm.threshold, 10.0);
        assert_eq!(fm.actual_value, 15.0);
        assert_eq!(fm.delta, 5.0);
        assert!(fm.is_blocking);
    }

    #[test]
    fn test_trigger_display() {
        let trigger = make_gate_failure_trigger();
        let display = trigger.to_string();
        assert!(display.contains("gate_failure"));
        assert!(display.contains("QGC-0001"));
        assert!(display.contains("2 metrics failed"));
    }

    #[test]
    fn test_trigger_trend_degradation_display() {
        let trigger = RemediationTrigger::TrendDegradation {
            degrading_metrics: vec!["errors".to_string(), "warnings".to_string()],
            consecutive_regressions: 3,
        };
        let display = trigger.to_string();
        assert!(display.contains("trend_degradation"));
        assert!(display.contains("2 metrics"));
        assert!(display.contains("3 consecutive"));
    }

    #[test]
    fn test_trigger_manual_display() {
        let trigger = RemediationTrigger::Manual {
            triggered_by: "engineer".to_string(),
            reason: "Spotted code smell".to_string(),
        };
        assert!(trigger.to_string().contains("manual(engineer)"));
    }

    #[test]
    fn test_remediation_loop_creation() {
        let rl = make_loop();
        assert_eq!(rl.id, "RL-001");
        assert_eq!(rl.phase, RemediationLoopPhase::Triggered);
        assert!(rl.actions.is_empty());
        assert!(rl.investigation_refs.is_empty());
        assert_eq!(rl.verification_attempts, 0);
        assert!(!rl.is_complete());
    }

    #[test]
    fn test_valid_phase_transitions() {
        let mut rl = make_loop();
        assert!(rl.transition(RemediationLoopPhase::Investigating).is_ok());
        assert_eq!(rl.phase, RemediationLoopPhase::Investigating);

        assert!(rl.transition(RemediationLoopPhase::ProposalReady).is_ok());
        assert!(rl.transition(RemediationLoopPhase::Remediating).is_ok());
        assert!(rl.transition(RemediationLoopPhase::Verifying).is_ok());
        assert!(rl.transition(RemediationLoopPhase::Resolved).is_ok());
        assert!(rl.is_complete());
    }

    #[test]
    fn test_invalid_phase_transition() {
        let mut rl = make_loop();
        // Cannot skip from Triggered to Verifying
        let err = rl.transition(RemediationLoopPhase::Verifying).unwrap_err();
        assert!(err.contains("Cannot transition"));
    }

    #[test]
    fn test_verification_loop_back() {
        let mut rl = make_loop();
        rl.transition(RemediationLoopPhase::Investigating).unwrap();
        rl.transition(RemediationLoopPhase::ProposalReady).unwrap();
        rl.transition(RemediationLoopPhase::Remediating).unwrap();
        rl.transition(RemediationLoopPhase::Verifying).unwrap();

        // Verification failed — loop back to investigating
        rl.record_verification_attempt();
        assert!(rl.transition(RemediationLoopPhase::Investigating).is_ok());
        assert_eq!(rl.verification_attempts, 1);
    }

    #[test]
    fn test_verification_budget() {
        let mut rl = make_loop();
        assert!(!rl.verification_budget_exhausted());
        rl.record_verification_attempt();
        rl.record_verification_attempt();
        assert!(!rl.verification_budget_exhausted());
        rl.record_verification_attempt();
        assert!(rl.verification_budget_exhausted());
    }

    #[test]
    fn test_close_from_any_non_terminal() {
        // Can close from Triggered
        let mut rl1 = make_loop();
        assert!(rl1.transition(RemediationLoopPhase::Closed).is_ok());

        // Can close from Investigating
        let mut rl2 = make_loop();
        rl2.transition(RemediationLoopPhase::Investigating).unwrap();
        assert!(rl2.transition(RemediationLoopPhase::Closed).is_ok());
    }

    #[test]
    fn test_cannot_transition_from_terminal() {
        let mut rl = make_loop();
        rl.transition(RemediationLoopPhase::Closed).unwrap();
        let err = rl
            .transition(RemediationLoopPhase::Investigating)
            .unwrap_err();
        assert!(err.contains("Cannot transition"));
    }

    #[test]
    fn test_add_action() {
        let mut rl = make_loop();
        rl.add_action(RemediationAction::CreateInvestigation {
            suggested_title: "Investigate lint errors spike".to_string(),
            trigger_refs: vec!["QGC-0001".to_string()],
        });
        assert_eq!(rl.actions.len(), 1);
    }

    #[test]
    fn test_link_investigation_deduplicates() {
        let mut rl = make_loop();
        rl.link_investigation("AI-0001");
        rl.link_investigation("AI-0001"); // duplicate
        rl.link_investigation("AI-0002");
        assert_eq!(rl.investigation_refs.len(), 2);
    }

    #[test]
    fn test_link_remediation_record_deduplicates() {
        let mut rl = make_loop();
        rl.link_remediation_record("RR-0001");
        rl.link_remediation_record("RR-0001"); // duplicate
        assert_eq!(rl.remediation_refs.len(), 1);
    }

    #[test]
    fn test_phase_roundtrip() {
        for phase in &[
            RemediationLoopPhase::Triggered,
            RemediationLoopPhase::Investigating,
            RemediationLoopPhase::ProposalReady,
            RemediationLoopPhase::Remediating,
            RemediationLoopPhase::Verifying,
            RemediationLoopPhase::Resolved,
            RemediationLoopPhase::Closed,
        ] {
            let s = phase.as_str();
            let parsed: RemediationLoopPhase = s.parse().unwrap();
            assert_eq!(parsed, *phase);
        }
    }

    #[test]
    fn test_phase_terminal_states() {
        assert!(!RemediationLoopPhase::Triggered.is_terminal());
        assert!(!RemediationLoopPhase::Investigating.is_terminal());
        assert!(!RemediationLoopPhase::ProposalReady.is_terminal());
        assert!(!RemediationLoopPhase::Remediating.is_terminal());
        assert!(!RemediationLoopPhase::Verifying.is_terminal());
        assert!(RemediationLoopPhase::Resolved.is_terminal());
        assert!(RemediationLoopPhase::Closed.is_terminal());
    }

    #[test]
    fn test_action_display() {
        let action = RemediationAction::CreateInvestigation {
            suggested_title: "Test".to_string(),
            trigger_refs: vec![],
        };
        assert!(action.to_string().contains("create_investigation"));

        let action2 = RemediationAction::EscalateToHuman {
            reason: "too complex".to_string(),
            context: HashMap::new(),
        };
        assert!(action2.to_string().contains("escalate_to_human"));
    }

    #[test]
    fn test_remediation_loop_serde_roundtrip() {
        let mut rl = make_loop();
        rl.add_action(RemediationAction::CreateInvestigation {
            suggested_title: "Investigate".to_string(),
            trigger_refs: vec!["REF-001".to_string()],
        });
        rl.link_investigation("AI-0001");

        let json = serde_json::to_string(&rl).unwrap();
        let deserialized: RemediationLoop = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, rl.id);
        assert_eq!(deserialized.phase, rl.phase);
        assert_eq!(deserialized.actions.len(), 1);
        assert_eq!(deserialized.investigation_refs, vec!["AI-0001"]);
    }
}

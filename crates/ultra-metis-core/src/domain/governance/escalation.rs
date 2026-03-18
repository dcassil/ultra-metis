//! Escalation trigger types and detection logic.
//!
//! Defines the conditions under which the system should pause and involve a
//! human.  Each trigger is a typed condition with severity, context, and an
//! actionable message for the human.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// ---------------------------------------------------------------------------
// EscalationTrigger -- the 9 escalation trigger types
// ---------------------------------------------------------------------------

/// Typed conditions that trigger escalation to a human.
///
/// When the system detects any of these conditions, it should pause the
/// current workflow and surface the issue to a human decision-maker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationTrigger {
    /// Insufficient evidence exists for the current gate to pass.
    InsufficientEvidence,
    /// An unresolved contradiction between artifacts or rules.
    UnresolvedContradiction,
    /// A rule says one thing but context suggests another.
    PolicyConflict,
    /// The change affects many files, crosses module boundaries, etc.
    HighImpactChange,
    /// The proposed change violates the reference architecture.
    ArchitectureMismatch,
    /// A security or safety concern has been detected.
    SecuritySafetyConcern,
    /// A required validation is failing.
    FailingRequiredValidation,
    /// The system's uncertainty exceeds the allowed threshold for the mode.
    UncertaintyAboveThreshold,
    /// Ambiguity in user/business requirements with material impact.
    BusinessAmbiguity,
}

impl EscalationTrigger {
    /// Returns all 9 trigger types in canonical order.
    pub fn all() -> &'static [EscalationTrigger] {
        &[
            EscalationTrigger::InsufficientEvidence,
            EscalationTrigger::UnresolvedContradiction,
            EscalationTrigger::PolicyConflict,
            EscalationTrigger::HighImpactChange,
            EscalationTrigger::ArchitectureMismatch,
            EscalationTrigger::SecuritySafetyConcern,
            EscalationTrigger::FailingRequiredValidation,
            EscalationTrigger::UncertaintyAboveThreshold,
            EscalationTrigger::BusinessAmbiguity,
        ]
    }

    /// Human-readable description of the trigger.
    pub fn description(&self) -> &'static str {
        match self {
            Self::InsufficientEvidence => "Insufficient evidence for the current gate",
            Self::UnresolvedContradiction => "Unresolved contradiction between artifacts or rules",
            Self::PolicyConflict => "Policy conflict: rule says X, context suggests Y",
            Self::HighImpactChange => "High-impact change affecting many files or boundaries",
            Self::ArchitectureMismatch => "Proposed change violates reference architecture",
            Self::SecuritySafetyConcern => "Security or safety concern detected",
            Self::FailingRequiredValidation => "A required validation is failing",
            Self::UncertaintyAboveThreshold => "Uncertainty exceeds allowed threshold for mode",
            Self::BusinessAmbiguity => "Business/user ambiguity with material impact",
        }
    }

    /// Snake_case identifier for serialization.
    pub fn identifier(&self) -> &'static str {
        match self {
            Self::InsufficientEvidence => "insufficient_evidence",
            Self::UnresolvedContradiction => "unresolved_contradiction",
            Self::PolicyConflict => "policy_conflict",
            Self::HighImpactChange => "high_impact_change",
            Self::ArchitectureMismatch => "architecture_mismatch",
            Self::SecuritySafetyConcern => "security_safety_concern",
            Self::FailingRequiredValidation => "failing_required_validation",
            Self::UncertaintyAboveThreshold => "uncertainty_above_threshold",
            Self::BusinessAmbiguity => "business_ambiguity",
        }
    }

    /// Default severity for this trigger type.
    pub fn default_severity(&self) -> EscalationSeverity {
        match self {
            Self::SecuritySafetyConcern => EscalationSeverity::Critical,
            Self::ArchitectureMismatch | Self::HighImpactChange => EscalationSeverity::High,
            Self::PolicyConflict
            | Self::UnresolvedContradiction
            | Self::FailingRequiredValidation
            | Self::UncertaintyAboveThreshold => EscalationSeverity::Medium,
            Self::InsufficientEvidence | Self::BusinessAmbiguity => EscalationSeverity::Low,
        }
    }
}

impl fmt::Display for EscalationTrigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.identifier())
    }
}

impl FromStr for EscalationTrigger {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "insufficient_evidence" | "insufficient" => Ok(Self::InsufficientEvidence),
            "unresolved_contradiction" | "contradiction" => Ok(Self::UnresolvedContradiction),
            "policy_conflict" | "policy" => Ok(Self::PolicyConflict),
            "high_impact_change" | "high_impact" => Ok(Self::HighImpactChange),
            "architecture_mismatch" | "arch_mismatch" => Ok(Self::ArchitectureMismatch),
            "security_safety_concern" | "security" | "safety" => Ok(Self::SecuritySafetyConcern),
            "failing_required_validation" | "failing_validation" => {
                Ok(Self::FailingRequiredValidation)
            }
            "uncertainty_above_threshold" | "uncertainty" => Ok(Self::UncertaintyAboveThreshold),
            "business_ambiguity" | "ambiguity" => Ok(Self::BusinessAmbiguity),
            _ => Err(format!("Unknown escalation trigger: {}", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// EscalationSeverity -- how urgent the escalation is
// ---------------------------------------------------------------------------

/// How urgent the escalation is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationSeverity {
    /// Informational -- human should be aware but work can continue.
    Low,
    /// Important -- human should review soon, work may continue cautiously.
    Medium,
    /// Significant -- human should review before further work proceeds.
    High,
    /// Urgent -- work must stop immediately until human reviews.
    Critical,
}

impl fmt::Display for EscalationSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

// ---------------------------------------------------------------------------
// EscalationEvent -- a concrete escalation instance
// ---------------------------------------------------------------------------

/// A concrete escalation event with full context for the human.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EscalationEvent {
    /// What triggered this escalation.
    pub trigger: EscalationTrigger,
    /// Severity of this specific escalation.
    pub severity: EscalationSeverity,
    /// Human-readable summary of why escalation was triggered.
    pub summary: String,
    /// Detailed context for the human decision-maker.
    pub context: Vec<String>,
    /// Suggested actions the human could take.
    pub suggested_actions: Vec<String>,
    /// The workflow step or gate where this was detected.
    pub detected_at: String,
}

impl EscalationEvent {
    /// Create a new escalation event.
    pub fn new(
        trigger: EscalationTrigger,
        summary: impl Into<String>,
        detected_at: impl Into<String>,
    ) -> Self {
        Self {
            severity: trigger.default_severity(),
            trigger,
            summary: summary.into(),
            context: Vec::new(),
            suggested_actions: Vec::new(),
            detected_at: detected_at.into(),
        }
    }

    /// Override the severity.
    pub fn with_severity(mut self, severity: EscalationSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Add context information.
    pub fn with_context(mut self, ctx: impl Into<String>) -> Self {
        self.context.push(ctx.into());
        self
    }

    /// Add a suggested action.
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.suggested_actions.push(action.into());
        self
    }

    /// Whether this event should block all work.
    pub fn blocks_work(&self) -> bool {
        self.severity >= EscalationSeverity::High
    }

    /// Format as an actionable message for the human.
    pub fn actionable_message(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!(
            "[{} ESCALATION] {} (at {})",
            self.severity.to_string().to_uppercase(),
            self.summary,
            self.detected_at,
        ));

        if !self.context.is_empty() {
            lines.push("Context:".into());
            for ctx in &self.context {
                lines.push(format!("  - {}", ctx));
            }
        }

        if !self.suggested_actions.is_empty() {
            lines.push("Suggested actions:".into());
            for action in &self.suggested_actions {
                lines.push(format!("  - {}", action));
            }
        }

        lines.join("\n")
    }
}

impl fmt::Display for EscalationEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.severity, self.trigger, self.summary)
    }
}

// ---------------------------------------------------------------------------
// EscalationDetector -- stateless detection of escalation conditions
// ---------------------------------------------------------------------------

/// Input signals for escalation detection.
///
/// A caller populates this with available signals, and the detector
/// evaluates which triggers fire.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EscalationSignals {
    /// Number of required evidence items missing.
    pub missing_evidence_count: usize,
    /// Whether contradictions were detected between artifacts.
    pub contradictions_detected: bool,
    /// Whether a policy conflict was detected.
    pub policy_conflict_detected: bool,
    /// Number of files affected by the change.
    pub files_affected: usize,
    /// Whether the change crosses module boundaries.
    pub crosses_boundaries: bool,
    /// Whether an architecture mismatch was detected.
    pub architecture_mismatch: bool,
    /// Whether a security/safety concern was detected.
    pub security_concern: bool,
    /// Number of required validations failing.
    pub failing_validations: usize,
    /// Estimated uncertainty level (0.0 = certain, 1.0 = completely uncertain).
    pub uncertainty_level: f64,
    /// Whether business-level ambiguity was detected.
    pub business_ambiguity: bool,
}

/// Thresholds for escalation detection, influenced by autonomy mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationThresholds {
    /// How many files affected before triggering high-impact escalation.
    pub high_impact_file_threshold: usize,
    /// Maximum uncertainty level before escalating.
    pub uncertainty_threshold: f64,
}

impl Default for EscalationThresholds {
    fn default() -> Self {
        Self {
            high_impact_file_threshold: 20,
            uncertainty_threshold: 0.7,
        }
    }
}

/// Stateless detector that evaluates signals against thresholds.
pub struct EscalationDetector;

impl EscalationDetector {
    /// Detect all escalation triggers that fire for the given signals.
    pub fn detect(
        signals: &EscalationSignals,
        thresholds: &EscalationThresholds,
    ) -> Vec<EscalationTrigger> {
        let mut triggers = Vec::new();

        if signals.missing_evidence_count > 0 {
            triggers.push(EscalationTrigger::InsufficientEvidence);
        }
        if signals.contradictions_detected {
            triggers.push(EscalationTrigger::UnresolvedContradiction);
        }
        if signals.policy_conflict_detected {
            triggers.push(EscalationTrigger::PolicyConflict);
        }
        if signals.files_affected >= thresholds.high_impact_file_threshold
            || signals.crosses_boundaries
        {
            triggers.push(EscalationTrigger::HighImpactChange);
        }
        if signals.architecture_mismatch {
            triggers.push(EscalationTrigger::ArchitectureMismatch);
        }
        if signals.security_concern {
            triggers.push(EscalationTrigger::SecuritySafetyConcern);
        }
        if signals.failing_validations > 0 {
            triggers.push(EscalationTrigger::FailingRequiredValidation);
        }
        if signals.uncertainty_level > thresholds.uncertainty_threshold {
            triggers.push(EscalationTrigger::UncertaintyAboveThreshold);
        }
        if signals.business_ambiguity {
            triggers.push(EscalationTrigger::BusinessAmbiguity);
        }

        triggers
    }

    /// Detect triggers and return the highest severity found.
    pub fn detect_max_severity(
        signals: &EscalationSignals,
        thresholds: &EscalationThresholds,
    ) -> Option<EscalationSeverity> {
        let triggers = Self::detect(signals, thresholds);
        triggers.iter().map(|t| t.default_severity()).max()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_triggers_returns_9() {
        assert_eq!(EscalationTrigger::all().len(), 9);
    }

    #[test]
    fn test_trigger_identifiers_are_unique() {
        let ids: Vec<&str> = EscalationTrigger::all()
            .iter()
            .map(|t| t.identifier())
            .collect();
        let mut deduped = ids.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(ids.len(), deduped.len());
    }

    #[test]
    fn test_trigger_roundtrip_from_str() {
        for t in EscalationTrigger::all() {
            let parsed: EscalationTrigger = t.identifier().parse().unwrap();
            assert_eq!(*t, parsed);
        }
    }

    #[test]
    fn test_trigger_from_str_aliases() {
        assert_eq!(
            "insufficient".parse::<EscalationTrigger>().unwrap(),
            EscalationTrigger::InsufficientEvidence
        );
        assert_eq!(
            "security".parse::<EscalationTrigger>().unwrap(),
            EscalationTrigger::SecuritySafetyConcern
        );
        assert_eq!(
            "ambiguity".parse::<EscalationTrigger>().unwrap(),
            EscalationTrigger::BusinessAmbiguity
        );
    }

    #[test]
    fn test_trigger_from_str_invalid() {
        assert!("nonexistent".parse::<EscalationTrigger>().is_err());
    }

    #[test]
    fn test_severity_ordering() {
        assert!(EscalationSeverity::Low < EscalationSeverity::Medium);
        assert!(EscalationSeverity::Medium < EscalationSeverity::High);
        assert!(EscalationSeverity::High < EscalationSeverity::Critical);
    }

    #[test]
    fn test_default_severities() {
        assert_eq!(
            EscalationTrigger::SecuritySafetyConcern.default_severity(),
            EscalationSeverity::Critical,
        );
        assert_eq!(
            EscalationTrigger::HighImpactChange.default_severity(),
            EscalationSeverity::High,
        );
        assert_eq!(
            EscalationTrigger::PolicyConflict.default_severity(),
            EscalationSeverity::Medium,
        );
        assert_eq!(
            EscalationTrigger::InsufficientEvidence.default_severity(),
            EscalationSeverity::Low,
        );
    }

    #[test]
    fn test_escalation_event_creation() {
        let event = EscalationEvent::new(
            EscalationTrigger::SecuritySafetyConcern,
            "Potential SQL injection",
            "validation_gate",
        )
        .with_context("Found unsanitized input in query builder")
        .with_action("Review query builder for injection vulnerabilities");

        assert_eq!(event.trigger, EscalationTrigger::SecuritySafetyConcern);
        assert_eq!(event.severity, EscalationSeverity::Critical);
        assert!(event.blocks_work());
        assert_eq!(event.context.len(), 1);
        assert_eq!(event.suggested_actions.len(), 1);
    }

    #[test]
    fn test_escalation_event_severity_override() {
        let event = EscalationEvent::new(
            EscalationTrigger::InsufficientEvidence,
            "Missing tests",
            "completion_gate",
        )
        .with_severity(EscalationSeverity::Critical);

        assert_eq!(event.severity, EscalationSeverity::Critical);
        assert!(event.blocks_work());
    }

    #[test]
    fn test_escalation_event_display() {
        let event = EscalationEvent::new(
            EscalationTrigger::PolicyConflict,
            "Rule conflict",
            "solution_gate",
        );
        let display = event.to_string();
        assert!(display.contains("medium"));
        assert!(display.contains("policy_conflict"));
        assert!(display.contains("Rule conflict"));
    }

    #[test]
    fn test_escalation_event_actionable_message() {
        let event = EscalationEvent::new(
            EscalationTrigger::HighImpactChange,
            "Large refactor",
            "execution_readiness_gate",
        )
        .with_context("47 files affected")
        .with_action("Review change scope with team lead");

        let msg = event.actionable_message();
        assert!(msg.contains("HIGH ESCALATION"));
        assert!(msg.contains("Large refactor"));
        assert!(msg.contains("47 files affected"));
        assert!(msg.contains("Review change scope"));
    }

    #[test]
    fn test_escalation_event_blocks_work() {
        // Low severity does not block
        let low = EscalationEvent::new(EscalationTrigger::InsufficientEvidence, "test", "gate");
        assert!(!low.blocks_work());

        // Medium does not block
        let medium = EscalationEvent::new(EscalationTrigger::PolicyConflict, "test", "gate");
        assert!(!medium.blocks_work());

        // High blocks
        let high = EscalationEvent::new(EscalationTrigger::HighImpactChange, "test", "gate");
        assert!(high.blocks_work());

        // Critical blocks
        let critical =
            EscalationEvent::new(EscalationTrigger::SecuritySafetyConcern, "test", "gate");
        assert!(critical.blocks_work());
    }

    #[test]
    fn test_detector_no_signals_no_triggers() {
        let signals = EscalationSignals::default();
        let thresholds = EscalationThresholds::default();
        let triggers = EscalationDetector::detect(&signals, &thresholds);
        assert!(triggers.is_empty());
    }

    #[test]
    fn test_detector_missing_evidence() {
        let signals = EscalationSignals {
            missing_evidence_count: 2,
            ..Default::default()
        };
        let triggers = EscalationDetector::detect(&signals, &EscalationThresholds::default());
        assert_eq!(triggers, vec![EscalationTrigger::InsufficientEvidence]);
    }

    #[test]
    fn test_detector_high_impact_files() {
        let signals = EscalationSignals {
            files_affected: 25,
            ..Default::default()
        };
        let triggers = EscalationDetector::detect(&signals, &EscalationThresholds::default());
        assert_eq!(triggers, vec![EscalationTrigger::HighImpactChange]);
    }

    #[test]
    fn test_detector_high_impact_boundary_crossing() {
        let signals = EscalationSignals {
            crosses_boundaries: true,
            ..Default::default()
        };
        let triggers = EscalationDetector::detect(&signals, &EscalationThresholds::default());
        assert_eq!(triggers, vec![EscalationTrigger::HighImpactChange]);
    }

    #[test]
    fn test_detector_uncertainty_threshold() {
        let signals = EscalationSignals {
            uncertainty_level: 0.8,
            ..Default::default()
        };
        let triggers = EscalationDetector::detect(&signals, &EscalationThresholds::default());
        assert_eq!(triggers, vec![EscalationTrigger::UncertaintyAboveThreshold]);
    }

    #[test]
    fn test_detector_uncertainty_at_threshold_does_not_fire() {
        let signals = EscalationSignals {
            uncertainty_level: 0.7, // exactly at threshold, not above
            ..Default::default()
        };
        let triggers = EscalationDetector::detect(&signals, &EscalationThresholds::default());
        assert!(triggers.is_empty());
    }

    #[test]
    fn test_detector_multiple_triggers() {
        let signals = EscalationSignals {
            missing_evidence_count: 1,
            security_concern: true,
            failing_validations: 3,
            ..Default::default()
        };
        let triggers = EscalationDetector::detect(&signals, &EscalationThresholds::default());
        assert_eq!(triggers.len(), 3);
        assert!(triggers.contains(&EscalationTrigger::InsufficientEvidence));
        assert!(triggers.contains(&EscalationTrigger::SecuritySafetyConcern));
        assert!(triggers.contains(&EscalationTrigger::FailingRequiredValidation));
    }

    #[test]
    fn test_detector_max_severity() {
        let signals = EscalationSignals {
            missing_evidence_count: 1, // Low
            security_concern: true,    // Critical
            ..Default::default()
        };
        let max =
            EscalationDetector::detect_max_severity(&signals, &EscalationThresholds::default());
        assert_eq!(max, Some(EscalationSeverity::Critical));
    }

    #[test]
    fn test_detector_max_severity_empty() {
        let signals = EscalationSignals::default();
        let max =
            EscalationDetector::detect_max_severity(&signals, &EscalationThresholds::default());
        assert_eq!(max, None);
    }

    #[test]
    fn test_detector_custom_thresholds() {
        let signals = EscalationSignals {
            files_affected: 5,
            uncertainty_level: 0.3,
            ..Default::default()
        };
        let thresholds = EscalationThresholds {
            high_impact_file_threshold: 3, // lower threshold
            uncertainty_threshold: 0.2,    // lower threshold
        };
        let triggers = EscalationDetector::detect(&signals, &thresholds);
        assert_eq!(triggers.len(), 2);
        assert!(triggers.contains(&EscalationTrigger::HighImpactChange));
        assert!(triggers.contains(&EscalationTrigger::UncertaintyAboveThreshold));
    }

    #[test]
    fn test_escalation_event_serde_roundtrip() {
        let event = EscalationEvent::new(
            EscalationTrigger::ArchitectureMismatch,
            "Module boundary violation",
            "solution_gate",
        )
        .with_context("New dependency crosses isolation boundary")
        .with_action("Review with architect");

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: EscalationEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_escalation_severity_display() {
        assert_eq!(EscalationSeverity::Low.to_string(), "low");
        assert_eq!(EscalationSeverity::Medium.to_string(), "medium");
        assert_eq!(EscalationSeverity::High.to_string(), "high");
        assert_eq!(EscalationSeverity::Critical.to_string(), "critical");
    }
}

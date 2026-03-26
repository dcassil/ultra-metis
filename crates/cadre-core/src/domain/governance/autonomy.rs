//! Autonomy mode configuration and mode-dependent gate behavior.
//!
//! Defines the three autonomy modes (tight, mixed, autonomous) and how they
//! affect gate behavior, escalation sensitivity, and system permissions.
//! The autonomy configuration is persisted as a durable artifact.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use super::escalation::EscalationThresholds;
use super::gates::{GateFailureBehavior, GateType};

// ---------------------------------------------------------------------------
// AutonomyMode -- the three operating modes
// ---------------------------------------------------------------------------

/// The three autonomy modes that control how much human oversight is required.
///
/// The mode is a deliberate human choice, not an AI decision.  It affects
/// gate strictness, escalation sensitivity, and what the system can do
/// without explicit approval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum AutonomyMode {
    /// Human approval required at most gates.
    ///
    /// Best for: early adoption, risky repos, architecture changes,
    /// org-sensitive work.
    Tight,
    /// AI proceeds within bounds, escalates on risk or ambiguity.
    ///
    /// This is the default mode.  The system handles routine decisions
    /// but involves humans at key decision points.
    #[default]
    Mixed,
    /// AI proceeds without routine approval, respects gates and thresholds.
    ///
    /// Only appropriate when rules, architecture, validations, and repo
    /// maturity are strong.
    Autonomous,
}

impl AutonomyMode {
    /// Returns all 3 modes in order of increasing autonomy.
    pub fn all() -> &'static [Self] {
        &[
            Self::Tight,
            Self::Mixed,
            Self::Autonomous,
        ]
    }

    /// Human-readable description of the mode.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Tight => "Human approval required at most gates",
            Self::Mixed => "AI proceeds within bounds, escalates on risk or ambiguity",
            Self::Autonomous => {
                "AI proceeds without routine approval, respects gates and thresholds"
            }
        }
    }

    /// Snake_case identifier for serialization.
    pub fn identifier(&self) -> &'static str {
        match self {
            Self::Tight => "tight",
            Self::Mixed => "mixed",
            Self::Autonomous => "autonomous",
        }
    }
}


impl fmt::Display for AutonomyMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.identifier())
    }
}

impl FromStr for AutonomyMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "tight" | "collaborative" | "strict" => Ok(Self::Tight),
            "mixed" | "default" | "balanced" => Ok(Self::Mixed),
            "autonomous" | "auto" | "full" => Ok(Self::Autonomous),
            _ => Err(format!("Unknown autonomy mode: {s}")),
        }
    }
}

// ---------------------------------------------------------------------------
// EvidenceLevel -- how much evidence is required
// ---------------------------------------------------------------------------

/// How much evidence must be present before the system can proceed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceLevel {
    /// All evidence (required + optional) must be present.
    High,
    /// All required evidence must be present.
    Medium,
    /// Only critical evidence must be present.
    Standard,
}

impl fmt::Display for EvidenceLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::High => write!(f, "high"),
            Self::Medium => write!(f, "medium"),
            Self::Standard => write!(f, "standard"),
        }
    }
}

// ---------------------------------------------------------------------------
// EscalationLevel -- how aggressively to escalate
// ---------------------------------------------------------------------------

/// How aggressively the system should escalate issues to humans.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationLevel {
    /// Escalate on any hint of uncertainty or risk.
    High,
    /// Escalate on significant risk or ambiguity.
    Medium,
    /// Only escalate on clear, high-impact issues.
    Low,
}

impl fmt::Display for EscalationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::High => write!(f, "high"),
            Self::Medium => write!(f, "medium"),
            Self::Low => write!(f, "low"),
        }
    }
}

// ---------------------------------------------------------------------------
// ToleranceLevel -- how much ambiguity/contradiction is acceptable
// ---------------------------------------------------------------------------

/// How much ambiguity or contradiction the system can tolerate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToleranceLevel {
    /// Almost no ambiguity tolerated -- escalate immediately.
    VeryLow,
    /// Minor ambiguity acceptable, significant ambiguity escalated.
    Low,
    /// Moderate ambiguity acceptable if the system can reason through it.
    Medium,
}

impl fmt::Display for ToleranceLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VeryLow => write!(f, "very_low"),
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
        }
    }
}

// ---------------------------------------------------------------------------
// AutonomyConfig -- the full autonomy configuration
// ---------------------------------------------------------------------------

/// Full autonomy configuration for a project.
///
/// This is persisted as a durable artifact and controls how the system
/// behaves at gates, escalation points, and decomposition boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutonomyConfig {
    /// The operating mode.
    pub mode: AutonomyMode,
    /// Per-gate behavior overrides (gate_type -> behavior).
    pub gate_overrides: HashMap<GateType, GateFailureBehavior>,
    /// How aggressively to escalate.
    pub escalation_sensitivity: EscalationLevel,
    /// Whether the system can decompose work without asking.
    pub auto_decompose: bool,
    /// Whether the system can dispatch subtasks without asking.
    pub auto_dispatch: bool,
    /// How much evidence is required before proceeding.
    pub evidence_threshold: EvidenceLevel,
    /// How much ambiguity is acceptable.
    pub contradiction_tolerance: ToleranceLevel,
}

impl AutonomyConfig {
    /// Create a new configuration for the given mode with sensible defaults.
    pub fn new(mode: AutonomyMode) -> Self {
        match mode {
            AutonomyMode::Tight => Self::tight(),
            AutonomyMode::Mixed => Self::mixed(),
            AutonomyMode::Autonomous => Self::autonomous(),
        }
    }

    /// Create the tight collaboration configuration.
    pub fn tight() -> Self {
        Self {
            mode: AutonomyMode::Tight,
            gate_overrides: HashMap::new(),
            escalation_sensitivity: EscalationLevel::High,
            auto_decompose: false,
            auto_dispatch: false,
            evidence_threshold: EvidenceLevel::High,
            contradiction_tolerance: ToleranceLevel::VeryLow,
        }
    }

    /// Create the mixed mode configuration (default).
    pub fn mixed() -> Self {
        Self {
            mode: AutonomyMode::Mixed,
            gate_overrides: HashMap::new(),
            escalation_sensitivity: EscalationLevel::Medium,
            auto_decompose: false,
            auto_dispatch: false,
            evidence_threshold: EvidenceLevel::Medium,
            contradiction_tolerance: ToleranceLevel::Low,
        }
    }

    /// Create the autonomous mode configuration.
    pub fn autonomous() -> Self {
        Self {
            mode: AutonomyMode::Autonomous,
            gate_overrides: HashMap::new(),
            escalation_sensitivity: EscalationLevel::Low,
            auto_decompose: true,
            auto_dispatch: true,
            evidence_threshold: EvidenceLevel::Standard,
            contradiction_tolerance: ToleranceLevel::Medium,
        }
    }

    /// Override the behavior for a specific gate type.
    pub fn with_gate_override(mut self, gate: GateType, behavior: GateFailureBehavior) -> Self {
        self.gate_overrides.insert(gate, behavior);
        self
    }

    /// Override the escalation sensitivity.
    pub fn with_escalation_sensitivity(mut self, level: EscalationLevel) -> Self {
        self.escalation_sensitivity = level;
        self
    }

    /// Override the evidence threshold.
    pub fn with_evidence_threshold(mut self, level: EvidenceLevel) -> Self {
        self.evidence_threshold = level;
        self
    }

    /// Override the contradiction tolerance.
    pub fn with_contradiction_tolerance(mut self, level: ToleranceLevel) -> Self {
        self.contradiction_tolerance = level;
        self
    }

    /// Resolve the effective failure behavior for a gate, considering mode
    /// defaults and per-gate overrides.
    pub fn effective_gate_behavior(
        &self,
        gate_type: GateType,
        default_behavior: GateFailureBehavior,
    ) -> GateFailureBehavior {
        // Per-gate override takes priority
        if let Some(override_behavior) = self.gate_overrides.get(&gate_type) {
            return *override_behavior;
        }

        // Mode-dependent defaults
        match self.mode {
            AutonomyMode::Tight => {
                // In tight mode, most gates escalate to human
                match gate_type {
                    GateType::Escalation => GateFailureBehavior::EscalateToHuman,
                    _ => GateFailureBehavior::EscalateToHuman,
                }
            }
            AutonomyMode::Mixed => {
                // In mixed mode, key gates escalate, others use defaults
                match gate_type {
                    GateType::Solution | GateType::Completion | GateType::Escalation => {
                        GateFailureBehavior::EscalateToHuman
                    }
                    _ => default_behavior,
                }
            }
            AutonomyMode::Autonomous => {
                // In autonomous mode, gates block or warn, rarely escalate
                match gate_type {
                    GateType::Escalation => GateFailureBehavior::EscalateToHuman,
                    GateType::Entry | GateType::Validation | GateType::Completion => {
                        GateFailureBehavior::Block
                    }
                    _ => GateFailureBehavior::Warn,
                }
            }
        }
    }

    /// Returns the escalation thresholds appropriate for this mode.
    pub fn escalation_thresholds(&self) -> EscalationThresholds {
        match self.mode {
            AutonomyMode::Tight => EscalationThresholds {
                high_impact_file_threshold: 5,
                uncertainty_threshold: 0.3,
            },
            AutonomyMode::Mixed => EscalationThresholds {
                high_impact_file_threshold: 20,
                uncertainty_threshold: 0.7,
            },
            AutonomyMode::Autonomous => EscalationThresholds {
                high_impact_file_threshold: 50,
                uncertainty_threshold: 0.9,
            },
        }
    }

    /// Whether an evidence requirement must be satisfied given this mode.
    ///
    /// In `High` evidence mode, both required and optional evidence must exist.
    /// In `Medium`, only required evidence must exist.
    /// In `Standard`, only critical evidence (i.e., required) must exist but
    /// the bar for what counts as "critical" is lower.
    pub fn requires_evidence(&self, is_required: bool) -> bool {
        match self.evidence_threshold {
            EvidenceLevel::High => true, // all evidence required
            EvidenceLevel::Medium | EvidenceLevel::Standard => is_required,
        }
    }
}

impl Default for AutonomyConfig {
    fn default() -> Self {
        Self::mixed()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_modes_returns_3() {
        assert_eq!(AutonomyMode::all().len(), 3);
    }

    #[test]
    fn test_mode_default_is_mixed() {
        assert_eq!(AutonomyMode::default(), AutonomyMode::Mixed);
    }

    #[test]
    fn test_mode_roundtrip_from_str() {
        for mode in AutonomyMode::all() {
            let parsed: AutonomyMode = mode.identifier().parse().unwrap();
            assert_eq!(*mode, parsed);
        }
    }

    #[test]
    fn test_mode_from_str_aliases() {
        assert_eq!(
            "collaborative".parse::<AutonomyMode>().unwrap(),
            AutonomyMode::Tight
        );
        assert_eq!(
            "strict".parse::<AutonomyMode>().unwrap(),
            AutonomyMode::Tight
        );
        assert_eq!(
            "default".parse::<AutonomyMode>().unwrap(),
            AutonomyMode::Mixed
        );
        assert_eq!(
            "balanced".parse::<AutonomyMode>().unwrap(),
            AutonomyMode::Mixed
        );
        assert_eq!(
            "auto".parse::<AutonomyMode>().unwrap(),
            AutonomyMode::Autonomous
        );
        assert_eq!(
            "full".parse::<AutonomyMode>().unwrap(),
            AutonomyMode::Autonomous
        );
    }

    #[test]
    fn test_mode_from_str_invalid() {
        assert!("nonexistent".parse::<AutonomyMode>().is_err());
    }

    #[test]
    fn test_config_tight_defaults() {
        let config = AutonomyConfig::tight();
        assert_eq!(config.mode, AutonomyMode::Tight);
        assert_eq!(config.escalation_sensitivity, EscalationLevel::High);
        assert!(!config.auto_decompose);
        assert!(!config.auto_dispatch);
        assert_eq!(config.evidence_threshold, EvidenceLevel::High);
        assert_eq!(config.contradiction_tolerance, ToleranceLevel::VeryLow);
    }

    #[test]
    fn test_config_mixed_defaults() {
        let config = AutonomyConfig::mixed();
        assert_eq!(config.mode, AutonomyMode::Mixed);
        assert_eq!(config.escalation_sensitivity, EscalationLevel::Medium);
        assert!(!config.auto_decompose);
        assert!(!config.auto_dispatch);
        assert_eq!(config.evidence_threshold, EvidenceLevel::Medium);
        assert_eq!(config.contradiction_tolerance, ToleranceLevel::Low);
    }

    #[test]
    fn test_config_autonomous_defaults() {
        let config = AutonomyConfig::autonomous();
        assert_eq!(config.mode, AutonomyMode::Autonomous);
        assert_eq!(config.escalation_sensitivity, EscalationLevel::Low);
        assert!(config.auto_decompose);
        assert!(config.auto_dispatch);
        assert_eq!(config.evidence_threshold, EvidenceLevel::Standard);
        assert_eq!(config.contradiction_tolerance, ToleranceLevel::Medium);
    }

    #[test]
    fn test_config_new_delegates_to_mode() {
        let tight = AutonomyConfig::new(AutonomyMode::Tight);
        assert_eq!(tight, AutonomyConfig::tight());

        let mixed = AutonomyConfig::new(AutonomyMode::Mixed);
        assert_eq!(mixed, AutonomyConfig::mixed());

        let auto = AutonomyConfig::new(AutonomyMode::Autonomous);
        assert_eq!(auto, AutonomyConfig::autonomous());
    }

    #[test]
    fn test_config_default_is_mixed() {
        assert_eq!(AutonomyConfig::default(), AutonomyConfig::mixed());
    }

    #[test]
    fn test_effective_gate_behavior_tight() {
        let config = AutonomyConfig::tight();
        // All gates escalate in tight mode
        for gt in GateType::all() {
            let behavior = config.effective_gate_behavior(*gt, GateFailureBehavior::Block);
            assert_eq!(behavior, GateFailureBehavior::EscalateToHuman);
        }
    }

    #[test]
    fn test_effective_gate_behavior_mixed() {
        let config = AutonomyConfig::mixed();
        // Solution, Completion, Escalation escalate; others use default
        assert_eq!(
            config.effective_gate_behavior(GateType::Solution, GateFailureBehavior::Block),
            GateFailureBehavior::EscalateToHuman,
        );
        assert_eq!(
            config.effective_gate_behavior(GateType::Entry, GateFailureBehavior::Block),
            GateFailureBehavior::Block,
        );
        assert_eq!(
            config.effective_gate_behavior(GateType::Validation, GateFailureBehavior::Block),
            GateFailureBehavior::Block,
        );
    }

    #[test]
    fn test_effective_gate_behavior_autonomous() {
        let config = AutonomyConfig::autonomous();
        // Entry, Validation, Completion block; Escalation escalates; others warn
        assert_eq!(
            config.effective_gate_behavior(GateType::Entry, GateFailureBehavior::EscalateToHuman),
            GateFailureBehavior::Block,
        );
        assert_eq!(
            config
                .effective_gate_behavior(GateType::ContextSufficiency, GateFailureBehavior::Block),
            GateFailureBehavior::Warn,
        );
        assert_eq!(
            config.effective_gate_behavior(GateType::Escalation, GateFailureBehavior::Block),
            GateFailureBehavior::EscalateToHuman,
        );
    }

    #[test]
    fn test_gate_override_takes_priority() {
        let config =
            AutonomyConfig::tight().with_gate_override(GateType::Entry, GateFailureBehavior::Warn);

        // Override should take priority over tight mode's escalate_to_human
        assert_eq!(
            config.effective_gate_behavior(GateType::Entry, GateFailureBehavior::Block),
            GateFailureBehavior::Warn,
        );
        // Other gates still escalate
        assert_eq!(
            config.effective_gate_behavior(GateType::Solution, GateFailureBehavior::Block),
            GateFailureBehavior::EscalateToHuman,
        );
    }

    #[test]
    fn test_escalation_thresholds_by_mode() {
        let tight = AutonomyConfig::tight().escalation_thresholds();
        let mixed = AutonomyConfig::mixed().escalation_thresholds();
        let auto = AutonomyConfig::autonomous().escalation_thresholds();

        // Tight has lowest thresholds (escalates more)
        assert!(tight.high_impact_file_threshold < mixed.high_impact_file_threshold);
        assert!(mixed.high_impact_file_threshold < auto.high_impact_file_threshold);
        assert!(tight.uncertainty_threshold < mixed.uncertainty_threshold);
        assert!(mixed.uncertainty_threshold < auto.uncertainty_threshold);
    }

    #[test]
    fn test_requires_evidence_high() {
        let config = AutonomyConfig::tight(); // High evidence
        assert!(config.requires_evidence(true));
        assert!(config.requires_evidence(false)); // even optional
    }

    #[test]
    fn test_requires_evidence_medium() {
        let config = AutonomyConfig::mixed(); // Medium evidence
        assert!(config.requires_evidence(true));
        assert!(!config.requires_evidence(false)); // optional not required
    }

    #[test]
    fn test_requires_evidence_standard() {
        let config = AutonomyConfig::autonomous(); // Standard evidence
        assert!(config.requires_evidence(true));
        assert!(!config.requires_evidence(false));
    }

    #[test]
    fn test_config_serde_roundtrip() {
        let config = AutonomyConfig::mixed()
            .with_gate_override(GateType::Entry, GateFailureBehavior::Warn)
            .with_escalation_sensitivity(EscalationLevel::High);

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AutonomyConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_config_serde_roundtrip_all_modes() {
        for mode in AutonomyMode::all() {
            let config = AutonomyConfig::new(*mode);
            let json = serde_json::to_string(&config).unwrap();
            let deserialized: AutonomyConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(config, deserialized);
        }
    }

    #[test]
    fn test_mode_display() {
        assert_eq!(AutonomyMode::Tight.to_string(), "tight");
        assert_eq!(AutonomyMode::Mixed.to_string(), "mixed");
        assert_eq!(AutonomyMode::Autonomous.to_string(), "autonomous");
    }

    #[test]
    fn test_evidence_level_display() {
        assert_eq!(EvidenceLevel::High.to_string(), "high");
        assert_eq!(EvidenceLevel::Medium.to_string(), "medium");
        assert_eq!(EvidenceLevel::Standard.to_string(), "standard");
    }

    #[test]
    fn test_escalation_level_display() {
        assert_eq!(EscalationLevel::High.to_string(), "high");
        assert_eq!(EscalationLevel::Medium.to_string(), "medium");
        assert_eq!(EscalationLevel::Low.to_string(), "low");
    }

    #[test]
    fn test_tolerance_level_display() {
        assert_eq!(ToleranceLevel::VeryLow.to_string(), "very_low");
        assert_eq!(ToleranceLevel::Low.to_string(), "low");
        assert_eq!(ToleranceLevel::Medium.to_string(), "medium");
    }
}

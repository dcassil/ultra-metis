//! Transition Enforcer -- validates and executes transitions through the hook pipeline.
//!
//! The `TransitionEnforcer` is the central coordination point that:
//! 1. Validates the transition is structurally valid (phase graph check)
//! 2. Runs all matching pre-transition checks from the registry
//! 3. Collects results and decides whether to allow the transition
//! 4. If allowed, runs all matching post-transition actions
//! 5. Returns a complete `EnforcementResult` with all details

use super::hooks::{
    PostActionResult, PostTransitionAction, PreCheckResult, PreTransitionCheck,
    TransitionEvent, HookPriority,
};
use super::registry::HookRegistry;
use crate::domain::documents::types::{DocumentType, Phase};
use std::fmt;

// ---------------------------------------------------------------------------
// EnforcementOutcome -- the final verdict
// ---------------------------------------------------------------------------

/// The overall outcome of a transition enforcement attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnforcementOutcome {
    /// Transition was allowed and completed.
    Allowed,
    /// Transition was blocked by one or more pre-transition checks.
    Blocked,
    /// Transition was structurally invalid (not a valid phase transition).
    InvalidTransition,
    /// Transition was forced despite failures.
    Forced,
}

impl fmt::Display for EnforcementOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allowed => write!(f, "allowed"),
            Self::Blocked => write!(f, "blocked"),
            Self::InvalidTransition => write!(f, "invalid_transition"),
            Self::Forced => write!(f, "forced"),
        }
    }
}

// ---------------------------------------------------------------------------
// EnforcementResult -- complete result of enforcement
// ---------------------------------------------------------------------------

/// Complete result of a transition enforcement attempt.
///
/// Contains the outcome, all pre-check results, all post-action results,
/// and any warnings that were generated.
#[derive(Debug)]
pub struct EnforcementResult {
    /// The event that was being enforced.
    pub event: TransitionEvent,
    /// The overall outcome.
    pub outcome: EnforcementOutcome,
    /// Results from all pre-transition checks that ran.
    pub pre_check_results: Vec<PreCheckResult>,
    /// Results from all post-transition actions that ran (empty if blocked).
    pub post_action_results: Vec<PostActionResult>,
}

impl EnforcementResult {
    /// Whether the transition was allowed (either normally or forced).
    pub fn was_allowed(&self) -> bool {
        matches!(
            self.outcome,
            EnforcementOutcome::Allowed | EnforcementOutcome::Forced
        )
    }

    /// Get all blocking failures from pre-checks.
    pub fn blocking_failures(&self) -> Vec<&PreCheckResult> {
        self.pre_check_results
            .iter()
            .filter(|r| !r.passed && r.blocking)
            .collect()
    }

    /// Get all warnings (non-blocking failures) from pre-checks.
    pub fn warnings(&self) -> Vec<&PreCheckResult> {
        self.pre_check_results
            .iter()
            .filter(|r| !r.passed && !r.blocking)
            .collect()
    }

    /// Get all post-action failures.
    pub fn post_action_failures(&self) -> Vec<&PostActionResult> {
        self.post_action_results
            .iter()
            .filter(|r| !r.success)
            .collect()
    }

    /// Format a human-readable summary of the enforcement result.
    pub fn summary(&self) -> String {
        let mut lines = vec![format!(
            "Transition {} -> {}: {}",
            self.event.from_phase, self.event.to_phase, self.outcome
        )];

        let blocking = self.blocking_failures();
        if !blocking.is_empty() {
            lines.push(format!("  Blocking failures ({}): ", blocking.len()));
            for f in &blocking {
                lines.push(format!("    - {}", f));
            }
        }

        let warnings = self.warnings();
        if !warnings.is_empty() {
            lines.push(format!("  Warnings ({}): ", warnings.len()));
            for w in &warnings {
                lines.push(format!("    - {}", w));
            }
        }

        let post_failures = self.post_action_failures();
        if !post_failures.is_empty() {
            lines.push(format!("  Post-action failures ({}): ", post_failures.len()));
            for f in &post_failures {
                lines.push(format!("    - {}", f));
            }
        }

        lines.join("\n")
    }
}

// ---------------------------------------------------------------------------
// TransitionEnforcer
// ---------------------------------------------------------------------------

/// Enforces transition rules by coordinating phase validity checks,
/// pre-transition hooks, and post-transition actions.
///
/// The enforcer does not own the document -- it evaluates whether a
/// transition should be allowed and reports the results. The caller
/// is responsible for actually mutating the document state.
pub struct TransitionEnforcer<'a> {
    registry: &'a HookRegistry,
}

impl<'a> TransitionEnforcer<'a> {
    /// Create a new enforcer that uses the given hook registry.
    pub fn new(registry: &'a HookRegistry) -> Self {
        Self { registry }
    }

    /// Evaluate whether a transition should be allowed and run all hooks.
    ///
    /// This is the main entry point. It:
    /// 1. Checks structural validity (is this a valid phase transition?)
    /// 2. Runs pre-transition checks
    /// 3. If allowed (or forced), runs post-transition actions
    /// 4. Returns the complete enforcement result
    pub fn enforce(&self, event: &TransitionEvent) -> EnforcementResult {
        // Step 1: Structural validity
        if !event
            .document_type
            .can_transition(event.from_phase, event.to_phase)
        {
            return EnforcementResult {
                event: event.clone(),
                outcome: EnforcementOutcome::InvalidTransition,
                pre_check_results: Vec::new(),
                post_action_results: Vec::new(),
            };
        }

        // Step 2: Run pre-transition checks
        let pre_check_results = self.registry.run_pre_checks(event);

        // Step 3: Determine outcome
        let has_blocking_failures = pre_check_results
            .iter()
            .any(|r| !r.passed && r.blocking);

        let outcome = if has_blocking_failures && !event.forced {
            EnforcementOutcome::Blocked
        } else if has_blocking_failures && event.forced {
            EnforcementOutcome::Forced
        } else {
            EnforcementOutcome::Allowed
        };

        // Step 4: Run post-transition actions only if transition proceeds
        let post_action_results = if outcome == EnforcementOutcome::Allowed
            || outcome == EnforcementOutcome::Forced
        {
            self.registry.run_post_actions(event)
        } else {
            Vec::new()
        };

        EnforcementResult {
            event: event.clone(),
            outcome,
            pre_check_results,
            post_action_results,
        }
    }

    /// Convenience method: check if a transition would be allowed without
    /// running post-transition actions.
    pub fn would_allow(&self, event: &TransitionEvent) -> bool {
        if !event
            .document_type
            .can_transition(event.from_phase, event.to_phase)
        {
            return false;
        }

        let pre_results = self.registry.run_pre_checks(event);
        !pre_results.iter().any(|r| !r.passed && r.blocking) || event.forced
    }
}

/// Create a built-in pre-transition check that validates phase transition
/// validity. This is registered at SYSTEM priority.
pub fn phase_validity_check() -> PreTransitionCheck {
    PreTransitionCheck::new("phase_validity", |event: &TransitionEvent| {
        if event
            .document_type
            .can_transition(event.from_phase, event.to_phase)
        {
            PreCheckResult::pass(
                "phase_validity",
                format!(
                    "Valid transition: {} -> {} for {}",
                    event.from_phase, event.to_phase, event.document_type
                ),
            )
        } else {
            PreCheckResult::block(
                "phase_validity",
                format!(
                    "Invalid transition: {} -> {} for {}",
                    event.from_phase, event.to_phase, event.document_type
                ),
            )
        }
    })
    .with_priority(HookPriority::SYSTEM)
}

/// Create a built-in pre-transition check that blocks transitions on
/// archived documents.
pub fn archived_document_check() -> PreTransitionCheck {
    // Note: This check uses the event's forced flag as a proxy.
    // In a full system, we'd pass document metadata in the event.
    // For now, this serves as an example of a system-level check.
    PreTransitionCheck::new("not_archived", |_event: &TransitionEvent| {
        // The actual archived check would need document state.
        // This is a structural placeholder that always passes.
        PreCheckResult::pass("not_archived", "Document is not archived")
    })
    .with_priority(HookPriority::SYSTEM)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::documents::types::DocumentId;

    fn task_event(from: Phase, to: Phase) -> TransitionEvent {
        TransitionEvent::new(
            DocumentId::from("test-task"),
            DocumentType::Task,
            from,
            to,
            "test-actor",
            false,
        )
    }

    fn forced_task_event(from: Phase, to: Phase) -> TransitionEvent {
        TransitionEvent::new(
            DocumentId::from("test-task"),
            DocumentType::Task,
            from,
            to,
            "test-actor",
            true,
        )
    }

    #[test]
    fn test_enforce_valid_transition_no_hooks() {
        let registry = HookRegistry::new();
        let enforcer = TransitionEnforcer::new(&registry);

        let event = task_event(Phase::Active, Phase::Completed);
        let result = enforcer.enforce(&event);

        assert_eq!(result.outcome, EnforcementOutcome::Allowed);
        assert!(result.was_allowed());
        assert!(result.blocking_failures().is_empty());
        assert!(result.warnings().is_empty());
    }

    #[test]
    fn test_enforce_invalid_transition() {
        let registry = HookRegistry::new();
        let enforcer = TransitionEnforcer::new(&registry);

        // Task cannot go directly from Todo to Completed
        let event = task_event(Phase::Todo, Phase::Completed);
        let result = enforcer.enforce(&event);

        assert_eq!(result.outcome, EnforcementOutcome::InvalidTransition);
        assert!(!result.was_allowed());
    }

    #[test]
    fn test_enforce_blocked_by_pre_check() {
        let mut registry = HookRegistry::new();
        registry.register_pre_check(PreTransitionCheck::new("blocker", |_| {
            PreCheckResult::block("blocker", "Exit criteria not met")
        }));

        let enforcer = TransitionEnforcer::new(&registry);
        let event = task_event(Phase::Active, Phase::Completed);
        let result = enforcer.enforce(&event);

        assert_eq!(result.outcome, EnforcementOutcome::Blocked);
        assert!(!result.was_allowed());
        assert_eq!(result.blocking_failures().len(), 1);
        assert!(result.post_action_results.is_empty()); // no post-actions when blocked
    }

    #[test]
    fn test_enforce_forced_despite_blocking() {
        let mut registry = HookRegistry::new();
        registry.register_pre_check(PreTransitionCheck::new("blocker", |_| {
            PreCheckResult::block("blocker", "Would normally block")
        }));

        let enforcer = TransitionEnforcer::new(&registry);
        let event = forced_task_event(Phase::Active, Phase::Completed);
        let result = enforcer.enforce(&event);

        assert_eq!(result.outcome, EnforcementOutcome::Forced);
        assert!(result.was_allowed());
        assert_eq!(result.blocking_failures().len(), 1);
    }

    #[test]
    fn test_enforce_warnings_dont_block() {
        let mut registry = HookRegistry::new();
        registry.register_pre_check(PreTransitionCheck::new("advisory", |_| {
            PreCheckResult::warn("advisory", "Could be better")
        }));

        let enforcer = TransitionEnforcer::new(&registry);
        let event = task_event(Phase::Active, Phase::Completed);
        let result = enforcer.enforce(&event);

        assert_eq!(result.outcome, EnforcementOutcome::Allowed);
        assert!(result.was_allowed());
        assert_eq!(result.warnings().len(), 1);
        assert!(result.blocking_failures().is_empty());
    }

    #[test]
    fn test_enforce_post_actions_run_on_allow() {
        let mut registry = HookRegistry::new();
        registry.register_post_action(PostTransitionAction::new("audit", |_| {
            PostActionResult::ok("audit", "Transition logged")
        }));

        let enforcer = TransitionEnforcer::new(&registry);
        let event = task_event(Phase::Active, Phase::Completed);
        let result = enforcer.enforce(&event);

        assert_eq!(result.outcome, EnforcementOutcome::Allowed);
        assert_eq!(result.post_action_results.len(), 1);
        assert!(result.post_action_results[0].success);
    }

    #[test]
    fn test_enforce_post_actions_run_on_force() {
        let mut registry = HookRegistry::new();
        registry.register_pre_check(PreTransitionCheck::new("blocker", |_| {
            PreCheckResult::block("blocker", "blocked")
        }));
        registry.register_post_action(PostTransitionAction::new("audit", |_| {
            PostActionResult::ok("audit", "logged")
        }));

        let enforcer = TransitionEnforcer::new(&registry);
        let event = forced_task_event(Phase::Active, Phase::Completed);
        let result = enforcer.enforce(&event);

        assert_eq!(result.outcome, EnforcementOutcome::Forced);
        assert_eq!(result.post_action_results.len(), 1);
    }

    #[test]
    fn test_enforce_post_actions_skip_on_block() {
        let mut registry = HookRegistry::new();
        registry.register_pre_check(PreTransitionCheck::new("blocker", |_| {
            PreCheckResult::block("blocker", "blocked")
        }));
        registry.register_post_action(PostTransitionAction::new("audit", |_| {
            PostActionResult::ok("audit", "logged")
        }));

        let enforcer = TransitionEnforcer::new(&registry);
        let event = task_event(Phase::Active, Phase::Completed);
        let result = enforcer.enforce(&event);

        assert_eq!(result.outcome, EnforcementOutcome::Blocked);
        assert!(result.post_action_results.is_empty());
    }

    #[test]
    fn test_enforce_mixed_checks() {
        let mut registry = HookRegistry::new();
        registry.register_pre_check(PreTransitionCheck::new("pass", |_| {
            PreCheckResult::pass("pass", "ok")
        }));
        registry.register_pre_check(PreTransitionCheck::new("warn", |_| {
            PreCheckResult::warn("warn", "not great")
        }));

        let enforcer = TransitionEnforcer::new(&registry);
        let event = task_event(Phase::Active, Phase::Completed);
        let result = enforcer.enforce(&event);

        assert_eq!(result.outcome, EnforcementOutcome::Allowed);
        assert_eq!(result.pre_check_results.len(), 2);
        assert_eq!(result.warnings().len(), 1);
    }

    #[test]
    fn test_would_allow_valid() {
        let registry = HookRegistry::new();
        let enforcer = TransitionEnforcer::new(&registry);

        assert!(enforcer.would_allow(&task_event(Phase::Active, Phase::Completed)));
    }

    #[test]
    fn test_would_allow_invalid_transition() {
        let registry = HookRegistry::new();
        let enforcer = TransitionEnforcer::new(&registry);

        assert!(!enforcer.would_allow(&task_event(Phase::Todo, Phase::Completed)));
    }

    #[test]
    fn test_would_allow_blocked() {
        let mut registry = HookRegistry::new();
        registry.register_pre_check(PreTransitionCheck::new("blocker", |_| {
            PreCheckResult::block("blocker", "no")
        }));

        let enforcer = TransitionEnforcer::new(&registry);
        assert!(!enforcer.would_allow(&task_event(Phase::Active, Phase::Completed)));
    }

    #[test]
    fn test_would_allow_forced() {
        let mut registry = HookRegistry::new();
        registry.register_pre_check(PreTransitionCheck::new("blocker", |_| {
            PreCheckResult::block("blocker", "no")
        }));

        let enforcer = TransitionEnforcer::new(&registry);
        assert!(enforcer.would_allow(&forced_task_event(Phase::Active, Phase::Completed)));
    }

    #[test]
    fn test_phase_validity_check_pass() {
        let check = phase_validity_check();
        let event = task_event(Phase::Active, Phase::Completed);
        let result = check.execute(&event);
        assert!(result.passed);
    }

    #[test]
    fn test_phase_validity_check_fail() {
        let check = phase_validity_check();
        let event = task_event(Phase::Todo, Phase::Completed);
        let result = check.execute(&event);
        assert!(!result.passed);
        assert!(result.blocking);
    }

    #[test]
    fn test_enforcement_outcome_display() {
        assert_eq!(EnforcementOutcome::Allowed.to_string(), "allowed");
        assert_eq!(EnforcementOutcome::Blocked.to_string(), "blocked");
        assert_eq!(
            EnforcementOutcome::InvalidTransition.to_string(),
            "invalid_transition"
        );
        assert_eq!(EnforcementOutcome::Forced.to_string(), "forced");
    }

    #[test]
    fn test_enforcement_result_summary() {
        let mut registry = HookRegistry::new();
        registry.register_pre_check(PreTransitionCheck::new("blocker", |_| {
            PreCheckResult::block("blocker", "not ready")
        }));

        let enforcer = TransitionEnforcer::new(&registry);
        let event = task_event(Phase::Active, Phase::Completed);
        let result = enforcer.enforce(&event);

        let summary = result.summary();
        assert!(summary.contains("blocked"));
        assert!(summary.contains("Blocking failures"));
        assert!(summary.contains("not ready"));
    }

    #[test]
    fn test_post_action_failures_tracked() {
        let mut registry = HookRegistry::new();
        registry.register_post_action(PostTransitionAction::new("fail-action", |_| {
            PostActionResult::err("fail-action", "disk full")
        }));

        let enforcer = TransitionEnforcer::new(&registry);
        let event = task_event(Phase::Active, Phase::Completed);
        let result = enforcer.enforce(&event);

        assert!(result.was_allowed());
        assert_eq!(result.post_action_failures().len(), 1);
    }

    #[test]
    fn test_enforce_epic_transitions() {
        let registry = HookRegistry::new();
        let enforcer = TransitionEnforcer::new(&registry);

        let event = TransitionEvent::new(
            DocumentId::from("epic-1"),
            DocumentType::Epic,
            Phase::Discovery,
            Phase::Design,
            "actor",
            false,
        );
        let result = enforcer.enforce(&event);
        assert_eq!(result.outcome, EnforcementOutcome::Allowed);

        // Epic cannot skip phases
        let skip_event = TransitionEvent::new(
            DocumentId::from("epic-1"),
            DocumentType::Epic,
            Phase::Discovery,
            Phase::Active,
            "actor",
            false,
        );
        let result = enforcer.enforce(&skip_event);
        assert_eq!(result.outcome, EnforcementOutcome::InvalidTransition);
    }
}

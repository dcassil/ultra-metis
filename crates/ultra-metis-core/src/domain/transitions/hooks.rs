//! Transition hook types.
//!
//! Defines the core hook abstractions: `TransitionEvent` describes what is
//! happening, `PreTransitionCheck` runs before a transition to allow/deny it,
//! and `PostTransitionAction` runs after a successful transition for side effects.

use crate::domain::documents::types::{DocumentId, DocumentType, Phase};
use std::fmt;

// ---------------------------------------------------------------------------
// TransitionEvent -- describes a phase transition attempt
// ---------------------------------------------------------------------------

/// Describes a phase transition that is being attempted or has completed.
///
/// This is the common context passed to all hooks so they can decide whether
/// to fire and what to inspect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionEvent {
    /// The document being transitioned.
    pub document_id: DocumentId,
    /// The type of document being transitioned.
    pub document_type: DocumentType,
    /// The phase the document is currently in.
    pub from_phase: Phase,
    /// The phase the document is transitioning to.
    pub to_phase: Phase,
    /// Who or what triggered the transition.
    pub actor: String,
    /// Whether this transition is being forced (bypassing checks).
    pub forced: bool,
}

impl TransitionEvent {
    pub fn new(
        document_id: DocumentId,
        document_type: DocumentType,
        from_phase: Phase,
        to_phase: Phase,
        actor: impl Into<String>,
        forced: bool,
    ) -> Self {
        Self {
            document_id,
            document_type,
            from_phase,
            to_phase,
            actor: actor.into(),
            forced,
        }
    }
}

impl fmt::Display for TransitionEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}) {} -> {}{}",
            self.document_id,
            self.document_type,
            self.from_phase,
            self.to_phase,
            if self.forced { " [FORCED]" } else { "" }
        )
    }
}

// ---------------------------------------------------------------------------
// HookPriority -- determines execution order
// ---------------------------------------------------------------------------

/// Priority for hook execution ordering. Lower values execute first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HookPriority(pub u32);

impl HookPriority {
    /// System-level hooks run first (e.g., phase validity).
    pub const SYSTEM: Self = Self(100);
    /// Gate checks run after system checks.
    pub const GATE: Self = Self(200);
    /// User-defined hooks run after gates.
    pub const USER: Self = Self(500);
    /// Advisory hooks run last.
    pub const ADVISORY: Self = Self(900);
}

impl Default for HookPriority {
    fn default() -> Self {
        Self::USER
    }
}

// ---------------------------------------------------------------------------
// PreTransitionCheck -- runs before a transition
// ---------------------------------------------------------------------------

/// Result of a pre-transition check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreCheckResult {
    /// Name of the check that produced this result.
    pub check_name: String,
    /// Whether the check passed.
    pub passed: bool,
    /// Whether a failure should block the transition or just warn.
    pub blocking: bool,
    /// Human-readable explanation.
    pub message: String,
}

impl PreCheckResult {
    /// Create a passing result.
    pub fn pass(check_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            check_name: check_name.into(),
            passed: true,
            blocking: false,
            message: message.into(),
        }
    }

    /// Create a blocking failure result.
    pub fn block(check_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            check_name: check_name.into(),
            passed: false,
            blocking: true,
            message: message.into(),
        }
    }

    /// Create a warning (non-blocking failure) result.
    pub fn warn(check_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            check_name: check_name.into(),
            passed: false,
            blocking: false,
            message: message.into(),
        }
    }
}

impl fmt::Display for PreCheckResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.passed {
            "PASS"
        } else if self.blocking {
            "BLOCK"
        } else {
            "WARN"
        };
        write!(f, "[{}] {}: {}", status, self.check_name, self.message)
    }
}

/// A pre-transition check that can be registered with the hook registry.
///
/// Pre-transition checks inspect the `TransitionEvent` and return a
/// `PreCheckResult` indicating whether the transition should proceed.
pub struct PreTransitionCheck {
    /// Unique name for this check.
    pub name: String,
    /// Execution priority (lower runs first).
    pub priority: HookPriority,
    /// Optional filter: only run for specific document types.
    pub document_type_filter: Option<Vec<DocumentType>>,
    /// Optional filter: only run for transitions from specific phases.
    pub from_phase_filter: Option<Vec<Phase>>,
    /// Optional filter: only run for transitions to specific phases.
    pub to_phase_filter: Option<Vec<Phase>>,
    /// The check function. Returns a PreCheckResult.
    check_fn: Box<dyn Fn(&TransitionEvent) -> PreCheckResult + Send + Sync>,
}

impl PreTransitionCheck {
    /// Create a new pre-transition check.
    pub fn new(
        name: impl Into<String>,
        check_fn: impl Fn(&TransitionEvent) -> PreCheckResult + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            priority: HookPriority::default(),
            document_type_filter: None,
            from_phase_filter: None,
            to_phase_filter: None,
            check_fn: Box::new(check_fn),
        }
    }

    /// Set the priority for this check.
    pub fn with_priority(mut self, priority: HookPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Only run this check for the given document types.
    pub fn for_document_types(mut self, types: Vec<DocumentType>) -> Self {
        self.document_type_filter = Some(types);
        self
    }

    /// Only run this check for transitions from the given phases.
    pub fn for_from_phases(mut self, phases: Vec<Phase>) -> Self {
        self.from_phase_filter = Some(phases);
        self
    }

    /// Only run this check for transitions to the given phases.
    pub fn for_to_phases(mut self, phases: Vec<Phase>) -> Self {
        self.to_phase_filter = Some(phases);
        self
    }

    /// Check whether this hook should fire for the given event.
    pub fn matches(&self, event: &TransitionEvent) -> bool {
        if let Some(ref types) = self.document_type_filter {
            if !types.contains(&event.document_type) {
                return false;
            }
        }
        if let Some(ref phases) = self.from_phase_filter {
            if !phases.contains(&event.from_phase) {
                return false;
            }
        }
        if let Some(ref phases) = self.to_phase_filter {
            if !phases.contains(&event.to_phase) {
                return false;
            }
        }
        true
    }

    /// Execute the check against the given event.
    pub fn execute(&self, event: &TransitionEvent) -> PreCheckResult {
        (self.check_fn)(event)
    }
}

impl fmt::Debug for PreTransitionCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreTransitionCheck")
            .field("name", &self.name)
            .field("priority", &self.priority)
            .field("document_type_filter", &self.document_type_filter)
            .field("from_phase_filter", &self.from_phase_filter)
            .field("to_phase_filter", &self.to_phase_filter)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// PostTransitionAction -- runs after a successful transition
// ---------------------------------------------------------------------------

/// Result of a post-transition action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostActionResult {
    /// Name of the action that produced this result.
    pub action_name: String,
    /// Whether the action succeeded.
    pub success: bool,
    /// Human-readable explanation.
    pub message: String,
}

impl PostActionResult {
    /// Create a successful result.
    pub fn ok(action_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            action_name: action_name.into(),
            success: true,
            message: message.into(),
        }
    }

    /// Create a failure result.
    pub fn err(action_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            action_name: action_name.into(),
            success: false,
            message: message.into(),
        }
    }
}

impl fmt::Display for PostActionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.success { "OK" } else { "ERR" };
        write!(f, "[{}] {}: {}", status, self.action_name, self.message)
    }
}

/// A post-transition action that runs after a successful transition.
///
/// Post-transition actions are for side effects like recording audit logs,
/// creating child documents, sending notifications, etc. Their failures
/// do not roll back the transition.
pub struct PostTransitionAction {
    /// Unique name for this action.
    pub name: String,
    /// Execution priority (lower runs first).
    pub priority: HookPriority,
    /// Optional filter: only run for specific document types.
    pub document_type_filter: Option<Vec<DocumentType>>,
    /// Optional filter: only run for transitions from specific phases.
    pub from_phase_filter: Option<Vec<Phase>>,
    /// Optional filter: only run for transitions to specific phases.
    pub to_phase_filter: Option<Vec<Phase>>,
    /// The action function.
    action_fn: Box<dyn Fn(&TransitionEvent) -> PostActionResult + Send + Sync>,
}

impl PostTransitionAction {
    /// Create a new post-transition action.
    pub fn new(
        name: impl Into<String>,
        action_fn: impl Fn(&TransitionEvent) -> PostActionResult + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            priority: HookPriority::default(),
            document_type_filter: None,
            from_phase_filter: None,
            to_phase_filter: None,
            action_fn: Box::new(action_fn),
        }
    }

    /// Set the priority for this action.
    pub fn with_priority(mut self, priority: HookPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Only run this action for the given document types.
    pub fn for_document_types(mut self, types: Vec<DocumentType>) -> Self {
        self.document_type_filter = Some(types);
        self
    }

    /// Only run this action for transitions to the given phases.
    pub fn for_to_phases(mut self, phases: Vec<Phase>) -> Self {
        self.to_phase_filter = Some(phases);
        self
    }

    /// Check whether this hook should fire for the given event.
    pub fn matches(&self, event: &TransitionEvent) -> bool {
        if let Some(ref types) = self.document_type_filter {
            if !types.contains(&event.document_type) {
                return false;
            }
        }
        if let Some(ref phases) = self.from_phase_filter {
            if !phases.contains(&event.from_phase) {
                return false;
            }
        }
        if let Some(ref phases) = self.to_phase_filter {
            if !phases.contains(&event.to_phase) {
                return false;
            }
        }
        true
    }

    /// Execute the action for the given event.
    pub fn execute(&self, event: &TransitionEvent) -> PostActionResult {
        (self.action_fn)(event)
    }
}

impl fmt::Debug for PostTransitionAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PostTransitionAction")
            .field("name", &self.name)
            .field("priority", &self.priority)
            .field("document_type_filter", &self.document_type_filter)
            .field("from_phase_filter", &self.from_phase_filter)
            .field("to_phase_filter", &self.to_phase_filter)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event() -> TransitionEvent {
        TransitionEvent::new(
            DocumentId::from("test-doc"),
            DocumentType::Task,
            Phase::Active,
            Phase::Completed,
            "test-actor",
            false,
        )
    }

    #[test]
    fn test_transition_event_display() {
        let event = sample_event();
        let display = event.to_string();
        assert!(display.contains("test-doc"));
        assert!(display.contains("active"));
        assert!(display.contains("completed"));
        assert!(!display.contains("FORCED"));
    }

    #[test]
    fn test_transition_event_forced_display() {
        let event = TransitionEvent::new(
            DocumentId::from("doc"),
            DocumentType::Task,
            Phase::Active,
            Phase::Completed,
            "actor",
            true,
        );
        assert!(event.to_string().contains("[FORCED]"));
    }

    #[test]
    fn test_hook_priority_ordering() {
        assert!(HookPriority::SYSTEM < HookPriority::GATE);
        assert!(HookPriority::GATE < HookPriority::USER);
        assert!(HookPriority::USER < HookPriority::ADVISORY);
    }

    #[test]
    fn test_pre_check_result_pass() {
        let r = PreCheckResult::pass("test", "all good");
        assert!(r.passed);
        assert!(!r.blocking);
        assert!(r.to_string().contains("[PASS]"));
    }

    #[test]
    fn test_pre_check_result_block() {
        let r = PreCheckResult::block("test", "not ready");
        assert!(!r.passed);
        assert!(r.blocking);
        assert!(r.to_string().contains("[BLOCK]"));
    }

    #[test]
    fn test_pre_check_result_warn() {
        let r = PreCheckResult::warn("test", "could be better");
        assert!(!r.passed);
        assert!(!r.blocking);
        assert!(r.to_string().contains("[WARN]"));
    }

    #[test]
    fn test_pre_transition_check_matches_all() {
        let check = PreTransitionCheck::new("always", |_| PreCheckResult::pass("always", "ok"));
        assert!(check.matches(&sample_event()));
    }

    #[test]
    fn test_pre_transition_check_document_type_filter() {
        let check = PreTransitionCheck::new("epic-only", |_| PreCheckResult::pass("epic-only", "ok"))
            .for_document_types(vec![DocumentType::Epic]);

        let task_event = sample_event();
        assert!(!check.matches(&task_event));

        let epic_event = TransitionEvent::new(
            DocumentId::from("epic-1"),
            DocumentType::Epic,
            Phase::Active,
            Phase::Completed,
            "actor",
            false,
        );
        assert!(check.matches(&epic_event));
    }

    #[test]
    fn test_pre_transition_check_phase_filters() {
        let check = PreTransitionCheck::new("completion-gate", |_| {
            PreCheckResult::pass("completion-gate", "ok")
        })
        .for_to_phases(vec![Phase::Completed]);

        let to_completed = sample_event();
        assert!(check.matches(&to_completed));

        let to_active = TransitionEvent::new(
            DocumentId::from("doc"),
            DocumentType::Task,
            Phase::Todo,
            Phase::Active,
            "actor",
            false,
        );
        assert!(!check.matches(&to_active));
    }

    #[test]
    fn test_pre_transition_check_execute() {
        let check = PreTransitionCheck::new("gate", |event| {
            if event.forced {
                PreCheckResult::warn("gate", "forced, skipping")
            } else {
                PreCheckResult::block("gate", "not forced")
            }
        });

        let event = sample_event();
        let result = check.execute(&event);
        assert!(!result.passed);
        assert!(result.blocking);
    }

    #[test]
    fn test_post_action_result_ok() {
        let r = PostActionResult::ok("audit", "logged");
        assert!(r.success);
        assert!(r.to_string().contains("[OK]"));
    }

    #[test]
    fn test_post_action_result_err() {
        let r = PostActionResult::err("audit", "failed to log");
        assert!(!r.success);
        assert!(r.to_string().contains("[ERR]"));
    }

    #[test]
    fn test_post_transition_action_matches_and_executes() {
        let action = PostTransitionAction::new("log", |event| {
            PostActionResult::ok("log", format!("logged transition for {}", event.document_id))
        })
        .for_document_types(vec![DocumentType::Task, DocumentType::Story]);

        let event = sample_event();
        assert!(action.matches(&event));

        let result = action.execute(&event);
        assert!(result.success);
        assert!(result.message.contains("test-doc"));
    }

    #[test]
    fn test_post_transition_action_filter_mismatch() {
        let action = PostTransitionAction::new("epic-only", |_| {
            PostActionResult::ok("epic-only", "ok")
        })
        .for_document_types(vec![DocumentType::Epic]);

        assert!(!action.matches(&sample_event())); // sample is Task
    }
}

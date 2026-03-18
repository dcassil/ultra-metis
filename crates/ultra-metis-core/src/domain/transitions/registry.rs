//! Hook Registry -- manages registered pre-transition checks and post-transition actions.
//!
//! The registry is the central collection point for all hooks. Hooks can be
//! registered, unregistered by name, and queried for a given transition event.

use super::hooks::{
    PostActionResult, PostTransitionAction, PreCheckResult, PreTransitionCheck, TransitionEvent,
};

// ---------------------------------------------------------------------------
// HookRegistry
// ---------------------------------------------------------------------------

/// Central registry for transition hooks.
///
/// Manages pre-transition checks and post-transition actions. When queried
/// for a transition event, returns matching hooks sorted by priority.
pub struct HookRegistry {
    pre_checks: Vec<PreTransitionCheck>,
    post_actions: Vec<PostTransitionAction>,
}

impl HookRegistry {
    /// Create an empty hook registry.
    pub fn new() -> Self {
        Self {
            pre_checks: Vec::new(),
            post_actions: Vec::new(),
        }
    }

    // -- Registration -------------------------------------------------------

    /// Register a pre-transition check.
    pub fn register_pre_check(&mut self, check: PreTransitionCheck) {
        self.pre_checks.push(check);
    }

    /// Register a post-transition action.
    pub fn register_post_action(&mut self, action: PostTransitionAction) {
        self.post_actions.push(action);
    }

    /// Unregister a pre-transition check by name. Returns true if found and removed.
    pub fn unregister_pre_check(&mut self, name: &str) -> bool {
        let before = self.pre_checks.len();
        self.pre_checks.retain(|c| c.name != name);
        self.pre_checks.len() < before
    }

    /// Unregister a post-transition action by name. Returns true if found and removed.
    pub fn unregister_post_action(&mut self, name: &str) -> bool {
        let before = self.post_actions.len();
        self.post_actions.retain(|a| a.name != name);
        self.post_actions.len() < before
    }

    // -- Queries ------------------------------------------------------------

    /// Get the number of registered pre-transition checks.
    pub fn pre_check_count(&self) -> usize {
        self.pre_checks.len()
    }

    /// Get the number of registered post-transition actions.
    pub fn post_action_count(&self) -> usize {
        self.post_actions.len()
    }

    /// Get the names of all registered pre-transition checks.
    pub fn pre_check_names(&self) -> Vec<&str> {
        self.pre_checks.iter().map(|c| c.name.as_str()).collect()
    }

    /// Get the names of all registered post-transition actions.
    pub fn post_action_names(&self) -> Vec<&str> {
        self.post_actions.iter().map(|a| a.name.as_str()).collect()
    }

    // -- Execution ----------------------------------------------------------

    /// Run all matching pre-transition checks for the given event.
    ///
    /// Returns results sorted by hook priority (lower priority values first).
    pub fn run_pre_checks(&self, event: &TransitionEvent) -> Vec<PreCheckResult> {
        let mut matching: Vec<&PreTransitionCheck> = self
            .pre_checks
            .iter()
            .filter(|c| c.matches(event))
            .collect();

        // Sort by priority (lower values first)
        matching.sort_by_key(|c| c.priority);

        matching.iter().map(|c| c.execute(event)).collect()
    }

    /// Run all matching post-transition actions for the given event.
    ///
    /// Returns results sorted by hook priority (lower priority values first).
    pub fn run_post_actions(&self, event: &TransitionEvent) -> Vec<PostActionResult> {
        let mut matching: Vec<&PostTransitionAction> = self
            .post_actions
            .iter()
            .filter(|a| a.matches(event))
            .collect();

        // Sort by priority (lower values first)
        matching.sort_by_key(|a| a.priority);

        matching.iter().map(|a| a.execute(event)).collect()
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for HookRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HookRegistry")
            .field("pre_checks", &self.pre_check_names())
            .field("post_actions", &self.post_action_names())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::documents::types::{DocumentId, DocumentType, Phase};
    use crate::domain::transitions::hooks::HookPriority;

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
    fn test_empty_registry() {
        let registry = HookRegistry::new();
        assert_eq!(registry.pre_check_count(), 0);
        assert_eq!(registry.post_action_count(), 0);
    }

    #[test]
    fn test_register_and_count() {
        let mut registry = HookRegistry::new();

        registry.register_pre_check(PreTransitionCheck::new("check1", |_| {
            PreCheckResult::pass("check1", "ok")
        }));
        registry.register_pre_check(PreTransitionCheck::new("check2", |_| {
            PreCheckResult::pass("check2", "ok")
        }));
        registry.register_post_action(PostTransitionAction::new("action1", |_| {
            PostActionResult::ok("action1", "done")
        }));

        assert_eq!(registry.pre_check_count(), 2);
        assert_eq!(registry.post_action_count(), 1);
        assert_eq!(registry.pre_check_names(), vec!["check1", "check2"]);
        assert_eq!(registry.post_action_names(), vec!["action1"]);
    }

    #[test]
    fn test_unregister_pre_check() {
        let mut registry = HookRegistry::new();
        registry.register_pre_check(PreTransitionCheck::new("a", |_| {
            PreCheckResult::pass("a", "ok")
        }));
        registry.register_pre_check(PreTransitionCheck::new("b", |_| {
            PreCheckResult::pass("b", "ok")
        }));

        assert!(registry.unregister_pre_check("a"));
        assert_eq!(registry.pre_check_count(), 1);
        assert_eq!(registry.pre_check_names(), vec!["b"]);

        // Unregistering non-existent returns false
        assert!(!registry.unregister_pre_check("nonexistent"));
    }

    #[test]
    fn test_unregister_post_action() {
        let mut registry = HookRegistry::new();
        registry.register_post_action(PostTransitionAction::new("x", |_| {
            PostActionResult::ok("x", "ok")
        }));

        assert!(registry.unregister_post_action("x"));
        assert_eq!(registry.post_action_count(), 0);
        assert!(!registry.unregister_post_action("x"));
    }

    #[test]
    fn test_run_pre_checks_all_pass() {
        let mut registry = HookRegistry::new();
        registry.register_pre_check(PreTransitionCheck::new("gate1", |_| {
            PreCheckResult::pass("gate1", "ok")
        }));
        registry.register_pre_check(PreTransitionCheck::new("gate2", |_| {
            PreCheckResult::pass("gate2", "ok")
        }));

        let results = registry.run_pre_checks(&sample_event());
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.passed));
    }

    #[test]
    fn test_run_pre_checks_with_blocking_failure() {
        let mut registry = HookRegistry::new();
        registry.register_pre_check(PreTransitionCheck::new("pass", |_| {
            PreCheckResult::pass("pass", "ok")
        }));
        registry.register_pre_check(PreTransitionCheck::new("fail", |_| {
            PreCheckResult::block("fail", "not ready")
        }));

        let results = registry.run_pre_checks(&sample_event());
        assert_eq!(results.len(), 2);
        assert!(results[0].passed);
        assert!(!results[1].passed);
        assert!(results[1].blocking);
    }

    #[test]
    fn test_run_pre_checks_respects_priority_order() {
        let mut registry = HookRegistry::new();

        // Register in wrong order; should execute by priority
        registry.register_pre_check(
            PreTransitionCheck::new("low", |_| PreCheckResult::pass("low", "low"))
                .with_priority(HookPriority::ADVISORY),
        );
        registry.register_pre_check(
            PreTransitionCheck::new("high", |_| PreCheckResult::pass("high", "high"))
                .with_priority(HookPriority::SYSTEM),
        );

        let results = registry.run_pre_checks(&sample_event());
        assert_eq!(results[0].check_name, "high");
        assert_eq!(results[1].check_name, "low");
    }

    #[test]
    fn test_run_pre_checks_respects_filters() {
        let mut registry = HookRegistry::new();

        registry.register_pre_check(
            PreTransitionCheck::new("epic-only", |_| PreCheckResult::pass("epic-only", "ok"))
                .for_document_types(vec![DocumentType::Epic]),
        );
        registry.register_pre_check(PreTransitionCheck::new("all", |_| {
            PreCheckResult::pass("all", "ok")
        }));

        let results = registry.run_pre_checks(&sample_event()); // Task event
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].check_name, "all");
    }

    #[test]
    fn test_run_post_actions() {
        let mut registry = HookRegistry::new();
        registry.register_post_action(PostTransitionAction::new("audit", |_| {
            PostActionResult::ok("audit", "logged")
        }));

        let results = registry.run_post_actions(&sample_event());
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
    }

    #[test]
    fn test_run_post_actions_respects_priority() {
        let mut registry = HookRegistry::new();

        registry.register_post_action(
            PostTransitionAction::new("second", |_| PostActionResult::ok("second", "ok"))
                .with_priority(HookPriority::USER),
        );
        registry.register_post_action(
            PostTransitionAction::new("first", |_| PostActionResult::ok("first", "ok"))
                .with_priority(HookPriority::SYSTEM),
        );

        let results = registry.run_post_actions(&sample_event());
        assert_eq!(results[0].action_name, "first");
        assert_eq!(results[1].action_name, "second");
    }

    #[test]
    fn test_default_trait() {
        let registry = HookRegistry::default();
        assert_eq!(registry.pre_check_count(), 0);
    }

    #[test]
    fn test_debug_display() {
        let mut registry = HookRegistry::new();
        registry.register_pre_check(PreTransitionCheck::new("check", |_| {
            PreCheckResult::pass("check", "ok")
        }));
        let debug = format!("{:?}", registry);
        assert!(debug.contains("check"));
    }
}

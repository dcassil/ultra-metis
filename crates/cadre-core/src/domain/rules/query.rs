//! Rule query engine with scope inheritance and filtering.
//!
//! Provides programmatic querying of RulesConfig documents by category,
//! scope, protection level, and other attributes. Supports scope inheritance
//! so that higher-scope rules apply to lower scopes unless overridden.

use crate::domain::documents::rules_config::{
    ProtectionLevel, RuleCategory, RuleScope, RulesConfig,
};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// RuleQuery — filter criteria for searching rules
// ---------------------------------------------------------------------------

/// A query for filtering RulesConfig documents.
///
/// All fields are optional — an empty query matches everything.
/// Multiple fields are combined with AND semantics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuleQuery {
    /// Filter by rule category.
    pub category: Option<RuleCategory>,
    /// Filter by scope (exact match, not inheritance-aware).
    pub scope: Option<RuleScope>,
    /// Filter by protection level.
    pub protection_level: Option<ProtectionLevel>,
    /// Filter by source architecture reference (exact match on short code).
    pub source_architecture_ref: Option<String>,
    /// If true, include archived rules. Default: false.
    pub include_archived: bool,
}

impl RuleQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_category(mut self, category: RuleCategory) -> Self {
        self.category = Some(category);
        self
    }

    pub fn with_scope(mut self, scope: RuleScope) -> Self {
        self.scope = Some(scope);
        self
    }

    pub fn with_protection_level(mut self, level: ProtectionLevel) -> Self {
        self.protection_level = Some(level);
        self
    }

    pub fn with_source_architecture_ref(mut self, ref_code: &str) -> Self {
        self.source_architecture_ref = Some(ref_code.to_string());
        self
    }

    pub fn including_archived(mut self) -> Self {
        self.include_archived = true;
        self
    }
}

// ---------------------------------------------------------------------------
// Scope inheritance
// ---------------------------------------------------------------------------

/// Returns the ordered scope hierarchy from broadest to narrowest.
///
/// Platform > Organization > Repository > Package > Component > Task
pub fn scope_hierarchy() -> &'static [RuleScope] {
    &[
        RuleScope::Platform,
        RuleScope::Organization,
        RuleScope::Repository,
        RuleScope::Package,
        RuleScope::Component,
        RuleScope::Task,
    ]
}

/// Returns the numeric rank of a scope (0 = broadest, 5 = narrowest).
pub fn scope_rank(scope: RuleScope) -> usize {
    match scope {
        RuleScope::Platform => 0,
        RuleScope::Organization => 1,
        RuleScope::Repository => 2,
        RuleScope::Package => 3,
        RuleScope::Component => 4,
        RuleScope::Task => 5,
    }
}

/// Returns true if `parent_scope` is equal to or broader than `child_scope`.
///
/// A Platform rule applies to all scopes. A Package rule applies to Package,
/// Component, and Task scopes.
pub fn scope_applies(parent_scope: RuleScope, child_scope: RuleScope) -> bool {
    scope_rank(parent_scope) <= scope_rank(child_scope)
}

// ---------------------------------------------------------------------------
// RuleQueryEngine
// ---------------------------------------------------------------------------

/// The rule query engine filters and retrieves applicable rules.
///
/// It operates over a collection of RulesConfig references and supports
/// both exact-match filtering and scope-inheritance-aware queries.
pub struct RuleQueryEngine<'a> {
    rules: &'a [&'a RulesConfig],
}

impl<'a> RuleQueryEngine<'a> {
    /// Create a new query engine from a slice of RulesConfig references.
    pub fn new(rules: &'a [&'a RulesConfig]) -> Self {
        Self { rules }
    }

    /// Execute a query and return matching rules.
    pub fn query(&self, q: &RuleQuery) -> Vec<&'a RulesConfig> {
        self.rules
            .iter()
            .filter(|rule| self.matches(rule, q))
            .copied()
            .collect()
    }

    /// Find all rules that apply to a given scope via inheritance.
    ///
    /// Returns rules whose scope is equal to or broader than the target scope.
    /// For example, querying for `Package` scope returns Platform, Organization,
    /// Repository, and Package rules.
    pub fn applicable_at_scope(&self, target_scope: RuleScope) -> Vec<&'a RulesConfig> {
        self.rules
            .iter()
            .filter(|rule| !rule.archived() && scope_applies(rule.scope, target_scope))
            .copied()
            .collect()
    }

    /// Find all rules that apply to a given scope, filtered by category.
    pub fn applicable_at_scope_with_category(
        &self,
        target_scope: RuleScope,
        category: RuleCategory,
    ) -> Vec<&'a RulesConfig> {
        self.applicable_at_scope(target_scope)
            .into_iter()
            .filter(|rule| {
                // Check if any tag matches the category
                rule.tags().iter().any(|tag| {
                    if let crate::domain::documents::types::Tag::Label(label) = tag {
                        label == &category.to_string()
                    } else {
                        false
                    }
                })
            })
            .collect()
    }

    /// Find all protected rules (useful for governance auditing).
    pub fn protected_rules(&self) -> Vec<&'a RulesConfig> {
        self.rules
            .iter()
            .filter(|rule| !rule.archived() && rule.protection_level == ProtectionLevel::Protected)
            .copied()
            .collect()
    }

    /// Find rules seeded from a specific architecture reference.
    pub fn rules_from_architecture(&self, arch_ref: &str) -> Vec<&'a RulesConfig> {
        self.rules
            .iter()
            .filter(|rule| {
                !rule.archived()
                    && rule
                        .source_architecture_ref
                        .as_deref()
                        .map(|r| r == arch_ref)
                        .unwrap_or(false)
            })
            .copied()
            .collect()
    }

    fn matches(&self, rule: &RulesConfig, q: &RuleQuery) -> bool {
        // Archived filter
        if !q.include_archived && rule.archived() {
            return false;
        }

        // Scope filter (exact match)
        if let Some(scope) = q.scope {
            if rule.scope != scope {
                return false;
            }
        }

        // Protection level filter
        if let Some(level) = q.protection_level {
            if rule.protection_level != level {
                return false;
            }
        }

        // Source architecture ref filter
        if let Some(ref arch_ref) = q.source_architecture_ref {
            match &rule.source_architecture_ref {
                Some(r) if r == arch_ref => {}
                _ => return false,
            }
        }

        // Category filter — check tags
        if let Some(category) = q.category {
            let cat_str = category.to_string();
            let has_category = rule.tags().iter().any(|tag| {
                if let crate::domain::documents::types::Tag::Label(label) = tag {
                    label == &cat_str
                } else {
                    false
                }
            });
            if !has_category {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::documents::types::{Phase, Tag};

    fn make_rule(
        title: &str,
        short_code: &str,
        scope: RuleScope,
        protection: ProtectionLevel,
        arch_ref: Option<&str>,
        tags: Vec<Tag>,
    ) -> RulesConfig {
        RulesConfig::new(
            title.to_string(),
            tags,
            false,
            short_code.to_string(),
            protection,
            scope,
            arch_ref.map(|s| s.to_string()),
        )
        .unwrap()
    }

    #[test]
    fn test_scope_hierarchy_ordering() {
        assert!(scope_rank(RuleScope::Platform) < scope_rank(RuleScope::Organization));
        assert!(scope_rank(RuleScope::Organization) < scope_rank(RuleScope::Repository));
        assert!(scope_rank(RuleScope::Repository) < scope_rank(RuleScope::Package));
        assert!(scope_rank(RuleScope::Package) < scope_rank(RuleScope::Component));
        assert!(scope_rank(RuleScope::Component) < scope_rank(RuleScope::Task));
    }

    #[test]
    fn test_scope_applies() {
        // Platform applies everywhere
        assert!(scope_applies(RuleScope::Platform, RuleScope::Platform));
        assert!(scope_applies(RuleScope::Platform, RuleScope::Task));

        // Task only applies to task
        assert!(scope_applies(RuleScope::Task, RuleScope::Task));
        assert!(!scope_applies(RuleScope::Task, RuleScope::Component));

        // Repository applies to repo, package, component, task
        assert!(scope_applies(RuleScope::Repository, RuleScope::Repository));
        assert!(scope_applies(RuleScope::Repository, RuleScope::Package));
        assert!(!scope_applies(
            RuleScope::Repository,
            RuleScope::Organization
        ));
    }

    #[test]
    fn test_query_all_rules() {
        let r1 = make_rule(
            "Platform Rules",
            "RC-0001",
            RuleScope::Platform,
            ProtectionLevel::Protected,
            None,
            vec![Tag::Phase(Phase::Published)],
        );
        let r2 = make_rule(
            "Repo Rules",
            "RC-0002",
            RuleScope::Repository,
            ProtectionLevel::Standard,
            None,
            vec![Tag::Phase(Phase::Draft)],
        );
        let rules: Vec<&RulesConfig> = vec![&r1, &r2];
        let engine = RuleQueryEngine::new(&rules);

        let results = engine.query(&RuleQuery::new());
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_by_scope() {
        let r1 = make_rule(
            "Platform Rules",
            "RC-0001",
            RuleScope::Platform,
            ProtectionLevel::Protected,
            None,
            vec![Tag::Phase(Phase::Published)],
        );
        let r2 = make_rule(
            "Repo Rules",
            "RC-0002",
            RuleScope::Repository,
            ProtectionLevel::Standard,
            None,
            vec![Tag::Phase(Phase::Draft)],
        );
        let rules: Vec<&RulesConfig> = vec![&r1, &r2];
        let engine = RuleQueryEngine::new(&rules);

        let results = engine.query(&RuleQuery::new().with_scope(RuleScope::Platform));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title(), "Platform Rules");
    }

    #[test]
    fn test_query_by_protection_level() {
        let r1 = make_rule(
            "Protected",
            "RC-0001",
            RuleScope::Repository,
            ProtectionLevel::Protected,
            None,
            vec![Tag::Phase(Phase::Published)],
        );
        let r2 = make_rule(
            "Standard",
            "RC-0002",
            RuleScope::Repository,
            ProtectionLevel::Standard,
            None,
            vec![Tag::Phase(Phase::Draft)],
        );
        let rules: Vec<&RulesConfig> = vec![&r1, &r2];
        let engine = RuleQueryEngine::new(&rules);

        let results = engine.protected_rules();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title(), "Protected");
    }

    #[test]
    fn test_applicable_at_scope_with_inheritance() {
        let platform = make_rule(
            "Platform",
            "RC-0001",
            RuleScope::Platform,
            ProtectionLevel::Protected,
            None,
            vec![Tag::Phase(Phase::Published)],
        );
        let repo = make_rule(
            "Repo",
            "RC-0002",
            RuleScope::Repository,
            ProtectionLevel::Standard,
            None,
            vec![Tag::Phase(Phase::Draft)],
        );
        let component = make_rule(
            "Component",
            "RC-0003",
            RuleScope::Component,
            ProtectionLevel::Standard,
            None,
            vec![Tag::Phase(Phase::Draft)],
        );
        let rules: Vec<&RulesConfig> = vec![&platform, &repo, &component];
        let engine = RuleQueryEngine::new(&rules);

        // Package scope should inherit Platform and Repo rules
        let applicable = engine.applicable_at_scope(RuleScope::Package);
        assert_eq!(applicable.len(), 2);
        let titles: Vec<&str> = applicable.iter().map(|r| r.title()).collect();
        assert!(titles.contains(&"Platform"));
        assert!(titles.contains(&"Repo"));
        assert!(!titles.contains(&"Component"));

        // Component scope should get all three
        let applicable = engine.applicable_at_scope(RuleScope::Component);
        assert_eq!(applicable.len(), 3);
    }

    #[test]
    fn test_rules_from_architecture() {
        let r1 = make_rule(
            "Arch Rules",
            "RC-0001",
            RuleScope::Repository,
            ProtectionLevel::Protected,
            Some("ARCH-0001"),
            vec![Tag::Phase(Phase::Published)],
        );
        let r2 = make_rule(
            "Manual Rules",
            "RC-0002",
            RuleScope::Repository,
            ProtectionLevel::Standard,
            None,
            vec![Tag::Phase(Phase::Draft)],
        );
        let rules: Vec<&RulesConfig> = vec![&r1, &r2];
        let engine = RuleQueryEngine::new(&rules);

        let arch_rules = engine.rules_from_architecture("ARCH-0001");
        assert_eq!(arch_rules.len(), 1);
        assert_eq!(arch_rules[0].title(), "Arch Rules");

        let no_rules = engine.rules_from_architecture("ARCH-9999");
        assert!(no_rules.is_empty());
    }

    #[test]
    fn test_query_with_category_tag() {
        let r1 = make_rule(
            "Arch Rules",
            "RC-0001",
            RuleScope::Repository,
            ProtectionLevel::Standard,
            None,
            vec![
                Tag::Phase(Phase::Draft),
                Tag::Label("architectural".to_string()),
            ],
        );
        let r2 = make_rule(
            "Behavior Rules",
            "RC-0002",
            RuleScope::Repository,
            ProtectionLevel::Standard,
            None,
            vec![
                Tag::Phase(Phase::Draft),
                Tag::Label("behavioral".to_string()),
            ],
        );
        let rules: Vec<&RulesConfig> = vec![&r1, &r2];
        let engine = RuleQueryEngine::new(&rules);

        let results = engine.query(&RuleQuery::new().with_category(RuleCategory::Architectural));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title(), "Arch Rules");
    }

    #[test]
    fn test_query_excludes_archived_by_default() {
        let active = make_rule(
            "Active",
            "RC-0001",
            RuleScope::Repository,
            ProtectionLevel::Standard,
            None,
            vec![Tag::Phase(Phase::Draft)],
        );
        let archived = RulesConfig::new(
            "Archived".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            true, // archived
            "RC-0002".to_string(),
            ProtectionLevel::Standard,
            RuleScope::Repository,
            None,
        )
        .unwrap();
        let rules: Vec<&RulesConfig> = vec![&active, &archived];
        let engine = RuleQueryEngine::new(&rules);

        let results = engine.query(&RuleQuery::new());
        assert_eq!(results.len(), 1);

        let results = engine.query(&RuleQuery::new().including_archived());
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_combined_query_filters() {
        let r1 = make_rule(
            "Protected Platform Arch",
            "RC-0001",
            RuleScope::Platform,
            ProtectionLevel::Protected,
            Some("ARCH-0001"),
            vec![
                Tag::Phase(Phase::Published),
                Tag::Label("architectural".to_string()),
            ],
        );
        let r2 = make_rule(
            "Standard Repo Arch",
            "RC-0002",
            RuleScope::Repository,
            ProtectionLevel::Standard,
            Some("ARCH-0001"),
            vec![
                Tag::Phase(Phase::Draft),
                Tag::Label("architectural".to_string()),
            ],
        );
        let r3 = make_rule(
            "Protected Repo Behavioral",
            "RC-0003",
            RuleScope::Repository,
            ProtectionLevel::Protected,
            None,
            vec![
                Tag::Phase(Phase::Published),
                Tag::Label("behavioral".to_string()),
            ],
        );
        let rules: Vec<&RulesConfig> = vec![&r1, &r2, &r3];
        let engine = RuleQueryEngine::new(&rules);

        // Protected + architectural
        let results = engine.query(
            &RuleQuery::new()
                .with_protection_level(ProtectionLevel::Protected)
                .with_category(RuleCategory::Architectural),
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title(), "Protected Platform Arch");

        // From ARCH-0001
        let results = engine.query(&RuleQuery::new().with_source_architecture_ref("ARCH-0001"));
        assert_eq!(results.len(), 2);
    }
}

//! Architecture-derived rule seeding from catalog entries.
//!
//! When a Reference Architecture is selected, this module generates a starter
//! set of RulesConfig documents from the architecture pattern's seed data.
//! Each seeded rule carries a `source_architecture_ref` linking it back to
//! the originating catalog entry for traceability.

use crate::domain::documents::architecture_catalog_entry::ArchitectureCatalogEntry;
use crate::domain::documents::rules_config::{
    ProtectionLevel, RuleCategory, RuleScope, RulesConfig,
};
use crate::domain::documents::traits::DocumentValidationError;
use crate::domain::documents::types::{Phase, Tag};

// ---------------------------------------------------------------------------
// SeedRule — an intermediate representation of a rule to be created
// ---------------------------------------------------------------------------

/// A seed rule definition derived from an architecture catalog entry.
///
/// This is an intermediate representation that captures the intent before
/// the actual RulesConfig document is created.
#[derive(Debug, Clone)]
pub struct SeedRule {
    /// Title for the generated RulesConfig.
    pub title: String,
    /// The category of rule being seeded.
    pub category: RuleCategory,
    /// Scope at which this rule applies.
    pub scope: RuleScope,
    /// Protection level for the seeded rule.
    pub protection_level: ProtectionLevel,
    /// The rule text to include in the content body.
    pub rule_text: String,
}

// ---------------------------------------------------------------------------
// RuleSeeder
// ---------------------------------------------------------------------------

/// Generates seed rules from an ArchitectureCatalogEntry.
///
/// The seeder extracts structured data from the catalog entry and produces
/// a set of `SeedRule` definitions that can be materialized into RulesConfig
/// documents.
pub struct RuleSeeder;

impl RuleSeeder {
    /// Extract seed rules from an architecture catalog entry.
    ///
    /// Generates rules from:
    /// - `folder_layout` -> Architectural rules
    /// - `dependency_rules` -> Architectural rules
    /// - `naming_conventions` -> Behavioral rules
    /// - `module_boundaries` -> Architectural rules
    /// - `rules_seed_hints` -> Mixed (inferred from content)
    pub fn extract_seeds(entry: &ArchitectureCatalogEntry) -> Vec<SeedRule> {
        let mut seeds = Vec::new();

        // Folder layout rules
        if !entry.folder_layout.is_empty() {
            seeds.push(SeedRule {
                title: format!("{} - Folder Structure Rules", entry.title()),
                category: RuleCategory::Architectural,
                scope: RuleScope::Repository,
                protection_level: ProtectionLevel::Protected,
                rule_text: Self::format_rule_list("Folder Structure", &entry.folder_layout),
            });
        }

        // Dependency direction rules
        if !entry.dependency_rules.is_empty() {
            seeds.push(SeedRule {
                title: format!("{} - Dependency Direction Rules", entry.title()),
                category: RuleCategory::Architectural,
                scope: RuleScope::Repository,
                protection_level: ProtectionLevel::Protected,
                rule_text: Self::format_rule_list("Dependency Direction", &entry.dependency_rules),
            });
        }

        // Naming convention rules
        if !entry.naming_conventions.is_empty() {
            seeds.push(SeedRule {
                title: format!("{} - Naming Conventions", entry.title()),
                category: RuleCategory::Behavioral,
                scope: RuleScope::Repository,
                protection_level: ProtectionLevel::Standard,
                rule_text: Self::format_rule_list("Naming Conventions", &entry.naming_conventions),
            });
        }

        // Module boundary rules
        if !entry.module_boundaries.is_empty() {
            seeds.push(SeedRule {
                title: format!("{} - Module Boundary Rules", entry.title()),
                category: RuleCategory::Architectural,
                scope: RuleScope::Package,
                protection_level: ProtectionLevel::Protected,
                rule_text: Self::format_rule_list("Module Boundaries", &entry.module_boundaries),
            });
        }

        // Explicit rule seed hints (these are additional rules the architect specified)
        for hint in &entry.rules_seed_hints {
            seeds.push(SeedRule {
                title: format!("{} - {}", entry.title(), Self::title_from_hint(hint)),
                category: Self::infer_category(hint),
                scope: RuleScope::Repository,
                protection_level: ProtectionLevel::Standard,
                rule_text: hint.clone(),
            });
        }

        seeds
    }

    /// Materialize seed rules into RulesConfig documents.
    ///
    /// Each generated RulesConfig references the source catalog entry via
    /// `source_architecture_ref` for traceability.
    pub fn materialize(
        seeds: &[SeedRule],
        source_ref: &str,
        short_code_prefix: &str,
        start_index: u32,
    ) -> Result<Vec<RulesConfig>, DocumentValidationError> {
        let mut configs = Vec::new();

        for (i, seed) in seeds.iter().enumerate() {
            let short_code = format!("{}-{:04}", short_code_prefix, start_index + i as u32);

            let tags = vec![
                Tag::Phase(Phase::Draft),
                Tag::Label(seed.category.to_string()),
            ];

            let template = format!(
                "# {}\n\n## Rules\n\n{}\n\n## Scope\n\nApplies at {} scope.\n\n## Protection Policy\n\nProtection level: {}. {}\n\n## Change History\n\nSeeded from architecture catalog entry: {}",
                seed.title,
                seed.rule_text,
                seed.scope,
                seed.protection_level,
                if seed.protection_level == ProtectionLevel::Protected {
                    "Changes require a RuleChangeProposal."
                } else {
                    "Direct edits permitted."
                },
                source_ref,
            );

            let config = RulesConfig::new_with_template(
                seed.title.clone(),
                tags,
                false,
                short_code,
                seed.protection_level,
                seed.scope,
                Some(source_ref.to_string()),
                &template,
            )?;

            configs.push(config);
        }

        Ok(configs)
    }

    fn format_rule_list(heading: &str, items: &[String]) -> String {
        let mut out = format!("### {}\n", heading);
        for item in items {
            out.push_str(&format!("\n- {}", item));
        }
        out
    }

    fn title_from_hint(hint: &str) -> String {
        // Take the first ~50 chars as a title, cleaning up
        let clean: String = hint.chars().take(50).collect::<String>().trim().to_string();
        if clean.len() < hint.len() {
            format!("{}...", clean)
        } else {
            clean
        }
    }

    fn infer_category(hint: &str) -> RuleCategory {
        let lower = hint.to_lowercase();
        if lower.contains("test") || lower.contains("quality") || lower.contains("validation") {
            RuleCategory::ValidationQuality
        } else if lower.contains("dependency")
            || lower.contains("import")
            || lower.contains("boundary")
        {
            RuleCategory::Architectural
        } else if lower.contains("naming")
            || lower.contains("convention")
            || lower.contains("style")
        {
            RuleCategory::Behavioral
        } else if lower.contains("approval")
            || lower.contains("escalat")
            || lower.contains("review")
        {
            RuleCategory::ApprovalEscalation
        } else if lower.contains("safety") || lower.contains("security") || lower.contains("danger")
        {
            RuleCategory::ExecutionSafety
        } else if lower.contains("deploy") || lower.contains("ops") || lower.contains("ci") {
            RuleCategory::Operational
        } else {
            RuleCategory::Behavioral
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_catalog_entry() -> ArchitectureCatalogEntry {
        ArchitectureCatalogEntry::new(
            "Rust Workspace CLI".to_string(),
            vec![Tag::Phase(Phase::Published)],
            false,
            "ACE-0001".to_string(),
            "rust".to_string(),
            "cli".to_string(),
        )
        .unwrap()
    }

    fn make_catalog_entry_with_data() -> ArchitectureCatalogEntry {
        let mut entry = make_catalog_entry();
        entry.folder_layout = vec![
            "src/ contains library code".to_string(),
            "src/bin/ contains binary entry points".to_string(),
        ];
        entry.dependency_rules = vec![
            "domain/ must not depend on infrastructure/".to_string(),
            "No circular dependencies between crates".to_string(),
        ];
        entry.naming_conventions = vec![
            "Modules use snake_case".to_string(),
            "Types use PascalCase".to_string(),
        ];
        entry.module_boundaries =
            vec!["Public API surface limited to mod.rs re-exports".to_string()];
        entry.rules_seed_hints = vec![
            "All public functions must have doc comments".to_string(),
            "Tests must be co-located in the same file".to_string(),
        ];
        entry
    }

    #[test]
    fn test_extract_seeds_empty_entry() {
        let entry = make_catalog_entry();
        let seeds = RuleSeeder::extract_seeds(&entry);
        assert!(seeds.is_empty());
    }

    #[test]
    fn test_extract_seeds_with_data() {
        let entry = make_catalog_entry_with_data();
        let seeds = RuleSeeder::extract_seeds(&entry);

        // Should produce: folder_layout, dependency_rules, naming_conventions,
        // module_boundaries, + 2 seed hints = 6 total
        assert_eq!(seeds.len(), 6);

        // Check folder layout rule
        let folder = &seeds[0];
        assert!(folder.title.contains("Folder Structure"));
        assert_eq!(folder.category, RuleCategory::Architectural);
        assert_eq!(folder.scope, RuleScope::Repository);
        assert_eq!(folder.protection_level, ProtectionLevel::Protected);

        // Check naming conventions
        let naming = &seeds[2];
        assert!(naming.title.contains("Naming Conventions"));
        assert_eq!(naming.category, RuleCategory::Behavioral);
        assert_eq!(naming.protection_level, ProtectionLevel::Standard);

        // Check module boundary rule has package scope
        let boundaries = &seeds[3];
        assert_eq!(boundaries.scope, RuleScope::Package);
    }

    #[test]
    fn test_materialize_creates_rules_configs() {
        let entry = make_catalog_entry_with_data();
        let seeds = RuleSeeder::extract_seeds(&entry);

        let configs = RuleSeeder::materialize(&seeds, "ACE-0001", "RC", 100).unwrap();
        assert_eq!(configs.len(), 6);

        // Check first config
        let first = &configs[0];
        assert!(first.title().contains("Folder Structure"));
        assert_eq!(first.protection_level, ProtectionLevel::Protected);
        assert_eq!(first.scope, RuleScope::Repository);
        assert_eq!(first.source_architecture_ref.as_deref(), Some("ACE-0001"));
        assert_eq!(first.metadata().short_code, "RC-0100");

        // Check short codes are sequential
        assert_eq!(configs[1].metadata().short_code, "RC-0101");
        assert_eq!(configs[2].metadata().short_code, "RC-0102");
    }

    #[test]
    fn test_materialize_empty_seeds() {
        let configs = RuleSeeder::materialize(&[], "ACE-0001", "RC", 1).unwrap();
        assert!(configs.is_empty());
    }

    #[test]
    fn test_infer_category() {
        assert_eq!(
            RuleSeeder::infer_category("All tests must pass before merge"),
            RuleCategory::ValidationQuality
        );
        assert_eq!(
            RuleSeeder::infer_category("No imports from internal modules"),
            RuleCategory::Architectural
        );
        assert_eq!(
            RuleSeeder::infer_category("Follow standard naming conventions"),
            RuleCategory::Behavioral
        );
        assert_eq!(
            RuleSeeder::infer_category("Require approval for breaking changes"),
            RuleCategory::ApprovalEscalation
        );
        assert_eq!(
            RuleSeeder::infer_category("CI pipeline must run on all PRs"),
            RuleCategory::Operational
        );
        assert_eq!(
            RuleSeeder::infer_category("Never disable safety checks"),
            RuleCategory::ExecutionSafety
        );
    }

    #[test]
    fn test_title_from_hint() {
        assert_eq!(RuleSeeder::title_from_hint("Short hint"), "Short hint");

        let long_hint =
            "This is a very long hint that exceeds fifty characters and should be truncated";
        let title = RuleSeeder::title_from_hint(long_hint);
        assert!(title.ends_with("..."));
        assert!(title.len() <= 54); // 50 chars + "..."
    }

    #[test]
    fn test_seeded_rules_have_category_tags() {
        let entry = make_catalog_entry_with_data();
        let seeds = RuleSeeder::extract_seeds(&entry);
        let configs = RuleSeeder::materialize(&seeds, "ACE-0001", "RC", 1).unwrap();

        // Each config should have a category label tag
        for config in &configs {
            let has_label = config.tags().iter().any(|t| matches!(t, Tag::Label(_)));
            assert!(
                has_label,
                "Config '{}' should have a category label tag",
                config.title()
            );
        }
    }
}

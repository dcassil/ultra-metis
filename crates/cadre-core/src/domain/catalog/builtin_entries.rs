//! Built-in architecture catalog entries.
//!
//! Production: returns an empty vec (entries are fetched from the remote catalog).
//! Tests: provides `test_builtin_entries()` with 5 JavaScript patterns for
//! deterministic testing without network access.

use crate::domain::documents::architecture_catalog_entry::ArchitectureCatalogEntry;

#[cfg(any(test, feature = "test-utils"))]
use crate::domain::documents::content::DocumentContent;
#[cfg(any(test, feature = "test-utils"))]
use crate::domain::documents::metadata::DocumentMetadata;
#[cfg(any(test, feature = "test-utils"))]
use crate::domain::documents::types::{Phase, Tag};

/// All built-in catalog entries.
///
/// Returns an empty vec — catalog entries are now fetched at runtime from the
/// external `dcassil/cadre-architecture-docs` repository via [`super::remote_fetcher`].
pub fn builtin_entries() -> Vec<ArchitectureCatalogEntry> {
    vec![]
}

// --- Test utilities ---

/// Convert a slice of string slices into owned String vec.
#[cfg(any(test, feature = "test-utils"))]
fn strs(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

/// Common tags for all test entries.
#[cfg(any(test, feature = "test-utils"))]
fn test_tags() -> Vec<Tag> {
    vec![
        Tag::Label("architecture_catalog_entry".to_string()),
        Tag::Phase(Phase::Published),
    ]
}

/// Test-only catalog entries for deterministic testing without network access.
///
/// Returns the same 5 JavaScript patterns that were previously embedded in the binary.
#[cfg(any(test, feature = "test-utils"))]
pub fn test_builtin_entries() -> Vec<ArchitectureCatalogEntry> {
    vec![
        test_javascript_server(),
        test_javascript_react_app(),
        test_javascript_component_lib(),
        test_javascript_cli_tool(),
        test_javascript_node_util(),
    ]
}

#[cfg(any(test, feature = "test-utils"))]
pub fn test_javascript_server() -> ArchitectureCatalogEntry {
    ArchitectureCatalogEntry::from_parts(
        "JavaScript Server (Express/Fastify/Hono)".to_string(),
        DocumentMetadata::new("BUILTIN-AC-JS-SERVER".to_string()),
        DocumentContent::new("# JavaScript Server\n\nLayered backend server architecture."),
        test_tags(),
        false,
        "javascript".to_string(),
        "server".to_string(),
        strs(&[
            "src/", "src/routes/", "src/handlers/", "src/services/", "src/repositories/",
            "src/middleware/", "src/config/", "src/types/", "tests/", "tests/integration/",
            "tests/unit/",
        ]),
        strs(&["routes", "handlers", "services", "repositories"]),
        strs(&["routes", "handlers", "services", "repositories", "middleware", "config"]),
        strs(&[
            "routes -> handlers", "handlers -> services", "services -> repositories",
            "services -> services (same level, no cycles)", "no upward dependencies",
        ]),
        strs(&[
            "camelCase for functions and variables", "PascalCase for classes and types",
            "kebab-case for file names", "*.route.ts for route files",
            "*.handler.ts for handler files", "*.service.ts for service files",
            "*.repository.ts for repository files", "*.test.ts for test files",
        ]),
        strs(&[
            "business logic in route handlers", "direct database queries in handlers",
            "circular service dependencies", "god service modules",
            "untyped request/response objects",
        ]),
        strs(&[
            "enforce-layer-boundaries: routes->handlers->services->repositories",
            "require-typed-request-response", "no-direct-db-in-handlers",
            "test-co-location: unit tests beside source",
        ]),
        strs(&[
            "eslint-clean", "typescript-strict-if-ts", "no-circular-dependencies",
            "minimum-service-test-coverage-80",
        ]),
    )
}

#[cfg(any(test, feature = "test-utils"))]
pub fn test_javascript_react_app() -> ArchitectureCatalogEntry {
    ArchitectureCatalogEntry::from_parts(
        "JavaScript React App (SPA / Next.js)".to_string(),
        DocumentMetadata::new("BUILTIN-AC-JS-REACT".to_string()),
        DocumentContent::new("# JavaScript React App\n\nFeature-based React architecture."),
        test_tags(),
        false,
        "javascript".to_string(),
        "react-app".to_string(),
        strs(&[
            "src/", "src/features/", "src/features/{feature}/",
            "src/features/{feature}/components/", "src/features/{feature}/hooks/",
            "src/features/{feature}/services/", "src/features/{feature}/types/",
            "src/shared/", "src/shared/components/", "src/shared/hooks/",
            "src/shared/utils/", "src/app/",
        ]),
        strs(&["components", "hooks", "services"]),
        strs(&["features/{name}", "shared/components", "shared/hooks", "shared/utils"]),
        strs(&[
            "features may not import from other features",
            "components -> hooks (same feature or shared)",
            "hooks -> services (same feature or shared)",
            "shared has no feature dependencies",
        ]),
        strs(&[
            "PascalCase for component files and exports", "camelCase for hooks (useXxx pattern)",
            "camelCase for service functions", "kebab-case for feature directory names",
            "*.test.tsx for component tests", "*.test.ts for hook and service tests",
            "index.ts barrel exports per feature",
        ]),
        strs(&[
            "cross-feature imports", "god components with mixed concerns",
            "business logic in JSX", "prop drilling beyond 2 levels",
            "direct API calls in components",
        ]),
        strs(&[
            "enforce-feature-isolation: no cross-feature imports",
            "require-hook-abstraction: no API calls in components",
            "barrel-exports: each feature has index.ts",
            "co-locate-tests: tests beside source files",
        ]),
        strs(&[
            "eslint-clean", "typescript-strict", "no-cross-feature-imports",
            "minimum-component-test-coverage-70",
        ]),
    )
}

#[cfg(any(test, feature = "test-utils"))]
pub fn test_javascript_component_lib() -> ArchitectureCatalogEntry {
    ArchitectureCatalogEntry::from_parts(
        "JavaScript Component Library".to_string(),
        DocumentMetadata::new("BUILTIN-AC-JS-COMPLIB".to_string()),
        DocumentContent::new("# JavaScript Component Library\n\nShared UI library."),
        test_tags(),
        false,
        "javascript".to_string(),
        "component-lib".to_string(),
        strs(&[
            "src/", "src/components/", "src/components/{ComponentName}/",
            "src/components/{ComponentName}/{ComponentName}.tsx",
            "src/components/{ComponentName}/{ComponentName}.test.tsx",
            "src/components/{ComponentName}/{ComponentName}.stories.tsx",
            "src/components/{ComponentName}/index.ts", "src/utils/", "src/index.ts",
        ]),
        strs(&["components", "utils"]),
        strs(&["components/{Name}", "utils"]),
        strs(&[
            "components may import from utils only",
            "components should not import other components",
            "root index.ts is the only public API surface",
        ]),
        strs(&[
            "PascalCase for component folders and files", "PascalCase for component exports",
            "camelCase for utility functions", "{ComponentName}.tsx for implementation",
            "{ComponentName}.test.tsx for tests", "{ComponentName}.stories.tsx for storybook",
            "index.ts per component for re-export",
        ]),
        strs(&[
            "cross-component imports", "leaking internal utilities in public API",
            "framework-version-specific code in components", "missing storybook stories",
            "components without tests",
        ]),
        strs(&[
            "enforce-component-isolation: no cross-component imports",
            "require-public-api: all exports via root index.ts",
            "require-stories: each component has stories",
            "require-tests: each component has tests",
        ]),
        strs(&[
            "eslint-clean", "typescript-strict", "no-cross-component-imports",
            "storybook-coverage-100-percent", "minimum-component-test-coverage-90",
        ]),
    )
}

#[cfg(any(test, feature = "test-utils"))]
pub fn test_javascript_cli_tool() -> ArchitectureCatalogEntry {
    ArchitectureCatalogEntry::from_parts(
        "JavaScript CLI Tool".to_string(),
        DocumentMetadata::new("BUILTIN-AC-JS-CLI".to_string()),
        DocumentContent::new("# JavaScript CLI Tool\n\nCommand-based CLI architecture."),
        test_tags(),
        false,
        "javascript".to_string(),
        "cli-tool".to_string(),
        strs(&[
            "src/", "src/commands/", "src/core/", "src/utils/", "src/config/",
            "src/index.ts", "bin/", "tests/", "tests/unit/", "tests/integration/",
        ]),
        strs(&["commands", "core", "utils"]),
        strs(&["commands", "core", "utils", "config"]),
        strs(&[
            "commands -> core (thin wrappers only)", "core has no CLI framework dependencies",
            "core -> utils", "utils has no domain knowledge",
        ]),
        strs(&[
            "camelCase for functions and variables", "kebab-case for file names",
            "*.command.ts for command definitions", "*.test.ts for test files",
            "bin/ entry point for CLI executable",
        ]),
        strs(&[
            "business logic in command handlers", "CLI framework imports in core modules",
            "untestable I/O in core logic", "missing --help for commands",
            "hard-coded paths or config",
        ]),
        strs(&[
            "enforce-cli-core-separation: no CLI imports in core/",
            "require-thin-commands: commands delegate to core",
            "testable-core: core modules have no side effects",
        ]),
        strs(&[
            "eslint-clean", "typescript-strict-if-ts", "minimum-core-test-coverage-85",
            "no-cli-framework-in-core",
        ]),
    )
}

#[cfg(any(test, feature = "test-utils"))]
pub fn test_javascript_node_util() -> ArchitectureCatalogEntry {
    ArchitectureCatalogEntry::from_parts(
        "JavaScript Node Utility Library".to_string(),
        DocumentMetadata::new("BUILTIN-AC-JS-NODEUTIL".to_string()),
        DocumentContent::new("# JavaScript Node Utility Library\n\nUtility package for Node.js."),
        test_tags(),
        false,
        "javascript".to_string(),
        "node-util".to_string(),
        strs(&[
            "src/", "src/{module}.ts", "src/internal/", "src/index.ts",
            "tests/", "tests/{module}.test.ts",
        ]),
        strs(&["public", "internal"]),
        strs(&["public modules", "internal helpers"]),
        strs(&[
            "public modules -> internal helpers",
            "internal helpers should not import public modules",
            "no circular dependencies", "all public exports via root index.ts",
        ]),
        strs(&[
            "camelCase for functions and variables", "PascalCase for classes and types",
            "kebab-case for file names", "*.test.ts for test files",
            "index.ts as sole public API entry point",
        ]),
        strs(&[
            "kitchen-sink exports (exporting everything)", "circular dependencies between modules",
            "side effects on import", "missing JSDoc for public API",
            "no type exports for TypeScript consumers",
        ]),
        strs(&[
            "enforce-public-api: exports only via index.ts", "no-circular-dependencies",
            "no-side-effects-on-import", "require-jsdoc-public-api",
        ]),
        strs(&[
            "eslint-clean", "typescript-strict-if-ts", "no-circular-dependencies",
            "minimum-test-coverage-90", "jsdoc-coverage-public-api-100",
        ]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_entries_returns_empty() {
        let entries = builtin_entries();
        assert!(entries.is_empty(), "production builtin_entries() should return empty vec");
    }

    #[test]
    fn test_test_builtin_entries_count() {
        let entries = test_builtin_entries();
        assert_eq!(entries.len(), 5);
    }

    #[test]
    fn test_all_test_entries_are_javascript() {
        let entries = test_builtin_entries();
        for entry in &entries {
            assert_eq!(entry.language, "javascript");
        }
    }

    #[test]
    fn test_all_test_entries_are_published() {
        let entries = test_builtin_entries();
        for entry in &entries {
            assert_eq!(entry.phase().unwrap(), Phase::Published);
        }
    }

    #[test]
    fn test_all_test_entries_have_unique_project_types() {
        let entries = test_builtin_entries();
        let mut types: Vec<&str> = entries.iter().map(|e| e.project_type.as_str()).collect();
        types.sort_unstable();
        types.dedup();
        assert_eq!(types.len(), 5);
    }

    #[test]
    fn test_all_test_entries_have_content() {
        let entries = test_builtin_entries();
        for entry in &entries {
            assert!(!entry.folder_layout.is_empty(), "{} missing folder_layout", entry.title());
            assert!(!entry.layers.is_empty(), "{} missing layers", entry.title());
            assert!(!entry.dependency_rules.is_empty(), "{} missing dependency_rules", entry.title());
            assert!(!entry.naming_conventions.is_empty(), "{} missing naming_conventions", entry.title());
            assert!(!entry.anti_patterns.is_empty(), "{} missing anti_patterns", entry.title());
            assert!(!entry.rules_seed_hints.is_empty(), "{} missing rules_seed_hints", entry.title());
            assert!(!entry.analysis_expectations.is_empty(), "{} missing analysis_expectations", entry.title());
        }
    }

    #[test]
    fn test_server_entry_details() {
        let entry = test_javascript_server();
        assert_eq!(entry.project_type, "server");
        assert!(entry.layers.contains(&"routes".to_string()));
        assert!(entry.layers.contains(&"handlers".to_string()));
        assert!(entry.layers.contains(&"services".to_string()));
        assert!(entry.layers.contains(&"repositories".to_string()));
    }

    #[test]
    fn test_entries_roundtrip_serialization() {
        let entries = test_builtin_entries();
        for entry in &entries {
            let serialized = entry.to_content().unwrap();
            let loaded = ArchitectureCatalogEntry::from_content(&serialized).unwrap();
            assert_eq!(loaded.title(), entry.title());
            assert_eq!(loaded.language, entry.language);
            assert_eq!(loaded.project_type, entry.project_type);
            assert_eq!(loaded.folder_layout, entry.folder_layout);
            assert_eq!(loaded.layers, entry.layers);
            assert_eq!(loaded.dependency_rules, entry.dependency_rules);
        }
    }
}

//! Built-in architecture catalog entries for JavaScript/TypeScript projects.
//!
//! These entries ship with the Ultra-Metis binary and represent curated,
//! known-good architecture patterns for common JS/TS project types.

use crate::domain::documents::architecture_catalog_entry::ArchitectureCatalogEntry;
use crate::domain::documents::content::DocumentContent;
use crate::domain::documents::metadata::DocumentMetadata;
use crate::domain::documents::types::{Phase, Tag};

/// All built-in catalog entries.
pub fn builtin_entries() -> Vec<ArchitectureCatalogEntry> {
    vec![
        javascript_server(),
        javascript_react_app(),
        javascript_component_lib(),
        javascript_cli_tool(),
        javascript_node_util(),
    ]
}

/// Express/Fastify/Hono backend server pattern.
pub fn javascript_server() -> ArchitectureCatalogEntry {
    ArchitectureCatalogEntry::from_parts(
        "JavaScript Server (Express/Fastify/Hono)".to_string(),
        DocumentMetadata::new("BUILTIN-AC-JS-SERVER".to_string()),
        DocumentContent::new(concat!(
            "# JavaScript Server (Express/Fastify/Hono)\n\n",
            "## Overview\n\n",
            "Layered backend server architecture for Node.js using Express, Fastify, or Hono. ",
            "Clear separation between HTTP handling and business logic. ",
            "Suitable for REST APIs, GraphQL servers, and backend services.\n\n",
            "## Structure\n\n",
            "Routes define HTTP endpoints and delegate to handlers. Handlers orchestrate ",
            "business logic by calling services. Services contain pure business rules and ",
            "call repositories for data access. Repositories abstract database/external ",
            "service interactions.\n\n",
            "## Dependency Rules\n\n",
            "- Routes depend on handlers only\n",
            "- Handlers depend on services only\n",
            "- Services depend on repositories and other services\n",
            "- Repositories depend on database clients and external APIs\n",
            "- No layer may depend on a layer above it\n\n",
            "## Anti-Patterns\n\n",
            "- Business logic in route handlers\n",
            "- Direct database access from handlers\n",
            "- Circular dependencies between services\n",
            "- God services that handle multiple domains\n\n",
            "## Quality Expectations\n\n",
            "- ESLint clean with no-restricted-imports for layer enforcement\n",
            "- TypeScript strict mode if using TS\n",
            "- Unit tests for services, integration tests for routes\n\n",
            "## Rules Seed Data\n\n",
            "Generates rules for: layer-boundary enforcement, import restrictions, ",
            "naming conventions, and test co-location requirements.",
        )),
        vec![
            Tag::Label("architecture_catalog_entry".to_string()),
            Tag::Phase(Phase::Published),
        ],
        false,
        "javascript".to_string(),
        "server".to_string(),
        vec![
            "src/".to_string(),
            "src/routes/".to_string(),
            "src/handlers/".to_string(),
            "src/services/".to_string(),
            "src/repositories/".to_string(),
            "src/middleware/".to_string(),
            "src/config/".to_string(),
            "src/types/".to_string(),
            "tests/".to_string(),
            "tests/integration/".to_string(),
            "tests/unit/".to_string(),
        ],
        vec![
            "routes".to_string(),
            "handlers".to_string(),
            "services".to_string(),
            "repositories".to_string(),
        ],
        vec![
            "routes".to_string(),
            "handlers".to_string(),
            "services".to_string(),
            "repositories".to_string(),
            "middleware".to_string(),
            "config".to_string(),
        ],
        vec![
            "routes -> handlers".to_string(),
            "handlers -> services".to_string(),
            "services -> repositories".to_string(),
            "services -> services (same level, no cycles)".to_string(),
            "no upward dependencies".to_string(),
        ],
        vec![
            "camelCase for functions and variables".to_string(),
            "PascalCase for classes and types".to_string(),
            "kebab-case for file names".to_string(),
            "*.route.ts for route files".to_string(),
            "*.handler.ts for handler files".to_string(),
            "*.service.ts for service files".to_string(),
            "*.repository.ts for repository files".to_string(),
            "*.test.ts for test files".to_string(),
        ],
        vec![
            "business logic in route handlers".to_string(),
            "direct database queries in handlers".to_string(),
            "circular service dependencies".to_string(),
            "god service modules".to_string(),
            "untyped request/response objects".to_string(),
        ],
        vec![
            "enforce-layer-boundaries: routes->handlers->services->repositories".to_string(),
            "require-typed-request-response".to_string(),
            "no-direct-db-in-handlers".to_string(),
            "test-co-location: unit tests beside source".to_string(),
        ],
        vec![
            "eslint-clean".to_string(),
            "typescript-strict-if-ts".to_string(),
            "no-circular-dependencies".to_string(),
            "minimum-service-test-coverage-80".to_string(),
        ],
    )
}

/// React SPA or Next.js application pattern.
pub fn javascript_react_app() -> ArchitectureCatalogEntry {
    ArchitectureCatalogEntry::from_parts(
        "JavaScript React App (SPA / Next.js)".to_string(),
        DocumentMetadata::new("BUILTIN-AC-JS-REACT".to_string()),
        DocumentContent::new(concat!(
            "# JavaScript React App (SPA / Next.js)\n\n",
            "## Overview\n\n",
            "Feature-based React application architecture. Each feature is a self-contained ",
            "vertical slice with its own components, hooks, services, and tests. ",
            "Shared UI components live in a common directory. Suitable for SPAs and Next.js apps.\n\n",
            "## Structure\n\n",
            "Features are organized by domain (e.g., auth, dashboard, settings). Each feature ",
            "contains components, hooks, services, and types. Shared components, hooks, and ",
            "utilities live in common directories.\n\n",
            "## Dependency Rules\n\n",
            "- Features may import from shared/ but not from other features\n",
            "- Components depend on hooks and services within the same feature\n",
            "- Hooks orchestrate service calls and state management\n",
            "- Services handle API communication\n\n",
            "## Anti-Patterns\n\n",
            "- Cross-feature imports (feature A importing from feature B)\n",
            "- God components with too many responsibilities\n",
            "- Business logic in components instead of hooks/services\n\n",
            "## Quality Expectations\n\n",
            "- ESLint with React-specific rules\n",
            "- TypeScript strict mode\n",
            "- Component tests co-located with source\n",
        )),
        vec![
            Tag::Label("architecture_catalog_entry".to_string()),
            Tag::Phase(Phase::Published),
        ],
        false,
        "javascript".to_string(),
        "react-app".to_string(),
        vec![
            "src/".to_string(),
            "src/features/".to_string(),
            "src/features/{feature}/".to_string(),
            "src/features/{feature}/components/".to_string(),
            "src/features/{feature}/hooks/".to_string(),
            "src/features/{feature}/services/".to_string(),
            "src/features/{feature}/types/".to_string(),
            "src/shared/".to_string(),
            "src/shared/components/".to_string(),
            "src/shared/hooks/".to_string(),
            "src/shared/utils/".to_string(),
            "src/app/".to_string(),
        ],
        vec![
            "components".to_string(),
            "hooks".to_string(),
            "services".to_string(),
        ],
        vec![
            "features/{name}".to_string(),
            "shared/components".to_string(),
            "shared/hooks".to_string(),
            "shared/utils".to_string(),
        ],
        vec![
            "features may not import from other features".to_string(),
            "components -> hooks (same feature or shared)".to_string(),
            "hooks -> services (same feature or shared)".to_string(),
            "shared has no feature dependencies".to_string(),
        ],
        vec![
            "PascalCase for component files and exports".to_string(),
            "camelCase for hooks (useXxx pattern)".to_string(),
            "camelCase for service functions".to_string(),
            "kebab-case for feature directory names".to_string(),
            "*.test.tsx for component tests".to_string(),
            "*.test.ts for hook and service tests".to_string(),
            "index.ts barrel exports per feature".to_string(),
        ],
        vec![
            "cross-feature imports".to_string(),
            "god components with mixed concerns".to_string(),
            "business logic in JSX".to_string(),
            "prop drilling beyond 2 levels".to_string(),
            "direct API calls in components".to_string(),
        ],
        vec![
            "enforce-feature-isolation: no cross-feature imports".to_string(),
            "require-hook-abstraction: no API calls in components".to_string(),
            "barrel-exports: each feature has index.ts".to_string(),
            "co-locate-tests: tests beside source files".to_string(),
        ],
        vec![
            "eslint-clean".to_string(),
            "typescript-strict".to_string(),
            "no-cross-feature-imports".to_string(),
            "minimum-component-test-coverage-70".to_string(),
        ],
    )
}

/// Shared UI component library pattern.
pub fn javascript_component_lib() -> ArchitectureCatalogEntry {
    ArchitectureCatalogEntry::from_parts(
        "JavaScript Component Library".to_string(),
        DocumentMetadata::new("BUILTIN-AC-JS-COMPLIB".to_string()),
        DocumentContent::new(concat!(
            "# JavaScript Component Library\n\n",
            "## Overview\n\n",
            "Shared UI library with one component per folder. Storybook-friendly structure ",
            "with explicit public API via index exports. Designed for publishing to npm or ",
            "internal registries. Suitable for design system implementations.\n\n",
            "## Structure\n\n",
            "Each component lives in its own folder with its implementation, styles, tests, ",
            "and stories. A root index.ts re-exports the public API. Internal utilities ",
            "are in a utils/ directory not exposed publicly.\n\n",
            "## Dependency Rules\n\n",
            "- Components may depend on shared utilities only\n",
            "- Components should not depend on each other (compose at consumer level)\n",
            "- All public exports go through root index.ts\n\n",
            "## Anti-Patterns\n\n",
            "- Components importing other components directly\n",
            "- Leaking internal utilities through the public API\n",
            "- Tightly coupling to a specific framework version\n",
        )),
        vec![
            Tag::Label("architecture_catalog_entry".to_string()),
            Tag::Phase(Phase::Published),
        ],
        false,
        "javascript".to_string(),
        "component-lib".to_string(),
        vec![
            "src/".to_string(),
            "src/components/".to_string(),
            "src/components/{ComponentName}/".to_string(),
            "src/components/{ComponentName}/{ComponentName}.tsx".to_string(),
            "src/components/{ComponentName}/{ComponentName}.test.tsx".to_string(),
            "src/components/{ComponentName}/{ComponentName}.stories.tsx".to_string(),
            "src/components/{ComponentName}/index.ts".to_string(),
            "src/utils/".to_string(),
            "src/index.ts".to_string(),
        ],
        vec!["components".to_string(), "utils".to_string()],
        vec!["components/{Name}".to_string(), "utils".to_string()],
        vec![
            "components may import from utils only".to_string(),
            "components should not import other components".to_string(),
            "root index.ts is the only public API surface".to_string(),
        ],
        vec![
            "PascalCase for component folders and files".to_string(),
            "PascalCase for component exports".to_string(),
            "camelCase for utility functions".to_string(),
            "{ComponentName}.tsx for implementation".to_string(),
            "{ComponentName}.test.tsx for tests".to_string(),
            "{ComponentName}.stories.tsx for storybook".to_string(),
            "index.ts per component for re-export".to_string(),
        ],
        vec![
            "cross-component imports".to_string(),
            "leaking internal utilities in public API".to_string(),
            "framework-version-specific code in components".to_string(),
            "missing storybook stories".to_string(),
            "components without tests".to_string(),
        ],
        vec![
            "enforce-component-isolation: no cross-component imports".to_string(),
            "require-public-api: all exports via root index.ts".to_string(),
            "require-stories: each component has stories".to_string(),
            "require-tests: each component has tests".to_string(),
        ],
        vec![
            "eslint-clean".to_string(),
            "typescript-strict".to_string(),
            "no-cross-component-imports".to_string(),
            "storybook-coverage-100-percent".to_string(),
            "minimum-component-test-coverage-90".to_string(),
        ],
    )
}

/// Node.js CLI tool pattern.
pub fn javascript_cli_tool() -> ArchitectureCatalogEntry {
    ArchitectureCatalogEntry::from_parts(
        "JavaScript CLI Tool".to_string(),
        DocumentMetadata::new("BUILTIN-AC-JS-CLI".to_string()),
        DocumentContent::new(concat!(
            "# JavaScript CLI Tool\n\n",
            "## Overview\n\n",
            "Command-based Node.js CLI architecture. CLI argument parsing is strictly separated ",
            "from core logic. Commands are thin wrappers that validate input and delegate to ",
            "core modules. Suitable for developer tools, build scripts, and automation.\n\n",
            "## Structure\n\n",
            "Commands define the CLI interface (arguments, flags, help text) and delegate to ",
            "core modules for actual logic. Core modules are framework-agnostic and testable ",
            "without CLI infrastructure. Shared utilities handle I/O, formatting, and config.\n\n",
            "## Dependency Rules\n\n",
            "- Commands depend on core modules\n",
            "- Core modules are self-contained and CLI-framework-agnostic\n",
            "- Utils are shared helpers with no domain knowledge\n\n",
            "## Anti-Patterns\n\n",
            "- Business logic in command handlers\n",
            "- Core modules depending on CLI framework\n",
            "- Untestable I/O-heavy code in core\n",
        )),
        vec![
            Tag::Label("architecture_catalog_entry".to_string()),
            Tag::Phase(Phase::Published),
        ],
        false,
        "javascript".to_string(),
        "cli-tool".to_string(),
        vec![
            "src/".to_string(),
            "src/commands/".to_string(),
            "src/core/".to_string(),
            "src/utils/".to_string(),
            "src/config/".to_string(),
            "src/index.ts".to_string(),
            "bin/".to_string(),
            "tests/".to_string(),
            "tests/unit/".to_string(),
            "tests/integration/".to_string(),
        ],
        vec![
            "commands".to_string(),
            "core".to_string(),
            "utils".to_string(),
        ],
        vec![
            "commands".to_string(),
            "core".to_string(),
            "utils".to_string(),
            "config".to_string(),
        ],
        vec![
            "commands -> core (thin wrappers only)".to_string(),
            "core has no CLI framework dependencies".to_string(),
            "core -> utils".to_string(),
            "utils has no domain knowledge".to_string(),
        ],
        vec![
            "camelCase for functions and variables".to_string(),
            "kebab-case for file names".to_string(),
            "*.command.ts for command definitions".to_string(),
            "*.test.ts for test files".to_string(),
            "bin/ entry point for CLI executable".to_string(),
        ],
        vec![
            "business logic in command handlers".to_string(),
            "CLI framework imports in core modules".to_string(),
            "untestable I/O in core logic".to_string(),
            "missing --help for commands".to_string(),
            "hard-coded paths or config".to_string(),
        ],
        vec![
            "enforce-cli-core-separation: no CLI imports in core/".to_string(),
            "require-thin-commands: commands delegate to core".to_string(),
            "testable-core: core modules have no side effects".to_string(),
        ],
        vec![
            "eslint-clean".to_string(),
            "typescript-strict-if-ts".to_string(),
            "minimum-core-test-coverage-85".to_string(),
            "no-cli-framework-in-core".to_string(),
        ],
    )
}

/// Node.js utility/library package pattern.
pub fn javascript_node_util() -> ArchitectureCatalogEntry {
    ArchitectureCatalogEntry::from_parts(
        "JavaScript Node Utility Library".to_string(),
        DocumentMetadata::new("BUILTIN-AC-JS-NODEUTIL".to_string()),
        DocumentContent::new(concat!(
            "# JavaScript Node Utility Library\n\n",
            "## Overview\n\n",
            "Utility or library package for Node.js. Flat or domain-grouped source structure. ",
            "Comprehensive unit tests. Clean public API via root index.ts. ",
            "Suitable for shared libraries, SDK packages, and utility collections.\n\n",
            "## Structure\n\n",
            "Source files are organized flat or by domain group. Each module is self-contained. ",
            "A root index.ts defines the public API surface. Internal helpers are not exported. ",
            "Tests mirror the source structure.\n\n",
            "## Dependency Rules\n\n",
            "- Public modules may depend on internal helpers\n",
            "- Internal helpers should not depend on public modules\n",
            "- No circular dependencies between modules\n",
            "- All public exports via root index.ts\n\n",
            "## Anti-Patterns\n\n",
            "- Exporting everything (kitchen-sink public API)\n",
            "- Circular dependencies between modules\n",
            "- Side effects on import\n",
        )),
        vec![
            Tag::Label("architecture_catalog_entry".to_string()),
            Tag::Phase(Phase::Published),
        ],
        false,
        "javascript".to_string(),
        "node-util".to_string(),
        vec![
            "src/".to_string(),
            "src/{module}.ts".to_string(),
            "src/internal/".to_string(),
            "src/index.ts".to_string(),
            "tests/".to_string(),
            "tests/{module}.test.ts".to_string(),
        ],
        vec!["public".to_string(), "internal".to_string()],
        vec!["public modules".to_string(), "internal helpers".to_string()],
        vec![
            "public modules -> internal helpers".to_string(),
            "internal helpers should not import public modules".to_string(),
            "no circular dependencies".to_string(),
            "all public exports via root index.ts".to_string(),
        ],
        vec![
            "camelCase for functions and variables".to_string(),
            "PascalCase for classes and types".to_string(),
            "kebab-case for file names".to_string(),
            "*.test.ts for test files".to_string(),
            "index.ts as sole public API entry point".to_string(),
        ],
        vec![
            "kitchen-sink exports (exporting everything)".to_string(),
            "circular dependencies between modules".to_string(),
            "side effects on import".to_string(),
            "missing JSDoc for public API".to_string(),
            "no type exports for TypeScript consumers".to_string(),
        ],
        vec![
            "enforce-public-api: exports only via index.ts".to_string(),
            "no-circular-dependencies".to_string(),
            "no-side-effects-on-import".to_string(),
            "require-jsdoc-public-api".to_string(),
        ],
        vec![
            "eslint-clean".to_string(),
            "typescript-strict-if-ts".to_string(),
            "no-circular-dependencies".to_string(),
            "minimum-test-coverage-90".to_string(),
            "jsdoc-coverage-public-api-100".to_string(),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_entries_count() {
        let entries = builtin_entries();
        assert_eq!(entries.len(), 5);
    }

    #[test]
    fn test_all_entries_are_javascript() {
        let entries = builtin_entries();
        for entry in &entries {
            assert_eq!(entry.language, "javascript");
        }
    }

    #[test]
    fn test_all_entries_are_published() {
        let entries = builtin_entries();
        for entry in &entries {
            assert_eq!(entry.phase().unwrap(), Phase::Published);
        }
    }

    #[test]
    fn test_all_entries_have_unique_project_types() {
        let entries = builtin_entries();
        let mut types: Vec<&str> = entries.iter().map(|e| e.project_type.as_str()).collect();
        types.sort();
        types.dedup();
        assert_eq!(types.len(), 5);
    }

    #[test]
    fn test_all_entries_have_content() {
        let entries = builtin_entries();
        for entry in &entries {
            assert!(
                !entry.folder_layout.is_empty(),
                "{} missing folder_layout",
                entry.title()
            );
            assert!(!entry.layers.is_empty(), "{} missing layers", entry.title());
            assert!(
                !entry.dependency_rules.is_empty(),
                "{} missing dependency_rules",
                entry.title()
            );
            assert!(
                !entry.naming_conventions.is_empty(),
                "{} missing naming_conventions",
                entry.title()
            );
            assert!(
                !entry.anti_patterns.is_empty(),
                "{} missing anti_patterns",
                entry.title()
            );
            assert!(
                !entry.rules_seed_hints.is_empty(),
                "{} missing rules_seed_hints",
                entry.title()
            );
            assert!(
                !entry.analysis_expectations.is_empty(),
                "{} missing analysis_expectations",
                entry.title()
            );
        }
    }

    #[test]
    fn test_server_entry_details() {
        let entry = javascript_server();
        assert_eq!(entry.project_type, "server");
        assert!(entry.layers.contains(&"routes".to_string()));
        assert!(entry.layers.contains(&"handlers".to_string()));
        assert!(entry.layers.contains(&"services".to_string()));
        assert!(entry.layers.contains(&"repositories".to_string()));
    }

    #[test]
    fn test_react_app_entry_details() {
        let entry = javascript_react_app();
        assert_eq!(entry.project_type, "react-app");
        assert!(entry.layers.contains(&"components".to_string()));
        assert!(entry.layers.contains(&"hooks".to_string()));
        assert!(entry.layers.contains(&"services".to_string()));
    }

    #[test]
    fn test_component_lib_entry_details() {
        let entry = javascript_component_lib();
        assert_eq!(entry.project_type, "component-lib");
        assert!(entry.layers.contains(&"components".to_string()));
        assert!(entry.layers.contains(&"utils".to_string()));
    }

    #[test]
    fn test_cli_tool_entry_details() {
        let entry = javascript_cli_tool();
        assert_eq!(entry.project_type, "cli-tool");
        assert!(entry.layers.contains(&"commands".to_string()));
        assert!(entry.layers.contains(&"core".to_string()));
    }

    #[test]
    fn test_node_util_entry_details() {
        let entry = javascript_node_util();
        assert_eq!(entry.project_type, "node-util");
        assert!(entry.layers.contains(&"public".to_string()));
        assert!(entry.layers.contains(&"internal".to_string()));
    }

    #[test]
    fn test_entries_roundtrip_serialization() {
        let entries = builtin_entries();
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

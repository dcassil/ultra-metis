//! Bootstrap init flow orchestrator.
//!
//! Orchestrates the full bootstrap process: scans the repo for languages,
//! detects monorepo patterns, discovers packages, and detects development
//! tools. Produces a [`BootstrapResult`] containing all detected information
//! ready for downstream consumption (architecture selection, product doc
//! scaffolding, rules config generation).

use super::monorepo_detector::{MonorepoDetector, MonorepoInfo};
use super::repo_scanner::{RepoScanResult, RepoScanner};
use super::tool_detector::{ToolDetectionResult, ToolDetector};

/// The project type inferred from repo analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InferredProjectType {
    /// A server/backend application.
    Server,
    /// A frontend web application (React, Vue, etc.).
    WebApp,
    /// A CLI tool.
    CliTool,
    /// A library/utility package.
    Library,
    /// A component library (UI components).
    ComponentLibrary,
    /// A full-stack monorepo with both frontend and backend.
    FullStack,
    /// Could not determine a specific project type.
    Unknown,
}

impl std::fmt::Display for InferredProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InferredProjectType::Server => write!(f, "server"),
            InferredProjectType::WebApp => write!(f, "web-app"),
            InferredProjectType::CliTool => write!(f, "cli-tool"),
            InferredProjectType::Library => write!(f, "library"),
            InferredProjectType::ComponentLibrary => write!(f, "component-library"),
            InferredProjectType::FullStack => write!(f, "full-stack"),
            InferredProjectType::Unknown => write!(f, "unknown"),
        }
    }
}

/// The complete result of bootstrapping a repository.
#[derive(Debug)]
pub struct BootstrapResult {
    /// Language and build tool scan results.
    pub scan: RepoScanResult,
    /// Monorepo detection results.
    pub monorepo: MonorepoInfo,
    /// Development tool detection results.
    pub tools: ToolDetectionResult,
    /// Inferred project type based on combined analysis.
    pub project_type: InferredProjectType,
    /// Whether this appears to be a brownfield (existing) project.
    pub is_brownfield: bool,
    /// Summary of the bootstrap analysis for display.
    pub summary: BootstrapSummary,
}

/// Human-readable summary of bootstrap analysis.
#[derive(Debug)]
pub struct BootstrapSummary {
    /// One-line description of the project.
    pub description: String,
    /// Key facts about the project.
    pub facts: Vec<String>,
    /// Suggestions for next steps.
    pub suggestions: Vec<String>,
}

/// Orchestrates the full bootstrap flow.
pub struct BootstrapFlow;

impl BootstrapFlow {
    /// Run the full bootstrap analysis on a set of file paths.
    ///
    /// This is the main entry point for repo-aware initialization.
    pub fn analyze(file_paths: &[String]) -> BootstrapResult {
        let scan = RepoScanner::scan(file_paths);
        let monorepo = MonorepoDetector::detect(file_paths);
        let tools = ToolDetector::detect(file_paths);

        let project_type = Self::infer_project_type(&scan, &monorepo, file_paths);
        let is_brownfield = Self::detect_brownfield(file_paths, &tools);
        let summary = Self::build_summary(&scan, &monorepo, &tools, &project_type, is_brownfield);

        BootstrapResult {
            scan,
            monorepo,
            tools,
            project_type,
            is_brownfield,
            summary,
        }
    }

    /// Infer the project type from scan results and file structure.
    fn infer_project_type(
        scan: &RepoScanResult,
        monorepo: &MonorepoInfo,
        file_paths: &[String],
    ) -> InferredProjectType {
        // Monorepo with both apps and packages -> full-stack or library
        if monorepo.is_monorepo {
            let has_apps = !monorepo.apps().is_empty();
            let has_libs = !monorepo.libraries().is_empty();
            if has_apps && has_libs {
                return InferredProjectType::FullStack;
            }
            if has_libs && !has_apps {
                return InferredProjectType::Library;
            }
        }

        // Check for specific patterns in file paths
        let has_routes = file_paths.iter().any(|p| {
            p.contains("/routes/") || p.contains("/handlers/") || p.contains("/controllers/")
        });
        let has_components = file_paths.iter().any(|p| {
            p.contains("/components/")
                && (p.ends_with(".tsx") || p.ends_with(".jsx") || p.ends_with(".vue"))
        });
        let has_features = file_paths.iter().any(|p| p.contains("/features/"));
        let has_stories = file_paths.iter().any(|p| p.contains(".stories."));
        let has_bin = file_paths
            .iter()
            .any(|p| p.contains("/bin/") || p.contains("src/main.rs") || p.contains("/cmd/"));
        let has_storybook = file_paths.iter().any(|p| p.contains(".storybook/"));

        // Component library: has stories/storybook, mostly components
        if (has_stories || has_storybook) && has_components && !has_routes {
            return InferredProjectType::ComponentLibrary;
        }

        // Web app: has components, features, or framework indicators
        if has_components || has_features {
            if has_routes {
                return InferredProjectType::FullStack;
            }
            return InferredProjectType::WebApp;
        }

        // CLI tool: has bin/main entry point, command structure
        let has_commands = file_paths
            .iter()
            .any(|p| p.contains("/commands/") || p.contains("/cmd/"));
        if has_commands && has_bin {
            return InferredProjectType::CliTool;
        }

        // Server: has routes/handlers/controllers
        if has_routes {
            return InferredProjectType::Server;
        }

        // CLI: has main.rs or bin/ without routes
        if has_bin && !has_routes && !has_components {
            return InferredProjectType::CliTool;
        }

        // Library: has lib.rs or is mostly source with no clear app structure
        let has_lib = file_paths.iter().any(|p| {
            p.ends_with("src/lib.rs") || p.ends_with("src/index.ts") || p.ends_with("src/index.js")
        });
        if has_lib && !has_routes && !has_components && !has_bin {
            return InferredProjectType::Library;
        }

        // Check primary language for hints
        if let Some(lang) = scan.primary_language() {
            if lang.name == "rust" && has_bin {
                return InferredProjectType::CliTool;
            }
        }

        InferredProjectType::Unknown
    }

    /// Detect whether this is a brownfield (existing) project vs greenfield.
    ///
    /// Heuristics: significant source files, existing test infrastructure,
    /// CI configuration, git history artifacts.
    fn detect_brownfield(file_paths: &[String], tools: &ToolDetectionResult) -> bool {
        // Count source files (exclude configs, docs, etc.)
        let source_extensions: std::collections::HashSet<&str> = [
            "rs", "js", "jsx", "ts", "tsx", "py", "go", "java", "kt", "rb", "php", "cs", "fs",
            "scala", "swift", "c", "cpp", "h",
        ]
        .into_iter()
        .collect();

        let source_count = file_paths
            .iter()
            .filter(|p| {
                std::path::Path::new(p)
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| source_extensions.contains(e))
                    .unwrap_or(false)
            })
            .count();

        let has_tests = !tools.test_runners().is_empty();
        let has_ci = !tools.ci_systems().is_empty();

        // Brownfield: has meaningful source code or established tooling
        source_count >= 5 || (source_count >= 2 && (has_tests || has_ci))
    }

    /// Build a human-readable summary of the bootstrap analysis.
    fn build_summary(
        scan: &RepoScanResult,
        monorepo: &MonorepoInfo,
        tools: &ToolDetectionResult,
        project_type: &InferredProjectType,
        is_brownfield: bool,
    ) -> BootstrapSummary {
        let mut facts = Vec::new();
        let mut suggestions = Vec::new();

        // Language facts
        if let Some(primary) = scan.primary_language() {
            facts.push(format!("Primary language: {}", primary.name));
        }
        if scan.languages.len() > 1 {
            let others: Vec<&str> = scan
                .languages
                .iter()
                .skip(1)
                .map(|l| l.name.as_str())
                .collect();
            facts.push(format!("Additional languages: {}", others.join(", ")));
        }

        // Monorepo facts
        if monorepo.is_monorepo {
            let tool_names: Vec<String> = monorepo.tools.iter().map(|t| t.to_string()).collect();
            facts.push(format!(
                "Monorepo ({}) with {} packages",
                if tool_names.is_empty() {
                    "structural".to_string()
                } else {
                    tool_names.join(", ")
                },
                monorepo.packages.len()
            ));
            let app_count = monorepo.apps().len();
            let lib_count = monorepo.libraries().len();
            if app_count > 0 {
                facts.push(format!("{} app(s) detected", app_count));
            }
            if lib_count > 0 {
                facts.push(format!("{} library package(s) detected", lib_count));
            }
        }

        // Tool facts
        let linters = tools.linters();
        if !linters.is_empty() {
            let names: Vec<&str> = linters.iter().map(|t| t.name.as_str()).collect();
            facts.push(format!("Linters: {}", names.join(", ")));
        }
        let formatters = tools.formatters();
        if !formatters.is_empty() {
            let names: Vec<&str> = formatters.iter().map(|t| t.name.as_str()).collect();
            facts.push(format!("Formatters: {}", names.join(", ")));
        }
        let test_runners = tools.test_runners();
        if !test_runners.is_empty() {
            let names: Vec<&str> = test_runners.iter().map(|t| t.name.as_str()).collect();
            facts.push(format!("Test runners: {}", names.join(", ")));
        }
        let ci = tools.ci_systems();
        if !ci.is_empty() {
            let names: Vec<&str> = ci.iter().map(|t| t.name.as_str()).collect();
            facts.push(format!("CI: {}", names.join(", ")));
        }

        // Brownfield/greenfield
        if is_brownfield {
            facts.push("Existing project (brownfield)".to_string());
        } else {
            facts.push("New project (greenfield)".to_string());
        }

        // Suggestions
        if tools.linters().is_empty() {
            if let Some(lang) = scan.primary_language() {
                let suggestion = match lang.name.as_str() {
                    "javascript" | "typescript" => "Consider adding ESLint or Biome for linting",
                    "rust" => "Consider enabling Clippy for additional linting",
                    "python" => "Consider adding Ruff or Pylint for linting",
                    "go" => "Consider adding golangci-lint for linting",
                    _ => "Consider adding a linter for your primary language",
                };
                suggestions.push(suggestion.to_string());
            }
        }
        if tools.formatters().is_empty() {
            suggestions.push("Consider adding a code formatter for consistency".to_string());
        }
        if tools.test_runners().is_empty() {
            suggestions.push("Consider setting up a test runner".to_string());
        }
        if tools.ci_systems().is_empty() {
            suggestions.push("Consider setting up CI/CD".to_string());
        }

        // Description
        let lang_str = scan
            .primary_language()
            .map(|l| l.name.as_str())
            .unwrap_or("unknown language");
        let mono_str = if monorepo.is_monorepo {
            "monorepo"
        } else {
            "project"
        };
        let description = format!(
            "{} {} {} ({})",
            if is_brownfield { "Existing" } else { "New" },
            lang_str,
            mono_str,
            project_type,
        );

        BootstrapSummary {
            description,
            facts,
            suggestions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rust_cli_paths() -> Vec<String> {
        vec![
            "Cargo.toml".to_string(),
            "Cargo.lock".to_string(),
            "rustfmt.toml".to_string(),
            "clippy.toml".to_string(),
            ".github/workflows/ci.yml".to_string(),
            "src/main.rs".to_string(),
            "src/commands/init.rs".to_string(),
            "src/commands/build.rs".to_string(),
            "src/core/compiler.rs".to_string(),
            "src/core/resolver.rs".to_string(),
            "tests/integration_test.rs".to_string(),
        ]
    }

    fn js_server_paths() -> Vec<String> {
        vec![
            "package.json".to_string(),
            "package-lock.json".to_string(),
            "tsconfig.json".to_string(),
            ".eslintrc.json".to_string(),
            ".prettierrc".to_string(),
            "jest.config.ts".to_string(),
            ".github/workflows/ci.yml".to_string(),
            "src/index.ts".to_string(),
            "src/routes/users.ts".to_string(),
            "src/routes/auth.ts".to_string(),
            "src/handlers/users.handler.ts".to_string(),
            "src/services/auth.service.ts".to_string(),
            "src/repositories/users.repository.ts".to_string(),
            "tests/auth.test.ts".to_string(),
        ]
    }

    fn react_app_paths() -> Vec<String> {
        vec![
            "package.json".to_string(),
            "tsconfig.json".to_string(),
            ".eslintrc.json".to_string(),
            "vite.config.ts".to_string(),
            "src/App.tsx".to_string(),
            "src/features/auth/components/Login.tsx".to_string(),
            "src/features/dashboard/components/Dashboard.tsx".to_string(),
            "src/shared/components/Button.tsx".to_string(),
        ]
    }

    fn turborepo_fullstack_paths() -> Vec<String> {
        vec![
            "package.json".to_string(),
            "pnpm-workspace.yaml".to_string(),
            "pnpm-lock.yaml".to_string(),
            "turbo.json".to_string(),
            ".eslintrc.json".to_string(),
            ".prettierrc".to_string(),
            ".github/workflows/ci.yml".to_string(),
            "apps/web/package.json".to_string(),
            "apps/web/src/App.tsx".to_string(),
            "apps/web/src/components/Header.tsx".to_string(),
            "apps/api/package.json".to_string(),
            "apps/api/src/index.ts".to_string(),
            "apps/api/src/routes/users.ts".to_string(),
            "packages/ui/package.json".to_string(),
            "packages/ui/src/Button.tsx".to_string(),
            "packages/config/package.json".to_string(),
            "packages/config/src/index.ts".to_string(),
        ]
    }

    fn greenfield_paths() -> Vec<String> {
        vec!["Cargo.toml".to_string(), "src/lib.rs".to_string()]
    }

    fn component_lib_paths() -> Vec<String> {
        vec![
            "package.json".to_string(),
            "tsconfig.json".to_string(),
            ".storybook/main.ts".to_string(),
            "src/components/Button/Button.tsx".to_string(),
            "src/components/Button/Button.stories.tsx".to_string(),
            "src/components/Input/Input.tsx".to_string(),
            "src/components/Input/Input.stories.tsx".to_string(),
            "src/index.ts".to_string(),
        ]
    }

    #[test]
    fn test_bootstrap_rust_cli() {
        let result = BootstrapFlow::analyze(&rust_cli_paths());

        assert_eq!(result.scan.primary_language_name(), Some("rust"));
        assert_eq!(result.project_type, InferredProjectType::CliTool);
        assert!(result.is_brownfield);
        assert!(result.tools.has_tool("clippy"));
        assert!(result.tools.has_tool("rustfmt"));
        assert!(result.tools.has_tool("cargo-test"));
    }

    #[test]
    fn test_bootstrap_js_server() {
        let result = BootstrapFlow::analyze(&js_server_paths());

        assert_eq!(result.scan.primary_language_name(), Some("javascript"));
        assert_eq!(result.project_type, InferredProjectType::Server);
        assert!(result.is_brownfield);
        assert!(result.tools.has_tool("eslint"));
        assert!(result.tools.has_tool("prettier"));
        assert!(result.tools.has_tool("jest"));
    }

    #[test]
    fn test_bootstrap_react_app() {
        let result = BootstrapFlow::analyze(&react_app_paths());

        assert_eq!(result.project_type, InferredProjectType::WebApp);
    }

    #[test]
    fn test_bootstrap_turborepo_fullstack() {
        let result = BootstrapFlow::analyze(&turborepo_fullstack_paths());

        assert!(result.monorepo.is_monorepo);
        assert_eq!(result.project_type, InferredProjectType::FullStack);
        assert!(result.monorepo.packages.len() >= 4);
    }

    #[test]
    fn test_bootstrap_greenfield() {
        let result = BootstrapFlow::analyze(&greenfield_paths());

        assert!(!result.is_brownfield);
        assert!(result
            .summary
            .facts
            .iter()
            .any(|f| f.contains("greenfield")));
    }

    #[test]
    fn test_bootstrap_component_library() {
        let result = BootstrapFlow::analyze(&component_lib_paths());

        assert_eq!(result.project_type, InferredProjectType::ComponentLibrary);
    }

    #[test]
    fn test_summary_has_description() {
        let result = BootstrapFlow::analyze(&rust_cli_paths());

        assert!(!result.summary.description.is_empty());
        assert!(result.summary.description.contains("rust"));
    }

    #[test]
    fn test_summary_has_facts() {
        let result = BootstrapFlow::analyze(&js_server_paths());

        assert!(!result.summary.facts.is_empty());
        assert!(result
            .summary
            .facts
            .iter()
            .any(|f| f.contains("javascript")));
    }

    #[test]
    fn test_summary_suggestions_for_missing_tools() {
        // Minimal project with no tooling
        let paths = vec![
            "Cargo.toml".to_string(),
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "src/types.rs".to_string(),
        ];
        let result = BootstrapFlow::analyze(&paths);

        // Should suggest adding formatter, CI, etc.
        assert!(!result.summary.suggestions.is_empty());
    }

    #[test]
    fn test_empty_paths() {
        let result = BootstrapFlow::analyze(&[]);

        assert_eq!(result.project_type, InferredProjectType::Unknown);
        assert!(!result.is_brownfield);
        assert_eq!(result.scan.total_files, 0);
    }

    #[test]
    fn test_brownfield_detection_by_source_count() {
        let paths: Vec<String> = (0..10)
            .map(|i| format!("src/module_{}.rs", i))
            .chain(std::iter::once("Cargo.toml".to_string()))
            .collect();
        let result = BootstrapFlow::analyze(&paths);
        assert!(result.is_brownfield);
    }

    #[test]
    fn test_brownfield_detection_with_ci() {
        let paths = vec![
            "Cargo.toml".to_string(),
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            ".github/workflows/ci.yml".to_string(),
        ];
        let result = BootstrapFlow::analyze(&paths);
        assert!(result.is_brownfield);
    }

    #[test]
    fn test_rust_library_detection() {
        let paths = vec![
            "Cargo.toml".to_string(),
            "src/lib.rs".to_string(),
            "src/types.rs".to_string(),
            "src/parser.rs".to_string(),
            "src/utils.rs".to_string(),
            "tests/test_parser.rs".to_string(),
        ];
        let result = BootstrapFlow::analyze(&paths);
        assert_eq!(result.project_type, InferredProjectType::Library);
    }

    #[test]
    fn test_display_impls() {
        assert_eq!(InferredProjectType::Server.to_string(), "server");
        assert_eq!(InferredProjectType::WebApp.to_string(), "web-app");
        assert_eq!(InferredProjectType::CliTool.to_string(), "cli-tool");
        assert_eq!(InferredProjectType::Library.to_string(), "library");
        assert_eq!(InferredProjectType::FullStack.to_string(), "full-stack");
        assert_eq!(
            InferredProjectType::ComponentLibrary.to_string(),
            "component-library"
        );
        assert_eq!(InferredProjectType::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_monorepo_not_detected_for_single_project() {
        let result = BootstrapFlow::analyze(&rust_cli_paths());
        assert!(!result.monorepo.is_monorepo);
    }
}

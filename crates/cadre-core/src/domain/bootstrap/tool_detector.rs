//! Dev tool detector for linters, formatters, test runners, and CI systems.
//!
//! Detects development tooling present in a repository by scanning for
//! configuration files. Operates on file path lists only -- no filesystem I/O.

use std::collections::HashSet;
use std::path::Path;

/// Category of a detected dev tool.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ToolCategory {
    /// Linter (ESLint, Clippy, pylint, etc.)
    Linter,
    /// Formatter (Prettier, rustfmt, Black, etc.)
    Formatter,
    /// Test runner (Jest, pytest, cargo-test, etc.)
    TestRunner,
    /// Type checker (TypeScript, mypy, etc.)
    TypeChecker,
    /// CI/CD system (GitHub Actions, GitLab CI, etc.)
    CiSystem,
    /// Code coverage tool
    CoverageTool,
    /// Bundler / build tool overlap (already in repo_scanner, but config presence)
    Bundler,
}

impl std::fmt::Display for ToolCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolCategory::Linter => write!(f, "linter"),
            ToolCategory::Formatter => write!(f, "formatter"),
            ToolCategory::TestRunner => write!(f, "test-runner"),
            ToolCategory::TypeChecker => write!(f, "type-checker"),
            ToolCategory::CiSystem => write!(f, "ci-system"),
            ToolCategory::CoverageTool => write!(f, "coverage-tool"),
            ToolCategory::Bundler => write!(f, "bundler"),
        }
    }
}

/// A detected development tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedTool {
    /// Tool name (e.g., "eslint", "prettier", "jest").
    pub name: String,
    /// Category of the tool.
    pub category: ToolCategory,
    /// Config file(s) that indicated this tool's presence.
    pub config_files: Vec<String>,
}

/// Result of detecting development tools in a repository.
#[derive(Debug, Clone)]
pub struct ToolDetectionResult {
    /// All detected tools.
    pub tools: Vec<DetectedTool>,
}

impl ToolDetectionResult {
    /// Get tools by category.
    pub fn by_category(&self, category: &ToolCategory) -> Vec<&DetectedTool> {
        self.tools
            .iter()
            .filter(|t| t.category == *category)
            .collect()
    }

    /// Get all linters.
    pub fn linters(&self) -> Vec<&DetectedTool> {
        self.by_category(&ToolCategory::Linter)
    }

    /// Get all formatters.
    pub fn formatters(&self) -> Vec<&DetectedTool> {
        self.by_category(&ToolCategory::Formatter)
    }

    /// Get all test runners.
    pub fn test_runners(&self) -> Vec<&DetectedTool> {
        self.by_category(&ToolCategory::TestRunner)
    }

    /// Get all CI systems.
    pub fn ci_systems(&self) -> Vec<&DetectedTool> {
        self.by_category(&ToolCategory::CiSystem)
    }

    /// Check if a specific tool is detected by name.
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.iter().any(|t| t.name == name)
    }
}

/// A detection rule: config file pattern -> tool.
struct ToolRule {
    /// File names or path patterns to match.
    patterns: &'static [&'static str],
    /// Tool name.
    name: &'static str,
    /// Tool category.
    category: ToolCategory,
}

// Linter rules
const LINTER_RULES: &[ToolRule] = &[
    ToolRule {
        patterns: &[
            ".eslintrc",
            ".eslintrc.js",
            ".eslintrc.cjs",
            ".eslintrc.json",
            ".eslintrc.yml",
            ".eslintrc.yaml",
            "eslint.config.js",
            "eslint.config.mjs",
            "eslint.config.cjs",
        ],
        name: "eslint",
        category: ToolCategory::Linter,
    },
    ToolRule {
        patterns: &["clippy.toml", ".clippy.toml"],
        name: "clippy",
        category: ToolCategory::Linter,
    },
    ToolRule {
        patterns: &[".pylintrc", "pylintrc", ".pylintrc.toml"],
        name: "pylint",
        category: ToolCategory::Linter,
    },
    ToolRule {
        patterns: &[".flake8", "setup.cfg"],
        name: "flake8",
        category: ToolCategory::Linter,
    },
    ToolRule {
        patterns: &["ruff.toml", ".ruff.toml"],
        name: "ruff",
        category: ToolCategory::Linter,
    },
    ToolRule {
        patterns: &[".golangci.yml", ".golangci.yaml", ".golangci.json"],
        name: "golangci-lint",
        category: ToolCategory::Linter,
    },
    ToolRule {
        patterns: &[".rubocop.yml"],
        name: "rubocop",
        category: ToolCategory::Linter,
    },
    ToolRule {
        patterns: &["biome.json", "biome.jsonc"],
        name: "biome",
        category: ToolCategory::Linter,
    },
    ToolRule {
        patterns: &[
            ".stylelintrc",
            ".stylelintrc.json",
            "stylelint.config.js",
            "stylelint.config.mjs",
        ],
        name: "stylelint",
        category: ToolCategory::Linter,
    },
];

// Formatter rules
const FORMATTER_RULES: &[ToolRule] = &[
    ToolRule {
        patterns: &[
            ".prettierrc",
            ".prettierrc.js",
            ".prettierrc.cjs",
            ".prettierrc.json",
            ".prettierrc.yml",
            ".prettierrc.yaml",
            ".prettierrc.toml",
            "prettier.config.js",
            "prettier.config.cjs",
            "prettier.config.mjs",
        ],
        name: "prettier",
        category: ToolCategory::Formatter,
    },
    ToolRule {
        patterns: &["rustfmt.toml", ".rustfmt.toml"],
        name: "rustfmt",
        category: ToolCategory::Formatter,
    },
    ToolRule {
        patterns: &[".editorconfig"],
        name: "editorconfig",
        category: ToolCategory::Formatter,
    },
    ToolRule {
        patterns: &["deno.json", "deno.jsonc"],
        name: "deno-fmt",
        category: ToolCategory::Formatter,
    },
];

// Test runner rules
const TEST_RUNNER_RULES: &[ToolRule] = &[
    ToolRule {
        patterns: &[
            "jest.config.js",
            "jest.config.ts",
            "jest.config.cjs",
            "jest.config.mjs",
            "jest.config.json",
        ],
        name: "jest",
        category: ToolCategory::TestRunner,
    },
    ToolRule {
        patterns: &[
            "vitest.config.js",
            "vitest.config.ts",
            "vitest.config.mjs",
            "vitest.workspace.ts",
            "vitest.workspace.js",
        ],
        name: "vitest",
        category: ToolCategory::TestRunner,
    },
    ToolRule {
        patterns: &["pytest.ini", "conftest.py"],
        name: "pytest",
        category: ToolCategory::TestRunner,
    },
    ToolRule {
        patterns: &[
            ".mocharc.yml",
            ".mocharc.yaml",
            ".mocharc.json",
            ".mocharc.js",
        ],
        name: "mocha",
        category: ToolCategory::TestRunner,
    },
    ToolRule {
        patterns: &["playwright.config.ts", "playwright.config.js"],
        name: "playwright",
        category: ToolCategory::TestRunner,
    },
    ToolRule {
        patterns: &[
            "cypress.config.js",
            "cypress.config.ts",
            "cypress.config.cjs",
            "cypress.config.mjs",
        ],
        name: "cypress",
        category: ToolCategory::TestRunner,
    },
];

// Type checker rules
const TYPE_CHECKER_RULES: &[ToolRule] = &[
    ToolRule {
        patterns: &["tsconfig.json", "tsconfig.base.json"],
        name: "typescript",
        category: ToolCategory::TypeChecker,
    },
    ToolRule {
        patterns: &["mypy.ini", ".mypy.ini"],
        name: "mypy",
        category: ToolCategory::TypeChecker,
    },
    ToolRule {
        patterns: &["pyrightconfig.json"],
        name: "pyright",
        category: ToolCategory::TypeChecker,
    },
];

// CI system rules (check path patterns, not just file names)
const CI_RULES: &[(&str, &str)] = &[
    (".github/workflows/", "github-actions"),
    (".gitlab-ci.yml", "gitlab-ci"),
    ("Jenkinsfile", "jenkins"),
    (".circleci/", "circleci"),
    (".travis.yml", "travis-ci"),
    ("azure-pipelines.yml", "azure-devops"),
    ("bitbucket-pipelines.yml", "bitbucket-pipelines"),
    (".buildkite/", "buildkite"),
];

// Coverage tool rules
const COVERAGE_RULES: &[ToolRule] = &[
    ToolRule {
        patterns: &[".nycrc", ".nycrc.json", ".nycrc.yml"],
        name: "nyc",
        category: ToolCategory::CoverageTool,
    },
    ToolRule {
        patterns: &[".coveragerc", ".coverage"],
        name: "coverage-py",
        category: ToolCategory::CoverageTool,
    },
    ToolRule {
        patterns: &["codecov.yml", ".codecov.yml"],
        name: "codecov",
        category: ToolCategory::CoverageTool,
    },
];

/// Detects development tools present in a repository.
pub struct ToolDetector;

impl ToolDetector {
    /// Detect all development tools from file paths.
    pub fn detect(file_paths: &[String]) -> ToolDetectionResult {
        let file_names: HashSet<String> = file_paths
            .iter()
            .filter_map(|p| {
                Path::new(p)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
            })
            .collect();

        let mut tools = Vec::new();
        let mut seen_names: HashSet<String> = HashSet::new();

        // Check all rule sets
        let all_rules: Vec<&[ToolRule]> = vec![
            LINTER_RULES,
            FORMATTER_RULES,
            TEST_RUNNER_RULES,
            TYPE_CHECKER_RULES,
            COVERAGE_RULES,
        ];

        for rules in all_rules {
            for rule in rules {
                let matching_files: Vec<String> = rule
                    .patterns
                    .iter()
                    .filter(|pattern| file_names.contains(**pattern))
                    .map(|p| p.to_string())
                    .collect();

                if !matching_files.is_empty() && seen_names.insert(rule.name.to_string()) {
                    tools.push(DetectedTool {
                        name: rule.name.to_string(),
                        category: rule.category.clone(),
                        config_files: matching_files,
                    });
                }
            }
        }

        // Check pyproject.toml-based tools (ruff, mypy, pytest configs can live there)
        if file_names.contains("pyproject.toml") {
            if !seen_names.contains("ruff") {
                // pyproject.toml might contain ruff config -- we can't read contents,
                // but if ruff.toml isn't present and pyproject.toml is, it's a hint.
                // We skip this to avoid false positives.
            }
            if !seen_names.contains("pytest") {
                // Similarly for pytest -- pyproject.toml can contain [tool.pytest]
                // but we can't verify without reading file content.
            }
        }

        // Check CI systems (path-based patterns)
        for &(pattern, name) in CI_RULES {
            let matching_files: Vec<String> = file_paths
                .iter()
                .filter(|p| {
                    if pattern.ends_with('/') {
                        p.starts_with(pattern)
                    } else {
                        let file_name = Path::new(p)
                            .file_name()
                            .map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_default();
                        file_name == pattern || *p == pattern
                    }
                })
                .cloned()
                .collect();

            if !matching_files.is_empty() && seen_names.insert(name.to_string()) {
                tools.push(DetectedTool {
                    name: name.to_string(),
                    category: ToolCategory::CiSystem,
                    config_files: matching_files,
                });
            }
        }

        // Detect cargo test (Rust projects always have it if Cargo.toml exists)
        if file_names.contains("Cargo.toml") && seen_names.insert("cargo-test".to_string()) {
            tools.push(DetectedTool {
                name: "cargo-test".to_string(),
                category: ToolCategory::TestRunner,
                config_files: vec!["Cargo.toml".to_string()],
            });
        }

        // Detect go test
        if file_names.contains("go.mod") && seen_names.insert("go-test".to_string()) {
            tools.push(DetectedTool {
                name: "go-test".to_string(),
                category: ToolCategory::TestRunner,
                config_files: vec!["go.mod".to_string()],
            });
        }

        ToolDetectionResult { tools }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn js_project_paths() -> Vec<String> {
        vec![
            "package.json".to_string(),
            ".eslintrc.json".to_string(),
            ".prettierrc".to_string(),
            "tsconfig.json".to_string(),
            "jest.config.ts".to_string(),
            ".github/workflows/ci.yml".to_string(),
            "src/index.ts".to_string(),
        ]
    }

    fn rust_project_paths() -> Vec<String> {
        vec![
            "Cargo.toml".to_string(),
            "rustfmt.toml".to_string(),
            "clippy.toml".to_string(),
            ".github/workflows/ci.yml".to_string(),
            ".github/workflows/release.yml".to_string(),
            "src/main.rs".to_string(),
        ]
    }

    fn python_project_paths() -> Vec<String> {
        vec![
            "pyproject.toml".to_string(),
            "ruff.toml".to_string(),
            "mypy.ini".to_string(),
            "conftest.py".to_string(),
            ".coveragerc".to_string(),
            ".gitlab-ci.yml".to_string(),
            "src/main.py".to_string(),
        ]
    }

    #[test]
    fn test_detect_js_tools() {
        let result = ToolDetector::detect(&js_project_paths());

        assert!(result.has_tool("eslint"));
        assert!(result.has_tool("prettier"));
        assert!(result.has_tool("typescript"));
        assert!(result.has_tool("jest"));
        assert!(result.has_tool("github-actions"));
    }

    #[test]
    fn test_detect_rust_tools() {
        let result = ToolDetector::detect(&rust_project_paths());

        assert!(result.has_tool("clippy"));
        assert!(result.has_tool("rustfmt"));
        assert!(result.has_tool("cargo-test"));
        assert!(result.has_tool("github-actions"));
    }

    #[test]
    fn test_detect_python_tools() {
        let result = ToolDetector::detect(&python_project_paths());

        assert!(result.has_tool("ruff"));
        assert!(result.has_tool("mypy"));
        assert!(result.has_tool("pytest"));
        assert!(result.has_tool("coverage-py"));
        assert!(result.has_tool("gitlab-ci"));
    }

    #[test]
    fn test_linters_filter() {
        let result = ToolDetector::detect(&js_project_paths());
        let linters = result.linters();
        assert!(linters.iter().any(|t| t.name == "eslint"));
    }

    #[test]
    fn test_formatters_filter() {
        let result = ToolDetector::detect(&js_project_paths());
        let formatters = result.formatters();
        assert!(formatters.iter().any(|t| t.name == "prettier"));
    }

    #[test]
    fn test_test_runners_filter() {
        let result = ToolDetector::detect(&js_project_paths());
        let runners = result.test_runners();
        assert!(runners.iter().any(|t| t.name == "jest"));
    }

    #[test]
    fn test_ci_systems_filter() {
        let result = ToolDetector::detect(&rust_project_paths());
        let ci = result.ci_systems();
        assert!(ci.iter().any(|t| t.name == "github-actions"));
    }

    #[test]
    fn test_config_files_tracked() {
        let result = ToolDetector::detect(&js_project_paths());
        let eslint = result.tools.iter().find(|t| t.name == "eslint").unwrap();
        assert!(eslint.config_files.contains(&".eslintrc.json".to_string()));
    }

    #[test]
    fn test_empty_paths() {
        let result = ToolDetector::detect(&[]);
        assert!(result.tools.is_empty());
    }

    #[test]
    fn test_no_tools_detected() {
        let paths = vec!["README.md".to_string(), "src/main.txt".to_string()];
        let result = ToolDetector::detect(&paths);
        assert!(result.tools.is_empty());
    }

    #[test]
    fn test_go_project_tools() {
        let paths = vec![
            "go.mod".to_string(),
            "go.sum".to_string(),
            ".golangci.yml".to_string(),
            "main.go".to_string(),
        ];
        let result = ToolDetector::detect(&paths);

        assert!(result.has_tool("golangci-lint"));
        assert!(result.has_tool("go-test"));
    }

    #[test]
    fn test_vitest_detection() {
        let paths = vec![
            "package.json".to_string(),
            "vitest.config.ts".to_string(),
            "src/index.ts".to_string(),
        ];
        let result = ToolDetector::detect(&paths);
        assert!(result.has_tool("vitest"));
    }

    #[test]
    fn test_playwright_detection() {
        let paths = vec![
            "package.json".to_string(),
            "playwright.config.ts".to_string(),
        ];
        let result = ToolDetector::detect(&paths);
        assert!(result.has_tool("playwright"));
    }

    #[test]
    fn test_biome_detection() {
        let paths = vec!["package.json".to_string(), "biome.json".to_string()];
        let result = ToolDetector::detect(&paths);
        assert!(result.has_tool("biome"));
    }

    #[test]
    fn test_multiple_ci_systems_not_duplicated() {
        let paths = vec![
            ".github/workflows/ci.yml".to_string(),
            ".github/workflows/deploy.yml".to_string(),
            ".github/workflows/test.yml".to_string(),
        ];
        let result = ToolDetector::detect(&paths);
        let gh_count = result
            .tools
            .iter()
            .filter(|t| t.name == "github-actions")
            .count();
        assert_eq!(gh_count, 1);
        // But all workflow files should be in config_files
        let gh = result
            .tools
            .iter()
            .find(|t| t.name == "github-actions")
            .unwrap();
        assert_eq!(gh.config_files.len(), 3);
    }

    #[test]
    fn test_display_impl() {
        assert_eq!(ToolCategory::Linter.to_string(), "linter");
        assert_eq!(ToolCategory::Formatter.to_string(), "formatter");
        assert_eq!(ToolCategory::TestRunner.to_string(), "test-runner");
        assert_eq!(ToolCategory::CiSystem.to_string(), "ci-system");
    }

    #[test]
    fn test_editorconfig_detection() {
        let paths = vec![".editorconfig".to_string(), "src/main.rs".to_string()];
        let result = ToolDetector::detect(&paths);
        assert!(result.has_tool("editorconfig"));
    }

    #[test]
    fn test_eslint_flat_config() {
        let paths = vec!["package.json".to_string(), "eslint.config.mjs".to_string()];
        let result = ToolDetector::detect(&paths);
        assert!(result.has_tool("eslint"));
    }
}

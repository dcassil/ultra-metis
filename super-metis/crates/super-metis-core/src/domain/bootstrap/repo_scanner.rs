//! Repo scanner for language, package manager, and build tool detection.
//!
//! Scans file path lists to identify what languages, package managers, and
//! build tools are present in a repository. Operates purely on path strings
//! with no filesystem I/O.

use std::collections::HashSet;
use std::path::Path;

/// A detected language in the repository.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DetectedLanguage {
    /// Language name (e.g., "rust", "javascript", "python", "go").
    pub name: String,
    /// The manifest file(s) that indicated this language.
    pub evidence: Vec<String>,
    /// Whether this appears to be the primary language (most files).
    pub is_primary: bool,
}

/// A detected package manager.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PackageManager {
    /// Cargo (Rust)
    Cargo,
    /// npm (JavaScript/TypeScript)
    Npm,
    /// Yarn (JavaScript/TypeScript)
    Yarn,
    /// pnpm (JavaScript/TypeScript)
    Pnpm,
    /// Bun (JavaScript/TypeScript)
    Bun,
    /// Go modules
    GoModules,
    /// pip / pyproject.toml (Python)
    Pip,
    /// Poetry (Python)
    Poetry,
    /// Maven (Java/Kotlin)
    Maven,
    /// Gradle (Java/Kotlin)
    Gradle,
    /// .NET (C#/F#)
    DotNet,
}

impl std::fmt::Display for PackageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageManager::Cargo => write!(f, "cargo"),
            PackageManager::Npm => write!(f, "npm"),
            PackageManager::Yarn => write!(f, "yarn"),
            PackageManager::Pnpm => write!(f, "pnpm"),
            PackageManager::Bun => write!(f, "bun"),
            PackageManager::GoModules => write!(f, "go-modules"),
            PackageManager::Pip => write!(f, "pip"),
            PackageManager::Poetry => write!(f, "poetry"),
            PackageManager::Maven => write!(f, "maven"),
            PackageManager::Gradle => write!(f, "gradle"),
            PackageManager::DotNet => write!(f, "dotnet"),
        }
    }
}

/// A detected build tool or task runner.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BuildTool {
    /// Cargo build (Rust)
    CargoBuild,
    /// Make
    Make,
    /// Just (justfile)
    Just,
    /// Turborepo
    Turborepo,
    /// Nx
    Nx,
    /// Webpack
    Webpack,
    /// Vite
    Vite,
    /// esbuild
    Esbuild,
    /// Rollup
    Rollup,
    /// Gradle
    GradleBuild,
    /// Maven
    MavenBuild,
    /// CMake
    CMake,
    /// Bazel
    Bazel,
    /// Docker
    Docker,
}

impl std::fmt::Display for BuildTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildTool::CargoBuild => write!(f, "cargo-build"),
            BuildTool::Make => write!(f, "make"),
            BuildTool::Just => write!(f, "just"),
            BuildTool::Turborepo => write!(f, "turborepo"),
            BuildTool::Nx => write!(f, "nx"),
            BuildTool::Webpack => write!(f, "webpack"),
            BuildTool::Vite => write!(f, "vite"),
            BuildTool::Esbuild => write!(f, "esbuild"),
            BuildTool::Rollup => write!(f, "rollup"),
            BuildTool::GradleBuild => write!(f, "gradle"),
            BuildTool::MavenBuild => write!(f, "maven"),
            BuildTool::CMake => write!(f, "cmake"),
            BuildTool::Bazel => write!(f, "bazel"),
            BuildTool::Docker => write!(f, "docker"),
        }
    }
}

/// Result of scanning a repository for languages, package managers, and build tools.
#[derive(Debug, Clone)]
pub struct RepoScanResult {
    /// All detected languages, ordered by file count (primary first).
    pub languages: Vec<DetectedLanguage>,
    /// Detected package managers.
    pub package_managers: Vec<PackageManager>,
    /// Detected build tools.
    pub build_tools: Vec<BuildTool>,
    /// Total number of source files scanned.
    pub total_files: usize,
    /// File extension distribution (extension -> count).
    pub extension_counts: Vec<(String, usize)>,
}

impl RepoScanResult {
    /// Get the primary language, if any.
    pub fn primary_language(&self) -> Option<&DetectedLanguage> {
        self.languages.iter().find(|l| l.is_primary)
    }

    /// Get the primary language name as a string.
    pub fn primary_language_name(&self) -> Option<&str> {
        self.primary_language().map(|l| l.name.as_str())
    }
}

/// Language detection rules: manifest file -> language name.
struct LanguageRule {
    /// File name or pattern to match (exact file name).
    manifest: &'static str,
    /// Language name to assign.
    language: &'static str,
}

const LANGUAGE_RULES: &[LanguageRule] = &[
    LanguageRule { manifest: "Cargo.toml", language: "rust" },
    LanguageRule { manifest: "package.json", language: "javascript" },
    LanguageRule { manifest: "go.mod", language: "go" },
    LanguageRule { manifest: "pyproject.toml", language: "python" },
    LanguageRule { manifest: "setup.py", language: "python" },
    LanguageRule { manifest: "requirements.txt", language: "python" },
    LanguageRule { manifest: "Pipfile", language: "python" },
    LanguageRule { manifest: "pom.xml", language: "java" },
    LanguageRule { manifest: "build.gradle", language: "java" },
    LanguageRule { manifest: "build.gradle.kts", language: "kotlin" },
    LanguageRule { manifest: "Gemfile", language: "ruby" },
    LanguageRule { manifest: "composer.json", language: "php" },
    LanguageRule { manifest: "mix.exs", language: "elixir" },
    LanguageRule { manifest: "Package.swift", language: "swift" },
    LanguageRule { manifest: "CMakeLists.txt", language: "c-cpp" },
    LanguageRule { manifest: "Makefile.am", language: "c-cpp" },
];

/// Extension -> language mapping for file-count-based detection.
const EXTENSION_LANGUAGES: &[(&str, &str)] = &[
    ("rs", "rust"),
    ("js", "javascript"),
    ("jsx", "javascript"),
    ("ts", "typescript"),
    ("tsx", "typescript"),
    ("mjs", "javascript"),
    ("cjs", "javascript"),
    ("go", "go"),
    ("py", "python"),
    ("java", "java"),
    ("kt", "kotlin"),
    ("kts", "kotlin"),
    ("rb", "ruby"),
    ("php", "php"),
    ("ex", "elixir"),
    ("exs", "elixir"),
    ("swift", "swift"),
    ("c", "c-cpp"),
    ("cpp", "c-cpp"),
    ("cc", "c-cpp"),
    ("h", "c-cpp"),
    ("hpp", "c-cpp"),
    ("cs", "csharp"),
    ("fs", "fsharp"),
    ("scala", "scala"),
    ("clj", "clojure"),
    ("zig", "zig"),
    ("lua", "lua"),
    ("dart", "dart"),
];

/// Scans a repository's file paths to detect languages, package managers,
/// and build tools.
pub struct RepoScanner;

impl RepoScanner {
    /// Scan file paths and produce a [`RepoScanResult`].
    pub fn scan(file_paths: &[String]) -> RepoScanResult {
        let languages = Self::detect_languages(file_paths);
        let package_managers = Self::detect_package_managers(file_paths);
        let build_tools = Self::detect_build_tools(file_paths);
        let extension_counts = Self::count_extensions(file_paths);

        RepoScanResult {
            languages,
            package_managers,
            build_tools,
            total_files: file_paths.len(),
            extension_counts,
        }
    }

    /// Detect languages from manifest files and file extensions.
    fn detect_languages(file_paths: &[String]) -> Vec<DetectedLanguage> {
        // Step 1: Find manifest files at the root level
        let mut manifest_evidence: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        for path in file_paths {
            let file_name = Path::new(path)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default();

            // Check root-level manifests (depth 1 or 2 for monorepos)
            let depth = Path::new(path).components().count();
            if depth <= 2 {
                for rule in LANGUAGE_RULES {
                    if file_name == rule.manifest {
                        manifest_evidence
                            .entry(rule.language.to_string())
                            .or_default()
                            .push(path.clone());
                    }
                }
            }
        }

        // Step 2: Count source files by extension
        let mut ext_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for path in file_paths {
            if let Some(ext) = Path::new(path).extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                for &(file_ext, lang) in EXTENSION_LANGUAGES {
                    if ext_str == file_ext {
                        *ext_counts.entry(lang.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }

        // Merge JS and TS: if we see .ts files but manifest says "javascript",
        // treat typescript as a variant of javascript for language detection
        let ts_count = ext_counts.get("typescript").copied().unwrap_or(0);
        let js_count = ext_counts.get("javascript").copied().unwrap_or(0);
        if ts_count > 0 && manifest_evidence.contains_key("javascript") {
            // package.json projects with TS files are "javascript" (TypeScript) projects
            *ext_counts.entry("javascript".to_string()).or_insert(0) += ts_count;
        }

        // Step 3: Build language list from manifests + file counts
        let mut all_langs: HashSet<String> = HashSet::new();
        all_langs.extend(manifest_evidence.keys().cloned());
        all_langs.extend(ext_counts.keys().cloned());
        // Don't double-count typescript if we already merged it
        if manifest_evidence.contains_key("javascript") {
            all_langs.remove("typescript");
        }

        let mut languages: Vec<DetectedLanguage> = all_langs
            .into_iter()
            .map(|name| {
                let evidence = manifest_evidence
                    .get(&name)
                    .cloned()
                    .unwrap_or_default();
                let file_count = ext_counts.get(&name).copied().unwrap_or(0);
                DetectedLanguage {
                    name: name.clone(),
                    evidence,
                    is_primary: false, // Set below
                }
            })
            .filter(|lang| {
                // Only include languages with manifest evidence or significant file count
                !lang.evidence.is_empty()
                    || ext_counts.get(&lang.name).copied().unwrap_or(0) >= 3
            })
            .collect();

        // Sort by file count (descending)
        languages.sort_by(|a, b| {
            let count_a = ext_counts.get(&a.name).copied().unwrap_or(0);
            let count_b = ext_counts.get(&b.name).copied().unwrap_or(0);
            count_b.cmp(&count_a)
        });

        // Mark the first language as primary
        if let Some(first) = languages.first_mut() {
            first.is_primary = true;
        }

        languages
    }

    /// Detect package managers from file paths.
    fn detect_package_managers(file_paths: &[String]) -> Vec<PackageManager> {
        let file_names: HashSet<String> = file_paths
            .iter()
            .filter_map(|p| {
                let depth = Path::new(p).components().count();
                if depth <= 2 {
                    Path::new(p)
                        .file_name()
                        .map(|f| f.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .collect();

        let mut managers = Vec::new();

        if file_names.contains("Cargo.toml") || file_names.contains("Cargo.lock") {
            managers.push(PackageManager::Cargo);
        }
        if file_names.contains("pnpm-lock.yaml") || file_names.contains("pnpm-workspace.yaml") {
            managers.push(PackageManager::Pnpm);
        } else if file_names.contains("yarn.lock") {
            managers.push(PackageManager::Yarn);
        } else if file_names.contains("bun.lockb") || file_names.contains("bun.lock") {
            managers.push(PackageManager::Bun);
        } else if file_names.contains("package-lock.json") || file_names.contains("package.json") {
            managers.push(PackageManager::Npm);
        }
        if file_names.contains("go.mod") || file_names.contains("go.sum") {
            managers.push(PackageManager::GoModules);
        }
        if file_names.contains("poetry.lock") || file_names.contains("pyproject.toml") {
            // Check for poetry specifically
            if file_names.contains("poetry.lock") {
                managers.push(PackageManager::Poetry);
            } else {
                managers.push(PackageManager::Pip);
            }
        } else if file_names.contains("requirements.txt") || file_names.contains("Pipfile") || file_names.contains("setup.py") {
            managers.push(PackageManager::Pip);
        }
        if file_names.contains("pom.xml") {
            managers.push(PackageManager::Maven);
        }
        if file_names.contains("build.gradle") || file_names.contains("build.gradle.kts") {
            managers.push(PackageManager::Gradle);
        }
        if file_names.iter().any(|f| f.ends_with(".csproj") || f.ends_with(".sln") || f.ends_with(".fsproj")) {
            managers.push(PackageManager::DotNet);
        }

        managers
    }

    /// Detect build tools from file paths.
    fn detect_build_tools(file_paths: &[String]) -> Vec<BuildTool> {
        let file_names: HashSet<String> = file_paths
            .iter()
            .filter_map(|p| {
                Path::new(p)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
            })
            .collect();

        let mut tools = Vec::new();

        if file_names.contains("Cargo.toml") {
            tools.push(BuildTool::CargoBuild);
        }
        if file_names.contains("Makefile") || file_names.contains("makefile") || file_names.contains("GNUmakefile") {
            tools.push(BuildTool::Make);
        }
        if file_names.contains("justfile") || file_names.contains("Justfile") {
            tools.push(BuildTool::Just);
        }
        if file_names.contains("turbo.json") {
            tools.push(BuildTool::Turborepo);
        }
        if file_names.contains("nx.json") {
            tools.push(BuildTool::Nx);
        }
        if file_names.contains("webpack.config.js") || file_names.contains("webpack.config.ts") {
            tools.push(BuildTool::Webpack);
        }
        if file_names.contains("vite.config.js") || file_names.contains("vite.config.ts") || file_names.contains("vite.config.mjs") {
            tools.push(BuildTool::Vite);
        }
        if file_names.contains("rollup.config.js") || file_names.contains("rollup.config.ts") || file_names.contains("rollup.config.mjs") {
            tools.push(BuildTool::Rollup);
        }
        if file_names.contains("pom.xml") {
            tools.push(BuildTool::MavenBuild);
        }
        if file_names.contains("build.gradle") || file_names.contains("build.gradle.kts") {
            tools.push(BuildTool::GradleBuild);
        }
        if file_names.contains("CMakeLists.txt") {
            tools.push(BuildTool::CMake);
        }
        if file_names.contains("BUILD") || file_names.contains("BUILD.bazel") || file_names.contains("WORKSPACE") {
            tools.push(BuildTool::Bazel);
        }
        if file_names.contains("Dockerfile") || file_names.contains("docker-compose.yml") || file_names.contains("docker-compose.yaml") {
            tools.push(BuildTool::Docker);
        }

        tools
    }

    /// Count file extensions for distribution analysis.
    fn count_extensions(file_paths: &[String]) -> Vec<(String, usize)> {
        let mut counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for path in file_paths {
            if let Some(ext) = Path::new(path).extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                *counts.entry(ext_str).or_insert(0) += 1;
            }
        }
        let mut sorted: Vec<(String, usize)> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rust_project_paths() -> Vec<String> {
        vec![
            "Cargo.toml".to_string(),
            "Cargo.lock".to_string(),
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "src/domain/mod.rs".to_string(),
            "src/domain/types.rs".to_string(),
            "tests/integration_test.rs".to_string(),
        ]
    }

    fn js_project_paths() -> Vec<String> {
        vec![
            "package.json".to_string(),
            "package-lock.json".to_string(),
            "tsconfig.json".to_string(),
            "src/index.ts".to_string(),
            "src/routes/users.ts".to_string(),
            "src/services/auth.ts".to_string(),
            "src/utils/helpers.ts".to_string(),
            "tests/auth.test.ts".to_string(),
        ]
    }

    fn python_project_paths() -> Vec<String> {
        vec![
            "pyproject.toml".to_string(),
            "poetry.lock".to_string(),
            "src/main.py".to_string(),
            "src/models/user.py".to_string(),
            "src/services/auth.py".to_string(),
            "tests/test_auth.py".to_string(),
        ]
    }

    fn go_project_paths() -> Vec<String> {
        vec![
            "go.mod".to_string(),
            "go.sum".to_string(),
            "main.go".to_string(),
            "internal/handler.go".to_string(),
            "internal/service.go".to_string(),
            "pkg/utils.go".to_string(),
        ]
    }

    fn multi_language_paths() -> Vec<String> {
        vec![
            "package.json".to_string(),
            "Cargo.toml".to_string(),
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "src/domain/mod.rs".to_string(),
            "web/index.ts".to_string(),
            "web/app.ts".to_string(),
        ]
    }

    #[test]
    fn test_scan_rust_project() {
        let result = RepoScanner::scan(&rust_project_paths());

        assert_eq!(result.primary_language_name(), Some("rust"));
        assert!(result.package_managers.contains(&PackageManager::Cargo));
        assert!(result.build_tools.contains(&BuildTool::CargoBuild));
    }

    #[test]
    fn test_scan_javascript_project() {
        let result = RepoScanner::scan(&js_project_paths());

        assert_eq!(result.primary_language_name(), Some("javascript"));
        assert!(result.package_managers.contains(&PackageManager::Npm));
    }

    #[test]
    fn test_scan_python_project() {
        let result = RepoScanner::scan(&python_project_paths());

        assert_eq!(result.primary_language_name(), Some("python"));
        assert!(result.package_managers.contains(&PackageManager::Poetry));
    }

    #[test]
    fn test_scan_go_project() {
        let result = RepoScanner::scan(&go_project_paths());

        assert_eq!(result.primary_language_name(), Some("go"));
        assert!(result.package_managers.contains(&PackageManager::GoModules));
    }

    #[test]
    fn test_scan_multi_language() {
        let result = RepoScanner::scan(&multi_language_paths());

        assert!(result.languages.len() >= 2);
        assert!(result.languages.iter().any(|l| l.name == "rust"));
        assert!(result.languages.iter().any(|l| l.name == "javascript"));
        // Primary should be the one with more source files
        assert!(result.primary_language().is_some());
    }

    #[test]
    fn test_pnpm_detection() {
        let paths = vec![
            "package.json".to_string(),
            "pnpm-lock.yaml".to_string(),
            "pnpm-workspace.yaml".to_string(),
        ];
        let result = RepoScanner::scan(&paths);
        assert!(result.package_managers.contains(&PackageManager::Pnpm));
        assert!(!result.package_managers.contains(&PackageManager::Npm));
    }

    #[test]
    fn test_yarn_detection() {
        let paths = vec![
            "package.json".to_string(),
            "yarn.lock".to_string(),
        ];
        let result = RepoScanner::scan(&paths);
        assert!(result.package_managers.contains(&PackageManager::Yarn));
        assert!(!result.package_managers.contains(&PackageManager::Npm));
    }

    #[test]
    fn test_build_tools_detection() {
        let paths = vec![
            "package.json".to_string(),
            "turbo.json".to_string(),
            "Makefile".to_string(),
            "Dockerfile".to_string(),
            "vite.config.ts".to_string(),
        ];
        let result = RepoScanner::scan(&paths);
        assert!(result.build_tools.contains(&BuildTool::Turborepo));
        assert!(result.build_tools.contains(&BuildTool::Make));
        assert!(result.build_tools.contains(&BuildTool::Docker));
        assert!(result.build_tools.contains(&BuildTool::Vite));
    }

    #[test]
    fn test_extension_counts() {
        let paths = vec![
            "src/a.rs".to_string(),
            "src/b.rs".to_string(),
            "src/c.rs".to_string(),
            "tests/d.rs".to_string(),
            "build.toml".to_string(),
        ];
        let result = RepoScanner::scan(&paths);
        let rs_count = result.extension_counts.iter().find(|(ext, _)| ext == "rs");
        assert_eq!(rs_count, Some(&("rs".to_string(), 4)));
    }

    #[test]
    fn test_empty_paths() {
        let result = RepoScanner::scan(&[]);
        assert!(result.languages.is_empty());
        assert!(result.package_managers.is_empty());
        assert!(result.build_tools.is_empty());
        assert_eq!(result.total_files, 0);
    }

    #[test]
    fn test_primary_language_is_first() {
        let result = RepoScanner::scan(&rust_project_paths());
        assert!(result.languages[0].is_primary);
        // All others should not be primary
        for lang in result.languages.iter().skip(1) {
            assert!(!lang.is_primary);
        }
    }

    #[test]
    fn test_java_gradle_detection() {
        let paths = vec![
            "build.gradle.kts".to_string(),
            "settings.gradle.kts".to_string(),
            "src/main/java/App.java".to_string(),
            "src/main/java/Service.java".to_string(),
            "src/test/java/AppTest.java".to_string(),
        ];
        let result = RepoScanner::scan(&paths);
        assert!(result.package_managers.contains(&PackageManager::Gradle));
        assert!(result.build_tools.contains(&BuildTool::GradleBuild));
    }

    #[test]
    fn test_bazel_detection() {
        let paths = vec![
            "WORKSPACE".to_string(),
            "BUILD".to_string(),
            "src/BUILD".to_string(),
            "src/main.go".to_string(),
        ];
        let result = RepoScanner::scan(&paths);
        assert!(result.build_tools.contains(&BuildTool::Bazel));
    }

    #[test]
    fn test_display_impls() {
        assert_eq!(PackageManager::Cargo.to_string(), "cargo");
        assert_eq!(PackageManager::Pnpm.to_string(), "pnpm");
        assert_eq!(BuildTool::Turborepo.to_string(), "turborepo");
        assert_eq!(BuildTool::CargoBuild.to_string(), "cargo-build");
    }
}

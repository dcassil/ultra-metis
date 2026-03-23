//! Monorepo pattern detector and package discovery.
//!
//! Detects whether a repository uses a monorepo pattern (Cargo workspaces,
//! pnpm workspaces, Turborepo, Nx, Lerna) and discovers packages, apps,
//! and libraries within it. Operates on file path lists only.

use std::collections::HashSet;
use std::path::Path;

/// The type of monorepo tool detected.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MonorepoTool {
    /// Cargo workspace (Rust)
    CargoWorkspace,
    /// pnpm workspaces
    PnpmWorkspace,
    /// Yarn workspaces
    YarnWorkspace,
    /// npm workspaces
    NpmWorkspace,
    /// Turborepo (builds on top of npm/pnpm/yarn workspaces)
    Turborepo,
    /// Nx
    Nx,
    /// Lerna
    Lerna,
    /// Go workspace (go.work)
    GoWorkspace,
    /// .NET solution with multiple projects
    DotNetSolution,
}

impl std::fmt::Display for MonorepoTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonorepoTool::CargoWorkspace => write!(f, "cargo-workspace"),
            MonorepoTool::PnpmWorkspace => write!(f, "pnpm-workspace"),
            MonorepoTool::YarnWorkspace => write!(f, "yarn-workspace"),
            MonorepoTool::NpmWorkspace => write!(f, "npm-workspace"),
            MonorepoTool::Turborepo => write!(f, "turborepo"),
            MonorepoTool::Nx => write!(f, "nx"),
            MonorepoTool::Lerna => write!(f, "lerna"),
            MonorepoTool::GoWorkspace => write!(f, "go-workspace"),
            MonorepoTool::DotNetSolution => write!(f, "dotnet-solution"),
        }
    }
}

/// Classification of a discovered package within a monorepo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageKind {
    /// An application (deployable binary or service).
    App,
    /// A library (shared code consumed by other packages).
    Library,
    /// A package whose kind could not be determined.
    Unknown,
}

impl std::fmt::Display for PackageKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageKind::App => write!(f, "app"),
            PackageKind::Library => write!(f, "library"),
            PackageKind::Unknown => write!(f, "unknown"),
        }
    }
}

/// A discovered package within a monorepo.
#[derive(Debug, Clone)]
pub struct DiscoveredPackage {
    /// Relative path from the repo root to the package directory.
    pub path: String,
    /// Name of the package (directory name or inferred from manifest).
    pub name: String,
    /// Classification of the package.
    pub kind: PackageKind,
    /// The manifest file that indicates this is a package.
    pub manifest_file: String,
}

/// Result of monorepo detection.
#[derive(Debug, Clone)]
pub struct MonorepoInfo {
    /// Whether the repo is a monorepo.
    pub is_monorepo: bool,
    /// The monorepo tools detected (may be multiple, e.g. Turborepo + pnpm).
    pub tools: Vec<MonorepoTool>,
    /// Discovered packages within the monorepo.
    pub packages: Vec<DiscoveredPackage>,
    /// Common package directory prefixes (e.g., "packages/", "apps/", "crates/").
    pub package_dirs: Vec<String>,
}

impl MonorepoInfo {
    /// Create a result indicating this is not a monorepo.
    pub fn not_monorepo() -> Self {
        Self {
            is_monorepo: false,
            tools: vec![],
            packages: vec![],
            package_dirs: vec![],
        }
    }

    /// Get all app packages.
    pub fn apps(&self) -> Vec<&DiscoveredPackage> {
        self.packages
            .iter()
            .filter(|p| p.kind == PackageKind::App)
            .collect()
    }

    /// Get all library packages.
    pub fn libraries(&self) -> Vec<&DiscoveredPackage> {
        self.packages
            .iter()
            .filter(|p| p.kind == PackageKind::Library)
            .collect()
    }
}

/// Well-known monorepo package directory names.
const PACKAGE_DIRS: &[&str] = &[
    "packages",
    "apps",
    "libs",
    "crates",
    "modules",
    "services",
    "tools",
    "plugins",
    "extensions",
];

/// Well-known app directory names (heuristic for classification).
const APP_DIR_NAMES: &[&str] = &[
    "apps", "app", "services", "server", "web", "api", "cli", "bin", "cmd",
];

/// Well-known library directory names.
const LIB_DIR_NAMES: &[&str] = &[
    "packages", "libs", "lib", "shared", "common", "core", "utils", "internal",
];

/// Detects monorepo patterns and discovers packages.
pub struct MonorepoDetector;

impl MonorepoDetector {
    /// Detect monorepo patterns from file paths.
    pub fn detect(file_paths: &[String]) -> MonorepoInfo {
        let tools = Self::detect_tools(file_paths);

        if tools.is_empty() {
            // Even without explicit tool markers, check for multi-package structure
            let packages = Self::discover_packages_by_structure(file_paths);
            if packages.len() >= 2 {
                let package_dirs = Self::find_package_dirs(&packages);
                return MonorepoInfo {
                    is_monorepo: true,
                    tools: vec![],
                    packages,
                    package_dirs,
                };
            }
            return MonorepoInfo::not_monorepo();
        }

        let packages = Self::discover_packages(file_paths, &tools);
        let package_dirs = Self::find_package_dirs(&packages);

        MonorepoInfo {
            is_monorepo: true,
            tools,
            packages,
            package_dirs,
        }
    }

    /// Detect which monorepo tools are present.
    fn detect_tools(file_paths: &[String]) -> Vec<MonorepoTool> {
        let root_files: HashSet<String> = file_paths
            .iter()
            .filter(|p| Path::new(p).components().count() == 1)
            .map(|p| p.clone())
            .collect();

        let all_file_names: HashSet<String> = file_paths
            .iter()
            .filter_map(|p| {
                Path::new(p)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
            })
            .collect();

        let mut tools = Vec::new();

        // Turborepo (must check before workspace tools since it layers on top)
        if root_files.contains("turbo.json") {
            tools.push(MonorepoTool::Turborepo);
        }

        // Nx
        if root_files.contains("nx.json") {
            tools.push(MonorepoTool::Nx);
        }

        // Lerna
        if root_files.contains("lerna.json") {
            tools.push(MonorepoTool::Lerna);
        }

        // pnpm workspace
        if root_files.contains("pnpm-workspace.yaml") {
            tools.push(MonorepoTool::PnpmWorkspace);
        }

        // Yarn workspace (if yarn.lock exists and there are nested package.json files)
        if root_files.contains("yarn.lock") && Self::has_nested_package_json(file_paths) {
            tools.push(MonorepoTool::YarnWorkspace);
        }

        // npm workspace (package.json with nested package.json files, no yarn/pnpm)
        if root_files.contains("package.json")
            && !root_files.contains("yarn.lock")
            && !root_files.contains("pnpm-workspace.yaml")
            && Self::has_nested_package_json(file_paths)
        {
            // Only mark as npm workspace if we see nested manifests in known dirs
            let has_workspace_structure = file_paths.iter().any(|p| {
                let depth = Path::new(p).components().count();
                depth == 3
                    && Path::new(p)
                        .file_name()
                        .map(|f| f.to_string_lossy() == "package.json")
                        .unwrap_or(false)
                    && PACKAGE_DIRS
                        .iter()
                        .any(|dir| p.starts_with(&format!("{}/", dir)))
            });
            if has_workspace_structure {
                tools.push(MonorepoTool::NpmWorkspace);
            }
        }

        // Cargo workspace
        if Self::has_cargo_workspace_structure(file_paths) {
            tools.push(MonorepoTool::CargoWorkspace);
        }

        // Go workspace
        if root_files.contains("go.work") {
            tools.push(MonorepoTool::GoWorkspace);
        }

        // .NET solution
        if all_file_names.iter().any(|f| f.ends_with(".sln")) {
            let has_multiple_projects = file_paths
                .iter()
                .filter(|p| {
                    Path::new(p)
                        .extension()
                        .map(|e| e == "csproj" || e == "fsproj")
                        .unwrap_or(false)
                })
                .count()
                >= 2;
            if has_multiple_projects {
                tools.push(MonorepoTool::DotNetSolution);
            }
        }

        tools
    }

    /// Check if there are nested package.json files (depth > 1).
    fn has_nested_package_json(file_paths: &[String]) -> bool {
        file_paths.iter().any(|p| {
            let depth = Path::new(p).components().count();
            depth >= 3
                && Path::new(p)
                    .file_name()
                    .map(|f| f.to_string_lossy() == "package.json")
                    .unwrap_or(false)
        })
    }

    /// Check if this looks like a Cargo workspace (root Cargo.toml + nested Cargo.tomls).
    fn has_cargo_workspace_structure(file_paths: &[String]) -> bool {
        let has_root_cargo = file_paths.iter().any(|p| p == "Cargo.toml");
        let nested_cargo_count = file_paths
            .iter()
            .filter(|p| {
                let depth = Path::new(p).components().count();
                depth >= 3
                    && Path::new(p)
                        .file_name()
                        .map(|f| f.to_string_lossy() == "Cargo.toml")
                        .unwrap_or(false)
            })
            .count();
        has_root_cargo && nested_cargo_count >= 1
    }

    /// Discover packages based on detected tools.
    fn discover_packages(file_paths: &[String], tools: &[MonorepoTool]) -> Vec<DiscoveredPackage> {
        let mut packages = Vec::new();
        let mut seen_paths: HashSet<String> = HashSet::new();

        for tool in tools {
            match tool {
                MonorepoTool::CargoWorkspace => {
                    Self::discover_cargo_packages(file_paths, &mut packages, &mut seen_paths);
                }
                MonorepoTool::PnpmWorkspace
                | MonorepoTool::YarnWorkspace
                | MonorepoTool::NpmWorkspace
                | MonorepoTool::Turborepo
                | MonorepoTool::Lerna => {
                    Self::discover_js_packages(file_paths, &mut packages, &mut seen_paths);
                }
                MonorepoTool::Nx => {
                    Self::discover_js_packages(file_paths, &mut packages, &mut seen_paths);
                }
                MonorepoTool::GoWorkspace => {
                    Self::discover_go_packages(file_paths, &mut packages, &mut seen_paths);
                }
                MonorepoTool::DotNetSolution => {
                    Self::discover_dotnet_packages(file_paths, &mut packages, &mut seen_paths);
                }
            }
        }

        packages
    }

    /// Discover Cargo workspace members.
    fn discover_cargo_packages(
        file_paths: &[String],
        packages: &mut Vec<DiscoveredPackage>,
        seen: &mut HashSet<String>,
    ) {
        for path in file_paths {
            if Path::new(path)
                .file_name()
                .map(|f| f.to_string_lossy() == "Cargo.toml")
                .unwrap_or(false)
            {
                let depth = Path::new(path).components().count();
                if depth >= 3 {
                    // Nested Cargo.toml = workspace member
                    if let Some(parent) = Path::new(path).parent() {
                        let parent_str = parent.to_string_lossy().to_string();
                        if seen.insert(parent_str.clone()) {
                            let name = parent
                                .file_name()
                                .map(|f| f.to_string_lossy().to_string())
                                .unwrap_or_default();
                            let kind = Self::classify_package(&parent_str);
                            packages.push(DiscoveredPackage {
                                path: parent_str,
                                name,
                                kind,
                                manifest_file: path.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    /// Discover JS/TS packages (nested package.json files).
    fn discover_js_packages(
        file_paths: &[String],
        packages: &mut Vec<DiscoveredPackage>,
        seen: &mut HashSet<String>,
    ) {
        for path in file_paths {
            if Path::new(path)
                .file_name()
                .map(|f| f.to_string_lossy() == "package.json")
                .unwrap_or(false)
            {
                let depth = Path::new(path).components().count();
                if depth >= 3 {
                    if let Some(parent) = Path::new(path).parent() {
                        let parent_str = parent.to_string_lossy().to_string();
                        if seen.insert(parent_str.clone()) {
                            let name = parent
                                .file_name()
                                .map(|f| f.to_string_lossy().to_string())
                                .unwrap_or_default();
                            let kind = Self::classify_package(&parent_str);
                            packages.push(DiscoveredPackage {
                                path: parent_str,
                                name,
                                kind,
                                manifest_file: path.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    /// Discover Go workspace modules (nested go.mod files).
    fn discover_go_packages(
        file_paths: &[String],
        packages: &mut Vec<DiscoveredPackage>,
        seen: &mut HashSet<String>,
    ) {
        for path in file_paths {
            if Path::new(path)
                .file_name()
                .map(|f| f.to_string_lossy() == "go.mod")
                .unwrap_or(false)
            {
                let depth = Path::new(path).components().count();
                if depth >= 2 {
                    if let Some(parent) = Path::new(path).parent() {
                        let parent_str = parent.to_string_lossy().to_string();
                        if !parent_str.is_empty() && seen.insert(parent_str.clone()) {
                            let name = parent
                                .file_name()
                                .map(|f| f.to_string_lossy().to_string())
                                .unwrap_or_default();
                            let kind = Self::classify_package(&parent_str);
                            packages.push(DiscoveredPackage {
                                path: parent_str,
                                name,
                                kind,
                                manifest_file: path.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    /// Discover .NET projects within a solution.
    fn discover_dotnet_packages(
        file_paths: &[String],
        packages: &mut Vec<DiscoveredPackage>,
        seen: &mut HashSet<String>,
    ) {
        for path in file_paths {
            let ext = Path::new(path)
                .extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_default();
            if ext == "csproj" || ext == "fsproj" {
                if let Some(parent) = Path::new(path).parent() {
                    let parent_str = parent.to_string_lossy().to_string();
                    if !parent_str.is_empty() && seen.insert(parent_str.clone()) {
                        let name = parent
                            .file_name()
                            .map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let kind = Self::classify_package(&parent_str);
                        packages.push(DiscoveredPackage {
                            path: parent_str,
                            name,
                            kind,
                            manifest_file: path.clone(),
                        });
                    }
                }
            }
        }
    }

    /// Discover packages by structure alone (no tool markers).
    fn discover_packages_by_structure(file_paths: &[String]) -> Vec<DiscoveredPackage> {
        let mut packages = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();

        // Look for nested manifests in well-known package directories
        let manifests = ["Cargo.toml", "package.json", "go.mod"];

        for path in file_paths {
            let file_name = Path::new(path)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default();

            if manifests.contains(&file_name.as_str()) {
                let depth = Path::new(path).components().count();
                if depth >= 3 {
                    if let Some(parent) = Path::new(path).parent() {
                        let parent_str = parent.to_string_lossy().to_string();
                        if seen.insert(parent_str.clone()) {
                            let name = parent
                                .file_name()
                                .map(|f| f.to_string_lossy().to_string())
                                .unwrap_or_default();
                            let kind = Self::classify_package(&parent_str);
                            packages.push(DiscoveredPackage {
                                path: parent_str,
                                name,
                                kind,
                                manifest_file: path.clone(),
                            });
                        }
                    }
                }
            }
        }

        packages
    }

    /// Classify a package as app, library, or unknown based on its path.
    fn classify_package(path: &str) -> PackageKind {
        let components: Vec<&str> = Path::new(path)
            .components()
            .map(|c| c.as_os_str().to_str().unwrap_or(""))
            .collect();

        // Check if any path component suggests app or library
        for component in &components {
            let lower = component.to_lowercase();
            if APP_DIR_NAMES.iter().any(|d| lower == *d) {
                return PackageKind::App;
            }
            if LIB_DIR_NAMES.iter().any(|d| lower == *d) {
                return PackageKind::Library;
            }
        }

        // Check the parent directory name
        if components.len() >= 2 {
            let parent = components[0].to_lowercase();
            if APP_DIR_NAMES.iter().any(|d| parent == *d) {
                return PackageKind::App;
            }
            if LIB_DIR_NAMES.iter().any(|d| parent == *d) {
                return PackageKind::Library;
            }
        }

        PackageKind::Unknown
    }

    /// Find common package directory prefixes.
    fn find_package_dirs(packages: &[DiscoveredPackage]) -> Vec<String> {
        let mut dirs: HashSet<String> = HashSet::new();

        for pkg in packages {
            if let Some(parent) = Path::new(&pkg.path).parent() {
                let parent_str = parent.to_string_lossy().to_string();
                if !parent_str.is_empty() {
                    dirs.insert(parent_str);
                }
            }
        }

        let mut result: Vec<String> = dirs.into_iter().collect();
        result.sort();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cargo_workspace_paths() -> Vec<String> {
        vec![
            "Cargo.toml".to_string(),
            "Cargo.lock".to_string(),
            "crates/core/Cargo.toml".to_string(),
            "crates/core/src/lib.rs".to_string(),
            "crates/cli/Cargo.toml".to_string(),
            "crates/cli/src/main.rs".to_string(),
            "crates/utils/Cargo.toml".to_string(),
            "crates/utils/src/lib.rs".to_string(),
        ]
    }

    fn turborepo_paths() -> Vec<String> {
        vec![
            "package.json".to_string(),
            "pnpm-workspace.yaml".to_string(),
            "pnpm-lock.yaml".to_string(),
            "turbo.json".to_string(),
            "apps/web/package.json".to_string(),
            "apps/web/src/index.ts".to_string(),
            "apps/api/package.json".to_string(),
            "apps/api/src/index.ts".to_string(),
            "packages/ui/package.json".to_string(),
            "packages/ui/src/index.ts".to_string(),
            "packages/config/package.json".to_string(),
            "packages/config/src/index.ts".to_string(),
        ]
    }

    fn nx_paths() -> Vec<String> {
        vec![
            "package.json".to_string(),
            "nx.json".to_string(),
            "yarn.lock".to_string(),
            "apps/frontend/package.json".to_string(),
            "apps/frontend/src/main.tsx".to_string(),
            "libs/shared/package.json".to_string(),
            "libs/shared/src/index.ts".to_string(),
        ]
    }

    fn single_project_paths() -> Vec<String> {
        vec![
            "Cargo.toml".to_string(),
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "tests/test.rs".to_string(),
        ]
    }

    fn go_workspace_paths() -> Vec<String> {
        vec![
            "go.work".to_string(),
            "services/auth/go.mod".to_string(),
            "services/auth/main.go".to_string(),
            "libs/common/go.mod".to_string(),
            "libs/common/utils.go".to_string(),
        ]
    }

    #[test]
    fn test_detect_cargo_workspace() {
        let info = MonorepoDetector::detect(&cargo_workspace_paths());

        assert!(info.is_monorepo);
        assert!(info.tools.contains(&MonorepoTool::CargoWorkspace));
        assert_eq!(info.packages.len(), 3);
        assert!(info.packages.iter().any(|p| p.name == "core"));
        assert!(info.packages.iter().any(|p| p.name == "cli"));
        assert!(info.packages.iter().any(|p| p.name == "utils"));
    }

    #[test]
    fn test_detect_turborepo() {
        let info = MonorepoDetector::detect(&turborepo_paths());

        assert!(info.is_monorepo);
        assert!(info.tools.contains(&MonorepoTool::Turborepo));
        assert!(info.tools.contains(&MonorepoTool::PnpmWorkspace));
        assert_eq!(info.packages.len(), 4);
    }

    #[test]
    fn test_detect_nx() {
        let info = MonorepoDetector::detect(&nx_paths());

        assert!(info.is_monorepo);
        assert!(info.tools.contains(&MonorepoTool::Nx));
    }

    #[test]
    fn test_single_project_not_monorepo() {
        let info = MonorepoDetector::detect(&single_project_paths());

        assert!(!info.is_monorepo);
        assert!(info.tools.is_empty());
        assert!(info.packages.is_empty());
    }

    #[test]
    fn test_detect_go_workspace() {
        let info = MonorepoDetector::detect(&go_workspace_paths());

        assert!(info.is_monorepo);
        assert!(info.tools.contains(&MonorepoTool::GoWorkspace));
        assert_eq!(info.packages.len(), 2);
    }

    #[test]
    fn test_package_classification_apps() {
        let info = MonorepoDetector::detect(&turborepo_paths());

        let web = info.packages.iter().find(|p| p.name == "web").unwrap();
        assert_eq!(web.kind, PackageKind::App);

        let api = info.packages.iter().find(|p| p.name == "api").unwrap();
        assert_eq!(api.kind, PackageKind::App);
    }

    #[test]
    fn test_package_classification_libs() {
        let info = MonorepoDetector::detect(&turborepo_paths());

        let ui = info.packages.iter().find(|p| p.name == "ui").unwrap();
        assert_eq!(ui.kind, PackageKind::Library);
    }

    #[test]
    fn test_package_dirs() {
        let info = MonorepoDetector::detect(&turborepo_paths());

        assert!(info.package_dirs.contains(&"apps".to_string()));
        assert!(info.package_dirs.contains(&"packages".to_string()));
    }

    #[test]
    fn test_cargo_package_classification() {
        let info = MonorepoDetector::detect(&cargo_workspace_paths());

        // "crates/utils" should classify as library due to "utils" name
        let utils = info.packages.iter().find(|p| p.name == "utils").unwrap();
        assert_eq!(utils.kind, PackageKind::Library);
    }

    #[test]
    fn test_apps_and_libraries_helpers() {
        let info = MonorepoDetector::detect(&turborepo_paths());

        let apps = info.apps();
        assert!(apps.len() >= 2); // web + api

        let libs = info.libraries();
        assert!(!libs.is_empty()); // ui, config
    }

    #[test]
    fn test_not_monorepo_result() {
        let info = MonorepoInfo::not_monorepo();
        assert!(!info.is_monorepo);
        assert!(info.tools.is_empty());
        assert!(info.packages.is_empty());
        assert!(info.package_dirs.is_empty());
    }

    #[test]
    fn test_empty_paths() {
        let info = MonorepoDetector::detect(&[]);
        assert!(!info.is_monorepo);
    }

    #[test]
    fn test_display_impls() {
        assert_eq!(MonorepoTool::CargoWorkspace.to_string(), "cargo-workspace");
        assert_eq!(MonorepoTool::Turborepo.to_string(), "turborepo");
        assert_eq!(MonorepoTool::Nx.to_string(), "nx");
        assert_eq!(PackageKind::App.to_string(), "app");
        assert_eq!(PackageKind::Library.to_string(), "library");
    }

    #[test]
    fn test_dotnet_solution() {
        let paths = vec![
            "MySolution.sln".to_string(),
            "src/WebApi/WebApi.csproj".to_string(),
            "src/WebApi/Program.cs".to_string(),
            "src/Core/Core.csproj".to_string(),
            "src/Core/Service.cs".to_string(),
            "tests/Tests/Tests.csproj".to_string(),
        ];
        let info = MonorepoDetector::detect(&paths);

        assert!(info.is_monorepo);
        assert!(info.tools.contains(&MonorepoTool::DotNetSolution));
        assert!(info.packages.len() >= 2);
    }

    #[test]
    fn test_lerna_detection() {
        let paths = vec![
            "package.json".to_string(),
            "lerna.json".to_string(),
            "packages/core/package.json".to_string(),
            "packages/core/src/index.ts".to_string(),
            "packages/utils/package.json".to_string(),
            "packages/utils/src/index.ts".to_string(),
        ];
        let info = MonorepoDetector::detect(&paths);

        assert!(info.is_monorepo);
        assert!(info.tools.contains(&MonorepoTool::Lerna));
    }

    #[test]
    fn test_structural_monorepo_without_tool() {
        // Multiple nested manifests without any explicit tool marker
        let paths = vec![
            "packages/core/package.json".to_string(),
            "packages/core/src/index.ts".to_string(),
            "packages/utils/package.json".to_string(),
            "packages/utils/src/index.ts".to_string(),
        ];
        let info = MonorepoDetector::detect(&paths);

        assert!(info.is_monorepo);
        assert_eq!(info.packages.len(), 2);
    }
}

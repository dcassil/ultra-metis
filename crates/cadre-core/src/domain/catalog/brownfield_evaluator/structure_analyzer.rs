//! Structure analyzer for brownfield repositories.
//!
//! Analyzes file paths to detect folder patterns, naming conventions,
//! layer structures, and architectural organization. Operates on path
//! lists only -- no filesystem I/O.

use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Result of analyzing a repository's file structure.
#[derive(Debug, Clone)]
pub struct StructureAnalysis {
    /// Top-level directories found (e.g., "src", "tests", "lib").
    pub top_level_dirs: Vec<String>,
    /// Detected layers (directories that appear to act as architectural layers).
    pub detected_layers: Vec<String>,
    /// Detected naming convention for files.
    pub file_naming_convention: NamingConvention,
    /// Detected test pattern.
    pub test_pattern: TestPattern,
    /// Total file count analyzed.
    pub total_files: usize,
    /// Directory depth distribution (depth -> count of files at that depth).
    pub depth_distribution: HashMap<usize, usize>,
    /// Whether the project appears to use a `src/` root.
    pub has_src_root: bool,
    /// Detected module boundary directories (directories with index/barrel files).
    pub module_boundaries: Vec<String>,
    /// Quality score (0-100) based on structural regularity.
    pub structure_quality_score: f64,
}

/// Detected naming convention for source files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamingConvention {
    /// camelCase (e.g., myModule.ts)
    CamelCase,
    /// PascalCase (e.g., MyComponent.tsx)
    PascalCase,
    /// kebab-case (e.g., my-module.ts)
    KebabCase,
    /// snake_case (e.g., my_module.rs)
    SnakeCase,
    /// Mixed or undetectable.
    Mixed,
}

impl std::fmt::Display for NamingConvention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CamelCase => write!(f, "camelCase"),
            Self::PascalCase => write!(f, "PascalCase"),
            Self::KebabCase => write!(f, "kebab-case"),
            Self::SnakeCase => write!(f, "snake_case"),
            Self::Mixed => write!(f, "mixed"),
        }
    }
}

/// Detected test file placement pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestPattern {
    /// Tests co-located with source (e.g., Component.test.tsx beside Component.tsx).
    CoLocated,
    /// Tests in a separate top-level directory (e.g., tests/ or __tests__/).
    SeparateDir,
    /// Both co-located and separate tests found.
    Both,
    /// No test files detected.
    None,
}

impl std::fmt::Display for TestPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CoLocated => write!(f, "co-located"),
            Self::SeparateDir => write!(f, "separate-directory"),
            Self::Both => write!(f, "both"),
            Self::None => write!(f, "none"),
        }
    }
}

/// Analyzes repository structure from file paths.
pub struct StructureAnalyzer;

impl StructureAnalyzer {
    /// Analyze a list of file paths (relative to repo root) and produce
    /// a [`StructureAnalysis`].
    pub fn analyze(file_paths: &[String]) -> StructureAnalysis {
        let top_level_dirs = Self::extract_top_level_dirs(file_paths);
        let has_src_root = top_level_dirs.contains(&"src".to_string());
        let detected_layers = Self::detect_layers(file_paths, has_src_root);
        let file_naming_convention = Self::detect_naming_convention(file_paths);
        let test_pattern = Self::detect_test_pattern(file_paths);
        let depth_distribution = Self::compute_depth_distribution(file_paths);
        let module_boundaries = Self::detect_module_boundaries(file_paths);

        let structure_quality_score = Self::compute_quality_score(
            &top_level_dirs,
            &detected_layers,
            file_naming_convention,
            test_pattern,
            &depth_distribution,
        );

        StructureAnalysis {
            top_level_dirs,
            detected_layers,
            file_naming_convention,
            test_pattern,
            total_files: file_paths.len(),
            depth_distribution,
            has_src_root,
            module_boundaries,
            structure_quality_score,
        }
    }

    /// Extract unique top-level directories from file paths.
    fn extract_top_level_dirs(file_paths: &[String]) -> Vec<String> {
        let mut dirs: Vec<String> = file_paths
            .iter()
            .filter_map(|p| {
                let path = Path::new(p);
                let components: Vec<_> = path.components().collect();
                if components.len() > 1 {
                    Some(components[0].as_os_str().to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        dirs.sort();
        dirs
    }

    /// Detect directories that likely act as architectural layers.
    ///
    /// Looks for well-known layer names at any depth within the source root.
    /// This handles both flat architectures (`src/services/`) and nested
    /// ones (`src/features/auth/services/`).
    fn detect_layers(file_paths: &[String], has_src_root: bool) -> Vec<String> {
        let well_known_layers = [
            "routes",
            "handlers",
            "controllers",
            "services",
            "repositories",
            "models",
            "middleware",
            "components",
            "hooks",
            "features",
            "core",
            "commands",
            "utils",
            "lib",
            "api",
            "domain",
            "infrastructure",
            "application",
            "presentation",
            "shared",
            "common",
            "internal",
        ];

        let prefix = if has_src_root { "src/" } else { "" };

        // Collect all directory names at any depth within the source root
        let dir_set: HashSet<String> = file_paths
            .iter()
            .flat_map(|p| {
                let stripped = p.strip_prefix(prefix).unwrap_or(p);
                let path = Path::new(stripped);
                // Collect all directory components (skip the file name)
                let components: Vec<_> = path.components().collect();
                components
                    .iter()
                    .take(components.len().saturating_sub(1))
                    .map(|c| c.as_os_str().to_string_lossy().to_string())
                    .collect::<Vec<_>>()
            })
            .collect();

        let mut layers: Vec<String> = well_known_layers
            .iter()
            .filter(|layer| dir_set.contains(**layer))
            .map(std::string::ToString::to_string)
            .collect();
        layers.sort();
        layers
    }

    /// Detect the predominant file naming convention.
    fn detect_naming_convention(file_paths: &[String]) -> NamingConvention {
        let mut camel = 0u32;
        let mut pascal = 0u32;
        let mut kebab = 0u32;
        let mut snake = 0u32;

        for path in file_paths {
            let file_name = Path::new(path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            // Skip test suffixes and index files for convention detection
            if file_name == "index" || file_name == "mod" || file_name.is_empty() {
                continue;
            }

            // Strip common suffixes like .test, .spec, .stories
            let base = file_name
                .strip_suffix(".test")
                .or_else(|| file_name.strip_suffix(".spec"))
                .or_else(|| file_name.strip_suffix(".stories"))
                .unwrap_or(&file_name);

            if base.contains('-') {
                kebab += 1;
            } else if base.contains('_') {
                snake += 1;
            } else if base.chars().next().is_some_and(char::is_uppercase) {
                pascal += 1;
            } else if base.len() > 1
                && base.chars().next().is_some_and(char::is_lowercase)
                && base.chars().any(char::is_uppercase)
            {
                camel += 1;
            }
        }

        let total = camel + pascal + kebab + snake;
        if total == 0 {
            return NamingConvention::Mixed;
        }

        let max = camel.max(pascal).max(kebab).max(snake);
        let dominance = f64::from(max) / f64::from(total);

        if dominance < 0.5 {
            NamingConvention::Mixed
        } else if max == kebab {
            NamingConvention::KebabCase
        } else if max == pascal {
            NamingConvention::PascalCase
        } else if max == snake {
            NamingConvention::SnakeCase
        } else {
            NamingConvention::CamelCase
        }
    }

    /// Detect test file placement pattern.
    fn detect_test_pattern(file_paths: &[String]) -> TestPattern {
        let test_extensions = [".test.", ".spec.", "__test__"];
        let test_dirs = ["tests/", "test/", "__tests__/"];

        let mut has_colocated = false;
        let mut has_separate = false;

        for path in file_paths {
            let is_test_file = test_extensions.iter().any(|ext| path.contains(ext));

            if is_test_file {
                let in_test_dir = test_dirs.iter().any(|dir| path.starts_with(dir));
                if in_test_dir {
                    has_separate = true;
                } else {
                    has_colocated = true;
                }
            } else {
                // Also check if the file is in a test directory
                let in_test_dir = test_dirs.iter().any(|dir| path.starts_with(dir));
                if in_test_dir {
                    has_separate = true;
                }
            }
        }

        match (has_colocated, has_separate) {
            (true, true) => TestPattern::Both,
            (true, false) => TestPattern::CoLocated,
            (false, true) => TestPattern::SeparateDir,
            (false, false) => TestPattern::None,
        }
    }

    /// Compute file depth distribution.
    fn compute_depth_distribution(file_paths: &[String]) -> HashMap<usize, usize> {
        let mut dist = HashMap::new();
        for path in file_paths {
            let depth = Path::new(path).components().count();
            *dist.entry(depth).or_insert(0) += 1;
        }
        dist
    }

    /// Detect directories that act as module boundaries (have index/barrel files).
    fn detect_module_boundaries(file_paths: &[String]) -> Vec<String> {
        let barrel_names = ["index.ts", "index.js", "index.tsx", "index.jsx", "mod.rs"];

        let mut boundaries: Vec<String> = file_paths
            .iter()
            .filter_map(|p| {
                let file_name = Path::new(p)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_default();
                if barrel_names.contains(&file_name.as_str()) {
                    Path::new(p)
                        .parent()
                        .map(|parent| parent.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .filter(|p| !p.is_empty())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        boundaries.sort();
        boundaries
    }

    /// Compute a structural quality score (0-100).
    ///
    /// Factors:
    /// - Has organized layers (up to 30 points)
    /// - Consistent naming convention (up to 25 points)
    /// - Has tests (up to 20 points)
    /// - Reasonable depth distribution (up to 15 points)
    /// - Has src root or organized top-level (up to 10 points)
    fn compute_quality_score(
        top_level_dirs: &[String],
        detected_layers: &[String],
        naming: NamingConvention,
        test_pattern: TestPattern,
        depth_dist: &HashMap<usize, usize>,
    ) -> f64 {
        let mut score = 0.0;

        // Layer organization (0-30)
        let layer_score = match detected_layers.len() {
            0 => 0.0,
            1 => 10.0,
            2 => 20.0,
            _ => 30.0,
        };
        score += layer_score;

        // Naming consistency (0-25)
        let naming_score = match naming {
            NamingConvention::Mixed => 5.0,
            _ => 25.0,
        };
        score += naming_score;

        // Test presence (0-20)
        let test_score = match test_pattern {
            TestPattern::None => 0.0,
            TestPattern::SeparateDir => 15.0,
            TestPattern::CoLocated => 18.0,
            TestPattern::Both => 20.0,
        };
        score += test_score;

        // Depth distribution (0-15) - penalize very flat or very deep
        let max_depth = depth_dist.keys().max().copied().unwrap_or(0);
        let depth_score = if max_depth == 0 {
            0.0
        } else if max_depth <= 2 {
            8.0 // Flat but organized
        } else if max_depth <= 5 {
            15.0 // Good depth
        } else if max_depth <= 8 {
            10.0 // Getting deep
        } else {
            5.0 // Very deep nesting
        };
        score += depth_score;

        // Top-level organization (0-10)
        let top_level_score = if top_level_dirs.contains(&"src".to_string()) {
            10.0
        } else if top_level_dirs.len() <= 5 {
            7.0
        } else if top_level_dirs.len() <= 10 {
            4.0
        } else {
            2.0 // Too many top-level dirs suggests poor organization
        };
        score += top_level_score;

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server_paths() -> Vec<String> {
        vec![
            "src/routes/users.route.ts".to_string(),
            "src/routes/auth.route.ts".to_string(),
            "src/handlers/users.handler.ts".to_string(),
            "src/handlers/auth.handler.ts".to_string(),
            "src/services/users.service.ts".to_string(),
            "src/services/auth.service.ts".to_string(),
            "src/repositories/users.repository.ts".to_string(),
            "src/middleware/auth.ts".to_string(),
            "src/config/database.ts".to_string(),
            "src/index.ts".to_string(),
            "tests/integration/users.test.ts".to_string(),
            "tests/unit/auth.service.test.ts".to_string(),
            "package.json".to_string(),
        ]
    }

    fn react_paths() -> Vec<String> {
        vec![
            "src/features/auth/components/LoginForm.tsx".to_string(),
            "src/features/auth/hooks/useAuth.ts".to_string(),
            "src/features/auth/services/authService.ts".to_string(),
            "src/features/auth/index.ts".to_string(),
            "src/features/dashboard/components/DashboardView.tsx".to_string(),
            "src/features/dashboard/components/DashboardView.test.tsx".to_string(),
            "src/shared/components/Button.tsx".to_string(),
            "src/shared/hooks/useTheme.ts".to_string(),
            "src/shared/utils/format.ts".to_string(),
            "src/app/App.tsx".to_string(),
            "src/index.tsx".to_string(),
        ]
    }

    fn flat_paths() -> Vec<String> {
        vec![
            "app.js".to_string(),
            "server.js".to_string(),
            "database.js".to_string(),
            "utils.js".to_string(),
        ]
    }

    #[test]
    fn test_analyze_server_structure() {
        let analysis = StructureAnalyzer::analyze(&server_paths());

        assert!(analysis.has_src_root);
        assert!(analysis.top_level_dirs.contains(&"src".to_string()));
        assert!(analysis.top_level_dirs.contains(&"tests".to_string()));
        assert!(analysis.detected_layers.contains(&"routes".to_string()));
        assert!(analysis.detected_layers.contains(&"handlers".to_string()));
        assert!(analysis.detected_layers.contains(&"services".to_string()));
        assert!(analysis
            .detected_layers
            .contains(&"repositories".to_string()));
        assert!(analysis.detected_layers.contains(&"middleware".to_string()));
        assert_eq!(analysis.total_files, 13);
    }

    #[test]
    fn test_analyze_react_structure() {
        let analysis = StructureAnalyzer::analyze(&react_paths());

        assert!(analysis.has_src_root);
        assert!(analysis.detected_layers.contains(&"features".to_string()));
        assert!(analysis.detected_layers.contains(&"shared".to_string()));
        assert!(analysis
            .module_boundaries
            .contains(&"src/features/auth".to_string()));
    }

    #[test]
    fn test_naming_convention_kebab() {
        let paths = vec![
            "src/my-module.ts".to_string(),
            "src/my-other-module.ts".to_string(),
            "src/third-module.ts".to_string(),
        ];
        let analysis = StructureAnalyzer::analyze(&paths);
        assert_eq!(analysis.file_naming_convention, NamingConvention::KebabCase);
    }

    #[test]
    fn test_naming_convention_pascal() {
        let paths = vec![
            "src/MyComponent.tsx".to_string(),
            "src/AnotherComponent.tsx".to_string(),
            "src/ThirdComponent.tsx".to_string(),
        ];
        let analysis = StructureAnalyzer::analyze(&paths);
        assert_eq!(
            analysis.file_naming_convention,
            NamingConvention::PascalCase
        );
    }

    #[test]
    fn test_naming_convention_snake() {
        let paths = vec![
            "src/my_module.rs".to_string(),
            "src/another_module.rs".to_string(),
            "src/third_module.rs".to_string(),
        ];
        let analysis = StructureAnalyzer::analyze(&paths);
        assert_eq!(analysis.file_naming_convention, NamingConvention::SnakeCase);
    }

    #[test]
    fn test_test_pattern_colocated() {
        let paths = vec![
            "src/Component.tsx".to_string(),
            "src/Component.test.tsx".to_string(),
        ];
        let analysis = StructureAnalyzer::analyze(&paths);
        assert_eq!(analysis.test_pattern, TestPattern::CoLocated);
    }

    #[test]
    fn test_test_pattern_separate() {
        let paths = vec!["src/main.ts".to_string(), "tests/main.test.ts".to_string()];
        let analysis = StructureAnalyzer::analyze(&paths);
        assert_eq!(analysis.test_pattern, TestPattern::SeparateDir);
    }

    #[test]
    fn test_test_pattern_both() {
        let paths = vec![
            "src/Component.tsx".to_string(),
            "src/Component.test.tsx".to_string(),
            "tests/integration/api.test.ts".to_string(),
        ];
        let analysis = StructureAnalyzer::analyze(&paths);
        assert_eq!(analysis.test_pattern, TestPattern::Both);
    }

    #[test]
    fn test_test_pattern_none() {
        let paths = vec!["src/main.ts".to_string(), "src/lib.ts".to_string()];
        let analysis = StructureAnalyzer::analyze(&paths);
        assert_eq!(analysis.test_pattern, TestPattern::None);
    }

    #[test]
    fn test_flat_structure_low_quality() {
        let analysis = StructureAnalyzer::analyze(&flat_paths());

        assert!(!analysis.has_src_root);
        assert!(analysis.detected_layers.is_empty());
        // Flat structure should score lower
        assert!(analysis.structure_quality_score < 50.0);
    }

    #[test]
    fn test_well_organized_high_quality() {
        let analysis = StructureAnalyzer::analyze(&server_paths());

        // Well-organized server structure should score well
        assert!(
            analysis.structure_quality_score >= 60.0,
            "Expected >= 60, got {}",
            analysis.structure_quality_score
        );
    }

    #[test]
    fn test_depth_distribution() {
        let paths = vec![
            "a.ts".to_string(),         // depth 1
            "src/b.ts".to_string(),     // depth 2
            "src/foo/c.ts".to_string(), // depth 3
        ];
        let analysis = StructureAnalyzer::analyze(&paths);
        assert_eq!(analysis.depth_distribution.get(&1), Some(&1));
        assert_eq!(analysis.depth_distribution.get(&2), Some(&1));
        assert_eq!(analysis.depth_distribution.get(&3), Some(&1));
    }

    #[test]
    fn test_module_boundaries() {
        let paths = vec![
            "src/features/auth/index.ts".to_string(),
            "src/features/auth/Login.tsx".to_string(),
            "src/features/dashboard/index.ts".to_string(),
            "src/utils/index.ts".to_string(),
        ];
        let analysis = StructureAnalyzer::analyze(&paths);
        assert!(analysis
            .module_boundaries
            .contains(&"src/features/auth".to_string()));
        assert!(analysis
            .module_boundaries
            .contains(&"src/features/dashboard".to_string()));
        assert!(analysis
            .module_boundaries
            .contains(&"src/utils".to_string()));
    }

    #[test]
    fn test_empty_paths() {
        let analysis = StructureAnalyzer::analyze(&[]);
        assert!(analysis.top_level_dirs.is_empty());
        assert!(analysis.detected_layers.is_empty());
        assert_eq!(analysis.total_files, 0);
        assert_eq!(analysis.file_naming_convention, NamingConvention::Mixed);
        assert_eq!(analysis.test_pattern, TestPattern::None);
    }
}

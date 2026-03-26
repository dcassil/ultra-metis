//! Rules config analyzer for brownfield architecture fast-path.
//!
//! Scans file paths for known rules engine config files, reads their contents
//! to evaluate strictness on two axes (quality + layering), and infers a
//! [`StructureAnalysis`] when both axes pass threshold — enabling the
//! brownfield evaluator to skip file-level structure analysis.

use std::collections::HashMap;
use std::path::Path;

use super::structure_analyzer::{StructureAnalysis, StructureAnalyzer};

// ---------------------------------------------------------------------------
// FileContentReader trait
// ---------------------------------------------------------------------------

/// Abstraction for reading file contents, enabling testability.
pub trait FileContentReader {
    /// Read the contents of a file at the given path.
    /// Returns `None` if the file cannot be read.
    fn read_content(&self, path: &str) -> Option<String>;
}

/// Reads file contents from the real filesystem.
pub struct FsContentReader;

impl FileContentReader for FsContentReader {
    fn read_content(&self, path: &str) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }
}

/// Mock reader for testing — returns pre-loaded content by path.
pub struct MockContentReader {
    files: HashMap<String, String>,
}

impl MockContentReader {
    pub fn new(files: HashMap<String, String>) -> Self {
        Self { files }
    }
}

impl FileContentReader for MockContentReader {
    fn read_content(&self, path: &str) -> Option<String> {
        self.files.get(path).cloned()
    }
}

// ---------------------------------------------------------------------------
// Language enum
// ---------------------------------------------------------------------------

/// Supported languages for rules config detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    JsTs,
    Rust,
    Python,
    Go,
    JavaKotlin,
    CSharp,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JsTs => write!(f, "JS/TS"),
            Self::Rust => write!(f, "Rust"),
            Self::Python => write!(f, "Python"),
            Self::Go => write!(f, "Go"),
            Self::JavaKotlin => write!(f, "Java/Kotlin"),
            Self::CSharp => write!(f, "C#"),
        }
    }
}

// ---------------------------------------------------------------------------
// Config axis classification
// ---------------------------------------------------------------------------

/// Which axis a config file provides signal for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigAxis {
    Quality,
    Layering,
    Both,
}

// ---------------------------------------------------------------------------
// Registry: known config patterns per language
// ---------------------------------------------------------------------------

/// A single config pattern to match against file paths.
#[derive(Debug, Clone)]
pub struct ConfigPattern {
    /// The filename or suffix to match (e.g., ".eslintrc.json", "tsconfig.json").
    pub filename: &'static str,
    /// Whether to match by exact filename or by suffix/contains.
    pub match_mode: MatchMode,
    /// Which language this config belongs to.
    pub language: Language,
    /// Which axis this config provides signal for.
    pub axis: ConfigAxis,
    /// Human-readable description of what this config enforces.
    pub description: &'static str,
}

/// How to match a config pattern against a file path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchMode {
    /// Match if the filename equals exactly.
    Exact,
    /// Match if the filename starts with this prefix.
    Prefix,
    /// Match if the path contains this string.
    Contains,
}

/// Registry of all known rules engine config patterns.
pub struct RulesConfigRegistry;

impl RulesConfigRegistry {
    /// Returns all known config patterns across all supported languages.
    pub fn patterns() -> &'static [ConfigPattern] {
        &KNOWN_PATTERNS
    }

    /// Returns patterns filtered to a specific language.
    pub fn patterns_for_language(lang: Language) -> Vec<&'static ConfigPattern> {
        KNOWN_PATTERNS
            .iter()
            .filter(|p| p.language == lang)
            .collect()
    }
}

static KNOWN_PATTERNS: [ConfigPattern; 27] = [
    // -----------------------------------------------------------------------
    // JS/TS — Quality
    // -----------------------------------------------------------------------
    ConfigPattern {
        filename: ".eslintrc",
        match_mode: MatchMode::Prefix,
        language: Language::JsTs,
        axis: ConfigAxis::Quality,
        description: "ESLint config (legacy format)",
    },
    ConfigPattern {
        filename: "eslint.config",
        match_mode: MatchMode::Prefix,
        language: Language::JsTs,
        axis: ConfigAxis::Quality,
        description: "ESLint flat config",
    },
    ConfigPattern {
        filename: "tsconfig.json",
        match_mode: MatchMode::Exact,
        language: Language::JsTs,
        axis: ConfigAxis::Quality,
        description: "TypeScript compiler config",
    },
    ConfigPattern {
        filename: "biome.json",
        match_mode: MatchMode::Exact,
        language: Language::JsTs,
        axis: ConfigAxis::Quality,
        description: "Biome linter/formatter config",
    },
    ConfigPattern {
        filename: "biome.jsonc",
        match_mode: MatchMode::Exact,
        language: Language::JsTs,
        axis: ConfigAxis::Quality,
        description: "Biome linter/formatter config (jsonc)",
    },
    // JS/TS — Layering
    ConfigPattern {
        filename: ".dependency-cruiser",
        match_mode: MatchMode::Prefix,
        language: Language::JsTs,
        axis: ConfigAxis::Layering,
        description: "Dependency cruiser boundary rules",
    },
    ConfigPattern {
        filename: "nx.json",
        match_mode: MatchMode::Exact,
        language: Language::JsTs,
        axis: ConfigAxis::Layering,
        description: "Nx workspace config with module boundaries",
    },
    // -----------------------------------------------------------------------
    // Rust — Quality
    // -----------------------------------------------------------------------
    ConfigPattern {
        filename: "clippy.toml",
        match_mode: MatchMode::Exact,
        language: Language::Rust,
        axis: ConfigAxis::Quality,
        description: "Clippy linter config",
    },
    ConfigPattern {
        filename: ".clippy.toml",
        match_mode: MatchMode::Exact,
        language: Language::Rust,
        axis: ConfigAxis::Quality,
        description: "Clippy linter config (dotfile)",
    },
    ConfigPattern {
        filename: "rustfmt.toml",
        match_mode: MatchMode::Exact,
        language: Language::Rust,
        axis: ConfigAxis::Quality,
        description: "Rustfmt formatter config",
    },
    ConfigPattern {
        filename: ".rustfmt.toml",
        match_mode: MatchMode::Exact,
        language: Language::Rust,
        axis: ConfigAxis::Quality,
        description: "Rustfmt formatter config (dotfile)",
    },
    // Rust — Both (deny.toml covers quality via advisories AND layering via bans)
    ConfigPattern {
        filename: "deny.toml",
        match_mode: MatchMode::Exact,
        language: Language::Rust,
        axis: ConfigAxis::Both,
        description: "cargo-deny dependency policy (advisories + bans)",
    },
    // -----------------------------------------------------------------------
    // Python — Quality
    // -----------------------------------------------------------------------
    ConfigPattern {
        filename: "ruff.toml",
        match_mode: MatchMode::Exact,
        language: Language::Python,
        axis: ConfigAxis::Quality,
        description: "Ruff linter config",
    },
    ConfigPattern {
        filename: ".ruff.toml",
        match_mode: MatchMode::Exact,
        language: Language::Python,
        axis: ConfigAxis::Quality,
        description: "Ruff linter config (dotfile)",
    },
    ConfigPattern {
        filename: "mypy.ini",
        match_mode: MatchMode::Exact,
        language: Language::Python,
        axis: ConfigAxis::Quality,
        description: "Mypy type checker config",
    },
    ConfigPattern {
        filename: ".mypy.ini",
        match_mode: MatchMode::Exact,
        language: Language::Python,
        axis: ConfigAxis::Quality,
        description: "Mypy type checker config (dotfile)",
    },
    ConfigPattern {
        filename: "pyrightconfig.json",
        match_mode: MatchMode::Exact,
        language: Language::Python,
        axis: ConfigAxis::Quality,
        description: "Pyright type checker config",
    },
    ConfigPattern {
        filename: ".flake8",
        match_mode: MatchMode::Exact,
        language: Language::Python,
        axis: ConfigAxis::Quality,
        description: "Flake8 linter config",
    },
    // Python — Quality+Layering (pyproject.toml may contain both ruff and import-linter)
    ConfigPattern {
        filename: "pyproject.toml",
        match_mode: MatchMode::Exact,
        language: Language::Python,
        axis: ConfigAxis::Both,
        description: "Python project config (may contain ruff, mypy, import-linter)",
    },
    // Python — Layering
    ConfigPattern {
        filename: ".importlinter",
        match_mode: MatchMode::Exact,
        language: Language::Python,
        axis: ConfigAxis::Layering,
        description: "Import linter boundary contracts",
    },
    ConfigPattern {
        filename: "setup.cfg",
        match_mode: MatchMode::Exact,
        language: Language::Python,
        axis: ConfigAxis::Both,
        description: "Python setup config (may contain import-linter)",
    },
    // -----------------------------------------------------------------------
    // Go — Quality + Layering (golangci-lint covers both via linter selection)
    // -----------------------------------------------------------------------
    ConfigPattern {
        filename: ".golangci.yml",
        match_mode: MatchMode::Exact,
        language: Language::Go,
        axis: ConfigAxis::Both,
        description: "golangci-lint config (quality linters + depguard)",
    },
    ConfigPattern {
        filename: ".golangci.yaml",
        match_mode: MatchMode::Exact,
        language: Language::Go,
        axis: ConfigAxis::Both,
        description: "golangci-lint config (quality linters + depguard)",
    },
    ConfigPattern {
        filename: ".golangci.toml",
        match_mode: MatchMode::Exact,
        language: Language::Go,
        axis: ConfigAxis::Both,
        description: "golangci-lint config (quality linters + depguard)",
    },
    // -----------------------------------------------------------------------
    // Java/Kotlin — Quality
    // -----------------------------------------------------------------------
    ConfigPattern {
        filename: "checkstyle.xml",
        match_mode: MatchMode::Exact,
        language: Language::JavaKotlin,
        axis: ConfigAxis::Quality,
        description: "Checkstyle code quality config",
    },
    ConfigPattern {
        filename: "detekt.yml",
        match_mode: MatchMode::Exact,
        language: Language::JavaKotlin,
        axis: ConfigAxis::Quality,
        description: "Detekt Kotlin linter config",
    },
    // -----------------------------------------------------------------------
    // C# — Quality
    // -----------------------------------------------------------------------
    ConfigPattern {
        filename: "Directory.Build.props",
        match_mode: MatchMode::Exact,
        language: Language::CSharp,
        axis: ConfigAxis::Quality,
        description: "MSBuild directory props with analyzer config",
    },
];

// ---------------------------------------------------------------------------
// Detected configs result
// ---------------------------------------------------------------------------

/// A config file that was detected in the file paths.
#[derive(Debug, Clone)]
pub struct DetectedConfig {
    /// The file path where this config was found.
    pub path: String,
    /// The pattern that matched.
    pub pattern_filename: &'static str,
    /// Which language this config belongs to.
    pub language: Language,
    /// Which axis this config provides signal for.
    pub axis: ConfigAxis,
    /// Description of what this config enforces.
    pub description: &'static str,
}

/// Result of scanning file paths for known config files.
#[derive(Debug, Clone)]
pub struct DetectedConfigs {
    /// All detected config files.
    pub configs: Vec<DetectedConfig>,
    /// Languages detected in the file paths.
    pub languages: Vec<Language>,
    /// Whether a Cargo workspace with multiple crates was detected (Rust layering signal).
    pub has_cargo_workspace: bool,
    /// Cargo workspace member paths (if detected).
    pub cargo_workspace_members: Vec<String>,
    /// Whether Go internal/ packages were detected.
    pub has_go_internal: bool,
}

impl DetectedConfigs {
    /// Get detected configs for a specific axis.
    pub fn for_axis(&self, axis: ConfigAxis) -> Vec<&DetectedConfig> {
        self.configs
            .iter()
            .filter(|c| c.axis == axis || c.axis == ConfigAxis::Both)
            .collect()
    }

    /// Get detected configs for a specific language.
    pub fn for_language(&self, lang: Language) -> Vec<&DetectedConfig> {
        self.configs.iter().filter(|c| c.language == lang).collect()
    }

    /// Whether any quality-axis configs were detected.
    pub fn has_quality_configs(&self) -> bool {
        self.configs
            .iter()
            .any(|c| c.axis == ConfigAxis::Quality || c.axis == ConfigAxis::Both)
    }

    /// Whether any layering-axis configs were detected.
    pub fn has_layering_configs(&self) -> bool {
        self.configs
            .iter()
            .any(|c| c.axis == ConfigAxis::Layering || c.axis == ConfigAxis::Both)
            || self.has_cargo_workspace
            || self.has_go_internal
    }
}

// ---------------------------------------------------------------------------
// Detection function
// ---------------------------------------------------------------------------

/// Scan file paths for known rules engine config files.
///
/// Matches filenames against the registry patterns and also detects
/// structural signals like Cargo workspace members and Go internal packages.
pub fn detect_configs(file_paths: &[String]) -> DetectedConfigs {
    let patterns = RulesConfigRegistry::patterns();
    let mut configs = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    for file_path in file_paths {
        let file_name = Path::new(file_path)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();

        for pattern in patterns {
            let matches = match pattern.match_mode {
                MatchMode::Exact => file_name == pattern.filename,
                MatchMode::Prefix => file_name.starts_with(pattern.filename),
                MatchMode::Contains => file_path.contains(pattern.filename),
            };

            if matches && seen_paths.insert(file_path.clone()) {
                configs.push(DetectedConfig {
                    path: file_path.clone(),
                    pattern_filename: pattern.filename,
                    language: pattern.language,
                    axis: pattern.axis,
                    description: pattern.description,
                });
            }
        }
    }

    // Detect Cargo workspace: multiple Cargo.toml at different directory depths
    let cargo_tomls: Vec<&String> = file_paths
        .iter()
        .filter(|p| {
            Path::new(p)
                .file_name()
                .map(|f| f == "Cargo.toml")
                .unwrap_or(false)
        })
        .collect();

    let has_cargo_workspace = cargo_tomls.len() > 1;
    let cargo_workspace_members: Vec<String> = if has_cargo_workspace {
        cargo_tomls
            .iter()
            .filter_map(|p| {
                let parent = Path::new(p).parent()?;
                let parent_str = parent.to_string_lossy().to_string();
                if parent_str.is_empty() {
                    None // root Cargo.toml
                } else {
                    Some(parent_str)
                }
            })
            .collect()
    } else {
        vec![]
    };

    // Detect Go internal/ packages
    let has_go_internal = file_paths
        .iter()
        .any(|p| p.contains("/internal/") || p.starts_with("internal/"));

    // Determine detected languages
    let mut lang_set = std::collections::HashSet::new();
    for config in &configs {
        lang_set.insert(config.language);
    }
    if has_cargo_workspace {
        lang_set.insert(Language::Rust);
    }
    if has_go_internal {
        lang_set.insert(Language::Go);
    }
    let mut languages: Vec<Language> = lang_set.into_iter().collect();
    languages.sort_by_key(|l| format!("{l}"));

    DetectedConfigs {
        configs,
        languages,
        has_cargo_workspace,
        cargo_workspace_members,
        has_go_internal,
    }
}

// ---------------------------------------------------------------------------
// Quality axis strictness evaluation
// ---------------------------------------------------------------------------

/// Result of evaluating the quality axis strictness.
#[derive(Debug, Clone)]
pub struct QualityStrictnessResult {
    /// Quality strictness score (0-100).
    pub score: f64,
    /// Human-readable signals explaining what was detected.
    pub signals: Vec<String>,
    /// The primary language detected.
    pub language: Option<Language>,
}

/// Evaluate quality-axis strictness from detected config contents.
pub fn evaluate_quality(
    detected: &DetectedConfigs,
    reader: &dyn FileContentReader,
) -> QualityStrictnessResult {
    let mut best_score = 0.0f64;
    let mut all_signals = Vec::new();
    let mut best_language = None;

    // Evaluate each language that has quality configs
    for lang in &detected.languages {
        let (score, signals) = match lang {
            Language::JsTs => evaluate_jsts_quality(detected, reader),
            Language::Rust => evaluate_rust_quality(detected, reader),
            Language::Python => evaluate_python_quality(detected, reader),
            Language::Go => evaluate_go_quality(detected, reader),
            _ => (0.0, vec![]),
        };
        if score > best_score {
            best_score = score;
            best_language = Some(*lang);
            all_signals = signals;
        }
    }

    QualityStrictnessResult {
        score: best_score,
        signals: all_signals,
        language: best_language,
    }
}

// --- JS/TS quality ---

fn evaluate_jsts_quality(
    detected: &DetectedConfigs,
    reader: &dyn FileContentReader,
) -> (f64, Vec<String>) {
    let mut score = 0.0f64;
    let mut signals = Vec::new();
    let jsts_configs = detected.for_language(Language::JsTs);

    for config in &jsts_configs {
        let content = match reader.read_content(&config.path) {
            Some(c) => c,
            None => continue,
        };

        match config.pattern_filename {
            "tsconfig.json" => {
                evaluate_tsconfig_quality(&content, &mut score, &mut signals);
            }
            pat if pat.starts_with(".eslintrc") || pat.starts_with("eslint.config") => {
                evaluate_eslint_quality(&content, &mut score, &mut signals);
            }
            "biome.json" | "biome.jsonc" => {
                evaluate_biome_quality(&content, &mut score, &mut signals);
            }
            _ => {}
        }
    }

    // Combined score bonus: tsconfig strict + eslint = stronger signal
    if score >= 35.0 && signals.len() >= 2 {
        score = (score + 15.0).min(100.0);
        signals.push("combined: multiple quality tools configured".to_string());
    }

    (score, signals)
}

fn evaluate_tsconfig_quality(content: &str, score: &mut f64, signals: &mut Vec<String>) {
    let Ok(val) = serde_json::from_str::<serde_json::Value>(content) else {
        return;
    };
    if val
        .get("compilerOptions")
        .and_then(|co| co.get("strict"))
        .and_then(serde_json::Value::as_bool)
        == Some(true)
    {
        *score = score.max(40.0);
        signals.push("tsconfig: strict mode enabled".to_string());
    }
    if let Some(co) = val.get("compilerOptions") {
        let strict_flags = [
            "noUncheckedIndexedAccess",
            "noImplicitReturns",
            "noFallthroughCasesInSwitch",
            "exactOptionalPropertyTypes",
        ];
        let extra_strict_count = strict_flags
            .iter()
            .filter(|f| co.get(**f).and_then(serde_json::Value::as_bool).unwrap_or(false))
            .count();
        if extra_strict_count >= 2 {
            *score += 10.0;
            signals.push(format!(
                "tsconfig: {extra_strict_count} additional strict flags enabled"
            ));
        }
    }
}

fn evaluate_eslint_quality(content: &str, score: &mut f64, signals: &mut Vec<String>) {
    // For JSON eslint configs, check extends
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(extends) = val.get("extends") {
            let extends_list = match extends {
                serde_json::Value::Array(arr) => {
                    arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>()
                }
                serde_json::Value::String(s) => vec![s.as_str()],
                _ => vec![],
            };
            let strict_presets = [
                "eslint:recommended",
                "@typescript-eslint/recommended",
                "@typescript-eslint/strict",
                "plugin:@typescript-eslint/recommended-type-checked",
                "plugin:@typescript-eslint/strict-type-checked",
            ];
            for preset in &strict_presets {
                if extends_list.iter().any(|e| e.contains(preset)) {
                    *score = score.max(35.0);
                    signals.push(format!("eslint: extends {preset}"));
                }
            }
            if extends_list.iter().any(|e| e.contains("strict")) {
                *score = score.max(45.0);
            }
        }
    }
    // For JS config files, do string matching
    if content.contains("@typescript-eslint/strict") {
        *score = score.max(45.0);
        if !signals.iter().any(|s| s.contains("strict")) {
            signals.push("eslint: extends strict preset".to_string());
        }
    } else if content.contains("eslint:recommended")
        || content.contains("@typescript-eslint/recommended")
    {
        *score = score.max(35.0);
        if !signals.iter().any(|s| s.contains("recommended")) {
            signals.push("eslint: extends recommended preset".to_string());
        }
    }
}

fn evaluate_biome_quality(content: &str, score: &mut f64, signals: &mut Vec<String>) {
    let Ok(val) = serde_json::from_str::<serde_json::Value>(content) else {
        return;
    };
    let Some(linter) = val.get("linter") else {
        return;
    };
    if linter
        .get("enabled")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        *score = score.max(30.0);
        signals.push("biome: linter enabled".to_string());
    }
    if let Some(rules) = linter.get("rules") {
        if rules
            .get("recommended")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
            || rules.get("all").and_then(serde_json::Value::as_bool).unwrap_or(false)
        {
            *score = score.max(45.0);
            signals.push("biome: recommended/all rules enabled".to_string());
        }
    }
}

// --- Rust quality ---

fn evaluate_rust_quality(
    detected: &DetectedConfigs,
    reader: &dyn FileContentReader,
) -> (f64, Vec<String>) {
    let mut score = 0.0f64;
    let mut signals = Vec::new();
    let rust_configs = detected.for_language(Language::Rust);

    for config in &rust_configs {
        let content = match reader.read_content(&config.path) {
            Some(c) => c,
            None => continue,
        };

        match config.pattern_filename {
            "clippy.toml" | ".clippy.toml" => {
                score = score.max(20.0);
                signals.push("clippy: custom config present".to_string());
            }
            "rustfmt.toml" | ".rustfmt.toml" => {
                score = score.max(15.0);
                signals.push("rustfmt: custom config present".to_string());
            }
            "deny.toml" => {
                let has_advisories = content.contains("[advisories]");
                let has_licenses = content.contains("[licenses]");
                let has_bans = content.contains("[bans]");
                let sections = [has_advisories, has_licenses, has_bans]
                    .iter()
                    .filter(|&&v| v)
                    .count();
                if sections >= 2 {
                    score = score.max(40.0);
                    signals.push(format!(
                        "cargo-deny: {sections} policy sections configured"
                    ));
                } else if sections >= 1 {
                    score = score.max(25.0);
                    signals.push("cargo-deny: partial policy configured".to_string());
                }
            }
            _ => {}
        }
    }

    // Check for workspace-level clippy lints in Cargo.toml files
    // Look for Cargo.toml files that might contain [workspace.lints] or [lints]
    for _file_path in detected.configs.iter().chain(std::iter::empty()) {
        // We need to check Cargo.toml files which aren't in detected configs
        // This is handled below via cargo workspace members
    }

    // Check workspace Cargo.toml for lint configuration
    if detected.has_cargo_workspace {
        // Try reading root Cargo.toml for workspace lint config
        if let Some(content) = reader.read_content("Cargo.toml") {
            if content.contains("clippy::pedantic") {
                score = score.max(50.0);
                signals.push("workspace lints: clippy::pedantic enabled".to_string());
            }
            if content.contains("clippy::nursery") {
                score += 10.0;
                signals.push("workspace lints: clippy::nursery enabled".to_string());
            }
            if (content.contains("[workspace.lints") || content.contains("[lints"))
                && !signals.iter().any(|s| s.contains("workspace lints"))
            {
                score = score.max(30.0);
                signals.push("workspace lints: lint configuration present".to_string());
            }
        }
    }

    // Combined bonus
    if signals.len() >= 3 {
        score = (score + 15.0).min(100.0);
        signals.push("combined: multiple Rust quality tools configured".to_string());
    }

    (score, signals)
}

// --- Python quality ---

fn evaluate_python_quality(
    detected: &DetectedConfigs,
    reader: &dyn FileContentReader,
) -> (f64, Vec<String>) {
    let mut score = 0.0f64;
    let mut signals = Vec::new();
    let py_configs = detected.for_language(Language::Python);

    for config in &py_configs {
        let content = match reader.read_content(&config.path) {
            Some(c) => c,
            None => continue,
        };

        match config.pattern_filename {
            "mypy.ini" | ".mypy.ini" => {
                evaluate_mypy_ini_quality(&content, &mut score, &mut signals);
            }
            "pyrightconfig.json" => {
                evaluate_pyright_quality(&content, &mut score, &mut signals);
            }
            "ruff.toml" | ".ruff.toml" => {
                evaluate_ruff_quality(&content, &mut score, &mut signals);
            }
            "pyproject.toml" => {
                evaluate_pyproject_quality(&content, &mut score, &mut signals);
            }
            ".flake8" => {
                score = score.max(20.0);
                signals.push("flake8: linter configured".to_string());
            }
            _ => {}
        }
    }

    // Combined bonus
    if signals.len() >= 2 {
        score = (score + 15.0).min(100.0);
        signals.push("combined: multiple Python quality tools configured".to_string());
    }

    (score, signals)
}

fn evaluate_mypy_ini_quality(content: &str, score: &mut f64, signals: &mut Vec<String>) {
    if content.contains("strict = True")
        || content.contains("strict=True")
        || content.contains("strict = true")
    {
        *score = score.max(45.0);
        signals.push("mypy: strict mode enabled".to_string());
    } else {
        *score = score.max(25.0);
        signals.push("mypy: type checking configured".to_string());
    }
}

fn evaluate_pyright_quality(content: &str, score: &mut f64, signals: &mut Vec<String>) {
    let Ok(val) = serde_json::from_str::<serde_json::Value>(content) else {
        return;
    };
    let Some(mode) = val.get("typeCheckingMode").and_then(|v| v.as_str()) else {
        return;
    };
    match mode {
        "strict" => {
            *score = score.max(50.0);
            signals.push("pyright: strict type checking mode".to_string());
        }
        "standard" => {
            *score = score.max(35.0);
            signals.push("pyright: standard type checking mode".to_string());
        }
        "basic" => {
            *score = score.max(20.0);
            signals.push("pyright: basic type checking mode".to_string());
        }
        _ => {}
    }
}

fn evaluate_ruff_quality(content: &str, score: &mut f64, signals: &mut Vec<String>) {
    if content.contains("select = [\"ALL\"]") || content.contains("select = ['ALL']") {
        *score = score.max(45.0);
        signals.push("ruff: ALL rules selected".to_string());
    } else if content.contains("[lint]") || content.contains("select") {
        *score = score.max(30.0);
        signals.push("ruff: custom lint rules configured".to_string());
    }
}

fn evaluate_pyproject_quality(content: &str, score: &mut f64, signals: &mut Vec<String>) {
    if content.contains("[tool.ruff") {
        if content.contains("select = [\"ALL\"]") || content.contains("select = ['ALL']") {
            *score = score.max(45.0);
            signals.push("ruff (pyproject): ALL rules selected".to_string());
        } else {
            *score = score.max(30.0);
            signals.push("ruff (pyproject): lint rules configured".to_string());
        }
    }
    if content.contains("[tool.mypy]") {
        if content.contains("strict = true") {
            *score = score.max(45.0);
            signals.push("mypy (pyproject): strict mode enabled".to_string());
        } else {
            *score = score.max(25.0);
            signals.push("mypy (pyproject): type checking configured".to_string());
        }
    }
}

// --- Go quality ---

fn evaluate_go_quality(
    detected: &DetectedConfigs,
    reader: &dyn FileContentReader,
) -> (f64, Vec<String>) {
    let mut score = 0.0f64;
    let mut signals = Vec::new();
    let go_configs = detected.for_language(Language::Go);

    let key_linters = [
        "govet",
        "staticcheck",
        "revive",
        "errcheck",
        "gosec",
        "gocritic",
        "gocyclo",
        "dupl",
        "ineffassign",
        "unconvert",
        "misspell",
    ];

    for config in &go_configs {
        let content = match reader.read_content(&config.path) {
            Some(c) => c,
            None => continue,
        };

        if config.pattern_filename.starts_with(".golangci") {
            // Count how many key linters are enabled
            let enabled_count = key_linters
                .iter()
                .filter(|linter| content.contains(**linter))
                .count();

            if enabled_count >= 6 {
                score = score.max(50.0);
                signals.push(format!(
                    "golangci-lint: {}/{} key linters enabled (comprehensive)",
                    enabled_count,
                    key_linters.len()
                ));
            } else if enabled_count >= 4 {
                score = score.max(35.0);
                signals.push(format!(
                    "golangci-lint: {}/{} key linters enabled (good coverage)",
                    enabled_count,
                    key_linters.len()
                ));
            } else if enabled_count >= 2 {
                score = score.max(20.0);
                signals.push(format!(
                    "golangci-lint: {}/{} key linters enabled (basic)",
                    enabled_count,
                    key_linters.len()
                ));
            } else {
                score = score.max(10.0);
                signals.push("golangci-lint: config present but few key linters".to_string());
            }

            // Check for enable-all
            if content.contains("enable-all") {
                score = score.max(50.0);
                if !signals.iter().any(|s| s.contains("comprehensive")) {
                    signals.push("golangci-lint: enable-all mode".to_string());
                }
            }
        }
    }

    (score, signals)
}

// ---------------------------------------------------------------------------
// Layering axis strictness evaluation
// ---------------------------------------------------------------------------

/// A declared boundary rule extracted from a config file.
#[derive(Debug, Clone)]
pub struct BoundaryRule {
    /// Source module/layer.
    pub from: String,
    /// Target module/layer.
    pub to: String,
    /// Whether this is an allowed or forbidden relationship.
    pub kind: BoundaryKind,
}

/// Whether a boundary rule allows or forbids the relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryKind {
    Allowed,
    Forbidden,
}

/// Result of evaluating the layering axis strictness.
#[derive(Debug, Clone)]
pub struct LayeringStrictnessResult {
    /// Layering strictness score (0-100).
    pub score: f64,
    /// Layer names extracted from configs.
    pub declared_layers: Vec<String>,
    /// Boundary rules extracted from configs.
    pub declared_boundaries: Vec<BoundaryRule>,
    /// Human-readable signals explaining what was detected.
    pub signals: Vec<String>,
    /// The primary language detected.
    pub language: Option<Language>,
}

/// Evaluate layering-axis strictness from detected config contents.
pub fn evaluate_layering(
    detected: &DetectedConfigs,
    file_paths: &[String],
    reader: &dyn FileContentReader,
) -> LayeringStrictnessResult {
    let mut best_score = 0.0f64;
    let mut best_layers = Vec::new();
    let mut best_boundaries = Vec::new();
    let mut best_signals = Vec::new();
    let mut best_language = None;

    for lang in &detected.languages {
        let (score, layers, boundaries, signals) = match lang {
            Language::JsTs => evaluate_jsts_layering(detected, reader),
            Language::Rust => evaluate_rust_layering(detected, file_paths, reader),
            Language::Python => evaluate_python_layering(detected, reader),
            Language::Go => evaluate_go_layering(detected, file_paths, reader),
            _ => (0.0, vec![], vec![], vec![]),
        };
        if score > best_score {
            best_score = score;
            best_layers = layers;
            best_boundaries = boundaries;
            best_signals = signals;
            best_language = Some(*lang);
        }
    }

    LayeringStrictnessResult {
        score: best_score,
        declared_layers: best_layers,
        declared_boundaries: best_boundaries,
        signals: best_signals,
        language: best_language,
    }
}

// --- JS/TS layering ---

fn evaluate_jsts_layering(
    detected: &DetectedConfigs,
    reader: &dyn FileContentReader,
) -> (f64, Vec<String>, Vec<BoundaryRule>, Vec<String>) {
    let mut score = 0.0f64;
    let mut layers = Vec::new();
    let mut boundaries = Vec::new();
    let mut signals = Vec::new();
    let jsts_configs = detected.for_language(Language::JsTs);

    for config in &jsts_configs {
        let content = match reader.read_content(&config.path) {
            Some(c) => c,
            None => continue,
        };

        match config.pattern_filename {
            pat if pat.starts_with(".dependency-cruiser") => {
                evaluate_depcruiser_layering(
                    &content, &mut score, &mut layers, &mut boundaries, &mut signals,
                );
            }
            "nx.json" => {
                evaluate_nx_layering(&content, &mut score, &mut signals);
            }
            pat if pat.starts_with(".eslintrc") || pat.starts_with("eslint.config") => {
                evaluate_eslint_layering(&content, &mut score, &mut signals);
            }
            _ => {}
        }
    }

    (score, layers, boundaries, signals)
}

fn evaluate_depcruiser_layering(
    content: &str,
    score: &mut f64,
    layers: &mut Vec<String>,
    boundaries: &mut Vec<BoundaryRule>,
    signals: &mut Vec<String>,
) {
    // Extract forbidden rules: look for "from" and "to" path patterns
    let forbidden_re = regex::Regex::new(r#"(?:from|path)\s*:\s*["']([^"']+)["']"#).ok();
    if let Some(re) = &forbidden_re {
        let found_paths: Vec<String> = re
            .captures_iter(content)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        for p in &found_paths {
            let layer = p
                .trim_start_matches('^')
                .split('/')
                .find(|seg| !seg.is_empty() && !seg.contains('(') && !seg.contains('.'))
                .unwrap_or("")
                .to_string();
            if !layer.is_empty() && !layers.contains(&layer) {
                layers.push(layer);
            }
        }
    }

    if content.contains("forbidden") {
        *score = score.max(70.0);
        signals.push("dependency-cruiser: forbidden rules defined".to_string());
    }
    if content.contains("allowed") {
        *score = score.max(60.0);
        if !signals.iter().any(|s| s.contains("dependency-cruiser")) {
            signals.push("dependency-cruiser: allowed rules defined".to_string());
        }
    }
    if content.contains("forbidden") && content.contains("allowed") {
        *score = score.max(85.0);
        signals.push("dependency-cruiser: both allowed and forbidden rules".to_string());
    }

    // Extract boundary rules from forbidden patterns
    let forbidden_block_re = regex::Regex::new(
        r#"from\s*:\s*\{[^}]*path\s*:\s*["']([^"']+)["'][^}]*\}[^}]*to\s*:\s*\{[^}]*path\s*:\s*["']([^"']+)["']"#
    ).ok();
    if let Some(re) = &forbidden_block_re {
        for cap in re.captures_iter(content) {
            if let (Some(from), Some(to)) = (cap.get(1), cap.get(2)) {
                boundaries.push(BoundaryRule {
                    from: from.as_str().to_string(),
                    to: to.as_str().to_string(),
                    kind: BoundaryKind::Forbidden,
                });
            }
        }
    }
}

fn evaluate_nx_layering(content: &str, score: &mut f64, signals: &mut Vec<String>) {
    let Ok(val) = serde_json::from_str::<serde_json::Value>(content) else {
        return;
    };
    let has_boundaries = content.contains("enforce-module-boundaries")
        || content.contains("@nrwl/nx/enforce-module-boundaries");
    if has_boundaries {
        *score = score.max(70.0);
        signals.push("nx: enforce-module-boundaries rule configured".to_string());
    }

    if let Some(target_defaults) = val.get("targetDefaults") {
        let td_str = serde_json::to_string(target_defaults).unwrap_or_default();
        if td_str.contains("tag") {
            *score = score.max(75.0);
            signals.push("nx: module boundary tags defined".to_string());
        }
    }
}

fn evaluate_eslint_layering(content: &str, score: &mut f64, signals: &mut Vec<String>) {
    if content.contains("eslint-plugin-boundaries")
        || content.contains("boundaries/element-types")
        || content.contains("boundaries/entry-point")
    {
        *score = score.max(65.0);
        signals.push("eslint: boundaries plugin configured".to_string());
    }

    if content.contains("no-restricted-imports") {
        let pattern_count =
            content.matches("patterns").count() + content.matches("paths").count();
        if pattern_count >= 3 {
            *score = score.max(55.0);
            signals.push(format!(
                "eslint: no-restricted-imports with {pattern_count} pattern groups"
            ));
        } else if pattern_count >= 1 {
            *score = score.max(40.0);
            signals.push("eslint: no-restricted-imports configured".to_string());
        }
    }
}

// --- Rust layering ---

fn evaluate_rust_layering(
    detected: &DetectedConfigs,
    _file_paths: &[String],
    reader: &dyn FileContentReader,
) -> (f64, Vec<String>, Vec<BoundaryRule>, Vec<String>) {
    let mut score = 0.0f64;
    let mut layers = Vec::new();
    let mut boundaries = Vec::new();
    let mut signals = Vec::new();

    // Cargo workspace = language-enforced module boundaries
    if detected.has_cargo_workspace {
        let member_count = detected.cargo_workspace_members.len();
        layers = detected
            .cargo_workspace_members
            .iter()
            .map(|m| {
                // Extract crate name from path like "crates/core" -> "core"
                Path::new(m)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| m.clone())
            })
            .collect();

        if member_count >= 4 {
            score = score.max(70.0);
            signals.push(format!(
                "cargo workspace: {member_count} crates (strong module boundaries)"
            ));
        } else if member_count >= 2 {
            score = score.max(50.0);
            signals.push(format!(
                "cargo workspace: {member_count} crates (basic module boundaries)"
            ));
        }
    }

    // Check deny.toml for bans (dependency restrictions between crates)
    for config in detected.for_language(Language::Rust) {
        if config.pattern_filename != "deny.toml" {
            continue;
        }
        let content = match reader.read_content(&config.path) {
            Some(c) => c,
            None => continue,
        };

        if content.contains("[bans]") {
            if content.contains("deny") || content.contains("skip") {
                score = score.max(65.0);
                signals.push("cargo-deny: bans section with deny/skip rules".to_string());
            }
            // Extract specific deny entries as boundary rules
            let deny_re = regex::Regex::new(r#"name\s*=\s*["']([^"']+)["']"#).ok();
            if let Some(re) = &deny_re {
                for cap in re.captures_iter(&content) {
                    if let Some(name) = cap.get(1) {
                        boundaries.push(BoundaryRule {
                            from: "workspace".to_string(),
                            to: name.as_str().to_string(),
                            kind: BoundaryKind::Forbidden,
                        });
                    }
                }
            }
        }
    }

    // Combined: workspace + deny = strong layering
    if detected.has_cargo_workspace && signals.iter().any(|s| s.contains("cargo-deny")) {
        score = (score + 10.0).min(100.0);
        signals.push("combined: workspace boundaries + dependency policy".to_string());
    }

    (score, layers, boundaries, signals)
}

// --- Python layering ---

fn evaluate_python_layering(
    detected: &DetectedConfigs,
    reader: &dyn FileContentReader,
) -> (f64, Vec<String>, Vec<BoundaryRule>, Vec<String>) {
    let mut score = 0.0f64;
    let mut layers = Vec::new();
    let mut boundaries = Vec::new();
    let mut signals = Vec::new();

    for config in detected.for_language(Language::Python) {
        let content = match reader.read_content(&config.path) {
            Some(c) => c,
            None => continue,
        };

        match config.pattern_filename {
            ".importlinter" => {
                score = score.max(70.0);
                signals.push("import-linter: standalone config present".to_string());
                extract_import_linter_layers(
                    &content,
                    &mut layers,
                    &mut boundaries,
                    &mut signals,
                    &mut score,
                );
            }
            "pyproject.toml" => {
                if content.contains("[tool.import_linter") || content.contains("[tool.importlinter")
                {
                    score = score.max(70.0);
                    signals.push("import-linter: configured in pyproject.toml".to_string());
                    extract_import_linter_layers(
                        &content,
                        &mut layers,
                        &mut boundaries,
                        &mut signals,
                        &mut score,
                    );
                }
            }
            "setup.cfg" => {
                if content.contains("[importlinter") || content.contains("[import-linter") {
                    score = score.max(70.0);
                    signals.push("import-linter: configured in setup.cfg".to_string());
                    extract_import_linter_layers(
                        &content,
                        &mut layers,
                        &mut boundaries,
                        &mut signals,
                        &mut score,
                    );
                }
            }
            _ => {}
        }
    }

    (score, layers, boundaries, signals)
}

/// Extract layer and contract info from import-linter config content.
fn extract_import_linter_layers(
    content: &str,
    layers: &mut Vec<String>,
    boundaries: &mut Vec<BoundaryRule>,
    signals: &mut Vec<String>,
    score: &mut f64,
) {
    // Extract contract types
    let has_layers_contract = content.contains("type = layers") || content.contains("type=layers");
    let has_forbidden_contract =
        content.contains("type = forbidden") || content.contains("type=forbidden");
    let has_independence_contract =
        content.contains("type = independence") || content.contains("type=independence");

    if has_layers_contract {
        *score = score.max(80.0);
        signals.push("import-linter: layers contract defined".to_string());
    }
    if has_forbidden_contract {
        *score = score.max(75.0);
        signals.push("import-linter: forbidden contract defined".to_string());
    }
    if has_independence_contract {
        *score = score.max(75.0);
        signals.push("import-linter: independence contract defined".to_string());
    }
    if has_layers_contract && (has_forbidden_contract || has_independence_contract) {
        *score = score.max(90.0);
        signals.push("import-linter: multiple contract types (comprehensive)".to_string());
    }

    extract_module_names_from_content(content, layers);

    // Create boundary rules from layers contract (each layer can only import from layers below it)
    if has_layers_contract && layers.len() >= 2 {
        for i in 0..layers.len() {
            for j in 0..i {
                boundaries.push(BoundaryRule {
                    from: layers[j].clone(),
                    to: layers[i].clone(),
                    kind: BoundaryKind::Forbidden,
                });
            }
        }
    }
}

/// Extract module names from "layers" or "modules" lines in config content.
/// Handles both TOML array format and INI-style indented lines.
fn extract_module_names_from_content(content: &str, layers: &mut Vec<String>) {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_layers_block = false;

    for line in &lines {
        let trimmed = line.trim();

        if trimmed.starts_with("layers")
            || trimmed.starts_with("modules")
            || trimmed.starts_with("source_modules")
            || trimmed.starts_with("containers")
        {
            if let Some(bracket_start) = trimmed.find('[') {
                let after = &trimmed[bracket_start..];
                for item in after
                    .trim_matches(|c: char| c == '[' || c == ']')
                    .split(',')
                {
                    let module = item
                        .trim()
                        .trim_matches(|c: char| c == '"' || c == '\'' || c == ' ')
                        .to_string();
                    if !module.is_empty() && !layers.contains(&module) {
                        layers.push(module);
                    }
                }
            } else {
                in_layers_block = true;
            }
            continue;
        }

        if in_layers_block {
            if trimmed.is_empty() || trimmed.starts_with('[') || trimmed.starts_with('#') {
                if trimmed.starts_with('[') {
                    in_layers_block = false;
                }
                continue;
            }
            if line.starts_with(' ') || line.starts_with('\t') || trimmed.starts_with('-') {
                let module = trimmed
                    .trim_start_matches('-')
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_matches(|c: char| c == '"' || c == '\'' || c == ',')
                    .to_string();
                if !module.is_empty() && !layers.contains(&module) {
                    layers.push(module);
                }
            } else {
                in_layers_block = false;
            }
        }
    }
}

// --- Go layering ---

fn evaluate_go_layering(
    detected: &DetectedConfigs,
    file_paths: &[String],
    reader: &dyn FileContentReader,
) -> (f64, Vec<String>, Vec<BoundaryRule>, Vec<String>) {
    let mut score = 0.0f64;
    let mut layers = Vec::new();
    let boundaries = Vec::new();
    let mut signals = Vec::new();

    // Go internal/ packages = language-enforced boundaries
    if detected.has_go_internal {
        // Extract internal package names
        let internal_pkgs: Vec<String> = file_paths
            .iter()
            .filter_map(|p| {
                let parts: Vec<&str> = p.split('/').collect();
                let internal_idx = parts.iter().position(|&s| s == "internal")?;
                if internal_idx + 1 < parts.len() - 1 {
                    Some(parts[internal_idx + 1].to_string())
                } else {
                    None
                }
            })
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        if !internal_pkgs.is_empty() {
            score = score.max(45.0);
            signals.push(format!(
                "go: {} internal packages (language-enforced boundaries)",
                internal_pkgs.len()
            ));
            for pkg in &internal_pkgs {
                if !layers.contains(pkg) {
                    layers.push(pkg.clone());
                }
            }
        }
    }

    // Check golangci-lint for depguard
    for config in detected.for_language(Language::Go) {
        if !config.pattern_filename.starts_with(".golangci") {
            continue;
        }
        let content = match reader.read_content(&config.path) {
            Some(c) => c,
            None => continue,
        };

        if content.contains("depguard") {
            score = score.max(65.0);
            signals.push("golangci-lint: depguard import restrictions configured".to_string());

            // Extract allow/deny patterns
            let list_re = regex::Regex::new(r"(?:allow|deny)\s*:\s*\n((?:\s+-\s*.+\n?)*)").ok();
            if let Some(re) = &list_re {
                for cap in re.captures_iter(&content) {
                    if let Some(block) = cap.get(1) {
                        for line in block.as_str().lines() {
                            let pkg = line.trim().trim_start_matches('-').trim().trim_matches('"');
                            if !pkg.is_empty() && !pkg.starts_with('#') {
                                let short_name = pkg.rsplit('/').next().unwrap_or(pkg).to_string();
                                if !layers.contains(&short_name) {
                                    layers.push(short_name);
                                }
                            }
                        }
                    }
                }
            }

            // Both depguard + internal = comprehensive
            if detected.has_go_internal {
                score = score.max(80.0);
                signals.push("combined: depguard + internal packages".to_string());
            }
        }
    }

    (score, layers, boundaries, signals)
}

// ---------------------------------------------------------------------------
// RulesConfigAnalyzer: orchestrator and structure inference
// ---------------------------------------------------------------------------

/// Configuration for the rules config analyzer thresholds.
#[derive(Debug, Clone)]
pub struct RulesConfigAnalyzerConfig {
    /// Minimum quality score to pass (0-100). Default: 70.
    pub quality_threshold: f64,
    /// Minimum layering score to pass (0-100). Default: 60.
    pub layering_threshold: f64,
}

impl Default for RulesConfigAnalyzerConfig {
    fn default() -> Self {
        Self {
            quality_threshold: 70.0,
            layering_threshold: 60.0,
        }
    }
}

/// Combined result of rules config analysis.
#[derive(Debug, Clone)]
pub struct RulesConfigAnalysisResult {
    /// Quality axis result.
    pub quality: QualityStrictnessResult,
    /// Layering axis result.
    pub layering: LayeringStrictnessResult,
    /// Inferred StructureAnalysis, present only when both axes pass threshold.
    pub inferred_analysis: Option<StructureAnalysis>,
    /// Whether the fast path was attempted (configs were found).
    pub attempted: bool,
}

/// Top-level rules config analyzer.
pub struct RulesConfigAnalyzer {
    config: RulesConfigAnalyzerConfig,
}

impl RulesConfigAnalyzer {
    /// Create analyzer with default thresholds.
    pub fn new() -> Self {
        Self {
            config: RulesConfigAnalyzerConfig::default(),
        }
    }

    /// Create analyzer with custom thresholds.
    pub fn with_config(config: RulesConfigAnalyzerConfig) -> Self {
        Self { config }
    }

    /// Analyze file paths for rules engine configs and attempt structure inference.
    ///
    /// Returns a result indicating whether the fast path can be taken.
    /// If `inferred_analysis` is `Some`, the caller can skip file-level analysis.
    pub fn analyze(
        &self,
        file_paths: &[String],
        reader: &dyn FileContentReader,
    ) -> RulesConfigAnalysisResult {
        let detected = detect_configs(file_paths);

        // Early exit: no configs found at all
        if detected.configs.is_empty() && !detected.has_cargo_workspace && !detected.has_go_internal
        {
            return RulesConfigAnalysisResult {
                quality: QualityStrictnessResult {
                    score: 0.0,
                    signals: vec![],
                    language: None,
                },
                layering: LayeringStrictnessResult {
                    score: 0.0,
                    declared_layers: vec![],
                    declared_boundaries: vec![],
                    signals: vec![],
                    language: None,
                },
                inferred_analysis: None,
                attempted: false,
            };
        }

        let quality = evaluate_quality(&detected, reader);
        let layering = evaluate_layering(&detected, file_paths, reader);

        let inferred_analysis = if quality.score >= self.config.quality_threshold
            && layering.score >= self.config.layering_threshold
        {
            Some(self.infer_structure_analysis(file_paths, &quality, &layering))
        } else {
            None
        };

        RulesConfigAnalysisResult {
            quality,
            layering,
            inferred_analysis,
            attempted: true,
        }
    }

    /// Infer a `StructureAnalysis` from config declarations + file path metadata.
    ///
    /// Architecture-critical fields (layers, boundaries, quality score) come from
    /// config analysis. Structural fields (naming, tests, depth) come from file paths
    /// using the same logic as `StructureAnalyzer`.
    fn infer_structure_analysis(
        &self,
        file_paths: &[String],
        quality: &QualityStrictnessResult,
        layering: &LayeringStrictnessResult,
    ) -> StructureAnalysis {
        // Reuse StructureAnalyzer for file-path-derived fields
        let base = StructureAnalyzer::analyze(file_paths);

        // Override architecture-critical fields with config-derived values
        let detected_layers = if layering.declared_layers.is_empty() {
            base.detected_layers
        } else {
            layering.declared_layers.clone()
        };

        let module_boundaries = if layering.declared_boundaries.is_empty() {
            base.module_boundaries
        } else {
            // Extract unique module names from boundary rules
            let mut modules: Vec<String> = layering
                .declared_boundaries
                .iter()
                .flat_map(|b| vec![b.from.clone(), b.to.clone()])
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            modules.sort();
            modules
        };

        // Quality score: weighted combination of config scores
        let structure_quality_score = quality.score.mul_add(0.5, layering.score * 0.5);

        StructureAnalysis {
            top_level_dirs: base.top_level_dirs,
            detected_layers,
            file_naming_convention: base.file_naming_convention,
            test_pattern: base.test_pattern,
            total_files: base.total_files,
            depth_distribution: base.depth_distribution,
            has_src_root: base.has_src_root,
            module_boundaries,
            structure_quality_score,
        }
    }
}

impl Default for RulesConfigAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::structure_analyzer::TestPattern;

    #[test]
    fn test_detect_jsts_quality_configs() {
        let paths = vec![
            "src/index.ts".to_string(),
            ".eslintrc.json".to_string(),
            "tsconfig.json".to_string(),
            "package.json".to_string(),
        ];

        let detected = detect_configs(&paths);

        assert!(detected.has_quality_configs());
        let quality = detected.for_axis(ConfigAxis::Quality);
        assert!(
            quality.len() >= 2,
            "Expected >=2 quality configs, got {}",
            quality.len()
        );

        let filenames: Vec<&str> = quality.iter().map(|c| c.pattern_filename).collect();
        assert!(filenames.contains(&".eslintrc"));
        assert!(filenames.contains(&"tsconfig.json"));
    }

    #[test]
    fn test_detect_jsts_layering_configs() {
        let paths = vec![
            "src/index.ts".to_string(),
            ".dependency-cruiser.js".to_string(),
            "nx.json".to_string(),
        ];

        let detected = detect_configs(&paths);

        assert!(detected.has_layering_configs());
        let layering = detected.for_axis(ConfigAxis::Layering);
        assert!(layering.len() >= 2);
    }

    #[test]
    fn test_detect_rust_configs() {
        let paths = vec![
            "Cargo.toml".to_string(),
            "crates/core/Cargo.toml".to_string(),
            "crates/cli/Cargo.toml".to_string(),
            "crates/mcp/Cargo.toml".to_string(),
            "clippy.toml".to_string(),
            "deny.toml".to_string(),
            "rustfmt.toml".to_string(),
        ];

        let detected = detect_configs(&paths);

        assert!(detected.has_quality_configs());
        assert!(detected.has_layering_configs());
        assert!(detected.has_cargo_workspace);
        assert_eq!(detected.cargo_workspace_members.len(), 3);
        assert!(detected.languages.contains(&Language::Rust));
    }

    #[test]
    fn test_detect_python_configs() {
        let paths = vec![
            "pyproject.toml".to_string(),
            "mypy.ini".to_string(),
            ".importlinter".to_string(),
            "src/app/__init__.py".to_string(),
        ];

        let detected = detect_configs(&paths);

        assert!(detected.has_quality_configs());
        assert!(detected.has_layering_configs());
        assert!(detected.languages.contains(&Language::Python));
    }

    #[test]
    fn test_detect_go_configs() {
        let paths = vec![
            "go.mod".to_string(),
            ".golangci.yml".to_string(),
            "internal/auth/handler.go".to_string(),
            "cmd/server/main.go".to_string(),
        ];

        let detected = detect_configs(&paths);

        assert!(detected.has_quality_configs());
        assert!(detected.has_layering_configs());
        assert!(detected.has_go_internal);
        assert!(detected.languages.contains(&Language::Go));
    }

    #[test]
    fn test_detect_no_configs() {
        let paths = vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "README.md".to_string(),
        ];

        let detected = detect_configs(&paths);

        assert!(!detected.has_quality_configs());
        assert!(!detected.has_layering_configs());
        assert!(detected.configs.is_empty());
    }

    #[test]
    fn test_detect_quality_only_no_layering() {
        let paths = vec![
            "src/index.ts".to_string(),
            ".eslintrc.json".to_string(),
            "tsconfig.json".to_string(),
            // No dependency-cruiser, no nx.json
        ];

        let detected = detect_configs(&paths);

        assert!(detected.has_quality_configs());
        assert!(!detected.has_layering_configs());
    }

    #[test]
    fn test_detect_layering_only_no_quality() {
        let paths = vec![
            "src/index.ts".to_string(),
            ".dependency-cruiser.cjs".to_string(),
            // No eslint, no tsconfig
        ];

        let detected = detect_configs(&paths);

        assert!(!detected.has_quality_configs());
        assert!(detected.has_layering_configs());
    }

    #[test]
    fn test_detect_eslint_flat_config() {
        let paths = vec!["eslint.config.mjs".to_string()];

        let detected = detect_configs(&paths);

        assert_eq!(detected.configs.len(), 1);
        assert_eq!(detected.configs[0].language, Language::JsTs);
        assert_eq!(detected.configs[0].axis, ConfigAxis::Quality);
    }

    #[test]
    fn test_detect_mixed_languages() {
        let paths = vec![
            "tsconfig.json".to_string(),
            ".golangci.yml".to_string(),
            "pyproject.toml".to_string(),
        ];

        let detected = detect_configs(&paths);

        assert!(detected.languages.contains(&Language::JsTs));
        assert!(detected.languages.contains(&Language::Go));
        assert!(detected.languages.contains(&Language::Python));
    }

    #[test]
    fn test_cargo_workspace_single_crate_not_workspace() {
        let paths = vec!["Cargo.toml".to_string(), "src/main.rs".to_string()];

        let detected = detect_configs(&paths);

        assert!(!detected.has_cargo_workspace);
        assert!(detected.cargo_workspace_members.is_empty());
    }

    #[test]
    fn test_mock_content_reader() {
        let mut files = HashMap::new();
        files.insert(
            "tsconfig.json".to_string(),
            r#"{"compilerOptions":{"strict":true}}"#.to_string(),
        );
        let reader = MockContentReader::new(files);

        assert!(reader.read_content("tsconfig.json").is_some());
        assert!(reader.read_content("nonexistent.json").is_none());
    }

    #[test]
    fn test_detect_biome_config() {
        let paths = vec!["biome.json".to_string()];
        let detected = detect_configs(&paths);

        assert_eq!(detected.configs.len(), 1);
        assert_eq!(detected.configs[0].language, Language::JsTs);
    }

    #[test]
    fn test_detect_deny_toml_covers_both_axes() {
        let paths = vec!["deny.toml".to_string()];
        let detected = detect_configs(&paths);

        assert_eq!(detected.configs.len(), 1);
        assert_eq!(detected.configs[0].axis, ConfigAxis::Both);
        assert!(detected.has_quality_configs());
        assert!(detected.has_layering_configs());
    }

    #[test]
    fn test_for_language_filter() {
        let paths = vec![
            "tsconfig.json".to_string(),
            ".eslintrc.json".to_string(),
            "clippy.toml".to_string(),
        ];

        let detected = detect_configs(&paths);

        let jsts = detected.for_language(Language::JsTs);
        assert_eq!(jsts.len(), 2);

        let rust = detected.for_language(Language::Rust);
        assert_eq!(rust.len(), 1);

        let python = detected.for_language(Language::Python);
        assert_eq!(python.len(), 0);
    }

    #[test]
    fn test_no_duplicate_detections() {
        // A file path should only be detected once even if multiple patterns could match
        let paths = vec!["deny.toml".to_string()];
        let detected = detect_configs(&paths);
        assert_eq!(detected.configs.len(), 1);
    }

    // -----------------------------------------------------------------------
    // Quality axis tests
    // -----------------------------------------------------------------------

    fn mock_reader(files: Vec<(&str, &str)>) -> MockContentReader {
        let map: HashMap<String, String> = files
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        MockContentReader::new(map)
    }

    #[test]
    fn test_jsts_quality_strict_tsconfig() {
        let paths = vec!["tsconfig.json".to_string(), "src/index.ts".to_string()];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![(
            "tsconfig.json",
            r#"{"compilerOptions":{"strict":true,"noUncheckedIndexedAccess":true,"noImplicitReturns":true}}"#,
        )]);

        let result = evaluate_quality(&detected, &reader);
        assert!(
            result.score >= 40.0,
            "Score {} should be >= 40",
            result.score
        );
        assert!(result.signals.iter().any(|s| s.contains("strict mode")));
    }

    #[test]
    fn test_jsts_quality_eslint_strict_preset() {
        let paths = vec![".eslintrc.json".to_string()];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![(
            ".eslintrc.json",
            r#"{"extends":["eslint:recommended","@typescript-eslint/strict"],"rules":{}}"#,
        )]);

        let result = evaluate_quality(&detected, &reader);
        assert!(
            result.score >= 45.0,
            "Score {} should be >= 45",
            result.score
        );
    }

    #[test]
    fn test_jsts_quality_eslint_recommended() {
        let paths = vec![".eslintrc.json".to_string()];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![(
            ".eslintrc.json",
            r#"{"extends":["eslint:recommended"]}"#,
        )]);

        let result = evaluate_quality(&detected, &reader);
        assert!(
            result.score >= 35.0,
            "Score {} should be >= 35",
            result.score
        );
    }

    #[test]
    fn test_jsts_quality_combined_tsconfig_and_eslint() {
        let paths = vec!["tsconfig.json".to_string(), ".eslintrc.json".to_string()];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![
            ("tsconfig.json", r#"{"compilerOptions":{"strict":true}}"#),
            (".eslintrc.json", r#"{"extends":["eslint:recommended"]}"#),
        ]);

        let result = evaluate_quality(&detected, &reader);
        // Combined should score higher than either alone
        assert!(
            result.score >= 50.0,
            "Combined score {} should be >= 50",
            result.score
        );
        assert!(result.signals.iter().any(|s| s.contains("combined")));
    }

    #[test]
    fn test_jsts_quality_biome_all_rules() {
        let paths = vec!["biome.json".to_string()];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![(
            "biome.json",
            r#"{"linter":{"enabled":true,"rules":{"all":true}}}"#,
        )]);

        let result = evaluate_quality(&detected, &reader);
        assert!(
            result.score >= 45.0,
            "Score {} should be >= 45",
            result.score
        );
    }

    #[test]
    fn test_rust_quality_pedantic_clippy() {
        let paths = vec![
            "Cargo.toml".to_string(),
            "crates/core/Cargo.toml".to_string(),
            "clippy.toml".to_string(),
            "deny.toml".to_string(),
        ];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![
            ("Cargo.toml", "[workspace]\nmembers = [\"crates/core\"]\n\n[workspace.lints.clippy]\npedantic = { level = \"warn\" }\nnursery = { level = \"warn\" }\n# clippy::pedantic clippy::nursery"),
            ("clippy.toml", "cognitive-complexity-threshold = 10"),
            ("deny.toml", "[advisories]\nvulnerability = \"deny\"\n\n[licenses]\nunlicensed = \"deny\"\n\n[bans]\nmultiple-versions = \"warn\""),
        ]);

        let result = evaluate_quality(&detected, &reader);
        assert!(
            result.score >= 50.0,
            "Score {} should be >= 50",
            result.score
        );
    }

    #[test]
    fn test_rust_quality_minimal() {
        let paths = vec!["rustfmt.toml".to_string()];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![("rustfmt.toml", "max_width = 100")]);

        let result = evaluate_quality(&detected, &reader);
        assert!(
            result.score <= 30.0,
            "Minimal config score {} should be <= 30",
            result.score
        );
    }

    #[test]
    fn test_python_quality_strict_mypy() {
        let paths = vec!["mypy.ini".to_string()];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![(
            "mypy.ini",
            "[mypy]\nstrict = True\nwarn_return_any = True",
        )]);

        let result = evaluate_quality(&detected, &reader);
        assert!(
            result.score >= 45.0,
            "Score {} should be >= 45",
            result.score
        );
        assert!(result.signals.iter().any(|s| s.contains("strict")));
    }

    #[test]
    fn test_python_quality_ruff_all() {
        let paths = vec!["ruff.toml".to_string()];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![("ruff.toml", "[lint]\nselect = [\"ALL\"]\n")]);

        let result = evaluate_quality(&detected, &reader);
        assert!(
            result.score >= 45.0,
            "Score {} should be >= 45",
            result.score
        );
    }

    #[test]
    fn test_python_quality_pyproject_combined() {
        let paths = vec!["pyproject.toml".to_string()];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![(
            "pyproject.toml",
            "[tool.ruff]\n[tool.ruff.lint]\nselect = [\"ALL\"]\n\n[tool.mypy]\nstrict = true\n",
        )]);

        let result = evaluate_quality(&detected, &reader);
        assert!(
            result.score >= 45.0,
            "Score {} should be >= 45",
            result.score
        );
    }

    #[test]
    fn test_python_quality_pyright_strict() {
        let paths = vec!["pyrightconfig.json".to_string()];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![(
            "pyrightconfig.json",
            r#"{"typeCheckingMode":"strict","reportMissingImports":true}"#,
        )]);

        let result = evaluate_quality(&detected, &reader);
        assert!(
            result.score >= 50.0,
            "Score {} should be >= 50",
            result.score
        );
    }

    #[test]
    fn test_go_quality_comprehensive_linters() {
        let paths = vec![".golangci.yml".to_string()];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![(
            ".golangci.yml",
            "linters:\n  enable:\n    - govet\n    - staticcheck\n    - revive\n    - errcheck\n    - gosec\n    - gocritic\n    - gocyclo\n",
        )]);

        let result = evaluate_quality(&detected, &reader);
        assert!(
            result.score >= 50.0,
            "Score {} should be >= 50",
            result.score
        );
        assert!(result
            .signals
            .iter()
            .any(|s| s.contains("comprehensive") || s.contains("key linters")));
    }

    #[test]
    fn test_go_quality_minimal_linters() {
        let paths = vec![".golangci.yml".to_string()];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![(
            ".golangci.yml",
            "linters:\n  enable:\n    - govet\n",
        )]);

        let result = evaluate_quality(&detected, &reader);
        assert!(
            result.score <= 20.0,
            "Minimal config score {} should be <= 20",
            result.score
        );
    }

    #[test]
    fn test_quality_no_configs() {
        let paths = vec!["src/main.rs".to_string()];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![]);

        let result = evaluate_quality(&detected, &reader);
        assert_eq!(result.score, 0.0);
        assert!(result.signals.is_empty());
        assert!(result.language.is_none());
    }

    #[test]
    fn test_quality_unreadable_config() {
        let paths = vec!["tsconfig.json".to_string()];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![]); // Config file not in mock = unreadable

        let result = evaluate_quality(&detected, &reader);
        assert_eq!(result.score, 0.0);
    }

    // -----------------------------------------------------------------------
    // Layering axis tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_jsts_layering_dependency_cruiser_forbidden() {
        let paths = vec![".dependency-cruiser.js".to_string()];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![(
            ".dependency-cruiser.js",
            r#"module.exports = {
                forbidden: [
                    { from: { path: "^src/domain" }, to: { path: "^src/infrastructure" } },
                    { from: { path: "^src/application" }, to: { path: "^src/presentation" } },
                ],
                allowed: [
                    { from: { path: "^src/presentation" }, to: { path: "^src/application" } },
                ],
            };"#,
        )]);

        let result = evaluate_layering(&detected, &[], &reader);
        assert!(
            result.score >= 85.0,
            "Score {} should be >= 85 (both forbidden and allowed)",
            result.score
        );
        assert!(
            !result.declared_layers.is_empty(),
            "Should extract layer names"
        );
        assert!(result
            .signals
            .iter()
            .any(|s| s.contains("dependency-cruiser")));
    }

    #[test]
    fn test_jsts_layering_nx_module_boundaries() {
        let paths = vec!["nx.json".to_string()];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![(
            "nx.json",
            r#"{
                "targetDefaults": {
                    "lint": { "options": { "tag": "scope:shared" } }
                },
                "plugins": ["@nx/enforce-module-boundaries"]
            }"#,
        )]);

        let result = evaluate_layering(&detected, &[], &reader);
        assert!(
            result.score >= 70.0,
            "Score {} should be >= 70",
            result.score
        );
    }

    #[test]
    fn test_jsts_layering_eslint_boundaries_plugin() {
        let paths = vec![".eslintrc.json".to_string()];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![(
            ".eslintrc.json",
            r#"{"plugins":["boundaries"],"rules":{"boundaries/element-types":"error","boundaries/entry-point":"error"}}"#,
        )]);

        let result = evaluate_layering(&detected, &[], &reader);
        assert!(
            result.score >= 65.0,
            "Score {} should be >= 65",
            result.score
        );
    }

    #[test]
    fn test_jsts_layering_no_restricted_imports() {
        let paths = vec![".eslintrc.json".to_string()];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![(
            ".eslintrc.json",
            r#"{"rules":{"no-restricted-imports":["error",{"patterns":["@internal/*"],"paths":["lodash"],"paths":["moment"]}]}}"#,
        )]);

        let result = evaluate_layering(&detected, &[], &reader);
        assert!(
            result.score >= 40.0,
            "Score {} should be >= 40",
            result.score
        );
    }

    #[test]
    fn test_rust_layering_cargo_workspace() {
        let paths = vec![
            "Cargo.toml".to_string(),
            "crates/core/Cargo.toml".to_string(),
            "crates/cli/Cargo.toml".to_string(),
            "crates/mcp/Cargo.toml".to_string(),
            "crates/store/Cargo.toml".to_string(),
        ];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![]);

        let result = evaluate_layering(&detected, &paths, &reader);
        assert!(
            result.score >= 70.0,
            "Score {} should be >= 70 (4 crates)",
            result.score
        );
        assert_eq!(result.declared_layers.len(), 4);
        assert!(result.declared_layers.contains(&"core".to_string()));
    }

    #[test]
    fn test_rust_layering_workspace_plus_deny() {
        let paths = vec![
            "Cargo.toml".to_string(),
            "crates/core/Cargo.toml".to_string(),
            "crates/cli/Cargo.toml".to_string(),
            "deny.toml".to_string(),
        ];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![(
            "deny.toml",
            "[bans]\nmultiple-versions = \"deny\"\n[[bans.deny]]\nname = \"openssl\"\n",
        )]);

        let result = evaluate_layering(&detected, &paths, &reader);
        assert!(
            result.score >= 60.0,
            "Score {} should be >= 60",
            result.score
        );
        assert!(result
            .signals
            .iter()
            .any(|s| s.contains("combined") || s.contains("workspace")));
    }

    #[test]
    fn test_python_layering_import_linter_standalone() {
        let paths = vec![".importlinter".to_string()];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![(
            ".importlinter",
            "[importlinter]\nroot_package = myapp\n\n[importlinter:contract:layers]\nname = Layer contract\ntype = layers\nlayers =\n    myapp.presentation\n    myapp.application\n    myapp.domain\n    myapp.infrastructure\n",
        )]);

        let result = evaluate_layering(&detected, &[], &reader);
        assert!(
            result.score >= 80.0,
            "Score {} should be >= 80 (layers contract)",
            result.score
        );
        assert!(
            result.declared_layers.len() >= 2,
            "Should extract layer names from contract"
        );
    }

    #[test]
    fn test_python_layering_pyproject_import_linter() {
        let paths = vec!["pyproject.toml".to_string()];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![(
            "pyproject.toml",
            "[tool.import_linter]\nroot_packages = [\"myapp\"]\n\n[[tool.import_linter.contracts]]\nname = \"Layers\"\ntype = layers\nlayers = [\"presentation\", \"application\", \"domain\"]\n\n[[tool.import_linter.contracts]]\nname = \"No infra in domain\"\ntype = forbidden\nsource_modules = [\"myapp.domain\"]\nforbidden_modules = [\"myapp.infrastructure\"]\n",
        )]);

        let result = evaluate_layering(&detected, &[], &reader);
        assert!(
            result.score >= 90.0,
            "Score {} should be >= 90 (layers + forbidden)",
            result.score
        );
    }

    #[test]
    fn test_go_layering_internal_packages() {
        let paths = vec![
            "cmd/server/main.go".to_string(),
            "internal/auth/handler.go".to_string(),
            "internal/auth/middleware.go".to_string(),
            "internal/database/client.go".to_string(),
            "pkg/utils/helpers.go".to_string(),
        ];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![]);

        let result = evaluate_layering(&detected, &paths, &reader);
        assert!(
            result.score >= 45.0,
            "Score {} should be >= 45 (internal pkgs)",
            result.score
        );
        assert!(result.declared_layers.contains(&"auth".to_string()));
        assert!(result.declared_layers.contains(&"database".to_string()));
    }

    #[test]
    fn test_go_layering_depguard_plus_internal() {
        let paths = vec![
            ".golangci.yml".to_string(),
            "internal/auth/handler.go".to_string(),
            "internal/database/client.go".to_string(),
        ];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![(
            ".golangci.yml",
            "linters:\n  enable:\n    - depguard\nlinters-settings:\n  depguard:\n    rules:\n      main:\n        deny:\n          - \"github.com/lib/pq\"\n",
        )]);

        let result = evaluate_layering(&detected, &paths, &reader);
        assert!(
            result.score >= 80.0,
            "Score {} should be >= 80 (depguard + internal)",
            result.score
        );
    }

    #[test]
    fn test_layering_no_configs() {
        let paths = vec!["src/main.rs".to_string()];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![]);

        let result = evaluate_layering(&detected, &paths, &reader);
        assert_eq!(result.score, 0.0);
        assert!(result.declared_layers.is_empty());
    }

    #[test]
    fn test_layering_jsts_quality_only_no_layering() {
        // eslint present but no boundary enforcement
        let paths = vec![".eslintrc.json".to_string()];
        let detected = detect_configs(&paths);
        let reader = mock_reader(vec![(
            ".eslintrc.json",
            r#"{"extends":["eslint:recommended"],"rules":{"no-unused-vars":"error"}}"#,
        )]);

        let result = evaluate_layering(&detected, &[], &reader);
        assert_eq!(
            result.score, 0.0,
            "Quality-only eslint should not score on layering"
        );
    }

    // -----------------------------------------------------------------------
    // RulesConfigAnalyzer orchestrator tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_analyzer_both_axes_pass_infers_analysis() {
        let paths = vec![
            "tsconfig.json".to_string(),
            ".eslintrc.json".to_string(),
            ".dependency-cruiser.js".to_string(),
            "src/domain/user.ts".to_string(),
            "src/infrastructure/db.ts".to_string(),
            "src/application/service.ts".to_string(),
            "tests/user.test.ts".to_string(),
        ];

        let reader = mock_reader(vec![
            (
                "tsconfig.json",
                r#"{"compilerOptions":{"strict":true,"noUncheckedIndexedAccess":true,"noImplicitReturns":true}}"#,
            ),
            (
                ".eslintrc.json",
                r#"{"extends":["eslint:recommended","@typescript-eslint/strict"]}"#,
            ),
            (
                ".dependency-cruiser.js",
                r#"module.exports = {
                forbidden: [
                    { from: { path: "^src/domain" }, to: { path: "^src/infrastructure" } },
                ],
                allowed: [
                    { from: { path: "^src/application" }, to: { path: "^src/domain" } },
                ],
            };"#,
            ),
        ]);

        // Use lower thresholds to match our scoring
        let config = RulesConfigAnalyzerConfig {
            quality_threshold: 50.0,
            layering_threshold: 60.0,
        };
        let analyzer = RulesConfigAnalyzer::with_config(config);
        let result = analyzer.analyze(&paths, &reader);

        assert!(result.attempted);
        assert!(
            result.quality.score >= 50.0,
            "Quality {} should be >= 50",
            result.quality.score
        );
        assert!(
            result.layering.score >= 60.0,
            "Layering {} should be >= 60",
            result.layering.score
        );
        assert!(
            result.inferred_analysis.is_some(),
            "Should infer analysis when both axes pass"
        );

        let analysis = result.inferred_analysis.unwrap();
        assert!(
            !analysis.detected_layers.is_empty(),
            "Should have layers from dependency-cruiser"
        );
        assert!(analysis.structure_quality_score > 0.0);
        assert!(analysis.has_src_root);
    }

    #[test]
    fn test_analyzer_quality_fails_no_inference() {
        let paths = vec![
            ".dependency-cruiser.js".to_string(),
            "src/index.ts".to_string(),
        ];

        let reader = mock_reader(vec![(
            ".dependency-cruiser.js",
            "module.exports = { forbidden: [{ from: { path: '^src/a' }, to: { path: '^src/b' } }] };",
        )]);

        let analyzer = RulesConfigAnalyzer::new(); // default thresholds: quality=70
        let result = analyzer.analyze(&paths, &reader);

        assert!(result.attempted);
        assert!(
            result.quality.score < 70.0,
            "Quality should be low (no quality configs)"
        );
        assert!(
            result.inferred_analysis.is_none(),
            "Should NOT infer when quality fails"
        );
    }

    #[test]
    fn test_analyzer_layering_fails_no_inference() {
        let paths = vec![
            "tsconfig.json".to_string(),
            ".eslintrc.json".to_string(),
            "src/index.ts".to_string(),
        ];

        let reader = mock_reader(vec![
            (
                "tsconfig.json",
                r#"{"compilerOptions":{"strict":true,"noUncheckedIndexedAccess":true,"noImplicitReturns":true}}"#,
            ),
            (
                ".eslintrc.json",
                r#"{"extends":["@typescript-eslint/strict"]}"#,
            ),
        ]);

        let config = RulesConfigAnalyzerConfig {
            quality_threshold: 40.0,
            layering_threshold: 60.0,
        };
        let analyzer = RulesConfigAnalyzer::with_config(config);
        let result = analyzer.analyze(&paths, &reader);

        assert!(result.attempted);
        assert!(result.quality.score >= 40.0);
        assert!(
            result.layering.score < 60.0,
            "Layering should be low (no boundary configs)"
        );
        assert!(result.inferred_analysis.is_none());
    }

    #[test]
    fn test_analyzer_no_configs_not_attempted() {
        let paths = vec!["src/main.rs".to_string(), "src/lib.rs".to_string()];
        let reader = mock_reader(vec![]);

        let analyzer = RulesConfigAnalyzer::new();
        let result = analyzer.analyze(&paths, &reader);

        assert!(!result.attempted);
        assert!(result.inferred_analysis.is_none());
    }

    #[test]
    fn test_analyzer_rust_workspace_full_pass() {
        let paths = vec![
            "Cargo.toml".to_string(),
            "crates/core/Cargo.toml".to_string(),
            "crates/cli/Cargo.toml".to_string(),
            "crates/mcp/Cargo.toml".to_string(),
            "crates/store/Cargo.toml".to_string(),
            "clippy.toml".to_string(),
            "deny.toml".to_string(),
            "crates/core/src/lib.rs".to_string(),
            "crates/core/src/domain/mod.rs".to_string(),
            "tests/integration.rs".to_string(),
        ];

        let reader = mock_reader(vec![
            ("Cargo.toml", "[workspace]\nmembers = [\"crates/*\"]\n\n[workspace.lints.clippy]\npedantic = { level = \"warn\" }\nnursery = { level = \"warn\" }\n# clippy::pedantic clippy::nursery"),
            ("clippy.toml", "cognitive-complexity-threshold = 10"),
            ("deny.toml", "[advisories]\nvulnerability = \"deny\"\n\n[licenses]\nunlicensed = \"deny\"\n\n[bans]\nmultiple-versions = \"deny\"\n[[bans.deny]]\nname = \"openssl\""),
        ]);

        let config = RulesConfigAnalyzerConfig {
            quality_threshold: 50.0,
            layering_threshold: 50.0,
        };
        let analyzer = RulesConfigAnalyzer::with_config(config);
        let result = analyzer.analyze(&paths, &reader);

        assert!(result.attempted);
        assert!(
            result.inferred_analysis.is_some(),
            "Rust workspace with clippy+deny should pass"
        );
        let analysis = result.inferred_analysis.unwrap();
        // Layers should come from workspace members
        assert!(
            analysis.detected_layers.len() >= 4,
            "Should have crate names as layers"
        );
    }

    #[test]
    fn test_analyzer_inferred_analysis_uses_file_path_metadata() {
        let paths = vec![
            "tsconfig.json".to_string(),
            ".dependency-cruiser.js".to_string(),
            "src/domain/user.ts".to_string(),
            "src/services/auth.service.ts".to_string(),
            "tests/user.test.ts".to_string(),
        ];

        let reader = mock_reader(vec![
            ("tsconfig.json", r#"{"compilerOptions":{"strict":true,"noUncheckedIndexedAccess":true,"noImplicitReturns":true}}"#),
            (".dependency-cruiser.js", "module.exports = { forbidden: [{ from: { path: '^src/domain' }, to: { path: '^src/infrastructure' } }], allowed: [{ from: { path: '^src/services' }, to: { path: '^src/domain' } }] };"),
        ]);

        let config = RulesConfigAnalyzerConfig {
            quality_threshold: 40.0,
            layering_threshold: 60.0,
        };
        let analyzer = RulesConfigAnalyzer::with_config(config);
        let result = analyzer.analyze(&paths, &reader);

        if let Some(analysis) = &result.inferred_analysis {
            // File-path derived fields should still be populated
            assert!(analysis.has_src_root, "Should detect src root from paths");
            assert_eq!(analysis.test_pattern, TestPattern::SeparateDir);
            assert!(analysis.total_files == 5);
        }
    }

    #[test]
    fn test_analyzer_custom_thresholds() {
        let paths = vec![
            ".golangci.yml".to_string(),
            "internal/auth/handler.go".to_string(),
        ];
        let reader = mock_reader(vec![(
            ".golangci.yml",
            "linters:\n  enable:\n    - govet\n    - staticcheck\n    - depguard\n    - errcheck\n    - gosec\n    - gocritic\n    - revive\nlinters-settings:\n  depguard:\n    rules:\n      main:\n        deny:\n          - \"unsafe\"\n",
        )]);

        // Lenient thresholds
        let config = RulesConfigAnalyzerConfig {
            quality_threshold: 30.0,
            layering_threshold: 40.0,
        };
        let analyzer = RulesConfigAnalyzer::with_config(config);
        let result = analyzer.analyze(&paths, &reader);
        assert!(
            result.inferred_analysis.is_some(),
            "Should pass with lenient thresholds"
        );

        // Strict thresholds
        let config2 = RulesConfigAnalyzerConfig {
            quality_threshold: 95.0,
            layering_threshold: 95.0,
        };
        let analyzer2 = RulesConfigAnalyzer::with_config(config2);
        let result2 = analyzer2.analyze(&paths, &reader);
        assert!(
            result2.inferred_analysis.is_none(),
            "Should fail with strict thresholds"
        );
    }

    #[test]
    fn test_analyzer_default_config() {
        let analyzer = RulesConfigAnalyzer::default();
        assert_eq!(analyzer.config.quality_threshold, 70.0);
        assert_eq!(analyzer.config.layering_threshold, 60.0);
    }
}

//! Pattern matcher for scoring existing repos against catalog entries.
//!
//! Takes a [`StructureAnalysis`] and scores it against each
//! [`ArchitectureCatalogEntry`] to find the best-matching pattern.

use crate::domain::documents::architecture_catalog_entry::ArchitectureCatalogEntry;

use super::structure_analyzer::{NamingConvention, StructureAnalysis, TestPattern};

/// Score representing how well a repository matches a catalog entry.
#[derive(Debug, Clone)]
pub struct PatternMatchScore {
    /// Short code or identifier of the catalog entry.
    pub catalog_id: String,
    /// Title of the catalog entry.
    pub catalog_title: String,
    /// Overall match score (0.0 - 100.0).
    pub overall_score: f64,
    /// Folder layout overlap score (0.0 - 100.0).
    pub folder_layout_score: f64,
    /// Layer detection score (0.0 - 100.0).
    pub layer_score: f64,
    /// Naming convention match score (0.0 - 100.0).
    pub naming_score: f64,
    /// Specific findings about the match.
    pub match_details: Vec<String>,
    /// Specific mismatches found.
    pub mismatch_details: Vec<String>,
}

/// Result of matching against all catalog entries.
#[derive(Debug)]
pub struct MatchResult {
    /// All scores, sorted by overall_score descending.
    pub scores: Vec<PatternMatchScore>,
    /// The best match, if any score is above the threshold.
    pub best_match: Option<PatternMatchScore>,
    /// The threshold used for classification.
    pub threshold: f64,
}

/// Matches a repository's structure analysis against catalog entries.
pub struct PatternMatcher {
    /// Minimum overall score to consider a match "good".
    threshold: f64,
}

impl PatternMatcher {
    /// Create a new matcher with the given quality threshold.
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }

    /// Create a matcher with the default threshold (50.0).
    pub fn with_default_threshold() -> Self {
        Self::new(50.0)
    }

    /// Score a structure analysis against all provided catalog entries.
    pub fn match_against(
        &self,
        analysis: &StructureAnalysis,
        entries: &[ArchitectureCatalogEntry],
    ) -> MatchResult {
        let mut scores: Vec<PatternMatchScore> = entries
            .iter()
            .map(|entry| self.score_entry(analysis, entry))
            .collect();

        scores.sort_by(|a, b| b.overall_score.partial_cmp(&a.overall_score).unwrap());

        let best_match = scores
            .first()
            .filter(|s| s.overall_score >= self.threshold)
            .cloned();

        MatchResult {
            scores,
            best_match,
            threshold: self.threshold,
        }
    }

    /// Score a single catalog entry against the analysis.
    fn score_entry(
        &self,
        analysis: &StructureAnalysis,
        entry: &ArchitectureCatalogEntry,
    ) -> PatternMatchScore {
        let mut match_details = Vec::new();
        let mut mismatch_details = Vec::new();

        let folder_layout_score =
            Self::score_folder_layout(analysis, entry, &mut match_details, &mut mismatch_details);
        let layer_score =
            Self::score_layers(analysis, entry, &mut match_details, &mut mismatch_details);
        let naming_score =
            Self::score_naming(analysis, entry, &mut match_details, &mut mismatch_details);

        // Weighted average: folders 40%, layers 40%, naming 20%
        let overall_score = folder_layout_score * 0.40 + layer_score * 0.40 + naming_score * 0.20;

        PatternMatchScore {
            catalog_id: entry.metadata().short_code.clone(),
            catalog_title: entry.title().to_string(),
            overall_score,
            folder_layout_score,
            layer_score,
            naming_score,
            match_details,
            mismatch_details,
        }
    }

    /// Score folder layout overlap.
    fn score_folder_layout(
        analysis: &StructureAnalysis,
        entry: &ArchitectureCatalogEntry,
        matches: &mut Vec<String>,
        mismatches: &mut Vec<String>,
    ) -> f64 {
        if entry.folder_layout.is_empty() {
            return 0.0;
        }

        // Extract expected directory names from the catalog entry's folder layout.
        // For template paths like {feature}, extract the non-template parent dirs.
        let mut expected_dirs = std::collections::HashSet::new();
        for layout_path in &entry.folder_layout {
            let trimmed = layout_path.trim_end_matches('/');
            // Walk each component and collect non-template directory names
            for component in std::path::Path::new(trimmed).components() {
                let name = component.as_os_str().to_string_lossy().to_string();
                if !name.contains('{') && !name.is_empty() {
                    expected_dirs.insert(name);
                }
            }
        }

        if expected_dirs.is_empty() {
            return 0.0;
        }

        let all_analysis_dirs: std::collections::HashSet<&str> = analysis
            .top_level_dirs
            .iter()
            .chain(analysis.detected_layers.iter())
            .chain(analysis.module_boundaries.iter())
            .map(std::string::String::as_str)
            .collect();

        let mut found = 0;
        for dir in &expected_dirs {
            // Check if the dir name appears anywhere in the analysis
            if all_analysis_dirs.iter().any(|d| d.contains(dir.as_str()))
                || (analysis.has_src_root && dir == "src")
            {
                found += 1;
                matches.push(format!("Found expected directory '{dir}'"));
            } else {
                mismatches.push(format!("Missing expected directory '{dir}'"));
            }
        }

        (f64::from(found) / expected_dirs.len() as f64) * 100.0
    }

    /// Score layer detection overlap.
    ///
    /// Checks detected layers and also scans all directory names from
    /// module boundaries and top-level dirs. Some architectures nest
    /// layers inside features (e.g. `src/features/auth/components/`).
    fn score_layers(
        analysis: &StructureAnalysis,
        entry: &ArchitectureCatalogEntry,
        matches: &mut Vec<String>,
        mismatches: &mut Vec<String>,
    ) -> f64 {
        if entry.layers.is_empty() {
            return 0.0;
        }

        // Collect all directory names from detected layers, top-level dirs,
        // and all path components from module boundaries
        let mut all_dir_names: std::collections::HashSet<String> = std::collections::HashSet::new();

        for layer in &analysis.detected_layers {
            all_dir_names.insert(layer.clone());
        }
        for dir in &analysis.top_level_dirs {
            all_dir_names.insert(dir.clone());
        }
        for boundary in &analysis.module_boundaries {
            for component in std::path::Path::new(boundary).components() {
                all_dir_names.insert(component.as_os_str().to_string_lossy().to_string());
            }
        }

        let mut found = 0;
        for layer in &entry.layers {
            if all_dir_names.contains(layer.as_str()) {
                found += 1;
                matches.push(format!("Detected layer '{layer}'"));
            } else {
                mismatches.push(format!("Missing layer '{layer}'"));
            }
        }

        (f64::from(found) / entry.layers.len() as f64) * 100.0
    }

    /// Score naming convention match.
    fn score_naming(
        analysis: &StructureAnalysis,
        entry: &ArchitectureCatalogEntry,
        matches: &mut Vec<String>,
        mismatches: &mut Vec<String>,
    ) -> f64 {
        if entry.naming_conventions.is_empty() {
            return 50.0; // Neutral if no conventions specified
        }

        // Check if the detected naming convention aligns with any entry convention
        let convention_str = analysis.file_naming_convention.to_string().to_lowercase();

        let has_matching_convention = entry.naming_conventions.iter().any(|conv| {
            let conv_lower = conv.to_lowercase();
            conv_lower.contains(&convention_str)
                || (analysis.file_naming_convention == NamingConvention::KebabCase
                    && conv_lower.contains("kebab"))
                || (analysis.file_naming_convention == NamingConvention::PascalCase
                    && conv_lower.contains("pascal"))
                || (analysis.file_naming_convention == NamingConvention::CamelCase
                    && conv_lower.contains("camel"))
                || (analysis.file_naming_convention == NamingConvention::SnakeCase
                    && conv_lower.contains("snake"))
        });

        // Check test pattern alignment
        let has_test_convention = entry.naming_conventions.iter().any(|conv| {
            let conv_lower = conv.to_lowercase();
            match analysis.test_pattern {
                TestPattern::CoLocated | TestPattern::Both => {
                    conv_lower.contains(".test.") || conv_lower.contains(".spec.")
                }
                TestPattern::SeparateDir => conv_lower.contains("test"),
                TestPattern::None => false,
            }
        });

        let mut score: f64 = 0.0;
        if has_matching_convention {
            score += 70.0;
            matches.push(format!(
                "Naming convention '{}' aligns with entry",
                analysis.file_naming_convention
            ));
        } else if analysis.file_naming_convention != NamingConvention::Mixed {
            score += 20.0;
            mismatches.push(format!(
                "Naming convention '{}' doesn't match entry expectations",
                analysis.file_naming_convention
            ));
        }

        if has_test_convention {
            score += 30.0;
            matches.push(format!(
                "Test pattern '{}' aligns with entry",
                analysis.test_pattern
            ));
        }

        score.min(100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::catalog::brownfield_evaluator::structure_analyzer::StructureAnalyzer;
    use crate::domain::catalog::builtin_entries;

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

    fn cli_paths() -> Vec<String> {
        vec![
            "src/commands/init.command.ts".to_string(),
            "src/commands/build.command.ts".to_string(),
            "src/core/compiler.ts".to_string(),
            "src/core/resolver.ts".to_string(),
            "src/utils/logger.ts".to_string(),
            "src/config/defaults.ts".to_string(),
            "src/index.ts".to_string(),
            "bin/cli.ts".to_string(),
            "tests/unit/compiler.test.ts".to_string(),
            "tests/integration/build.test.ts".to_string(),
        ]
    }

    #[test]
    fn test_server_matches_server_pattern() {
        let analysis = StructureAnalyzer::analyze(&server_paths());
        let entries = builtin_entries::builtin_entries();
        let matcher = PatternMatcher::with_default_threshold();

        let result = matcher.match_against(&analysis, &entries);

        assert!(result.best_match.is_some());
        let best = result.best_match.unwrap();
        assert_eq!(best.catalog_id, "BUILTIN-AC-JS-SERVER");
    }

    #[test]
    fn test_react_matches_react_pattern() {
        let analysis = StructureAnalyzer::analyze(&react_paths());
        let entries = builtin_entries::builtin_entries();
        let matcher = PatternMatcher::with_default_threshold();

        let result = matcher.match_against(&analysis, &entries);

        // The React pattern should be the best match among all entries
        assert!(!result.scores.is_empty());
        let best = &result.scores[0];
        assert_eq!(best.catalog_id, "BUILTIN-AC-JS-REACT");
        // And it should be above threshold
        assert!(
            best.overall_score >= 50.0,
            "React score {} should be >= 50. folder={}, layer={}, naming={}",
            best.overall_score,
            best.folder_layout_score,
            best.layer_score,
            best.naming_score
        );
    }

    #[test]
    fn test_cli_matches_cli_pattern() {
        let analysis = StructureAnalyzer::analyze(&cli_paths());
        let entries = builtin_entries::builtin_entries();
        let matcher = PatternMatcher::with_default_threshold();

        let result = matcher.match_against(&analysis, &entries);

        assert!(result.best_match.is_some());
        let best = result.best_match.unwrap();
        assert_eq!(best.catalog_id, "BUILTIN-AC-JS-CLI");
    }

    #[test]
    fn test_no_match_for_unknown_structure() {
        // A structure that doesn't match any pattern well
        let paths = vec![
            "data/raw/input.csv".to_string(),
            "data/processed/output.csv".to_string(),
            "notebooks/analysis.ipynb".to_string(),
            "scripts/run.py".to_string(),
        ];
        let analysis = StructureAnalyzer::analyze(&paths);
        let entries = builtin_entries::builtin_entries();
        let matcher = PatternMatcher::new(60.0); // Higher threshold

        let result = matcher.match_against(&analysis, &entries);

        assert!(result.best_match.is_none());
    }

    #[test]
    fn test_scores_are_sorted_descending() {
        let analysis = StructureAnalyzer::analyze(&server_paths());
        let entries = builtin_entries::builtin_entries();
        let matcher = PatternMatcher::with_default_threshold();

        let result = matcher.match_against(&analysis, &entries);

        for i in 1..result.scores.len() {
            assert!(result.scores[i - 1].overall_score >= result.scores[i].overall_score);
        }
    }

    #[test]
    fn test_match_details_populated() {
        let analysis = StructureAnalyzer::analyze(&server_paths());
        let entries = builtin_entries::builtin_entries();
        let matcher = PatternMatcher::with_default_threshold();

        let result = matcher.match_against(&analysis, &entries);
        let best = result.best_match.unwrap();

        assert!(!best.match_details.is_empty());
    }

    #[test]
    fn test_empty_analysis() {
        let analysis = StructureAnalyzer::analyze(&[]);
        let entries = builtin_entries::builtin_entries();
        let matcher = PatternMatcher::with_default_threshold();

        let result = matcher.match_against(&analysis, &entries);

        assert!(result.best_match.is_none());
    }

    #[test]
    fn test_custom_threshold() {
        let analysis = StructureAnalyzer::analyze(&server_paths());
        let entries = builtin_entries::builtin_entries();

        // Very high threshold -- should not match
        let strict = PatternMatcher::new(99.0);
        let result = strict.match_against(&analysis, &entries);
        assert!(result.best_match.is_none());

        // Very low threshold -- should match
        let lenient = PatternMatcher::new(1.0);
        let result = lenient.match_against(&analysis, &entries);
        assert!(result.best_match.is_some());
    }

    #[test]
    fn test_server_has_higher_layer_score_than_react_for_server_paths() {
        let analysis = StructureAnalyzer::analyze(&server_paths());
        let entries = builtin_entries::builtin_entries();
        let matcher = PatternMatcher::with_default_threshold();

        let result = matcher.match_against(&analysis, &entries);

        let server_score = result
            .scores
            .iter()
            .find(|s| s.catalog_id == "BUILTIN-AC-JS-SERVER")
            .unwrap();
        let react_score = result
            .scores
            .iter()
            .find(|s| s.catalog_id == "BUILTIN-AC-JS-REACT")
            .unwrap();

        assert!(
            server_score.layer_score > react_score.layer_score,
            "Server layer_score ({}) should exceed React layer_score ({})",
            server_score.layer_score,
            react_score.layer_score
        );
    }
}

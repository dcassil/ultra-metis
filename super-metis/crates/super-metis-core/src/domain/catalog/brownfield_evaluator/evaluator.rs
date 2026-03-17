//! Brownfield architecture evaluator orchestrator.
//!
//! Combines [`StructureAnalyzer`] and [`PatternMatcher`] to evaluate an
//! existing repository and produce an [`EvaluationResult`] with one of
//! four outcomes:
//!
//! 1. **CatalogMatch** -- good architecture that matches a catalog entry
//! 2. **DerivedArchitecture** -- good architecture with no catalog match
//! 3. **RecommendCatalogPattern** -- bad architecture, recommends a pattern
//! 4. **RecordAsIs** -- user declined recommendation, record current state

use crate::domain::documents::architecture_catalog_entry::ArchitectureCatalogEntry;
use crate::domain::documents::content::DocumentContent;
use crate::domain::documents::metadata::DocumentMetadata;
use crate::domain::documents::reference_architecture::{ArchitectureStatus, ReferenceArchitecture};
use crate::domain::documents::types::{Phase, Tag};
use crate::domain::quality::types::{FindingEntry, MetricEntry, ParsedToolOutput, Severity};

use super::pattern_matcher::{MatchResult, PatternMatchScore, PatternMatcher};
use super::structure_analyzer::{StructureAnalysis, StructureAnalyzer};

/// The four possible outcomes of a brownfield evaluation.
#[derive(Debug)]
pub enum EvaluationOutcome {
    /// Good architecture that matches a known catalog pattern.
    CatalogMatch {
        /// The created reference architecture linked to the catalog entry.
        reference_architecture: ReferenceArchitecture,
        /// The matched catalog entry's short code.
        catalog_entry_id: String,
        /// Match score details.
        match_score: PatternMatchScore,
    },
    /// Good architecture that doesn't match any catalog pattern.
    DerivedArchitecture {
        /// The created derived reference architecture.
        reference_architecture: ReferenceArchitecture,
        /// The structure analysis that was captured.
        analysis: StructureAnalysis,
    },
    /// Bad architecture -- system recommends a catalog pattern.
    RecommendCatalogPattern {
        /// The recommended catalog entry's short code.
        recommended_entry_id: String,
        /// Title of the recommended entry.
        recommended_title: String,
        /// Quality findings explaining why the architecture is poor.
        findings: ParsedToolOutput,
        /// The structure analysis.
        analysis: StructureAnalysis,
        /// Match scores against all catalog entries.
        match_result: MatchResult,
    },
    /// User declined recommendation -- record architecture as-is.
    RecordAsIs {
        /// The reference architecture recorded as-is.
        reference_architecture: ReferenceArchitecture,
        /// Quality findings preserved for reference.
        findings: ParsedToolOutput,
    },
}

/// Result of a brownfield evaluation.
#[derive(Debug)]
pub struct EvaluationResult {
    /// The evaluation outcome.
    pub outcome: EvaluationOutcome,
    /// The structure analysis performed.
    pub structure_analysis: StructureAnalysis,
    /// Overall quality score (0-100).
    pub quality_score: f64,
}

/// Configuration for the brownfield evaluator.
#[derive(Debug, Clone)]
pub struct EvaluatorConfig {
    /// Quality threshold: repos scoring above this are "good architecture".
    pub quality_threshold: f64,
    /// Match threshold: minimum score to consider a catalog match valid.
    pub match_threshold: f64,
}

impl Default for EvaluatorConfig {
    fn default() -> Self {
        Self {
            quality_threshold: 70.0,
            match_threshold: 50.0,
        }
    }
}

/// Orchestrates brownfield architecture evaluation.
///
/// Takes file paths and catalog entries, runs analysis and matching,
/// and produces an [`EvaluationResult`].
pub struct BrownfieldEvaluator {
    config: EvaluatorConfig,
}

impl BrownfieldEvaluator {
    /// Create a new evaluator with default configuration.
    pub fn new() -> Self {
        Self {
            config: EvaluatorConfig::default(),
        }
    }

    /// Create a new evaluator with custom configuration.
    pub fn with_config(config: EvaluatorConfig) -> Self {
        Self { config }
    }

    /// Evaluate a repository's architecture.
    ///
    /// Returns an [`EvaluationResult`] indicating whether the repo has
    /// good architecture (matched or derived) or bad architecture
    /// (recommendation needed).
    pub fn evaluate(
        &self,
        file_paths: &[String],
        catalog_entries: &[ArchitectureCatalogEntry],
        short_code: String,
    ) -> EvaluationResult {
        // Step 1: Analyze structure
        let analysis = StructureAnalyzer::analyze(file_paths);

        // Step 2: Compute quality score
        let quality_score = analysis.structure_quality_score;

        // Step 3: Match against catalog
        let matcher = PatternMatcher::new(self.config.match_threshold);
        let match_result = matcher.match_against(&analysis, catalog_entries);

        // Step 4: Determine outcome
        let outcome = if quality_score >= self.config.quality_threshold {
            // Good architecture path
            if let Some(best_match) = &match_result.best_match {
                // Matched a catalog entry
                let ra = self.create_catalog_linked_ra(
                    &analysis,
                    best_match,
                    short_code,
                );
                EvaluationOutcome::CatalogMatch {
                    reference_architecture: ra,
                    catalog_entry_id: best_match.catalog_id.clone(),
                    match_score: best_match.clone(),
                }
            } else {
                // No catalog match -- derive from analysis
                let ra = self.create_derived_ra(&analysis, short_code);
                EvaluationOutcome::DerivedArchitecture {
                    reference_architecture: ra,
                    analysis: analysis.clone(),
                }
            }
        } else {
            // Bad architecture path -- recommend best catalog pattern
            let findings = self.produce_quality_findings(&analysis, quality_score);

            if let Some(best_match) = &match_result.best_match {
                EvaluationOutcome::RecommendCatalogPattern {
                    recommended_entry_id: best_match.catalog_id.clone(),
                    recommended_title: best_match.catalog_title.clone(),
                    findings,
                    analysis: analysis.clone(),
                    match_result,
                }
            } else if let Some(top_score) = match_result.scores.first() {
                // Even below threshold, recommend the closest match
                EvaluationOutcome::RecommendCatalogPattern {
                    recommended_entry_id: top_score.catalog_id.clone(),
                    recommended_title: top_score.catalog_title.clone(),
                    findings,
                    analysis: analysis.clone(),
                    match_result,
                }
            } else {
                // No catalog entries at all -- record as-is
                let ra = self.create_as_is_ra(&analysis, short_code);
                EvaluationOutcome::RecordAsIs {
                    reference_architecture: ra,
                    findings,
                }
            }
        };

        EvaluationResult {
            outcome,
            structure_analysis: analysis,
            quality_score,
        }
    }

    /// Accept a recommendation: create a catalog-linked reference architecture.
    ///
    /// Called when the user accepts the recommended catalog pattern for a
    /// bad-architecture repo.
    pub fn accept_recommendation(
        &self,
        analysis: &StructureAnalysis,
        recommended_score: &PatternMatchScore,
        short_code: String,
    ) -> ReferenceArchitecture {
        self.create_catalog_linked_ra(analysis, recommended_score, short_code)
    }

    /// Decline a recommendation: record the current architecture as-is.
    ///
    /// Called when the user declines the recommended catalog pattern.
    pub fn decline_recommendation(
        &self,
        analysis: &StructureAnalysis,
        quality_score: f64,
        short_code: String,
    ) -> (ReferenceArchitecture, ParsedToolOutput) {
        let ra = self.create_as_is_ra(analysis, short_code);
        let findings = self.produce_quality_findings(analysis, quality_score);
        (ra, findings)
    }

    // --- private helpers ---

    fn create_catalog_linked_ra(
        &self,
        analysis: &StructureAnalysis,
        match_score: &PatternMatchScore,
        short_code: String,
    ) -> ReferenceArchitecture {
        let title = format!(
            "Reference Architecture: {} (brownfield match)",
            match_score.catalog_title
        );

        let content_body = format!(
            "# {}\n\n\
             ## Source\n\n\
             Matched from catalog entry: {} (score: {:.1}%)\n\n\
             ## Detected Structure\n\n\
             - Top-level dirs: {}\n\
             - Detected layers: {}\n\
             - Naming convention: {}\n\
             - Test pattern: {}\n\n\
             ## Match Details\n\n{}\n\n\
             ## Mismatches\n\n{}",
            title,
            match_score.catalog_title,
            match_score.overall_score,
            analysis.top_level_dirs.join(", "),
            analysis.detected_layers.join(", "),
            analysis.file_naming_convention,
            analysis.test_pattern,
            match_score
                .match_details
                .iter()
                .map(|d| format!("- {}", d))
                .collect::<Vec<_>>()
                .join("\n"),
            match_score
                .mismatch_details
                .iter()
                .map(|d| format!("- {}", d))
                .collect::<Vec<_>>()
                .join("\n"),
        );

        ReferenceArchitecture::from_parts(
            title,
            DocumentMetadata::new(short_code),
            DocumentContent::new(&content_body),
            vec![
                Tag::Label("reference_architecture".to_string()),
                Tag::Phase(Phase::Draft),
            ],
            false,
            Some(match_score.catalog_id.clone()),
            false,
            ArchitectureStatus::Draft,
            analysis.detected_layers.clone(),
            analysis.module_boundaries.clone(),
            vec![],
            None,
            None,
            vec![],
        )
    }

    fn create_derived_ra(
        &self,
        analysis: &StructureAnalysis,
        short_code: String,
    ) -> ReferenceArchitecture {
        let title = "Reference Architecture: Derived from existing structure".to_string();

        let content_body = format!(
            "# {}\n\n\
             ## Source\n\n\
             Derived from existing repository structure (no catalog match).\n\n\
             ## Detected Structure\n\n\
             - Top-level dirs: {}\n\
             - Detected layers: {}\n\
             - Naming convention: {}\n\
             - Test pattern: {}\n\
             - Module boundaries: {}\n\
             - Quality score: {:.1}",
            title,
            analysis.top_level_dirs.join(", "),
            analysis.detected_layers.join(", "),
            analysis.file_naming_convention,
            analysis.test_pattern,
            analysis.module_boundaries.join(", "),
            analysis.structure_quality_score,
        );

        ReferenceArchitecture::from_parts(
            title,
            DocumentMetadata::new(short_code),
            DocumentContent::new(&content_body),
            vec![
                Tag::Label("reference_architecture".to_string()),
                Tag::Phase(Phase::Draft),
            ],
            false,
            None,
            true,
            ArchitectureStatus::Draft,
            analysis.detected_layers.clone(),
            analysis.module_boundaries.clone(),
            vec![],
            None,
            None,
            vec![],
        )
    }

    fn create_as_is_ra(
        &self,
        analysis: &StructureAnalysis,
        short_code: String,
    ) -> ReferenceArchitecture {
        let title = "Reference Architecture: Recorded as-is".to_string();

        let content_body = format!(
            "# {}\n\n\
             ## Source\n\n\
             Current architecture recorded as-is (recommendation declined or no catalog entries).\n\n\
             ## Detected Structure\n\n\
             - Top-level dirs: {}\n\
             - Detected layers: {}\n\
             - Naming convention: {}\n\
             - Test pattern: {}\n\
             - Quality score: {:.1}",
            title,
            analysis.top_level_dirs.join(", "),
            analysis.detected_layers.join(", "),
            analysis.file_naming_convention,
            analysis.test_pattern,
            analysis.structure_quality_score,
        );

        ReferenceArchitecture::from_parts(
            title,
            DocumentMetadata::new(short_code),
            DocumentContent::new(&content_body),
            vec![
                Tag::Label("reference_architecture".to_string()),
                Tag::Phase(Phase::Draft),
            ],
            false,
            None,
            true,
            ArchitectureStatus::Draft,
            analysis.detected_layers.clone(),
            analysis.module_boundaries.clone(),
            vec![],
            None,
            None,
            vec![],
        )
    }

    fn produce_quality_findings(
        &self,
        analysis: &StructureAnalysis,
        quality_score: f64,
    ) -> ParsedToolOutput {
        let mut output = ParsedToolOutput::new("brownfield_evaluation");

        // Quality score metric
        output.metrics.push(MetricEntry::new(
            "structure_quality_score",
            quality_score,
            "percent",
        ));
        output.metrics.push(MetricEntry::new(
            "total_files",
            analysis.total_files as f64,
            "count",
        ));
        output.metrics.push(MetricEntry::new(
            "detected_layers",
            analysis.detected_layers.len() as f64,
            "count",
        ));

        // Findings based on analysis
        if analysis.detected_layers.is_empty() {
            output.findings.push(FindingEntry::new(
                "no_layers_detected",
                Severity::Warning,
                "No architectural layers detected. Consider organizing code into layers.",
                "repository",
            ));
        }

        if analysis.test_pattern == super::structure_analyzer::TestPattern::None {
            output.findings.push(FindingEntry::new(
                "no_tests_detected",
                Severity::Warning,
                "No test files detected. Test coverage is important for maintainability.",
                "repository",
            ));
        }

        if analysis.file_naming_convention == super::structure_analyzer::NamingConvention::Mixed {
            output.findings.push(FindingEntry::new(
                "inconsistent_naming",
                Severity::Warning,
                "File naming convention is inconsistent. Consider standardizing.",
                "repository",
            ));
        }

        if !analysis.has_src_root && analysis.total_files > 5 {
            output.findings.push(FindingEntry::new(
                "no_src_root",
                Severity::Info,
                "No 'src/' root directory. Consider organizing source under src/.",
                "repository",
            ));
        }

        if analysis.module_boundaries.is_empty() && analysis.total_files > 10 {
            output.findings.push(FindingEntry::new(
                "no_module_boundaries",
                Severity::Info,
                "No barrel/index files detected. Module boundaries help define public APIs.",
                "repository",
            ));
        }

        output
            .summary
            .insert("quality_score".to_string(), quality_score);
        output.summary.insert(
            "quality_threshold".to_string(),
            self.config.quality_threshold,
        );

        output
    }
}

impl Default for BrownfieldEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn poor_paths() -> Vec<String> {
        vec![
            "app.js".to_string(),
            "server.js".to_string(),
            "database.js".to_string(),
            "utils.js".to_string(),
            "helpers.js".to_string(),
        ]
    }

    fn custom_good_paths() -> Vec<String> {
        // A well-organized repo that doesn't match any JS catalog pattern
        vec![
            "src/domain/models/user.py".to_string(),
            "src/domain/models/order.py".to_string(),
            "src/infrastructure/database.py".to_string(),
            "src/infrastructure/cache.py".to_string(),
            "src/application/user_service.py".to_string(),
            "src/application/order_service.py".to_string(),
            "src/presentation/api.py".to_string(),
            "src/shared/utils.py".to_string(),
            "tests/test_user_service.py".to_string(),
            "tests/test_order_service.py".to_string(),
        ]
    }

    #[test]
    fn test_good_architecture_catalog_match() {
        let evaluator = BrownfieldEvaluator::new();
        let entries = builtin_entries::builtin_entries();

        let result = evaluator.evaluate(&server_paths(), &entries, "RA-BF-001".to_string());

        assert!(result.quality_score >= 70.0, "Score: {}", result.quality_score);
        match &result.outcome {
            EvaluationOutcome::CatalogMatch {
                catalog_entry_id,
                match_score,
                ..
            } => {
                assert_eq!(catalog_entry_id, "BUILTIN-AC-JS-SERVER");
                assert!(match_score.overall_score >= 50.0);
            }
            other => panic!("Expected CatalogMatch, got {:?}", std::mem::discriminant(other)),
        }
    }

    #[test]
    fn test_good_architecture_derived() {
        // Use config with lower quality threshold so the Python paths count as "good"
        let config = EvaluatorConfig {
            quality_threshold: 60.0,
            match_threshold: 70.0, // High match threshold so nothing matches
        };
        let evaluator = BrownfieldEvaluator::with_config(config);
        let entries = builtin_entries::builtin_entries();

        let result = evaluator.evaluate(&custom_good_paths(), &entries, "RA-BF-002".to_string());

        match &result.outcome {
            EvaluationOutcome::DerivedArchitecture {
                reference_architecture,
                ..
            } => {
                assert!(reference_architecture.is_derived);
                assert!(reference_architecture.source_catalog_ref.is_none());
            }
            other => panic!(
                "Expected DerivedArchitecture, got {:?}",
                std::mem::discriminant(other)
            ),
        }
    }

    #[test]
    fn test_bad_architecture_recommendation() {
        let evaluator = BrownfieldEvaluator::new();
        let entries = builtin_entries::builtin_entries();

        let result = evaluator.evaluate(&poor_paths(), &entries, "RA-BF-003".to_string());

        assert!(result.quality_score < 70.0);
        match &result.outcome {
            EvaluationOutcome::RecommendCatalogPattern {
                findings,
                ..
            } => {
                assert!(!findings.findings.is_empty());
                assert!(findings.summary.contains_key("quality_score"));
            }
            other => panic!(
                "Expected RecommendCatalogPattern, got {:?}",
                std::mem::discriminant(other)
            ),
        }
    }

    #[test]
    fn test_bad_architecture_no_catalog_entries() {
        let evaluator = BrownfieldEvaluator::new();
        let entries: Vec<ArchitectureCatalogEntry> = vec![];

        let result = evaluator.evaluate(&poor_paths(), &entries, "RA-BF-004".to_string());

        match &result.outcome {
            EvaluationOutcome::RecordAsIs {
                reference_architecture,
                findings,
            } => {
                assert!(reference_architecture.is_derived);
                assert!(!findings.findings.is_empty());
            }
            other => panic!(
                "Expected RecordAsIs, got {:?}",
                std::mem::discriminant(other)
            ),
        }
    }

    #[test]
    fn test_decline_recommendation() {
        let evaluator = BrownfieldEvaluator::new();
        let analysis = StructureAnalyzer::analyze(&poor_paths());

        let (ra, findings) =
            evaluator.decline_recommendation(&analysis, 40.0, "RA-BF-005".to_string());

        assert!(ra.is_derived);
        assert!(ra.title().contains("as-is"));
        assert!(!findings.findings.is_empty());
    }

    #[test]
    fn test_accept_recommendation() {
        let evaluator = BrownfieldEvaluator::new();
        let entries = builtin_entries::builtin_entries();
        let analysis = StructureAnalyzer::analyze(&poor_paths());
        let matcher = PatternMatcher::new(0.0); // Accept any match
        let match_result = matcher.match_against(&analysis, &entries);

        let best = &match_result.scores[0];
        let ra = evaluator.accept_recommendation(&analysis, best, "RA-BF-006".to_string());

        assert!(!ra.is_derived);
        assert!(ra.source_catalog_ref.is_some());
        assert!(ra.is_catalog_linked());
    }

    #[test]
    fn test_quality_findings_content() {
        let evaluator = BrownfieldEvaluator::new();
        let entries = builtin_entries::builtin_entries();

        let result = evaluator.evaluate(&poor_paths(), &entries, "RA-BF-007".to_string());

        if let EvaluationOutcome::RecommendCatalogPattern { findings, .. } = &result.outcome {
            // Should have findings about missing layers, tests, etc.
            let finding_ids: Vec<&str> =
                findings.findings.iter().map(|f| f.rule_id.as_str()).collect();
            assert!(
                finding_ids.contains(&"no_tests_detected"),
                "Expected no_tests_detected finding"
            );
        }
    }

    #[test]
    fn test_evaluator_default() {
        let evaluator = BrownfieldEvaluator::default();
        assert_eq!(evaluator.config.quality_threshold, 70.0);
        assert_eq!(evaluator.config.match_threshold, 50.0);
    }

    #[test]
    fn test_catalog_match_ra_has_content() {
        let evaluator = BrownfieldEvaluator::new();
        let entries = builtin_entries::builtin_entries();

        let result = evaluator.evaluate(&server_paths(), &entries, "RA-BF-008".to_string());

        if let EvaluationOutcome::CatalogMatch {
            reference_architecture,
            ..
        } = &result.outcome
        {
            let body = &reference_architecture.content().body;
            assert!(body.contains("Detected Structure"));
            assert!(body.contains("Match Details"));
        } else {
            panic!("Expected CatalogMatch");
        }
    }

    #[test]
    fn test_empty_file_paths() {
        let evaluator = BrownfieldEvaluator::new();
        let entries = builtin_entries::builtin_entries();

        let result = evaluator.evaluate(&[], &entries, "RA-BF-009".to_string());

        assert_eq!(result.structure_analysis.total_files, 0);
        // Empty repo should have low quality score
        assert!(result.quality_score < 70.0);
    }
}

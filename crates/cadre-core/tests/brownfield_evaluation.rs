//! Integration tests for the brownfield architecture evaluation system.
//!
//! These tests exercise the full flow across modules:
//! StructureAnalyzer → PatternMatcher → BrownfieldEvaluator → ReferenceArchitecture

use cadre_core::{
    builtin_entries, BrownfieldEvaluator, EvaluationOutcome, EvaluatorConfig, NamingConvention,
    PatternMatcher, StructureAnalyzer, TestPattern,
};

/// Simulates a well-structured Express server repo.
fn express_server_repo() -> Vec<String> {
    vec![
        "src/routes/users.route.ts".to_string(),
        "src/routes/auth.route.ts".to_string(),
        "src/routes/products.route.ts".to_string(),
        "src/handlers/users.handler.ts".to_string(),
        "src/handlers/auth.handler.ts".to_string(),
        "src/handlers/products.handler.ts".to_string(),
        "src/services/users.service.ts".to_string(),
        "src/services/auth.service.ts".to_string(),
        "src/services/products.service.ts".to_string(),
        "src/repositories/users.repository.ts".to_string(),
        "src/repositories/products.repository.ts".to_string(),
        "src/middleware/auth.ts".to_string(),
        "src/middleware/rate-limit.ts".to_string(),
        "src/config/database.ts".to_string(),
        "src/config/app.ts".to_string(),
        "src/types/user.ts".to_string(),
        "src/types/product.ts".to_string(),
        "src/index.ts".to_string(),
        "tests/integration/users.test.ts".to_string(),
        "tests/integration/auth.test.ts".to_string(),
        "tests/unit/auth.service.test.ts".to_string(),
        "tests/unit/users.service.test.ts".to_string(),
        "package.json".to_string(),
        "tsconfig.json".to_string(),
    ]
}

/// Simulates a feature-based React SPA.
fn react_spa_repo() -> Vec<String> {
    vec![
        "src/features/auth/components/LoginForm.tsx".to_string(),
        "src/features/auth/components/LoginForm.test.tsx".to_string(),
        "src/features/auth/hooks/useAuth.ts".to_string(),
        "src/features/auth/services/authApi.ts".to_string(),
        "src/features/auth/types.ts".to_string(),
        "src/features/auth/index.ts".to_string(),
        "src/features/dashboard/components/DashboardView.tsx".to_string(),
        "src/features/dashboard/components/DashboardView.test.tsx".to_string(),
        "src/features/dashboard/hooks/useDashboard.ts".to_string(),
        "src/features/dashboard/index.ts".to_string(),
        "src/features/settings/components/SettingsPage.tsx".to_string(),
        "src/features/settings/index.ts".to_string(),
        "src/shared/components/Button.tsx".to_string(),
        "src/shared/components/Modal.tsx".to_string(),
        "src/shared/hooks/useTheme.ts".to_string(),
        "src/shared/utils/format.ts".to_string(),
        "src/shared/utils/validate.ts".to_string(),
        "src/app/App.tsx".to_string(),
        "src/app/routes.tsx".to_string(),
        "src/index.tsx".to_string(),
        "package.json".to_string(),
    ]
}

/// Simulates a poorly-organized "everything in root" project.
fn messy_repo() -> Vec<String> {
    vec![
        "app.js".to_string(),
        "server.js".to_string(),
        "database.js".to_string(),
        "auth.js".to_string(),
        "utils.js".to_string(),
        "helpers.js".to_string(),
        "config.js".to_string(),
        "types.js".to_string(),
        "api.js".to_string(),
        "middleware.js".to_string(),
    ]
}

/// Full flow: Express server → analyze → match → catalog-linked RA
#[test]
fn test_full_flow_express_server_catalog_match() {
    let evaluator = BrownfieldEvaluator::new();
    let entries = builtin_entries::builtin_entries();

    let result = evaluator.evaluate(&express_server_repo(), &entries, "RA-INT-001".to_string());

    // Should be good quality
    assert!(
        result.quality_score >= 70.0,
        "Expected quality >= 70, got {}",
        result.quality_score
    );

    // Should match the server catalog entry
    match &result.outcome {
        EvaluationOutcome::CatalogMatch {
            reference_architecture,
            catalog_entry_id,
            match_score,
        } => {
            assert_eq!(catalog_entry_id, "BUILTIN-AC-JS-SERVER");
            assert!(match_score.overall_score >= 50.0);
            assert!(!reference_architecture.is_derived);
            assert!(reference_architecture.is_catalog_linked());
            assert_eq!(
                reference_architecture.source_catalog_ref.as_deref(),
                Some("BUILTIN-AC-JS-SERVER")
            );

            // RA should serialize and deserialize
            let serialized = reference_architecture.to_content().unwrap();
            let loaded = cadre_core::ReferenceArchitecture::from_content(&serialized).unwrap();
            assert_eq!(loaded.title(), reference_architecture.title());
            assert_eq!(
                loaded.source_catalog_ref,
                reference_architecture.source_catalog_ref
            );
        }
        other => panic!(
            "Expected CatalogMatch, got {:?}",
            std::mem::discriminant(other)
        ),
    }
}

/// Full flow: React SPA → analyze → match → catalog-linked RA
#[test]
fn test_full_flow_react_spa_catalog_match() {
    let evaluator = BrownfieldEvaluator::new();
    let entries = builtin_entries::builtin_entries();

    let result = evaluator.evaluate(&react_spa_repo(), &entries, "RA-INT-002".to_string());

    assert!(result.quality_score >= 70.0);

    match &result.outcome {
        EvaluationOutcome::CatalogMatch {
            catalog_entry_id, ..
        } => {
            assert_eq!(catalog_entry_id, "BUILTIN-AC-JS-REACT");
        }
        other => panic!(
            "Expected CatalogMatch, got {:?}",
            std::mem::discriminant(other)
        ),
    }
}

/// Full flow: messy repo → analyze → recommend → decline → record as-is
#[test]
fn test_full_flow_messy_repo_recommend_then_decline() {
    let evaluator = BrownfieldEvaluator::new();
    let entries = builtin_entries::builtin_entries();

    let result = evaluator.evaluate(&messy_repo(), &entries, "RA-INT-003".to_string());

    // Should be poor quality
    assert!(result.quality_score < 70.0);

    match &result.outcome {
        EvaluationOutcome::RecommendCatalogPattern {
            recommended_entry_id,
            findings,
            analysis,
            ..
        } => {
            // Should have quality findings
            assert!(!findings.findings.is_empty());
            assert!(findings.summary.contains_key("quality_score"));

            // Simulate user declining
            let (ra, decline_findings) = evaluator.decline_recommendation(
                analysis,
                result.quality_score,
                "RA-INT-003-DECLINED".to_string(),
            );

            assert!(ra.is_derived);
            assert!(ra.title().contains("as-is"));
            assert!(!decline_findings.findings.is_empty());

            // Now simulate accepting instead
            let matcher = PatternMatcher::new(0.0);
            let match_result = matcher.match_against(analysis, &entries);
            let best = &match_result.scores[0];

            let accepted_ra =
                evaluator.accept_recommendation(analysis, best, "RA-INT-003-ACCEPTED".to_string());

            assert!(!accepted_ra.is_derived);
            assert!(accepted_ra.is_catalog_linked());
            assert_eq!(
                accepted_ra.source_catalog_ref.as_deref(),
                Some(recommended_entry_id.as_str())
            );
        }
        other => panic!(
            "Expected RecommendCatalogPattern, got {:?}",
            std::mem::discriminant(other)
        ),
    }
}

/// Verify structure analysis produces correct details for a real-ish repo.
#[test]
fn test_structure_analysis_details() {
    let analysis = StructureAnalyzer::analyze(&express_server_repo());

    assert!(analysis.has_src_root);
    assert!(analysis.top_level_dirs.contains(&"src".to_string()));
    assert!(analysis.top_level_dirs.contains(&"tests".to_string()));

    // Should detect multiple layers
    assert!(analysis.detected_layers.contains(&"routes".to_string()));
    assert!(analysis.detected_layers.contains(&"handlers".to_string()));
    assert!(analysis.detected_layers.contains(&"services".to_string()));
    assert!(analysis
        .detected_layers
        .contains(&"repositories".to_string()));
    assert!(analysis.detected_layers.contains(&"middleware".to_string()));

    // Naming should be kebab-case (kebab file names like rate-limit.ts)
    assert_eq!(analysis.file_naming_convention, NamingConvention::KebabCase);

    // Should detect tests in separate directory
    assert!(matches!(
        analysis.test_pattern,
        TestPattern::SeparateDir | TestPattern::Both
    ));
}

/// Verify pattern matcher correctly ranks entries.
#[test]
fn test_pattern_matcher_ranking() {
    let analysis = StructureAnalyzer::analyze(&express_server_repo());
    let entries = builtin_entries::builtin_entries();
    let matcher = PatternMatcher::with_default_threshold();

    let result = matcher.match_against(&analysis, &entries);

    // Server should be best match
    assert_eq!(result.scores[0].catalog_id, "BUILTIN-AC-JS-SERVER");

    // All scores should be present
    assert_eq!(result.scores.len(), 5);

    // Scores should be descending
    for window in result.scores.windows(2) {
        assert!(window[0].overall_score >= window[1].overall_score);
    }
}

/// Custom evaluator config changes behavior.
#[test]
fn test_custom_config_thresholds() {
    let entries = builtin_entries::builtin_entries();

    // Very strict quality threshold -- even good repos are "bad"
    let strict = BrownfieldEvaluator::with_config(EvaluatorConfig {
        quality_threshold: 99.0,
        match_threshold: 50.0,
    });
    let result = strict.evaluate(&express_server_repo(), &entries, "RA-INT-004".to_string());
    assert!(matches!(
        result.outcome,
        EvaluationOutcome::RecommendCatalogPattern { .. }
    ));

    // Very lenient quality threshold with express server -- should still match catalog
    let lenient = BrownfieldEvaluator::with_config(EvaluatorConfig {
        quality_threshold: 10.0,
        match_threshold: 10.0,
    });
    let result = lenient.evaluate(&express_server_repo(), &entries, "RA-INT-005".to_string());
    assert!(matches!(
        result.outcome,
        EvaluationOutcome::CatalogMatch { .. }
    ));

    // Messy repo with lenient quality but high match threshold → derived
    let lenient_quality = BrownfieldEvaluator::with_config(EvaluatorConfig {
        quality_threshold: 1.0,
        match_threshold: 99.0,
    });
    let result = lenient_quality.evaluate(&messy_repo(), &entries, "RA-INT-006".to_string());
    assert!(matches!(
        result.outcome,
        EvaluationOutcome::DerivedArchitecture { .. }
    ));
}

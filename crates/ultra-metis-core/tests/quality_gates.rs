//! Integration tests for the quality gate system.
//!
//! These tests exercise the full flow across modules:
//! QualityGateConfig → GateCheckEngine → GateCheckResult → GateOverride → GateOverrideAuditEntry

use std::collections::HashMap;
use ultra_metis_core::Phase;
use ultra_metis_core::Tag;
use ultra_metis_core::{
    GateCheckEngine, GateOverride, GateOverrideAuditEntry, GateSeverity, MetricGateRule,
    OverrideType, QualityGateConfig, ThresholdType, TransitionGateConfig, TrendRequirement,
};

fn metrics(pairs: &[(&str, f64)]) -> HashMap<String, f64> {
    pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

/// Full flow: config → check → all pass
#[test]
fn test_full_flow_all_gates_pass() {
    let config = QualityGateConfig::new(
        "CI Quality Gates".to_string(),
        vec![Tag::Phase(Phase::Draft)],
        false,
        "QGC-0001".to_string(),
        GateSeverity::Blocking,
        vec![
            MetricGateRule::blocking_absolute("lint_errors", 0.0),
            MetricGateRule::blocking_relative("total_warnings", 10.0),
            MetricGateRule::advisory_absolute("info_count", 100.0),
        ],
        vec![],
    )
    .unwrap();

    let current = metrics(&[
        ("lint_errors", 0.0),
        ("total_warnings", 5.0),
        ("info_count", 42.0),
    ]);
    let baseline = metrics(&[("total_warnings", 5.0)]);

    let result = GateCheckEngine::check(
        &current,
        &config,
        Some("ready"),
        Some("active"),
        Some(&baseline),
        None,
    );

    assert!(result.passed);
    assert_eq!(result.total_failures(), 0);
    assert_eq!(result.failure_summary(), "All quality gates passed.");
}

/// Full flow: config → check → failure → override → audit entry
#[test]
fn test_full_flow_failure_to_override() {
    let config = QualityGateConfig::new(
        "Strict Gates".to_string(),
        vec![Tag::Phase(Phase::Draft)],
        false,
        "QGC-0002".to_string(),
        GateSeverity::Blocking,
        vec![
            MetricGateRule::blocking_absolute("lint_errors", 0.0),
            MetricGateRule::blocking_absolute("type_errors", 0.0),
        ],
        vec![],
    )
    .unwrap();

    let current = metrics(&[("lint_errors", 3.0), ("type_errors", 0.0)]);

    // Step 1: Check gates — should fail
    let result = GateCheckEngine::check(&current, &config, None, None, None, None);
    assert!(!result.passed);
    assert_eq!(result.blocking_failures.len(), 1);
    assert_eq!(result.blocking_failures[0].metric, "lint_errors");

    // Step 2: Create override decision
    let failed_metrics: Vec<String> = result
        .blocking_failures
        .iter()
        .map(|f| f.metric.clone())
        .collect();

    let gate_override = GateOverride::new(
        "deploy-bot",
        "Critical security patch, lint issues are cosmetic",
        failed_metrics,
        OverrideType::Emergency,
    );
    assert!(gate_override.validate().is_ok());

    // Step 3: Create audit entry from override
    let failure_details = result.failure_summary();
    let audit_entry = GateOverrideAuditEntry::from_override(
        &gate_override,
        "GOA-0001".to_string(),
        Some("QR-0001".to_string()),
        Some("QGC-0002".to_string()),
        &failure_details,
    )
    .unwrap();

    assert!(audit_entry.validate().is_ok());
    assert_eq!(audit_entry.overrider, "deploy-bot");
    assert_eq!(audit_entry.override_type, OverrideType::Emergency);
    assert_eq!(audit_entry.gates_bypassed, vec!["lint_errors"]);
    assert_eq!(
        audit_entry.linked_quality_record.as_deref(),
        Some("QR-0001")
    );

    // Step 4: Verify audit entry round-trips
    let content = audit_entry.to_content().unwrap();
    let reloaded = GateOverrideAuditEntry::from_content(&content).unwrap();
    assert_eq!(reloaded.overrider, "deploy-bot");
    assert_eq!(reloaded.gates_bypassed, vec!["lint_errors"]);
    assert_eq!(reloaded.override_type, OverrideType::Emergency);
}

/// Transition-specific overrides with stricter thresholds
#[test]
fn test_transition_specific_gates() {
    let config = QualityGateConfig::new(
        "Progressive Gates".to_string(),
        vec![Tag::Phase(Phase::Draft)],
        false,
        "QGC-0003".to_string(),
        GateSeverity::Blocking,
        vec![
            MetricGateRule::blocking_absolute("lint_errors", 10.0), // lenient default
            MetricGateRule::advisory_absolute("warnings", 50.0),
        ],
        vec![TransitionGateConfig::new(
            "active",
            "completed",
            vec![
                MetricGateRule::blocking_absolute("lint_errors", 0.0), // strict for completion
                MetricGateRule::blocking_absolute("warnings", 0.0),    // promoted to blocking
            ],
        )],
    )
    .unwrap();

    let current = metrics(&[("lint_errors", 5.0), ("warnings", 10.0)]);

    // ready→active: uses defaults, 5 lint errors < 10 threshold — passes
    let ready_active =
        GateCheckEngine::check(&current, &config, Some("ready"), Some("active"), None, None);
    assert!(ready_active.passed);

    // active→completed: uses overrides, 5 lint errors > 0 threshold — fails
    let active_completed = GateCheckEngine::check(
        &current,
        &config,
        Some("active"),
        Some("completed"),
        None,
        None,
    );
    assert!(!active_completed.passed);
    assert_eq!(active_completed.blocking_failures.len(), 2);
}

/// Mixed threshold types in a single config
#[test]
fn test_mixed_threshold_types() {
    let config = QualityGateConfig::new(
        "Mixed Gates".to_string(),
        vec![Tag::Phase(Phase::Draft)],
        false,
        "QGC-0004".to_string(),
        GateSeverity::Blocking,
        vec![
            MetricGateRule::blocking_absolute("lint_errors", 5.0),
            MetricGateRule::blocking_relative("total_warnings", 10.0),
            MetricGateRule::new(
                "type_errors",
                ThresholdType::Trend(TrendRequirement::NotRegressing),
                GateSeverity::Blocking,
            ),
            MetricGateRule::advisory_absolute("info_count", 100.0),
        ],
        vec![],
    )
    .unwrap();

    let current = metrics(&[
        ("lint_errors", 3.0),
        ("total_warnings", 12.0),
        ("type_errors", 2.0),
        ("info_count", 150.0), // advisory, won't block
    ]);
    let baseline = metrics(&[("total_warnings", 10.0)]);
    let history = vec![metrics(&[("type_errors", 2.0)])]; // stable — ok

    let result = GateCheckEngine::check(
        &current,
        &config,
        None,
        None,
        Some(&baseline),
        Some(&history),
    );

    // lint_errors: 3 <= 5 ✓
    // total_warnings: 20% regression (10→12), threshold 10% ✗
    // type_errors: stable trend ✓
    // info_count: 150 > 100, but advisory ✓
    assert!(!result.passed);
    assert_eq!(result.blocking_failures.len(), 1);
    assert_eq!(result.blocking_failures[0].metric, "total_warnings");
    assert_eq!(result.advisory_failures.len(), 1);
    assert_eq!(result.advisory_failures[0].metric, "info_count");
}

/// Config round-trip with complex structure
#[tokio::test]
async fn test_config_roundtrip_complex() {
    let config = QualityGateConfig::new(
        "Complex Config".to_string(),
        vec![Tag::Phase(Phase::Draft), Tag::Label("ci".to_string())],
        false,
        "QGC-0005".to_string(),
        GateSeverity::Advisory,
        vec![
            MetricGateRule::blocking_absolute("lint_errors", 0.0),
            MetricGateRule::blocking_relative("coverage_regression", 2.5),
            MetricGateRule::advisory_absolute("warnings", 25.0),
        ],
        vec![
            TransitionGateConfig::new(
                "active",
                "completed",
                vec![
                    MetricGateRule::blocking_absolute("lint_errors", 0.0),
                    MetricGateRule::blocking_absolute("warnings", 0.0),
                ],
            ),
            TransitionGateConfig::new(
                "ready",
                "active",
                vec![MetricGateRule::advisory_absolute("lint_errors", 5.0)],
            ),
        ],
    )
    .unwrap();

    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("complex-config.md");

    config.to_file(&path).await.unwrap();
    let loaded = QualityGateConfig::from_file(&path).await.unwrap();

    assert_eq!(loaded.title(), "Complex Config");
    assert_eq!(loaded.gate_severity_default, GateSeverity::Advisory);
    assert_eq!(loaded.default_thresholds.len(), 3);
    assert_eq!(loaded.transition_overrides.len(), 2);

    // Verify override resolution works on loaded config
    let active_completed = loaded.thresholds_for_transition("active", "completed");
    assert_eq!(active_completed.len(), 2);

    let ready_active = loaded.thresholds_for_transition("ready", "active");
    assert_eq!(ready_active.len(), 1);

    // Unknown transition falls back to defaults
    let unknown = loaded.thresholds_for_transition("design", "ready");
    assert_eq!(unknown.len(), 3);
}

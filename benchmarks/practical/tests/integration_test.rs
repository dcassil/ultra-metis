use chrono::Utc;
use practical_benchmark::*;

/// Skip a test if ANTHROPIC_API_KEY is not set (requires live API).
macro_rules! require_api_key {
    () => {
        if std::env::var("ANTHROPIC_API_KEY").is_err() {
            eprintln!("Skipping: ANTHROPIC_API_KEY not set");
            return;
        }
    };
}

/// Live integration test — runs only when ANTHROPIC_API_KEY is available.
#[tokio::test]
async fn test_autonomous_runner_returns_valid_scaffold() {
    require_api_key!();

    let scenario_path = std::path::PathBuf::from("scenario");
    let results_dir = std::path::PathBuf::from("target/test-results");

    let harness = BenchmarkHarness::new(scenario_path, results_dir);
    let run = harness.run_autonomous().await.unwrap();

    assert_eq!(run.execution_mode, ExecutionMode::Autonomous);
    assert!(
        run.total_metrics.gate_effectiveness.is_none(),
        "autonomous run should not have gate metrics"
    );
    assert!(!run.run_id.is_empty());
    assert!(
        !run.initiatives.is_empty(),
        "autonomous run must populate initiatives from AI response"
    );
    assert!(
        run.total_metrics.total_tokens > 0,
        "must capture real token counts"
    );
}

/// Live integration test — runs only when ANTHROPIC_API_KEY is available.
#[tokio::test]
async fn test_validated_runner_returns_valid_scaffold() {
    require_api_key!();

    let scenario_path = std::path::PathBuf::from("scenario");
    let results_dir = std::path::PathBuf::from("target/test-results");

    let harness = BenchmarkHarness::new(scenario_path, results_dir);
    let run = harness.run_validated().await.unwrap();

    assert_eq!(run.execution_mode, ExecutionMode::Validated);
    assert!(
        run.total_metrics.gate_effectiveness.is_some(),
        "validated run must expose gate effectiveness"
    );
    assert!(!run.run_id.is_empty());
}

#[test]
fn test_comparison_analysis_produces_positive_roi() {
    let autonomous = BenchmarkRun {
        run_id: "auto".to_string(),
        timestamp: Utc::now(),
        scenario: ScenarioSummary {
            id: "test-scenario".to_string(),
            title: "Test Scenario".to_string(),
            root: "scenario".to_string(),
        },
        execution_mode: ExecutionMode::Autonomous,
        phases: vec![],
        trace: RunTrace::default(),
        artifacts: practical_benchmark::types::RunArtifacts::default(),
        initiatives: vec![],
        total_metrics: RunMetrics {
            total_tokens: 9000,
            total_time: std::time::Duration::from_secs(300),
            avg_code_quality: 72.5,
            avg_test_coverage: 78.0,
            avg_doc_accuracy: 65.0,
            avg_instruction_adherence: 82.0,
            gate_effectiveness: None,
        },
    };

    let validated = BenchmarkRun {
        run_id: "val".to_string(),
        timestamp: Utc::now(),
        scenario: ScenarioSummary {
            id: "test-scenario".to_string(),
            title: "Test Scenario".to_string(),
            root: "scenario".to_string(),
        },
        execution_mode: ExecutionMode::Validated,
        phases: vec![],
        trace: RunTrace::default(),
        artifacts: practical_benchmark::types::RunArtifacts::default(),
        initiatives: vec![],
        total_metrics: RunMetrics {
            total_tokens: 10200,
            total_time: std::time::Duration::from_secs(420),
            avg_code_quality: 82.0,
            avg_test_coverage: 88.0,
            avg_doc_accuracy: 78.0,
            avg_instruction_adherence: 91.0,
            gate_effectiveness: Some(68.0),
        },
    };

    let analysis = analysis::BenchmarkAnalysis::new(autonomous, validated);
    let report = analysis.compare();

    assert!(
        report.token_overhead > 0.0,
        "validated run should use more tokens"
    );
    assert!(
        report.quality_delta > 0.0,
        "validated run should have better quality"
    );
    assert!(report.roi > 0.0, "ROI should be positive");
    assert_eq!(report.gate_effectiveness, 68.0);
}

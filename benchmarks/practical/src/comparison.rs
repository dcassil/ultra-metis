use crate::scoring::ScoreBreakdown;
use crate::types::{BenchmarkRun, SystemUnderTest, ToolSurface};
use serde::{Deserialize, Serialize};

/// How to compare two systems.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonMode {
    /// Same model, same prompt, same budget, same stopping rules.
    /// Fair apples-to-apples test of tool quality holding everything else constant.
    Constrained,
    /// Same scenario and scoring, but each system uses its natural workflow.
    /// Shows what users actually experience.
    Realistic,
}

/// Configuration for a comparative benchmark.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonConfig {
    pub mode: ComparisonMode,
    pub scenario_id: String,
    pub model_id: String,
    pub token_budget: u64,
    pub time_budget_secs: u64,
}

impl Default for ComparisonConfig {
    fn default() -> Self {
        Self {
            mode: ComparisonMode::Constrained,
            scenario_id: "file-processing-toolkit".to_string(),
            model_id: "claude-haiku-4-5-20251001".to_string(),
            token_budget: 500_000,
            time_budget_secs: 600,
        }
    }
}

/// A side-by-side comparison result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub config: ComparisonConfig,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub original_metis: SystemResult,
    pub cadre: SystemResult,
    pub deltas: ComparisonDeltas,
}

/// Result from one system under test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResult {
    pub system: SystemUnderTest,
    pub tool_surface: ToolSurface,
    pub run_id: String,
    pub tokens_used: u64,
    pub time_ms: u64,
    pub scores: Option<ScoreBreakdown>,
    pub initiative_count: u32,
    pub task_count: u32,
}

/// Computed deltas between the two systems (cadre - original_metis).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonDeltas {
    /// Positive = cadre used more tokens.
    pub token_delta: i64,
    pub token_overhead_percent: f32,
    /// Positive = cadre took longer.
    pub time_delta_ms: i64,
    /// Positive = cadre scored higher on doc generation.
    pub doc_generation_delta: f32,
    /// Positive = cadre scored higher on decomposition.
    pub decomposition_delta: f32,
    /// Positive = cadre scored higher on build outcome.
    pub build_outcome_delta: f32,
    /// Positive = cadre scored higher on architecture conformance.
    pub architecture_delta: f32,
}

impl ComparisonResult {
    /// Build a comparison from two benchmark runs.
    pub fn from_runs(
        config: ComparisonConfig,
        original: &BenchmarkRun,
        ultra: &BenchmarkRun,
    ) -> Self {
        let original_scores = None; // Populated by caller if scoring was run
        let ultra_scores = None;

        let orig = SystemResult {
            system: SystemUnderTest::OriginalMetis,
            tool_surface: original.manifest.tool_surface.clone(),
            run_id: original.run_id.clone(),
            tokens_used: original.total_metrics.total_tokens,
            time_ms: original.total_metrics.total_time.as_millis() as u64,
            scores: original_scores,
            initiative_count: original.initiatives.len() as u32,
            task_count: original
                .initiatives
                .iter()
                .map(|i| i.tasks.len() as u32)
                .sum(),
        };

        let ult = SystemResult {
            system: SystemUnderTest::Cadre,
            tool_surface: ultra.manifest.tool_surface.clone(),
            run_id: ultra.run_id.clone(),
            tokens_used: ultra.total_metrics.total_tokens,
            time_ms: ultra.total_metrics.total_time.as_millis() as u64,
            scores: ultra_scores,
            initiative_count: ultra.initiatives.len() as u32,
            task_count: ultra.initiatives.iter().map(|i| i.tasks.len() as u32).sum(),
        };

        let deltas = compute_deltas(&orig, &ult);

        Self {
            config,
            timestamp: chrono::Utc::now(),
            original_metis: orig,
            cadre: ult,
            deltas,
        }
    }

    /// Build a comparison with pre-computed scores.
    pub fn from_runs_with_scores(
        config: ComparisonConfig,
        original: &BenchmarkRun,
        original_scores: ScoreBreakdown,
        ultra: &BenchmarkRun,
        ultra_scores: ScoreBreakdown,
    ) -> Self {
        let orig = SystemResult {
            system: SystemUnderTest::OriginalMetis,
            tool_surface: original.manifest.tool_surface.clone(),
            run_id: original.run_id.clone(),
            tokens_used: original.total_metrics.total_tokens,
            time_ms: original.total_metrics.total_time.as_millis() as u64,
            scores: Some(original_scores),
            initiative_count: original.initiatives.len() as u32,
            task_count: original
                .initiatives
                .iter()
                .map(|i| i.tasks.len() as u32)
                .sum(),
        };

        let ult = SystemResult {
            system: SystemUnderTest::Cadre,
            tool_surface: ultra.manifest.tool_surface.clone(),
            run_id: ultra.run_id.clone(),
            tokens_used: ultra.total_metrics.total_tokens,
            time_ms: ultra.total_metrics.total_time.as_millis() as u64,
            scores: Some(ultra_scores),
            initiative_count: ultra.initiatives.len() as u32,
            task_count: ultra.initiatives.iter().map(|i| i.tasks.len() as u32).sum(),
        };

        let deltas = compute_deltas(&orig, &ult);

        Self {
            config,
            timestamp: chrono::Utc::now(),
            original_metis: orig,
            cadre: ult,
            deltas,
        }
    }

    /// Generate a markdown comparison report.
    pub fn to_markdown(&self) -> String {
        let mode_label = match self.config.mode {
            ComparisonMode::Constrained => "Constrained (same model/budget/rules)",
            ComparisonMode::Realistic => "Realistic (natural workflow per system)",
        };

        let mut md = String::new();
        md.push_str(&format!(
            "# Benchmark Comparison: {} vs {}\n\n",
            "Original Metis", "Cadre"
        ));
        md.push_str(&format!("**Mode**: {mode_label}\n"));
        md.push_str(&format!("**Scenario**: {}\n", self.config.scenario_id));
        md.push_str(&format!("**Model**: {}\n", self.config.model_id));
        md.push_str(&format!(
            "**Date**: {}\n\n",
            self.timestamp.format("%Y-%m-%d %H:%M UTC")
        ));

        md.push_str("## Cost Metrics\n\n");
        md.push_str("| Metric | Original Metis | Cadre | Delta |\n");
        md.push_str("|--------|---------------|-------------|-------|\n");
        md.push_str(&format!(
            "| Tokens | {} | {} | {:+} ({:+.1}%) |\n",
            self.original_metis.tokens_used,
            self.cadre.tokens_used,
            self.deltas.token_delta,
            self.deltas.token_overhead_percent
        ));
        md.push_str(&format!(
            "| Time (ms) | {} | {} | {:+} |\n",
            self.original_metis.time_ms, self.cadre.time_ms, self.deltas.time_delta_ms
        ));
        md.push_str(&format!(
            "| Initiatives | {} | {} | |\n",
            self.original_metis.initiative_count, self.cadre.initiative_count,
        ));
        md.push_str(&format!(
            "| Tasks | {} | {} | |\n\n",
            self.original_metis.task_count, self.cadre.task_count,
        ));

        if self.original_metis.scores.is_some() && self.cadre.scores.is_some() {
            md.push_str("## Quality Scores\n\n");
            md.push_str("| Track | Original Metis | Cadre | Delta |\n");
            md.push_str("|-------|---------------|-------------|-------|\n");
            md.push_str(&format!(
                "| Doc Generation | — | — | {:+.1} |\n",
                self.deltas.doc_generation_delta
            ));
            md.push_str(&format!(
                "| Decomposition | — | — | {:+.1} |\n",
                self.deltas.decomposition_delta
            ));
            md.push_str(&format!(
                "| Build Outcome | — | — | {:+.1} |\n",
                self.deltas.build_outcome_delta
            ));
            md.push_str(&format!(
                "| Architecture | — | — | {:+.1} |\n\n",
                self.deltas.architecture_delta
            ));
        }

        md
    }
}

fn compute_deltas(original: &SystemResult, ultra: &SystemResult) -> ComparisonDeltas {
    let token_delta = ultra.tokens_used as i64 - original.tokens_used as i64;
    let token_overhead_percent = if original.tokens_used > 0 {
        (token_delta as f32 / original.tokens_used as f32) * 100.0
    } else {
        0.0
    };

    let time_delta_ms = ultra.time_ms as i64 - original.time_ms as i64;

    // Score deltas (ultra - original), only if both have scores
    let (doc_delta, decomp_delta, build_delta, arch_delta) = match (&original.scores, &ultra.scores)
    {
        (Some(orig_s), Some(ult_s)) => (
            ult_s.document_generation.percent() - orig_s.document_generation.percent(),
            ult_s.decomposition.percent() - orig_s.decomposition.percent(),
            ult_s.build_outcome.percent() - orig_s.build_outcome.percent(),
            ult_s.architecture_conformance - orig_s.architecture_conformance,
        ),
        _ => (0.0, 0.0, 0.0, 0.0),
    };

    ComparisonDeltas {
        token_delta,
        token_overhead_percent,
        time_delta_ms,
        doc_generation_delta: doc_delta,
        decomposition_delta: decomp_delta,
        build_outcome_delta: build_delta,
        architecture_delta: arch_delta,
    }
}

/// Validate that a constrained comparison meets fairness requirements.
pub fn validate_constrained_fairness(original: &BenchmarkRun, ultra: &BenchmarkRun) -> Vec<String> {
    let mut issues = Vec::new();

    if original.manifest.model_id != ultra.manifest.model_id {
        issues.push(format!(
            "Model mismatch: {} vs {}",
            original.manifest.model_id, ultra.manifest.model_id
        ));
    }

    if original.manifest.token_budget != ultra.manifest.token_budget {
        issues.push(format!(
            "Token budget mismatch: {} vs {}",
            original.manifest.token_budget, ultra.manifest.token_budget
        ));
    }

    if original.manifest.time_budget_secs != ultra.manifest.time_budget_secs {
        issues.push(format!(
            "Time budget mismatch: {} vs {}",
            original.manifest.time_budget_secs, ultra.manifest.time_budget_secs
        ));
    }

    if original.scenario.id != ultra.scenario.id {
        issues.push(format!(
            "Scenario mismatch: {} vs {}",
            original.scenario.id, ultra.scenario.id
        ));
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoring::{ScoringCheck, TrackScore};
    use crate::types::{
        CodeMetrics, ExecutionMode, InitiativeResult, RunArtifacts, RunEnvironment, RunManifest,
        RunMetrics, RunTrace, ScenarioSummary, TaskResult, TaskStatus,
    };
    use std::time::Duration;

    fn make_run(system: SystemUnderTest, tokens: u64, time_ms: u64) -> BenchmarkRun {
        BenchmarkRun {
            run_id: format!("{system:?}"),
            timestamp: chrono::Utc::now(),
            manifest: RunManifest {
                system_under_test: system,
                tool_surface: ToolSurface::Cli,
                model_id: "claude-haiku-4-5-20251001".to_string(),
                token_budget: 500_000,
                time_budget_secs: 600,
                git_commit: None,
                environment: RunEnvironment::default(),
            },
            scenario: ScenarioSummary {
                id: "test".to_string(),
                title: "Test".to_string(),
                root: ".".to_string(),
            },
            execution_mode: ExecutionMode::Autonomous,
            phases: vec![],
            trace: RunTrace::default(),
            artifacts: RunArtifacts::default(),
            initiatives: vec![InitiativeResult {
                initiative_id: "init".to_string(),
                initiative_title: "Test Init".to_string(),
                tasks: vec![TaskResult {
                    task_id: "t1".to_string(),
                    task_title: "Task 1".to_string(),
                    status: TaskStatus::Completed,
                    tokens_used: tokens,
                    time_elapsed: Duration::from_millis(time_ms),
                    code_metrics: CodeMetrics {
                        lines_of_code: 50,
                        test_coverage_percent: 80.0,
                        cyclomatic_complexity: 3.0,
                        doc_accuracy_percent: 85.0,
                        instruction_adherence_percent: 90.0,
                    },
                    validation_gate: None,
                }],
                total_tokens: tokens,
                total_time: Duration::from_millis(time_ms),
            }],
            total_metrics: RunMetrics {
                total_tokens: tokens,
                total_time: Duration::from_millis(time_ms),
                avg_code_quality: 80.0,
                avg_test_coverage: 80.0,
                avg_doc_accuracy: 85.0,
                avg_instruction_adherence: 90.0,
                gate_effectiveness: None,
            },
        }
    }

    fn make_score(doc_pct: f32, decomp_pct: f32, build_pct: f32) -> ScoreBreakdown {
        let make_track = |pct: f32| TrackScore {
            score: pct,
            max_score: 100.0,
            checks: vec![ScoringCheck {
                name: "test".to_string(),
                passed: pct > 50.0,
                weight: 100.0,
                detail: String::new(),
            }],
        };
        ScoreBreakdown {
            document_generation: make_track(doc_pct),
            decomposition: make_track(decomp_pct),
            build_outcome: make_track(build_pct),
            architecture_conformance: 100.0,
            static_tool_utilization: 50.0,
            tokens_total: 1000,
            time_total_ms: 5000,
        }
    }

    #[test]
    fn test_comparison_from_runs() {
        let original = make_run(SystemUnderTest::OriginalMetis, 9000, 30000);
        let ultra = make_run(SystemUnderTest::Cadre, 10000, 25000);
        let config = ComparisonConfig::default();

        let result = ComparisonResult::from_runs(config, &original, &ultra);

        assert_eq!(result.deltas.token_delta, 1000);
        assert!(result.deltas.token_overhead_percent > 0.0);
        assert_eq!(result.deltas.time_delta_ms, -5000);
        assert_eq!(result.original_metis.initiative_count, 1);
        assert_eq!(result.cadre.task_count, 1);
    }

    #[test]
    fn test_comparison_with_scores() {
        let original = make_run(SystemUnderTest::OriginalMetis, 9000, 30000);
        let ultra = make_run(SystemUnderTest::Cadre, 10000, 25000);
        let orig_scores = make_score(60.0, 50.0, 70.0);
        let ultra_scores = make_score(80.0, 75.0, 85.0);

        let result = ComparisonResult::from_runs_with_scores(
            ComparisonConfig::default(),
            &original,
            orig_scores,
            &ultra,
            ultra_scores,
        );

        assert!(result.deltas.doc_generation_delta > 0.0);
        assert!(result.deltas.decomposition_delta > 0.0);
        assert!(result.deltas.build_outcome_delta > 0.0);
    }

    #[test]
    fn test_constrained_fairness_validation() {
        let run_a = make_run(SystemUnderTest::OriginalMetis, 9000, 30000);
        let run_b = make_run(SystemUnderTest::Cadre, 10000, 25000);

        let issues = validate_constrained_fairness(&run_a, &run_b);
        assert!(
            issues.is_empty(),
            "Same config should have no fairness issues"
        );
    }

    #[test]
    fn test_constrained_fairness_catches_model_mismatch() {
        let run_a = make_run(SystemUnderTest::OriginalMetis, 9000, 30000);
        let mut run_b = make_run(SystemUnderTest::Cadre, 10000, 25000);
        run_b.manifest.model_id = "claude-sonnet-4-6".to_string();

        let issues = validate_constrained_fairness(&run_a, &run_b);
        assert!(!issues.is_empty());
        assert!(issues[0].contains("Model mismatch"));
    }

    #[test]
    fn test_markdown_report_generation() {
        let original = make_run(SystemUnderTest::OriginalMetis, 9000, 30000);
        let ultra = make_run(SystemUnderTest::Cadre, 10000, 25000);
        let config = ComparisonConfig::default();

        let result = ComparisonResult::from_runs(config, &original, &ultra);
        let md = result.to_markdown();

        assert!(md.contains("Original Metis"));
        assert!(md.contains("Cadre"));
        assert!(md.contains("Constrained"));
        assert!(md.contains("Tokens"));
    }
}

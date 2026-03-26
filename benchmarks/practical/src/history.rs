use crate::types::NormalizedResult;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const HISTORY_FILE: &str = "benchmark_history.json";
const REGRESSION_THRESHOLD_PERCENT: f32 = 5.0;

/// Append-only history of normalized benchmark results.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BenchmarkHistory {
    pub entries: Vec<NormalizedResult>,
}

impl BenchmarkHistory {
    /// Load history from disk, or create empty if file doesn't exist.
    pub fn load(results_dir: &Path) -> Result<Self> {
        let path = results_dir.join(HISTORY_FILE);
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        serde_json::from_str(&raw).with_context(|| format!("Failed to parse {}", path.display()))
    }

    /// Append a result and save to disk.
    pub fn append_and_save(
        &mut self,
        result: NormalizedResult,
        results_dir: &Path,
    ) -> Result<PathBuf> {
        self.entries.push(result);
        let path = results_dir.join(HISTORY_FILE);
        std::fs::create_dir_all(results_dir)?;
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(path)
    }

    /// Get the most recent entry for a given scenario and system.
    pub fn latest_for(
        &self,
        scenario_id: &str,
        system: &crate::types::SystemUnderTest,
    ) -> Option<&NormalizedResult> {
        self.entries
            .iter()
            .rev()
            .find(|e| e.scenario_id == scenario_id && &e.system_under_test == system)
    }

    /// Compare a new result against the most recent comparable baseline.
    pub fn detect_regressions(&self, new_result: &NormalizedResult) -> RegressionSummary {
        let baseline = self
            .entries
            .iter()
            .rev()
            .skip(1) // Skip the entry we just added
            .find(|e| {
                e.scenario_id == new_result.scenario_id
                    && e.system_under_test == new_result.system_under_test
                    && e.execution_mode == new_result.execution_mode
            });

        let Some(baseline) = baseline else {
            return RegressionSummary {
                has_baseline: false,
                regressions: vec![],
                improvements: vec![],
            };
        };

        let mut regressions = Vec::new();
        let mut improvements = Vec::new();

        check_metric(
            "doc_generation_score",
            baseline.doc_generation_score,
            new_result.doc_generation_score,
            &mut regressions,
            &mut improvements,
        );
        check_metric(
            "decomposition_score",
            baseline.decomposition_score,
            new_result.decomposition_score,
            &mut regressions,
            &mut improvements,
        );
        check_metric(
            "build_outcome_score",
            baseline.build_outcome_score,
            new_result.build_outcome_score,
            &mut regressions,
            &mut improvements,
        );
        check_metric(
            "architecture_conformance_score",
            baseline.architecture_conformance_score,
            new_result.architecture_conformance_score,
            &mut regressions,
            &mut improvements,
        );

        // Token cost: regression = using significantly more tokens
        let token_delta_pct = if baseline.total_tokens > 0 {
            ((new_result.total_tokens as f32 - baseline.total_tokens as f32)
                / baseline.total_tokens as f32)
                * 100.0
        } else {
            0.0
        };
        if token_delta_pct > REGRESSION_THRESHOLD_PERCENT * 2.0 {
            regressions.push(RegressionDelta {
                metric: "total_tokens".to_string(),
                baseline_value: baseline.total_tokens as f32,
                new_value: new_result.total_tokens as f32,
                delta_percent: token_delta_pct,
            });
        } else if token_delta_pct < -REGRESSION_THRESHOLD_PERCENT {
            improvements.push(RegressionDelta {
                metric: "total_tokens".to_string(),
                baseline_value: baseline.total_tokens as f32,
                new_value: new_result.total_tokens as f32,
                delta_percent: token_delta_pct,
            });
        }

        RegressionSummary {
            has_baseline: true,
            regressions,
            improvements,
        }
    }
}

fn check_metric(
    name: &str,
    baseline: f32,
    new: f32,
    regressions: &mut Vec<RegressionDelta>,
    improvements: &mut Vec<RegressionDelta>,
) {
    if baseline == 0.0 {
        return;
    }
    let delta_pct = ((new - baseline) / baseline) * 100.0;

    if delta_pct < -REGRESSION_THRESHOLD_PERCENT {
        regressions.push(RegressionDelta {
            metric: name.to_string(),
            baseline_value: baseline,
            new_value: new,
            delta_percent: delta_pct,
        });
    } else if delta_pct > REGRESSION_THRESHOLD_PERCENT {
        improvements.push(RegressionDelta {
            metric: name.to_string(),
            baseline_value: baseline,
            new_value: new,
            delta_percent: delta_pct,
        });
    }
}

/// Summary of regressions and improvements detected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionSummary {
    pub has_baseline: bool,
    pub regressions: Vec<RegressionDelta>,
    pub improvements: Vec<RegressionDelta>,
}

impl RegressionSummary {
    pub fn has_regressions(&self) -> bool {
        !self.regressions.is_empty()
    }

    /// Generate a markdown summary of changes.
    pub fn to_markdown(&self) -> String {
        if !self.has_baseline {
            return "No baseline available for comparison — this is the first run.\n".to_string();
        }

        let mut md = String::new();

        if self.regressions.is_empty() && self.improvements.is_empty() {
            md.push_str("No significant changes detected (within 5% threshold).\n");
            return md;
        }

        if !self.regressions.is_empty() {
            md.push_str("### Regressions\n\n");
            md.push_str("| Metric | Baseline | New | Delta |\n");
            md.push_str("|--------|----------|-----|-------|\n");
            for r in &self.regressions {
                md.push_str(&format!(
                    "| {} | {:.1} | {:.1} | {:+.1}% |\n",
                    r.metric, r.baseline_value, r.new_value, r.delta_percent
                ));
            }
            md.push('\n');
        }

        if !self.improvements.is_empty() {
            md.push_str("### Improvements\n\n");
            md.push_str("| Metric | Baseline | New | Delta |\n");
            md.push_str("|--------|----------|-----|-------|\n");
            for i in &self.improvements {
                md.push_str(&format!(
                    "| {} | {:.1} | {:.1} | {:+.1}% |\n",
                    i.metric, i.baseline_value, i.new_value, i.delta_percent
                ));
            }
            md.push('\n');
        }

        md
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionDelta {
    pub metric: String,
    pub baseline_value: f32,
    pub new_value: f32,
    pub delta_percent: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn make_result(doc_score: f32, tokens: u64) -> NormalizedResult {
        NormalizedResult {
            run_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            scenario_id: "test-scenario".to_string(),
            system_under_test: SystemUnderTest::Cadre,
            execution_mode: ExecutionMode::Autonomous,
            tool_surface: ToolSurface::Cli,
            model_id: "claude-haiku-4-5-20251001".to_string(),
            total_tokens: tokens,
            total_time_ms: 30000,
            doc_generation_score: doc_score,
            decomposition_score: 75.0,
            build_outcome_score: 80.0,
            architecture_conformance_score: 90.0,
            gate_effectiveness: None,
            initiative_count: 2,
            task_count: 4,
        }
    }

    #[test]
    fn test_empty_history_no_regressions() {
        let history = BenchmarkHistory::default();
        let result = make_result(80.0, 10000);
        let summary = history.detect_regressions(&result);
        assert!(!summary.has_baseline);
        assert!(!summary.has_regressions());
    }

    #[test]
    fn test_regression_detected() {
        let mut history = BenchmarkHistory::default();
        history.entries.push(make_result(80.0, 10000));
        // New result with lower doc score
        let new_result = make_result(70.0, 10000);
        history.entries.push(new_result.clone());

        let summary = history.detect_regressions(&new_result);
        assert!(summary.has_baseline);
        assert!(summary.has_regressions());
        assert!(summary
            .regressions
            .iter()
            .any(|r| r.metric == "doc_generation_score"));
    }

    #[test]
    fn test_improvement_detected() {
        let mut history = BenchmarkHistory::default();
        history.entries.push(make_result(60.0, 10000));
        let new_result = make_result(80.0, 10000);
        history.entries.push(new_result.clone());

        let summary = history.detect_regressions(&new_result);
        assert!(summary.has_baseline);
        assert!(!summary.has_regressions());
        assert!(!summary.improvements.is_empty());
        assert!(summary
            .improvements
            .iter()
            .any(|i| i.metric == "doc_generation_score"));
    }

    #[test]
    fn test_token_regression() {
        let mut history = BenchmarkHistory::default();
        history.entries.push(make_result(80.0, 10000));
        let new_result = make_result(80.0, 12000); // 20% more tokens
        history.entries.push(new_result.clone());

        let summary = history.detect_regressions(&new_result);
        assert!(summary
            .regressions
            .iter()
            .any(|r| r.metric == "total_tokens"));
    }

    #[test]
    fn test_persist_and_load_history() {
        let dir = tempfile::tempdir().unwrap();
        let mut history = BenchmarkHistory::default();
        let result = make_result(80.0, 10000);
        history.append_and_save(result.clone(), dir.path()).unwrap();

        let loaded = BenchmarkHistory::load(dir.path()).unwrap();
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].run_id, result.run_id);
    }

    #[test]
    fn test_regression_markdown() {
        let mut history = BenchmarkHistory::default();
        history.entries.push(make_result(80.0, 10000));
        let new_result = make_result(70.0, 10000);
        history.entries.push(new_result.clone());

        let summary = history.detect_regressions(&new_result);
        let md = summary.to_markdown();
        assert!(md.contains("Regressions"));
        assert!(md.contains("doc_generation_score"));
    }

    #[test]
    fn test_no_change_within_threshold() {
        let mut history = BenchmarkHistory::default();
        history.entries.push(make_result(80.0, 10000));
        let new_result = make_result(79.0, 10200); // < 5% change
        history.entries.push(new_result.clone());

        let summary = history.detect_regressions(&new_result);
        assert!(summary.regressions.is_empty());
        assert!(summary.improvements.is_empty());
    }
}

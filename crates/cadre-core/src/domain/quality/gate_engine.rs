use crate::domain::documents::quality_gate_config::{
    GateSeverity, MetricGateRule, QualityGateConfig, ThresholdType, TrendRequirement,
};
use std::collections::HashMap;
use std::fmt;

/// Result of checking a single metric against its gate threshold.
#[derive(Debug, Clone)]
pub struct MetricCheckResult {
    /// Name of the metric that was checked.
    pub metric: String,
    /// The threshold that was applied.
    pub threshold: ThresholdType,
    /// The actual metric value found.
    pub actual_value: f64,
    /// Whether this metric passed its gate.
    pub passed: bool,
    /// The severity of the gate (blocking vs advisory).
    pub severity: GateSeverity,
    /// Difference from the threshold (positive means exceeded).
    pub delta: f64,
    /// Baseline value used for relative comparisons (if applicable).
    pub baseline_value: Option<f64>,
}

impl MetricCheckResult {
    /// Format an actionable failure message.
    pub fn failure_message(&self) -> String {
        if self.passed {
            return format!("{}: passed", self.metric);
        }

        match &self.threshold {
            ThresholdType::Absolute(max) => {
                format!(
                    "{}: {} (threshold: {}, exceeded by {})",
                    self.metric,
                    self.actual_value,
                    max,
                    self.actual_value - max
                )
            }
            ThresholdType::RelativeRegression(max_pct) => {
                let baseline = self.baseline_value.unwrap_or(0.0);
                let pct_change = if baseline != 0.0 {
                    ((self.actual_value - baseline) / baseline) * 100.0
                } else {
                    0.0
                };
                format!(
                    "{}: regressed {:.1}% (baseline: {}, current: {}, max allowed: {}%)",
                    self.metric, pct_change, baseline, self.actual_value, max_pct
                )
            }
            ThresholdType::Trend(req) => {
                let req_str = match req {
                    TrendRequirement::Improving => "improving",
                    TrendRequirement::NotRegressing => "not regressing",
                };
                format!(
                    "{}: trend requirement not met (required: {}, current: {})",
                    self.metric, req_str, self.actual_value
                )
            }
        }
    }
}

impl fmt::Display for MetricCheckResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.failure_message())
    }
}

/// Overall result of checking all quality gates.
#[derive(Debug)]
pub struct GateCheckResult {
    /// Whether all blocking gates passed.
    pub passed: bool,
    /// Per-metric check results.
    pub metric_results: Vec<MetricCheckResult>,
    /// Metrics that failed with blocking severity.
    pub blocking_failures: Vec<MetricCheckResult>,
    /// Metrics that failed with advisory severity.
    pub advisory_failures: Vec<MetricCheckResult>,
}

impl GateCheckResult {
    /// Format all failures as an actionable summary.
    pub fn failure_summary(&self) -> String {
        let mut lines = Vec::new();

        if !self.blocking_failures.is_empty() {
            lines.push("BLOCKING failures:".to_string());
            for f in &self.blocking_failures {
                lines.push(format!("  - {}", f.failure_message()));
            }
        }

        if !self.advisory_failures.is_empty() {
            lines.push("Advisory warnings:".to_string());
            for f in &self.advisory_failures {
                lines.push(format!("  - {}", f.failure_message()));
            }
        }

        if lines.is_empty() {
            "All quality gates passed.".to_string()
        } else {
            lines.join("\n")
        }
    }

    /// Count of all failures (blocking + advisory).
    pub fn total_failures(&self) -> usize {
        self.blocking_failures.len() + self.advisory_failures.len()
    }
}

/// Stateless engine for evaluating quality metrics against gate thresholds.
pub struct GateCheckEngine;

impl GateCheckEngine {
    /// Check current metric values against a gate configuration.
    ///
    /// # Arguments
    /// * `current_metrics` - Current metric values (e.g., from ParsedToolOutput.summary)
    /// * `config` - The quality gate configuration with thresholds
    /// * `from_phase` - The phase being transitioned from (for transition-specific overrides)
    /// * `to_phase` - The phase being transitioned to (for transition-specific overrides)
    /// * `baseline_metrics` - Optional baseline values for relative threshold checks
    /// * `trend_history` - Optional historical values for trend checks (most recent first)
    pub fn check(
        current_metrics: &HashMap<String, f64>,
        config: &QualityGateConfig,
        from_phase: Option<&str>,
        to_phase: Option<&str>,
        baseline_metrics: Option<&HashMap<String, f64>>,
        trend_history: Option<&[HashMap<String, f64>]>,
    ) -> GateCheckResult {
        // Resolve which thresholds to use
        let rules = if let (Some(from), Some(to)) = (from_phase, to_phase) {
            config.thresholds_for_transition(from, to)
        } else {
            &config.default_thresholds
        };

        let mut metric_results = Vec::new();
        let mut blocking_failures = Vec::new();
        let mut advisory_failures = Vec::new();

        for rule in rules {
            let result = Self::check_metric(rule, current_metrics, baseline_metrics, trend_history);

            if !result.passed {
                match result.severity {
                    GateSeverity::Blocking => blocking_failures.push(result.clone()),
                    GateSeverity::Advisory => advisory_failures.push(result.clone()),
                }
            }

            metric_results.push(result);
        }

        let passed = blocking_failures.is_empty();

        GateCheckResult {
            passed,
            metric_results,
            blocking_failures,
            advisory_failures,
        }
    }

    /// Check a single metric against its gate rule.
    fn check_metric(
        rule: &MetricGateRule,
        current_metrics: &HashMap<String, f64>,
        baseline_metrics: Option<&HashMap<String, f64>>,
        trend_history: Option<&[HashMap<String, f64>]>,
    ) -> MetricCheckResult {
        let actual_value = current_metrics.get(&rule.metric).copied().unwrap_or(0.0);

        match &rule.threshold {
            ThresholdType::Absolute(max_value) => {
                let passed = actual_value <= *max_value;
                let delta = actual_value - max_value;

                MetricCheckResult {
                    metric: rule.metric.clone(),
                    threshold: rule.threshold.clone(),
                    actual_value,
                    passed,
                    severity: rule.severity,
                    delta,
                    baseline_value: None,
                }
            }

            ThresholdType::RelativeRegression(max_pct) => {
                let baseline_value = baseline_metrics
                    .and_then(|bm| bm.get(&rule.metric))
                    .copied()
                    .unwrap_or(0.0);

                let (passed, delta) = if baseline_value == 0.0 {
                    // No baseline or baseline is 0 — can't compute regression percentage.
                    // If current is also 0, pass. If current > 0, it's a new issue but
                    // we can't express it as a percentage, so pass to avoid false positives.
                    (true, 0.0)
                } else {
                    let pct_change =
                        ((actual_value - baseline_value) / baseline_value.abs()) * 100.0;
                    // For "lower is better" metrics: regression means value increased.
                    // pct_change > 0 means value went up (regressed for error counts).
                    // pct_change > max_pct means it regressed too much.
                    let passed = pct_change <= *max_pct;
                    (passed, pct_change)
                };

                MetricCheckResult {
                    metric: rule.metric.clone(),
                    threshold: rule.threshold.clone(),
                    actual_value,
                    passed,
                    severity: rule.severity,
                    delta,
                    baseline_value: Some(baseline_value),
                }
            }

            ThresholdType::Trend(requirement) => {
                let (passed, delta) =
                    Self::check_trend(&rule.metric, actual_value, requirement, trend_history);

                MetricCheckResult {
                    metric: rule.metric.clone(),
                    threshold: rule.threshold.clone(),
                    actual_value,
                    passed,
                    severity: rule.severity,
                    delta,
                    baseline_value: None,
                }
            }
        }
    }

    /// Check trend for a metric against historical values.
    fn check_trend(
        metric: &str,
        current_value: f64,
        requirement: &TrendRequirement,
        trend_history: Option<&[HashMap<String, f64>]>,
    ) -> (bool, f64) {
        let history = match trend_history {
            Some(h) if !h.is_empty() => h,
            _ => {
                // No history available — can't assess trend, so pass.
                return (true, 0.0);
            }
        };

        // Get the most recent historical value
        let previous_value = history
            .first()
            .and_then(|h| h.get(metric))
            .copied()
            .unwrap_or(current_value);

        let delta = current_value - previous_value;

        match requirement {
            TrendRequirement::Improving => {
                // For "lower is better" metrics, improving means delta < 0.
                // We use the convention that the metric is "lower is better" by default.
                // The caller should negate coverage-style metrics before passing them.
                let passed = delta < 0.0 || (delta.abs() < f64::EPSILON);
                (passed, delta)
            }
            TrendRequirement::NotRegressing => {
                // Not getting worse — delta must be <= 0 (or approximately 0).
                let passed = delta <= 0.0 || (delta.abs() < f64::EPSILON);
                (passed, delta)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::documents::quality_gate_config::{
        MetricGateRule, QualityGateConfig, TransitionGateConfig,
    };
    use crate::domain::documents::types::{Phase, Tag};

    fn make_config() -> QualityGateConfig {
        QualityGateConfig::new(
            "Test Gates".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-QGC-0001".to_string(),
            GateSeverity::Blocking,
            vec![
                MetricGateRule::blocking_absolute("lint_errors", 10.0),
                MetricGateRule::blocking_relative("total_warnings", 5.0),
                MetricGateRule::advisory_absolute("info_count", 50.0),
            ],
            vec![TransitionGateConfig::new(
                "active",
                "completed",
                vec![
                    MetricGateRule::blocking_absolute("lint_errors", 0.0),
                    MetricGateRule::blocking_absolute("total_warnings", 0.0),
                ],
            )],
        )
        .unwrap()
    }

    fn metrics(pairs: &[(&str, f64)]) -> HashMap<String, f64> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn test_all_pass() {
        let config = make_config();
        let current = metrics(&[
            ("lint_errors", 5.0),
            ("total_warnings", 10.0),
            ("info_count", 30.0),
        ]);
        let baseline = metrics(&[("total_warnings", 10.0)]);

        let result = GateCheckEngine::check(&current, &config, None, None, Some(&baseline), None);

        assert!(result.passed);
        assert!(result.blocking_failures.is_empty());
        assert!(result.advisory_failures.is_empty());
        assert_eq!(result.metric_results.len(), 3);
    }

    #[test]
    fn test_single_blocking_failure() {
        let config = make_config();
        let current = metrics(&[
            ("lint_errors", 15.0), // exceeds 10.0 threshold
            ("total_warnings", 10.0),
            ("info_count", 30.0),
        ]);
        let baseline = metrics(&[("total_warnings", 10.0)]);

        let result = GateCheckEngine::check(&current, &config, None, None, Some(&baseline), None);

        assert!(!result.passed);
        assert_eq!(result.blocking_failures.len(), 1);
        assert_eq!(result.blocking_failures[0].metric, "lint_errors");
        assert_eq!(result.advisory_failures.len(), 0);
    }

    #[test]
    fn test_multiple_failures() {
        let config = make_config();
        let current = metrics(&[
            ("lint_errors", 15.0),    // exceeds 10.0
            ("total_warnings", 20.0), // regressed >5% from baseline 10.0
            ("info_count", 60.0),     // exceeds advisory 50.0
        ]);
        let baseline = metrics(&[("total_warnings", 10.0)]);

        let result = GateCheckEngine::check(&current, &config, None, None, Some(&baseline), None);

        assert!(!result.passed);
        assert_eq!(result.blocking_failures.len(), 2);
        assert_eq!(result.advisory_failures.len(), 1);
        assert_eq!(result.advisory_failures[0].metric, "info_count");
    }

    #[test]
    fn test_advisory_only_failures_still_pass() {
        let config = make_config();
        let current = metrics(&[
            ("lint_errors", 5.0),
            ("total_warnings", 10.0),
            ("info_count", 60.0), // exceeds advisory 50.0
        ]);
        let baseline = metrics(&[("total_warnings", 10.0)]);

        let result = GateCheckEngine::check(&current, &config, None, None, Some(&baseline), None);

        assert!(result.passed); // advisory failures don't block
        assert!(result.blocking_failures.is_empty());
        assert_eq!(result.advisory_failures.len(), 1);
    }

    #[test]
    fn test_transition_override() {
        let config = make_config();
        let current = metrics(&[
            ("lint_errors", 0.0),
            ("total_warnings", 5.0), // passes default (relative) but fails override (absolute 0)
        ]);

        let result = GateCheckEngine::check(
            &current,
            &config,
            Some("active"),
            Some("completed"),
            None,
            None,
        );

        assert!(!result.passed);
        assert_eq!(result.blocking_failures.len(), 1);
        assert_eq!(result.blocking_failures[0].metric, "total_warnings");
    }

    #[test]
    fn test_transition_override_fallback_to_defaults() {
        let config = make_config();
        let current = metrics(&[
            ("lint_errors", 5.0),
            ("total_warnings", 10.0),
            ("info_count", 30.0),
        ]);
        let baseline = metrics(&[("total_warnings", 10.0)]);

        // No override for ready->active, falls back to defaults
        let result = GateCheckEngine::check(
            &current,
            &config,
            Some("ready"),
            Some("active"),
            Some(&baseline),
            None,
        );

        assert!(result.passed);
        assert_eq!(result.metric_results.len(), 3); // all 3 default rules checked
    }

    #[test]
    fn test_relative_threshold_zero_baseline() {
        let config = QualityGateConfig::new(
            "Rel Test".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-QGC-0002".to_string(),
            GateSeverity::Blocking,
            vec![MetricGateRule::blocking_relative("errors", 5.0)],
            vec![],
        )
        .unwrap();

        let current = metrics(&[("errors", 10.0)]);
        let baseline = metrics(&[("errors", 0.0)]); // zero baseline

        let result = GateCheckEngine::check(&current, &config, None, None, Some(&baseline), None);

        // Zero baseline — can't compute regression %, passes to avoid false positive
        assert!(result.passed);
    }

    #[test]
    fn test_relative_threshold_exactly_at_threshold() {
        let config = QualityGateConfig::new(
            "Exact Test".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-QGC-0003".to_string(),
            GateSeverity::Blocking,
            vec![MetricGateRule::blocking_relative("errors", 10.0)],
            vec![],
        )
        .unwrap();

        let current = metrics(&[("errors", 11.0)]); // exactly 10% regression from 10
        let baseline = metrics(&[("errors", 10.0)]);

        let result = GateCheckEngine::check(&current, &config, None, None, Some(&baseline), None);

        // 10% regression, threshold is 10% — should pass (<=)
        assert!(result.passed);
    }

    #[test]
    fn test_relative_threshold_over() {
        let config = QualityGateConfig::new(
            "Over Test".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-QGC-0004".to_string(),
            GateSeverity::Blocking,
            vec![MetricGateRule::blocking_relative("errors", 10.0)],
            vec![],
        )
        .unwrap();

        let current = metrics(&[("errors", 11.1)]); // >10% regression from 10
        let baseline = metrics(&[("errors", 10.0)]);

        let result = GateCheckEngine::check(&current, &config, None, None, Some(&baseline), None);

        assert!(!result.passed);
    }

    #[test]
    fn test_trend_improving_pass() {
        let config = QualityGateConfig::new(
            "Trend Test".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-QGC-0005".to_string(),
            GateSeverity::Blocking,
            vec![MetricGateRule::new(
                "errors",
                ThresholdType::Trend(TrendRequirement::Improving),
                GateSeverity::Blocking,
            )],
            vec![],
        )
        .unwrap();

        let current = metrics(&[("errors", 3.0)]);
        let history = vec![metrics(&[("errors", 5.0)])]; // was 5, now 3 — improving

        let result = GateCheckEngine::check(&current, &config, None, None, None, Some(&history));

        assert!(result.passed);
    }

    #[test]
    fn test_trend_improving_fail() {
        let config = QualityGateConfig::new(
            "Trend Fail Test".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-QGC-0006".to_string(),
            GateSeverity::Blocking,
            vec![MetricGateRule::new(
                "errors",
                ThresholdType::Trend(TrendRequirement::Improving),
                GateSeverity::Blocking,
            )],
            vec![],
        )
        .unwrap();

        let current = metrics(&[("errors", 8.0)]);
        let history = vec![metrics(&[("errors", 5.0)])]; // was 5, now 8 — regressed

        let result = GateCheckEngine::check(&current, &config, None, None, None, Some(&history));

        assert!(!result.passed);
    }

    #[test]
    fn test_trend_not_regressing_stable() {
        let config = QualityGateConfig::new(
            "Stable Test".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-QGC-0007".to_string(),
            GateSeverity::Blocking,
            vec![MetricGateRule::new(
                "errors",
                ThresholdType::Trend(TrendRequirement::NotRegressing),
                GateSeverity::Blocking,
            )],
            vec![],
        )
        .unwrap();

        let current = metrics(&[("errors", 5.0)]);
        let history = vec![metrics(&[("errors", 5.0)])]; // unchanged — ok

        let result = GateCheckEngine::check(&current, &config, None, None, None, Some(&history));

        assert!(result.passed);
    }

    #[test]
    fn test_trend_no_history_passes() {
        let config = QualityGateConfig::new(
            "No History Test".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-QGC-0008".to_string(),
            GateSeverity::Blocking,
            vec![MetricGateRule::new(
                "errors",
                ThresholdType::Trend(TrendRequirement::Improving),
                GateSeverity::Blocking,
            )],
            vec![],
        )
        .unwrap();

        let current = metrics(&[("errors", 5.0)]);

        let result = GateCheckEngine::check(
            &current, &config, None, None, None, None, // no history
        );

        // No history — can't assess trend, passes
        assert!(result.passed);
    }

    #[test]
    fn test_missing_metric_defaults_to_zero() {
        let config = QualityGateConfig::new(
            "Missing Metric Test".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-QGC-0009".to_string(),
            GateSeverity::Blocking,
            vec![MetricGateRule::blocking_absolute("lint_errors", 10.0)],
            vec![],
        )
        .unwrap();

        let current = HashMap::new(); // empty — lint_errors defaults to 0

        let result = GateCheckEngine::check(&current, &config, None, None, None, None);

        assert!(result.passed);
        assert_eq!(result.metric_results[0].actual_value, 0.0);
    }

    #[test]
    fn test_failure_message_format_absolute() {
        let result = MetricCheckResult {
            metric: "lint_errors".to_string(),
            threshold: ThresholdType::Absolute(10.0),
            actual_value: 15.0,
            passed: false,
            severity: GateSeverity::Blocking,
            delta: 5.0,
            baseline_value: None,
        };

        let msg = result.failure_message();
        assert!(msg.contains("lint_errors"));
        assert!(msg.contains("15"));
        assert!(msg.contains("threshold: 10"));
        assert!(msg.contains("exceeded by 5"));
    }

    #[test]
    fn test_failure_summary() {
        let config = make_config();
        let current = metrics(&[
            ("lint_errors", 15.0),
            ("total_warnings", 10.0),
            ("info_count", 60.0),
        ]);
        let baseline = metrics(&[("total_warnings", 10.0)]);

        let result = GateCheckEngine::check(&current, &config, None, None, Some(&baseline), None);

        let summary = result.failure_summary();
        assert!(summary.contains("BLOCKING"));
        assert!(summary.contains("lint_errors"));
        assert!(summary.contains("Advisory"));
        assert!(summary.contains("info_count"));
    }

    #[test]
    fn test_all_pass_summary() {
        let config = make_config();
        let current = metrics(&[
            ("lint_errors", 5.0),
            ("total_warnings", 10.0),
            ("info_count", 30.0),
        ]);
        let baseline = metrics(&[("total_warnings", 10.0)]);

        let result = GateCheckEngine::check(&current, &config, None, None, Some(&baseline), None);

        assert_eq!(result.failure_summary(), "All quality gates passed.");
    }

    #[test]
    fn test_total_failures() {
        let config = make_config();
        let current = metrics(&[
            ("lint_errors", 15.0),
            ("total_warnings", 20.0),
            ("info_count", 60.0),
        ]);
        let baseline = metrics(&[("total_warnings", 10.0)]);

        let result = GateCheckEngine::check(&current, &config, None, None, Some(&baseline), None);

        assert_eq!(result.total_failures(), 3);
    }
}

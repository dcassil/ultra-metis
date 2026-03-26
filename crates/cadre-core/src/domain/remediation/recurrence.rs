//! Recurring issue detection for remediation loops.
//!
//! Analyzes remediation history to detect patterns of recurring quality
//! issues, enabling proactive intervention for systemic problems.

use super::types::{RemediationLoop, RemediationLoopPhase, RemediationTrigger};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A detected pattern of recurring quality issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurrencePattern {
    /// The metric that keeps failing.
    pub metric_name: String,
    /// Number of times this metric has triggered remediation.
    pub occurrence_count: u32,
    /// Time span over which the recurrences were observed.
    pub time_span_days: u32,
    /// Average delta (how far over threshold) across occurrences.
    pub avg_delta: f64,
    /// Whether the pattern suggests a systemic issue.
    pub is_systemic: bool,
    /// Short codes of the remediation loops involved.
    pub loop_refs: Vec<String>,
}

/// Summary of remediation history analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurrenceReport {
    /// Total number of remediation loops analyzed.
    pub total_loops_analyzed: usize,
    /// Loops that resolved successfully.
    pub resolved_count: usize,
    /// Loops that were closed without resolution.
    pub closed_count: usize,
    /// Active (non-terminal) loops.
    pub active_count: usize,
    /// Detected recurrence patterns.
    pub patterns: Vec<RecurrencePattern>,
    /// Metrics sorted by failure frequency.
    pub top_failing_metrics: Vec<(String, u32)>,
}

/// Engine for detecting recurring quality issues across remediation loops.
pub struct RecurrenceDetector;

impl RecurrenceDetector {
    /// Analyze a collection of remediation loops to detect recurring patterns.
    ///
    /// # Arguments
    /// * `loops` - Historical remediation loops to analyze.
    /// * `recurrence_threshold` - Minimum number of occurrences to flag as recurring.
    /// * `window_days` - Only consider loops created within this many days of now.
    pub fn detect(
        loops: &[RemediationLoop],
        recurrence_threshold: u32,
        window_days: u32,
    ) -> RecurrenceReport {
        let cutoff = Utc::now() - Duration::days(i64::from(window_days));

        let in_window: Vec<&RemediationLoop> =
            loops.iter().filter(|l| l.created_at >= cutoff).collect();

        let resolved_count = in_window
            .iter()
            .filter(|l| l.phase == RemediationLoopPhase::Resolved)
            .count();
        let closed_count = in_window
            .iter()
            .filter(|l| l.phase == RemediationLoopPhase::Closed)
            .count();
        let active_count = in_window.iter().filter(|l| !l.phase.is_terminal()).count();

        // Collect all failed metrics from gate failure triggers
        let mut metric_occurrences: HashMap<String, Vec<MetricOccurrence>> = HashMap::new();

        for rl in &in_window {
            if let RemediationTrigger::GateFailure { failed_metrics, .. } = &rl.trigger {
                for fm in failed_metrics {
                    metric_occurrences
                        .entry(fm.metric_name.clone())
                        .or_default()
                        .push(MetricOccurrence {
                            loop_id: rl.id.clone(),
                            delta: fm.delta,
                            timestamp: rl.created_at,
                            is_blocking: fm.is_blocking,
                        });
                }
            }
        }

        // Detect patterns
        let mut patterns = Vec::new();
        for (metric_name, occurrences) in &metric_occurrences {
            let count = occurrences.len() as u32;
            if count >= recurrence_threshold {
                let avg_delta = occurrences.iter().map(|o| o.delta).sum::<f64>() / f64::from(count);

                let time_span_days = if occurrences.len() >= 2 {
                    let min_ts = occurrences.iter().map(|o| o.timestamp).min().unwrap();
                    let max_ts = occurrences.iter().map(|o| o.timestamp).max().unwrap();
                    (max_ts - min_ts).num_days().max(1) as u32
                } else {
                    1
                };

                // Systemic if recurring frequently relative to the time window
                let is_systemic = count >= 3 || (count >= 2 && time_span_days <= 7);

                let loop_refs: Vec<String> =
                    occurrences.iter().map(|o| o.loop_id.clone()).collect();

                patterns.push(RecurrencePattern {
                    metric_name: metric_name.clone(),
                    occurrence_count: count,
                    time_span_days,
                    avg_delta,
                    is_systemic,
                    loop_refs,
                });
            }
        }

        // Sort patterns by occurrence count (descending)
        patterns.sort_by(|a, b| b.occurrence_count.cmp(&a.occurrence_count));

        // Top failing metrics
        let mut metric_counts: Vec<(String, u32)> = metric_occurrences
            .iter()
            .map(|(k, v)| (k.clone(), v.len() as u32))
            .collect();
        metric_counts.sort_by(|a, b| b.1.cmp(&a.1));

        RecurrenceReport {
            total_loops_analyzed: in_window.len(),
            resolved_count,
            closed_count,
            active_count,
            patterns,
            top_failing_metrics: metric_counts,
        }
    }
}

/// Internal tracking for a metric occurrence.
struct MetricOccurrence {
    loop_id: String,
    delta: f64,
    timestamp: DateTime<Utc>,
    #[allow(dead_code)]
    is_blocking: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::remediation::types::FailedMetric;

    fn make_gate_loop(
        id: &str,
        metrics: Vec<FailedMetric>,
        phase: RemediationLoopPhase,
    ) -> RemediationLoop {
        let trigger = RemediationTrigger::GateFailure {
            gate_config_ref: "QGC-0001".to_string(),
            failed_metrics: metrics,
            from_phase: None,
            to_phase: None,
        };
        let mut rl = RemediationLoop::new(id.to_string(), trigger);
        // Force phase for testing
        rl.phase = phase;
        rl
    }

    #[test]
    fn test_no_recurrence_below_threshold() {
        let loops = vec![make_gate_loop(
            "RL-001",
            vec![FailedMetric::new("lint_errors", 10.0, 15.0, true)],
            RemediationLoopPhase::Resolved,
        )];

        let report = RecurrenceDetector::detect(&loops, 2, 90);
        assert!(report.patterns.is_empty());
        assert_eq!(report.total_loops_analyzed, 1);
        assert_eq!(report.resolved_count, 1);
    }

    #[test]
    fn test_recurrence_detected() {
        let loops = vec![
            make_gate_loop(
                "RL-001",
                vec![FailedMetric::new("lint_errors", 10.0, 15.0, true)],
                RemediationLoopPhase::Resolved,
            ),
            make_gate_loop(
                "RL-002",
                vec![FailedMetric::new("lint_errors", 10.0, 20.0, true)],
                RemediationLoopPhase::Resolved,
            ),
            make_gate_loop(
                "RL-003",
                vec![FailedMetric::new("lint_errors", 10.0, 12.0, true)],
                RemediationLoopPhase::Investigating,
            ),
        ];

        let report = RecurrenceDetector::detect(&loops, 2, 90);
        assert_eq!(report.patterns.len(), 1);
        assert_eq!(report.patterns[0].metric_name, "lint_errors");
        assert_eq!(report.patterns[0].occurrence_count, 3);
        assert!(report.patterns[0].is_systemic);
        assert_eq!(report.patterns[0].loop_refs.len(), 3);
    }

    #[test]
    fn test_multiple_metrics_tracked() {
        let loops = vec![
            make_gate_loop(
                "RL-001",
                vec![
                    FailedMetric::new("lint_errors", 10.0, 15.0, true),
                    FailedMetric::new("warnings", 20.0, 30.0, false),
                ],
                RemediationLoopPhase::Resolved,
            ),
            make_gate_loop(
                "RL-002",
                vec![FailedMetric::new("lint_errors", 10.0, 12.0, true)],
                RemediationLoopPhase::Resolved,
            ),
        ];

        let report = RecurrenceDetector::detect(&loops, 2, 90);
        // lint_errors appears twice (meets threshold), warnings only once
        assert_eq!(report.patterns.len(), 1);
        assert_eq!(report.patterns[0].metric_name, "lint_errors");
        // Top failing metrics should have both
        assert_eq!(report.top_failing_metrics.len(), 2);
        assert_eq!(report.top_failing_metrics[0].0, "lint_errors");
        assert_eq!(report.top_failing_metrics[0].1, 2);
    }

    #[test]
    fn test_report_counts() {
        let loops = vec![
            make_gate_loop(
                "RL-001",
                vec![FailedMetric::new("errors", 10.0, 15.0, true)],
                RemediationLoopPhase::Resolved,
            ),
            make_gate_loop(
                "RL-002",
                vec![FailedMetric::new("errors", 10.0, 20.0, true)],
                RemediationLoopPhase::Closed,
            ),
            make_gate_loop(
                "RL-003",
                vec![FailedMetric::new("errors", 10.0, 12.0, true)],
                RemediationLoopPhase::Investigating,
            ),
        ];

        let report = RecurrenceDetector::detect(&loops, 2, 90);
        assert_eq!(report.total_loops_analyzed, 3);
        assert_eq!(report.resolved_count, 1);
        assert_eq!(report.closed_count, 1);
        assert_eq!(report.active_count, 1);
    }

    #[test]
    fn test_avg_delta_calculation() {
        let loops = vec![
            make_gate_loop(
                "RL-001",
                vec![FailedMetric::new("errors", 10.0, 15.0, true)], // delta = 5
                RemediationLoopPhase::Resolved,
            ),
            make_gate_loop(
                "RL-002",
                vec![FailedMetric::new("errors", 10.0, 20.0, true)], // delta = 10
                RemediationLoopPhase::Resolved,
            ),
        ];

        let report = RecurrenceDetector::detect(&loops, 2, 90);
        assert_eq!(report.patterns.len(), 1);
        // Average delta should be (5 + 10) / 2 = 7.5
        assert!((report.patterns[0].avg_delta - 7.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_manual_triggers_not_counted_for_metric_recurrence() {
        let trigger = RemediationTrigger::Manual {
            triggered_by: "test".to_string(),
            reason: "test".to_string(),
        };
        let mut rl = RemediationLoop::new("RL-001".to_string(), trigger);
        rl.phase = RemediationLoopPhase::Resolved;

        let report = RecurrenceDetector::detect(&[rl], 1, 90);
        // Manual triggers have no failed metrics, so no patterns
        assert!(report.patterns.is_empty());
        assert!(report.top_failing_metrics.is_empty());
    }

    #[test]
    fn test_empty_loops() {
        let report = RecurrenceDetector::detect(&[], 2, 90);
        assert_eq!(report.total_loops_analyzed, 0);
        assert!(report.patterns.is_empty());
        assert!(report.top_failing_metrics.is_empty());
    }

    #[test]
    fn test_recurrence_report_serde() {
        let report = RecurrenceReport {
            total_loops_analyzed: 5,
            resolved_count: 3,
            closed_count: 1,
            active_count: 1,
            patterns: vec![RecurrencePattern {
                metric_name: "lint_errors".to_string(),
                occurrence_count: 3,
                time_span_days: 30,
                avg_delta: 5.0,
                is_systemic: true,
                loop_refs: vec!["RL-001".to_string()],
            }],
            top_failing_metrics: vec![("lint_errors".to_string(), 3)],
        };

        let json = serde_json::to_string(&report).unwrap();
        let deserialized: RecurrenceReport = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_loops_analyzed, 5);
        assert_eq!(deserialized.patterns.len(), 1);
    }
}

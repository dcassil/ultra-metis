use super::types::{MetricDelta, ParsedToolOutput, TrendDirection};
use crate::domain::documents::quality_record::{NewQualityRecordParams, QualityRecord, QualityStatus};
use crate::domain::documents::traits::DocumentValidationError;
use crate::domain::documents::types::{Phase, Tag};
use std::collections::{HashMap, HashSet};

/// Engine for comparing two baselines and producing a QualityRecord.
pub struct BaselineComparisonEngine;

/// Result of comparing two baselines.
#[derive(Debug)]
pub struct ComparisonResult {
    pub metric_deltas: Vec<MetricDelta>,
    pub new_findings: Vec<String>, // finding keys only present in "after"
    pub resolved_findings: Vec<String>, // finding keys only present in "before"
    pub overall_status: QualityStatus,
    pub files_improved: Vec<String>,
    pub files_regressed: Vec<String>,
}

impl BaselineComparisonEngine {
    /// Compare two ParsedToolOutputs (before and after) for the same tool.
    pub fn compare(
        before: &ParsedToolOutput,
        after: &ParsedToolOutput,
    ) -> Result<ComparisonResult, String> {
        if before.tool_name != after.tool_name {
            return Err(format!(
                "Cannot compare different tools: '{}' vs '{}'",
                before.tool_name, after.tool_name
            ));
        }

        let metric_deltas = Self::compute_metric_deltas(before, after);
        let (new_findings, resolved_findings) = Self::compute_finding_diff(before, after);
        let (files_improved, files_regressed) = Self::compute_file_changes(before, after);

        let overall_status =
            Self::determine_status(&metric_deltas, &new_findings, &resolved_findings);

        Ok(ComparisonResult {
            metric_deltas,
            new_findings,
            resolved_findings,
            overall_status,
            files_improved,
            files_regressed,
        })
    }

    /// Create a QualityRecord from a comparison result.
    pub fn to_quality_record(
        comparison: &ComparisonResult,
        before: &ParsedToolOutput,
        after: &ParsedToolOutput,
        short_code: &str,
        linked_baseline: Option<String>,
    ) -> Result<QualityRecord, DocumentValidationError> {
        let title = format!("{} Quality Comparison", capitalize(&after.tool_name));
        let body = Self::build_body(comparison, before, after);

        let tags = vec![
            Tag::Label("quality_record".to_string()),
            Tag::Phase(Phase::Draft),
        ];

        // Use new_with_template with our generated body as the template
        QualityRecord::new_with_template(
            title,
            tags,
            false,
            short_code.to_string(),
            NewQualityRecordParams {
                linked_baseline,
                record_date: after.timestamp.format("%Y-%m-%d").to_string(),
                overall_status: comparison.overall_status,
            },
            &body,
        )
    }

    fn compute_metric_deltas(
        before: &ParsedToolOutput,
        after: &ParsedToolOutput,
    ) -> Vec<MetricDelta> {
        let mut deltas = Vec::new();

        // Build maps of summary-level metrics
        let before_summary = &before.summary;
        let after_summary = &after.summary;

        let all_keys: HashSet<&String> =
            before_summary.keys().chain(after_summary.keys()).collect();
        let mut sorted_keys: Vec<&&String> = all_keys.iter().collect();
        sorted_keys.sort();

        for key in sorted_keys {
            let before_val = before_summary.get(*key).copied().unwrap_or(0.0);
            let after_val = after_summary.get(*key).copied().unwrap_or(0.0);
            let delta = after_val - before_val;

            // For error/warning counts, decreasing is improvement
            // For coverage, increasing is improvement
            let direction = if (delta.abs()) < f64::EPSILON {
                TrendDirection::Unchanged
            } else if is_higher_better(key) {
                if delta > 0.0 {
                    TrendDirection::Improved
                } else {
                    TrendDirection::Regressed
                }
            } else {
                // Lower is better (error counts, warning counts)
                if delta < 0.0 {
                    TrendDirection::Improved
                } else {
                    TrendDirection::Regressed
                }
            };

            deltas.push(MetricDelta {
                metric_name: (*key).clone(),
                before: before_val,
                after: after_val,
                delta,
                direction,
            });
        }

        deltas
    }

    fn compute_finding_diff(
        before: &ParsedToolOutput,
        after: &ParsedToolOutput,
    ) -> (Vec<String>, Vec<String>) {
        let before_keys: HashSet<String> =
            before.findings.iter().map(super::types::FindingEntry::finding_key).collect();
        let after_keys: HashSet<String> = after.findings.iter().map(super::types::FindingEntry::finding_key).collect();

        let new_findings: Vec<String> = after_keys.difference(&before_keys).cloned().collect();
        let resolved_findings: Vec<String> = before_keys.difference(&after_keys).cloned().collect();

        (new_findings, resolved_findings)
    }

    fn compute_file_changes(
        before: &ParsedToolOutput,
        after: &ParsedToolOutput,
    ) -> (Vec<String>, Vec<String>) {
        let mut before_counts: HashMap<String, usize> = HashMap::new();
        let mut after_counts: HashMap<String, usize> = HashMap::new();

        for f in &before.findings {
            *before_counts.entry(f.file_path.clone()).or_default() += 1;
        }
        for f in &after.findings {
            *after_counts.entry(f.file_path.clone()).or_default() += 1;
        }

        let all_files: HashSet<&String> = before_counts.keys().chain(after_counts.keys()).collect();

        let mut improved = Vec::new();
        let mut regressed = Vec::new();

        for file in all_files {
            let before_count = before_counts.get(file).copied().unwrap_or(0);
            let after_count = after_counts.get(file).copied().unwrap_or(0);

            if after_count < before_count {
                improved.push(file.clone());
            } else if after_count > before_count {
                regressed.push(file.clone());
            }
        }

        improved.sort();
        regressed.sort();
        (improved, regressed)
    }

    fn determine_status(
        deltas: &[MetricDelta],
        new_findings: &[String],
        resolved_findings: &[String],
    ) -> QualityStatus {
        let has_regressions = deltas
            .iter()
            .any(|d| d.direction == TrendDirection::Regressed);
        let has_new_issues = !new_findings.is_empty();
        let has_improvements = deltas
            .iter()
            .any(|d| d.direction == TrendDirection::Improved);
        let has_resolved = !resolved_findings.is_empty();

        if has_regressions || has_new_issues {
            if has_improvements || has_resolved {
                QualityStatus::Warn // Mixed results
            } else {
                QualityStatus::Fail
            }
        } else {
            QualityStatus::Pass // Improvements, resolved findings, or no changes
        }
    }

    fn build_body(
        comparison: &ComparisonResult,
        before: &ParsedToolOutput,
        after: &ParsedToolOutput,
    ) -> String {
        let mut body = String::new();

        body.push_str(&format!(
            "# {} Quality Comparison\n\n",
            capitalize(&after.tool_name)
        ));

        body.push_str("## Overview\n\n");
        body.push_str(&format!("- **Tool**: {}\n", after.tool_name));
        body.push_str(&format!(
            "- **Before**: {}\n",
            before.timestamp.to_rfc3339()
        ));
        body.push_str(&format!("- **After**: {}\n", after.timestamp.to_rfc3339()));
        body.push_str(&format!(
            "- **Status**: {}\n\n",
            comparison.overall_status.as_str()
        ));

        // Metric deltas
        if !comparison.metric_deltas.is_empty() {
            body.push_str("## Metric Deltas\n\n");
            body.push_str("| Metric | Before | After | Delta | Direction |\n");
            body.push_str("|--------|--------|-------|-------|-----------|\n");
            for delta in &comparison.metric_deltas {
                body.push_str(&format!(
                    "| {} | {:.2} | {:.2} | {:+.2} | {} |\n",
                    delta.metric_name, delta.before, delta.after, delta.delta, delta.direction
                ));
            }
            body.push('\n');
        }

        // New findings
        if !comparison.new_findings.is_empty() {
            body.push_str("## New Findings\n\n");
            for key in &comparison.new_findings {
                body.push_str(&format!("- {key}\n"));
            }
            body.push('\n');
        }

        // Resolved findings
        if !comparison.resolved_findings.is_empty() {
            body.push_str("## Resolved Findings\n\n");
            for key in &comparison.resolved_findings {
                body.push_str(&format!("- {key}\n"));
            }
            body.push('\n');
        }

        // File-level changes
        if !comparison.files_improved.is_empty() || !comparison.files_regressed.is_empty() {
            body.push_str("## File-Level Changes\n\n");
            if !comparison.files_improved.is_empty() {
                body.push_str("### Improved Files\n\n");
                for file in &comparison.files_improved {
                    body.push_str(&format!("- {file}\n"));
                }
                body.push('\n');
            }
            if !comparison.files_regressed.is_empty() {
                body.push_str("### Regressed Files\n\n");
                for file in &comparison.files_regressed {
                    body.push_str(&format!("- {file}\n"));
                }
                body.push('\n');
            }
        }

        body
    }
}

/// Returns true if a higher value for this metric is better.
fn is_higher_better(metric_name: &str) -> bool {
    matches!(
        metric_name,
        "line_coverage"
            | "function_coverage"
            | "branch_coverage"
            | "files_covered"
            | "total_lines_hit"
            | "total_functions_hit"
    )
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::quality::types::{FindingEntry, Severity};

    fn make_before() -> ParsedToolOutput {
        let mut output = ParsedToolOutput::new("eslint");
        output.summary.insert("total_errors".to_string(), 5.0);
        output.summary.insert("total_warnings".to_string(), 10.0);
        output
            .findings
            .push(FindingEntry::new("rule1", Severity::Error, "err1", "a.js").with_location(1, 1));
        output.findings.push(
            FindingEntry::new("rule2", Severity::Warning, "warn1", "a.js").with_location(5, 1),
        );
        output
            .findings
            .push(FindingEntry::new("rule3", Severity::Error, "err2", "b.js").with_location(10, 1));
        output
    }

    fn make_after() -> ParsedToolOutput {
        let mut output = ParsedToolOutput::new("eslint");
        output.summary.insert("total_errors".to_string(), 3.0);
        output.summary.insert("total_warnings".to_string(), 12.0);
        // rule1 in a.js resolved, rule2 still present, rule3 in b.js resolved
        output.findings.push(
            FindingEntry::new("rule2", Severity::Warning, "warn1", "a.js").with_location(5, 1),
        );
        // new finding
        output.findings.push(
            FindingEntry::new("rule4", Severity::Warning, "warn2", "c.js").with_location(1, 1),
        );
        output
    }

    #[test]
    fn test_compare_basic() {
        let before = make_before();
        let after = make_after();

        let result = BaselineComparisonEngine::compare(&before, &after).unwrap();

        // Errors went from 5 to 3 (improved), warnings went from 10 to 12 (regressed)
        let error_delta = result
            .metric_deltas
            .iter()
            .find(|d| d.metric_name == "total_errors")
            .unwrap();
        assert_eq!(error_delta.direction, TrendDirection::Improved);
        assert_eq!(error_delta.delta, -2.0);

        let warning_delta = result
            .metric_deltas
            .iter()
            .find(|d| d.metric_name == "total_warnings")
            .unwrap();
        assert_eq!(warning_delta.direction, TrendDirection::Regressed);
        assert_eq!(warning_delta.delta, 2.0);
    }

    #[test]
    fn test_compare_finding_diff() {
        let before = make_before();
        let after = make_after();

        let result = BaselineComparisonEngine::compare(&before, &after).unwrap();

        // rule1:a.js:1 and rule3:b.js:10 resolved
        assert_eq!(result.resolved_findings.len(), 2);

        // rule4:c.js:1 is new
        assert_eq!(result.new_findings.len(), 1);
        assert!(result.new_findings[0].contains("rule4"));
    }

    #[test]
    fn test_compare_file_changes() {
        let before = make_before();
        let after = make_after();

        let result = BaselineComparisonEngine::compare(&before, &after).unwrap();

        // a.js: 2 findings -> 1 (improved)
        assert!(result.files_improved.contains(&"a.js".to_string()));
        // b.js: 1 finding -> 0 (improved)
        assert!(result.files_improved.contains(&"b.js".to_string()));
        // c.js: 0 findings -> 1 (regressed)
        assert!(result.files_regressed.contains(&"c.js".to_string()));
    }

    #[test]
    fn test_compare_mixed_status() {
        let before = make_before();
        let after = make_after();

        let result = BaselineComparisonEngine::compare(&before, &after).unwrap();

        // Mixed: some improvements, some regressions
        assert_eq!(result.overall_status, QualityStatus::Warn);
    }

    #[test]
    fn test_compare_all_improved() {
        let mut before = ParsedToolOutput::new("eslint");
        before.summary.insert("total_errors".to_string(), 5.0);
        before
            .findings
            .push(FindingEntry::new("r1", Severity::Error, "e1", "a.js").with_location(1, 1));

        let mut after = ParsedToolOutput::new("eslint");
        after.summary.insert("total_errors".to_string(), 0.0);

        let result = BaselineComparisonEngine::compare(&before, &after).unwrap();
        assert_eq!(result.overall_status, QualityStatus::Pass);
    }

    #[test]
    fn test_compare_all_regressed() {
        let before = ParsedToolOutput::new("eslint");

        let mut after = ParsedToolOutput::new("eslint");
        after.summary.insert("total_errors".to_string(), 5.0);
        after
            .findings
            .push(FindingEntry::new("r1", Severity::Error, "e1", "a.js").with_location(1, 1));

        let result = BaselineComparisonEngine::compare(&before, &after).unwrap();
        assert_eq!(result.overall_status, QualityStatus::Fail);
    }

    #[test]
    fn test_compare_different_tools_error() {
        let before = ParsedToolOutput::new("eslint");
        let after = ParsedToolOutput::new("clippy");

        let err = BaselineComparisonEngine::compare(&before, &after).unwrap_err();
        assert!(err.contains("Cannot compare different tools"));
    }

    #[test]
    fn test_to_quality_record() {
        let before = make_before();
        let after = make_after();

        let result = BaselineComparisonEngine::compare(&before, &after).unwrap();
        let qr = BaselineComparisonEngine::to_quality_record(
            &result,
            &before,
            &after,
            "QR-0001",
            Some("AB-0001".to_string()),
        )
        .unwrap();

        assert_eq!(qr.title(), "Eslint Quality Comparison");
        assert_eq!(qr.linked_baseline.as_deref(), Some("AB-0001"));
        assert_eq!(qr.overall_status, QualityStatus::Warn);
    }
}

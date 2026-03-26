use super::types::ParsedToolOutput;
use crate::domain::documents::analysis_baseline::AnalysisBaseline;
use crate::domain::documents::content::DocumentContent;
use crate::domain::documents::metadata::DocumentMetadata;
use crate::domain::documents::traits::DocumentValidationError;
use crate::domain::documents::types::{Phase, Tag};

/// Service for capturing tool output into durable AnalysisBaseline documents.
pub struct BaselineCaptureService;

impl BaselineCaptureService {
    /// Create an AnalysisBaseline from parsed tool output.
    pub fn capture(
        parsed: &ParsedToolOutput,
        short_code: &str,
        linked_rules_config: Option<String>,
    ) -> Result<AnalysisBaseline, DocumentValidationError> {
        let title = format!("{} Baseline", capitalize_tool_name(&parsed.tool_name));
        let body = Self::build_body(parsed);

        let metadata = DocumentMetadata::new(short_code.to_string());
        let content = DocumentContent::new(&body);
        let tags = vec![
            Tag::Label("analysis_baseline".to_string()),
            Tag::Phase(Phase::Draft),
        ];

        Ok(AnalysisBaseline::from_parts(
            title,
            metadata,
            content,
            tags,
            false,
            linked_rules_config,
            parsed.timestamp.format("%Y-%m-%d").to_string(),
        ))
    }

    /// Build the markdown body for the baseline document.
    fn build_body(parsed: &ParsedToolOutput) -> String {
        let mut body = String::new();

        body.push_str(&format!(
            "# {} Baseline\n\n",
            capitalize_tool_name(&parsed.tool_name)
        ));

        // Summary section
        body.push_str("## Summary\n\n");
        body.push_str(&format!("- **Tool**: {}\n", parsed.tool_name));
        body.push_str(&format!(
            "- **Timestamp**: {}\n",
            parsed.timestamp.to_rfc3339()
        ));
        body.push_str(&format!(
            "- **Total findings**: {}\n",
            parsed.total_findings()
        ));
        body.push_str(&format!("- **Errors**: {}\n", parsed.error_count()));
        body.push_str(&format!("- **Warnings**: {}\n", parsed.warning_count()));
        body.push_str(&format!("- **Info**: {}\n\n", parsed.info_count()));

        // Metrics section
        if !parsed.metrics.is_empty() {
            body.push_str("## Metrics\n\n");
            body.push_str("| Metric | Value | Unit | File |\n");
            body.push_str("|--------|-------|------|------|\n");
            for metric in &parsed.metrics {
                let file = metric.file_path.as_deref().unwrap_or("-");
                body.push_str(&format!(
                    "| {} | {:.2} | {} | {} |\n",
                    metric.metric_name, metric.value, metric.unit, file
                ));
            }
            body.push('\n');
        }

        // Summary metrics
        if !parsed.summary.is_empty() {
            body.push_str("## Summary Metrics\n\n");
            let mut keys: Vec<&String> = parsed.summary.keys().collect();
            keys.sort();
            for key in keys {
                body.push_str(&format!("- **{}**: {:.2}\n", key, parsed.summary[key]));
            }
            body.push('\n');
        }

        // Findings by file
        if !parsed.findings.is_empty() {
            body.push_str("## Findings by File\n\n");
            let by_file = parsed.findings_by_file();
            let mut files: Vec<&&str> = by_file.keys().collect();
            files.sort();
            for file in files {
                let findings = &by_file[*file];
                body.push_str(&format!("### {file}\n\n"));
                for finding in findings {
                    let loc = match (finding.line, finding.column) {
                        (Some(l), Some(c)) => format!("{l}:{c}"),
                        (Some(l), None) => format!("{l}"),
                        _ => "-".to_string(),
                    };
                    body.push_str(&format!(
                        "- **{}** [{}] ({}): {}\n",
                        finding.severity, finding.rule_id, loc, finding.message
                    ));
                }
                body.push('\n');
            }
        }

        body
    }
}

fn capitalize_tool_name(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::quality::types::{FindingEntry, MetricEntry, Severity};

    fn make_parsed_output() -> ParsedToolOutput {
        let mut output = ParsedToolOutput::new("eslint");
        output
            .metrics
            .push(MetricEntry::new("total_errors", 2.0, "count"));
        output
            .metrics
            .push(MetricEntry::new("total_warnings", 1.0, "count"));
        output.findings.push(
            FindingEntry::new(
                "no-unused-vars",
                Severity::Error,
                "unused var",
                "src/app.js",
            )
            .with_location(10, 5),
        );
        output.findings.push(
            FindingEntry::new("semi", Severity::Warning, "missing semicolon", "src/app.js")
                .with_location(20, 1),
        );
        output.findings.push(
            FindingEntry::new("no-console", Severity::Error, "console use", "src/utils.js")
                .with_location(3, 1),
        );
        output.summary.insert("total_errors".to_string(), 2.0);
        output.summary.insert("total_warnings".to_string(), 1.0);
        output
    }

    #[test]
    fn test_capture_creates_baseline() {
        let parsed = make_parsed_output();
        let baseline = BaselineCaptureService::capture(&parsed, "AB-0001", None).unwrap();

        assert_eq!(baseline.title(), "Eslint Baseline");
        assert_eq!(baseline.phase().unwrap(), Phase::Draft);
    }

    #[test]
    fn test_capture_with_rules_config_ref() {
        let parsed = make_parsed_output();
        let baseline =
            BaselineCaptureService::capture(&parsed, "AB-0002", Some("RC-0001".to_string()))
                .unwrap();

        assert_eq!(baseline.linked_rules_config.as_deref(), Some("RC-0001"));
    }

    #[test]
    fn test_capture_body_contains_summary() {
        let parsed = make_parsed_output();
        let baseline = BaselineCaptureService::capture(&parsed, "AB-0003", None).unwrap();
        let body = &baseline.content().body;

        assert!(body.contains("Total findings"));
        assert!(body.contains("Errors"));
        assert!(body.contains("Warnings"));
    }

    #[test]
    fn test_capture_body_contains_findings() {
        let parsed = make_parsed_output();
        let baseline = BaselineCaptureService::capture(&parsed, "AB-0004", None).unwrap();
        let body = &baseline.content().body;

        assert!(body.contains("src/app.js"));
        assert!(body.contains("no-unused-vars"));
        assert!(body.contains("src/utils.js"));
    }

    #[tokio::test]
    async fn test_capture_roundtrip() {
        let parsed = make_parsed_output();
        let baseline = BaselineCaptureService::capture(&parsed, "AB-0005", None).unwrap();

        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("baseline.md");

        baseline.to_file(&path).await.unwrap();
        let loaded = AnalysisBaseline::from_file(&path).await.unwrap();

        assert_eq!(loaded.title(), baseline.title());
        assert_eq!(loaded.baseline_date, baseline.baseline_date);
        assert_eq!(loaded.linked_rules_config, baseline.linked_rules_config);
    }
}

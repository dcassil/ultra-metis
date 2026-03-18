use crate::domain::quality::parser::{ParserError, ToolOutputParser};
use crate::domain::quality::types::{FindingEntry, MetricEntry, ParsedToolOutput, Severity};
use serde::Deserialize;

/// Parser for ESLint JSON output format.
///
/// Expects the output from `eslint --format json`, which is an array
/// of file result objects.
pub struct EslintParser;

#[derive(Deserialize)]
struct EslintFileResult {
    #[serde(rename = "filePath")]
    file_path: String,
    messages: Vec<EslintMessage>,
    #[serde(rename = "errorCount")]
    error_count: u32,
    #[serde(rename = "warningCount")]
    warning_count: u32,
}

#[derive(Deserialize)]
struct EslintMessage {
    #[serde(rename = "ruleId")]
    rule_id: Option<String>,
    severity: u8, // 1 = warning, 2 = error
    message: String,
    line: Option<u32>,
    column: Option<u32>,
}

impl ToolOutputParser for EslintParser {
    fn tool_name(&self) -> &str {
        "eslint"
    }

    fn parse(&self, input: &str) -> Result<ParsedToolOutput, ParserError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(ParserError::InvalidFormat("Empty input".to_string()));
        }

        let file_results: Vec<EslintFileResult> = serde_json::from_str(trimmed)?;

        let mut output = ParsedToolOutput::new("eslint");

        let mut total_errors: u32 = 0;
        let mut total_warnings: u32 = 0;
        let mut files_with_issues: u32 = 0;

        for file_result in &file_results {
            total_errors += file_result.error_count;
            total_warnings += file_result.warning_count;

            if file_result.error_count > 0 || file_result.warning_count > 0 {
                files_with_issues += 1;
            }

            // Per-file metrics
            if file_result.error_count > 0 {
                output.metrics.push(
                    MetricEntry::new("error_count", file_result.error_count as f64, "count")
                        .with_file(&file_result.file_path),
                );
            }
            if file_result.warning_count > 0 {
                output.metrics.push(
                    MetricEntry::new("warning_count", file_result.warning_count as f64, "count")
                        .with_file(&file_result.file_path),
                );
            }

            for msg in &file_result.messages {
                let severity = match msg.severity {
                    2 => Severity::Error,
                    1 => Severity::Warning,
                    _ => Severity::Info,
                };

                let rule_id = msg.rule_id.as_deref().unwrap_or("unknown");
                let mut finding =
                    FindingEntry::new(rule_id, severity, &msg.message, &file_result.file_path);

                if let (Some(line), Some(col)) = (msg.line, msg.column) {
                    finding = finding.with_location(line, col);
                }

                output.findings.push(finding);
            }
        }

        // Summary metrics
        output.metrics.push(MetricEntry::new(
            "total_errors",
            total_errors as f64,
            "count",
        ));
        output.metrics.push(MetricEntry::new(
            "total_warnings",
            total_warnings as f64,
            "count",
        ));
        output.metrics.push(MetricEntry::new(
            "files_checked",
            file_results.len() as f64,
            "count",
        ));
        output.metrics.push(MetricEntry::new(
            "files_with_issues",
            files_with_issues as f64,
            "count",
        ));

        output
            .summary
            .insert("total_errors".to_string(), total_errors as f64);
        output
            .summary
            .insert("total_warnings".to_string(), total_warnings as f64);
        output
            .summary
            .insert("files_checked".to_string(), file_results.len() as f64);
        output
            .summary
            .insert("files_with_issues".to_string(), files_with_issues as f64);

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ESLINT_FIXTURE: &str = r#"[
  {
    "filePath": "/project/src/app.js",
    "messages": [
      {
        "ruleId": "no-unused-vars",
        "severity": 2,
        "message": "'foo' is defined but never used.",
        "line": 5,
        "column": 7,
        "nodeType": "Identifier"
      },
      {
        "ruleId": "semi",
        "severity": 1,
        "message": "Missing semicolon.",
        "line": 10,
        "column": 20,
        "nodeType": "ExpressionStatement"
      }
    ],
    "errorCount": 1,
    "warningCount": 1,
    "fixableErrorCount": 0,
    "fixableWarningCount": 1,
    "source": "..."
  },
  {
    "filePath": "/project/src/utils.js",
    "messages": [
      {
        "ruleId": "no-console",
        "severity": 1,
        "message": "Unexpected console statement.",
        "line": 3,
        "column": 1,
        "nodeType": "MemberExpression"
      }
    ],
    "errorCount": 0,
    "warningCount": 1,
    "fixableErrorCount": 0,
    "fixableWarningCount": 0,
    "source": "..."
  },
  {
    "filePath": "/project/src/clean.js",
    "messages": [],
    "errorCount": 0,
    "warningCount": 0,
    "fixableErrorCount": 0,
    "fixableWarningCount": 0,
    "source": "..."
  }
]"#;

    #[test]
    fn test_eslint_parser_basic() {
        let parser = EslintParser;
        assert_eq!(parser.tool_name(), "eslint");

        let output = parser.parse(ESLINT_FIXTURE).unwrap();
        assert_eq!(output.tool_name, "eslint");
        assert_eq!(output.total_findings(), 3);
        assert_eq!(output.error_count(), 1);
        assert_eq!(output.warning_count(), 2);
    }

    #[test]
    fn test_eslint_parser_summary() {
        let parser = EslintParser;
        let output = parser.parse(ESLINT_FIXTURE).unwrap();

        assert_eq!(output.summary.get("total_errors"), Some(&1.0));
        assert_eq!(output.summary.get("total_warnings"), Some(&2.0));
        assert_eq!(output.summary.get("files_checked"), Some(&3.0));
        assert_eq!(output.summary.get("files_with_issues"), Some(&2.0));
    }

    #[test]
    fn test_eslint_parser_findings_detail() {
        let parser = EslintParser;
        let output = parser.parse(ESLINT_FIXTURE).unwrap();

        let first = &output.findings[0];
        assert_eq!(first.rule_id, "no-unused-vars");
        assert_eq!(first.severity, Severity::Error);
        assert_eq!(first.file_path, "/project/src/app.js");
        assert_eq!(first.line, Some(5));
        assert_eq!(first.column, Some(7));

        let second = &output.findings[1];
        assert_eq!(second.rule_id, "semi");
        assert_eq!(second.severity, Severity::Warning);
    }

    #[test]
    fn test_eslint_parser_findings_by_file() {
        let parser = EslintParser;
        let output = parser.parse(ESLINT_FIXTURE).unwrap();

        let by_file = output.findings_by_file();
        assert_eq!(by_file.get("/project/src/app.js").unwrap().len(), 2);
        assert_eq!(by_file.get("/project/src/utils.js").unwrap().len(), 1);
        assert!(!by_file.contains_key("/project/src/clean.js"));
    }

    #[test]
    fn test_eslint_parser_empty_results() {
        let parser = EslintParser;
        let output = parser.parse("[]").unwrap();

        assert_eq!(output.total_findings(), 0);
        assert_eq!(output.summary.get("total_errors"), Some(&0.0));
    }

    #[test]
    fn test_eslint_parser_empty_input() {
        let parser = EslintParser;
        let err = parser.parse("").unwrap_err();
        assert!(matches!(err, ParserError::InvalidFormat(_)));
    }

    #[test]
    fn test_eslint_parser_invalid_json() {
        let parser = EslintParser;
        let err = parser.parse("not json").unwrap_err();
        assert!(matches!(err, ParserError::JsonError(_)));
    }

    #[test]
    fn test_eslint_parser_no_rule_id() {
        let input = r#"[{
            "filePath": "/test.js",
            "messages": [{"severity": 2, "message": "Parse error", "line": 1, "column": 1}],
            "errorCount": 1,
            "warningCount": 0
        }]"#;

        let parser = EslintParser;
        let output = parser.parse(input).unwrap();
        assert_eq!(output.findings[0].rule_id, "unknown");
    }
}

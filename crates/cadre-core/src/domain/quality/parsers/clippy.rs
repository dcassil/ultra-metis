use crate::domain::quality::parser::{ParserError, ToolOutputParser};
use crate::domain::quality::types::{FindingEntry, MetricEntry, ParsedToolOutput, Severity};
use serde::Deserialize;

/// Parser for `cargo clippy --message-format=json` output.
///
/// Clippy JSON output is one JSON object per line (newline-delimited JSON),
/// each representing a compiler message.
pub struct ClippyParser;

#[derive(Deserialize)]
struct ClippyMessage {
    reason: Option<String>,
    message: Option<ClippyDiagnostic>,
}

#[derive(Deserialize)]
struct ClippyDiagnostic {
    message: String,
    code: Option<ClippyCode>,
    level: String, // "warning", "error", "note", "help"
    spans: Vec<ClippySpan>,
}

#[derive(Deserialize)]
struct ClippyCode {
    code: String,
}

#[derive(Deserialize)]
struct ClippySpan {
    file_name: String,
    line_start: u32,
    column_start: u32,
    is_primary: bool,
}

impl ToolOutputParser for ClippyParser {
    fn tool_name(&self) -> &str {
        "clippy"
    }

    fn parse(&self, input: &str) -> Result<ParsedToolOutput, ParserError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(ParserError::InvalidFormat("Empty input".to_string()));
        }

        let mut output = ParsedToolOutput::new("clippy");
        let mut total_errors: u32 = 0;
        let mut total_warnings: u32 = 0;

        for line in trimmed.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let msg: ClippyMessage = match serde_json::from_str(line) {
                Ok(m) => m,
                Err(_) => continue, // Skip non-JSON lines
            };

            // Only process compiler-message reason types
            if msg.reason.as_deref() != Some("compiler-message") {
                continue;
            }

            let diagnostic = match msg.message {
                Some(d) => d,
                None => continue,
            };

            // Skip "note" and "help" level messages — only track warnings and errors
            let severity = match diagnostic.level.as_str() {
                "error" => {
                    total_errors += 1;
                    Severity::Error
                }
                "warning" => {
                    total_warnings += 1;
                    Severity::Warning
                }
                _ => continue,
            };

            let rule_id = diagnostic
                .code
                .as_ref()
                .map(|c| c.code.as_str())
                .unwrap_or("unknown");

            // Use primary span for location
            let primary_span = diagnostic.spans.iter().find(|s| s.is_primary);

            let (file_path, line_num, col_num) = match primary_span {
                Some(span) => (
                    span.file_name.as_str(),
                    Some(span.line_start),
                    Some(span.column_start),
                ),
                None => ("unknown", None, None),
            };

            let mut finding = FindingEntry::new(rule_id, severity, &diagnostic.message, file_path);
            if let (Some(l), Some(c)) = (line_num, col_num) {
                finding = finding.with_location(l, c);
            }

            output.findings.push(finding);
        }

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

        output
            .summary
            .insert("total_errors".to_string(), total_errors as f64);
        output
            .summary
            .insert("total_warnings".to_string(), total_warnings as f64);

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLIPPY_FIXTURE: &str = r#"{"reason":"compiler-artifact","package_id":"foo","target":{"kind":["lib"],"name":"foo"},"profile":{"opt_level":"0"},"features":[],"filenames":["target/debug/libfoo.rlib"]}
{"reason":"compiler-message","package_id":"foo","message":{"message":"unused variable: `x`","code":{"code":"unused_variables","explanation":null},"level":"warning","spans":[{"file_name":"src/lib.rs","byte_start":100,"byte_end":101,"line_start":10,"line_end":10,"column_start":9,"column_end":10,"is_primary":true,"text":[]}],"children":[],"rendered":"warning: unused variable"}}
{"reason":"compiler-message","package_id":"foo","message":{"message":"this function has too many arguments","code":{"code":"clippy::too_many_arguments","explanation":null},"level":"warning","spans":[{"file_name":"src/api.rs","byte_start":200,"byte_end":250,"line_start":25,"line_end":25,"column_start":1,"column_end":50,"is_primary":true,"text":[]}],"children":[],"rendered":"warning: too many arguments"}}
{"reason":"compiler-message","package_id":"foo","message":{"message":"cannot find value `y`","code":{"code":"E0425","explanation":null},"level":"error","spans":[{"file_name":"src/lib.rs","byte_start":300,"byte_end":301,"line_start":15,"line_end":15,"column_start":5,"column_end":6,"is_primary":true,"text":[]}],"children":[],"rendered":"error[E0425]: cannot find value"}}
{"reason":"build-finished","success":false}"#;

    #[test]
    fn test_clippy_parser_basic() {
        let parser = ClippyParser;
        assert_eq!(parser.tool_name(), "clippy");

        let output = parser.parse(CLIPPY_FIXTURE).unwrap();
        assert_eq!(output.tool_name, "clippy");
        assert_eq!(output.total_findings(), 3);
        assert_eq!(output.error_count(), 1);
        assert_eq!(output.warning_count(), 2);
    }

    #[test]
    fn test_clippy_parser_findings() {
        let parser = ClippyParser;
        let output = parser.parse(CLIPPY_FIXTURE).unwrap();

        let first = &output.findings[0];
        assert_eq!(first.rule_id, "unused_variables");
        assert_eq!(first.severity, Severity::Warning);
        assert_eq!(first.file_path, "src/lib.rs");
        assert_eq!(first.line, Some(10));

        let second = &output.findings[1];
        assert_eq!(second.rule_id, "clippy::too_many_arguments");

        let third = &output.findings[2];
        assert_eq!(third.rule_id, "E0425");
        assert_eq!(third.severity, Severity::Error);
    }

    #[test]
    fn test_clippy_parser_summary() {
        let parser = ClippyParser;
        let output = parser.parse(CLIPPY_FIXTURE).unwrap();

        assert_eq!(output.summary.get("total_errors"), Some(&1.0));
        assert_eq!(output.summary.get("total_warnings"), Some(&2.0));
    }

    #[test]
    fn test_clippy_parser_empty() {
        let parser = ClippyParser;
        let err = parser.parse("").unwrap_err();
        assert!(matches!(err, ParserError::InvalidFormat(_)));
    }

    #[test]
    fn test_clippy_parser_no_diagnostics() {
        let input = r#"{"reason":"compiler-artifact","package_id":"foo","target":{"kind":["lib"],"name":"foo"},"profile":{"opt_level":"0"},"features":[],"filenames":["target/debug/libfoo.rlib"]}
{"reason":"build-finished","success":true}"#;

        let parser = ClippyParser;
        let output = parser.parse(input).unwrap();
        assert_eq!(output.total_findings(), 0);
    }
}

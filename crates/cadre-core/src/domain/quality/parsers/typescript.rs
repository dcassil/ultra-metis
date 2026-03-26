use crate::domain::quality::parser::{ParserError, ToolOutputParser};
use crate::domain::quality::types::{FindingEntry, MetricEntry, ParsedToolOutput, Severity};
use regex::Regex;

/// Parser for TypeScript compiler (`tsc`) diagnostic output.
///
/// Expects standard tsc output with lines like:
/// `src/file.ts(10,5): error TS2304: Cannot find name 'foo'.`
pub struct TypeScriptParser;

impl ToolOutputParser for TypeScriptParser {
    fn tool_name(&self) -> &str {
        "typescript"
    }

    fn parse(&self, input: &str) -> Result<ParsedToolOutput, ParserError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(ParserError::InvalidFormat("Empty input".to_string()));
        }

        let mut output = ParsedToolOutput::new("typescript");
        let mut total_errors: u32 = 0;

        // Match: file(line,col): error TSxxxx: message
        let diagnostic_re =
            Regex::new(r"^(.+?)\((\d+),(\d+)\):\s+(error|warning)\s+(TS\d+):\s+(.+)$").unwrap();

        // Match summary line: Found N errors.
        let summary_re = Regex::new(r"Found (\d+) errors?").unwrap();

        for line in trimmed.lines() {
            let line = line.trim();

            if let Some(caps) = diagnostic_re.captures(line) {
                let file_path = caps.get(1).unwrap().as_str();
                let line_num: u32 = caps.get(2).unwrap().as_str().parse().unwrap_or(0);
                let col_num: u32 = caps.get(3).unwrap().as_str().parse().unwrap_or(0);
                let level = caps.get(4).unwrap().as_str();
                let code = caps.get(5).unwrap().as_str();
                let message = caps.get(6).unwrap().as_str();

                let severity = match level {
                    "error" => {
                        total_errors += 1;
                        Severity::Error
                    }
                    "warning" => Severity::Warning,
                    _ => Severity::Info,
                };

                output.findings.push(
                    FindingEntry::new(code, severity, message, file_path)
                        .with_location(line_num, col_num),
                );
            }

            if let Some(caps) = summary_re.captures(line) {
                if let Ok(count) = caps.get(1).unwrap().as_str().parse::<f64>() {
                    output.summary.insert("reported_errors".to_string(), count);
                }
            }
        }

        output.metrics.push(MetricEntry::new(
            "total_errors",
            f64::from(total_errors),
            "count",
        ));

        output
            .summary
            .insert("total_errors".to_string(), f64::from(total_errors));

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TSC_FIXTURE: &str = r"src/app.ts(10,5): error TS2304: Cannot find name 'foo'.
src/app.ts(15,10): error TS2551: Property 'baz' does not exist on type 'Bar'. Did you mean 'bar'?
src/utils.ts(3,1): error TS7006: Parameter 'x' implicitly has an 'any' type.

Found 3 errors.";

    #[test]
    fn test_typescript_parser_basic() {
        let parser = TypeScriptParser;
        assert_eq!(parser.tool_name(), "typescript");

        let output = parser.parse(TSC_FIXTURE).unwrap();
        assert_eq!(output.tool_name, "typescript");
        assert_eq!(output.total_findings(), 3);
        assert_eq!(output.error_count(), 3);
    }

    #[test]
    fn test_typescript_parser_findings() {
        let parser = TypeScriptParser;
        let output = parser.parse(TSC_FIXTURE).unwrap();

        let first = &output.findings[0];
        assert_eq!(first.rule_id, "TS2304");
        assert_eq!(first.severity, Severity::Error);
        assert_eq!(first.file_path, "src/app.ts");
        assert_eq!(first.line, Some(10));
        assert_eq!(first.column, Some(5));
        assert!(first.message.contains("Cannot find name"));
    }

    #[test]
    fn test_typescript_parser_summary() {
        let parser = TypeScriptParser;
        let output = parser.parse(TSC_FIXTURE).unwrap();

        assert_eq!(output.summary.get("total_errors"), Some(&3.0));
        assert_eq!(output.summary.get("reported_errors"), Some(&3.0));
    }

    #[test]
    fn test_typescript_parser_clean() {
        let parser = TypeScriptParser;
        let output = parser.parse("No errors found.").unwrap();
        assert_eq!(output.total_findings(), 0);
    }

    #[test]
    fn test_typescript_parser_empty() {
        let parser = TypeScriptParser;
        let err = parser.parse("").unwrap_err();
        assert!(matches!(err, ParserError::InvalidFormat(_)));
    }
}

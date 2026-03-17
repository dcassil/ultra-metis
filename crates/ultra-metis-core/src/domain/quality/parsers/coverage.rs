use crate::domain::quality::parser::{ParserError, ToolOutputParser};
use crate::domain::quality::types::{MetricEntry, ParsedToolOutput};

/// Parser for LCOV format coverage data.
///
/// LCOV format uses markers like:
/// - `SF:<file>` — source file
/// - `LF:<n>` — lines found (instrumentable)
/// - `LH:<n>` — lines hit (covered)
/// - `FNF:<n>` — functions found
/// - `FNH:<n>` — functions hit
/// - `BRF:<n>` — branches found
/// - `BRH:<n>` — branches hit
/// - `end_of_record`
pub struct CoverageParser;

impl ToolOutputParser for CoverageParser {
    fn tool_name(&self) -> &str {
        "coverage"
    }

    fn parse(&self, input: &str) -> Result<ParsedToolOutput, ParserError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(ParserError::InvalidFormat("Empty input".to_string()));
        }

        let mut output = ParsedToolOutput::new("coverage");

        let mut total_lines_found: u64 = 0;
        let mut total_lines_hit: u64 = 0;
        let mut total_functions_found: u64 = 0;
        let mut total_functions_hit: u64 = 0;
        let mut total_branches_found: u64 = 0;
        let mut total_branches_hit: u64 = 0;
        let mut current_file: Option<String> = None;
        let mut file_count: u32 = 0;

        for line in trimmed.lines() {
            let line = line.trim();

            if let Some(file) = line.strip_prefix("SF:") {
                current_file = Some(file.to_string());
                file_count += 1;
            } else if let Some(val) = line.strip_prefix("LF:") {
                if let Ok(n) = val.parse::<u64>() {
                    total_lines_found += n;
                    if let Some(ref file) = current_file {
                        output.metrics.push(
                            MetricEntry::new("lines_found", n as f64, "count").with_file(file),
                        );
                    }
                }
            } else if let Some(val) = line.strip_prefix("LH:") {
                if let Ok(n) = val.parse::<u64>() {
                    total_lines_hit += n;
                    if let Some(ref file) = current_file {
                        output.metrics.push(
                            MetricEntry::new("lines_hit", n as f64, "count").with_file(file),
                        );
                    }
                }
            } else if let Some(val) = line.strip_prefix("FNF:") {
                if let Ok(n) = val.parse::<u64>() {
                    total_functions_found += n;
                }
            } else if let Some(val) = line.strip_prefix("FNH:") {
                if let Ok(n) = val.parse::<u64>() {
                    total_functions_hit += n;
                }
            } else if let Some(val) = line.strip_prefix("BRF:") {
                if let Ok(n) = val.parse::<u64>() {
                    total_branches_found += n;
                }
            } else if let Some(val) = line.strip_prefix("BRH:") {
                if let Ok(n) = val.parse::<u64>() {
                    total_branches_hit += n;
                }
            } else if line == "end_of_record" {
                current_file = None;
            }
        }

        // Compute coverage percentages
        let line_coverage = if total_lines_found > 0 {
            (total_lines_hit as f64 / total_lines_found as f64) * 100.0
        } else {
            0.0
        };

        let function_coverage = if total_functions_found > 0 {
            (total_functions_hit as f64 / total_functions_found as f64) * 100.0
        } else {
            0.0
        };

        let branch_coverage = if total_branches_found > 0 {
            (total_branches_hit as f64 / total_branches_found as f64) * 100.0
        } else {
            0.0
        };

        output.metrics.push(MetricEntry::new(
            "line_coverage",
            line_coverage,
            "percent",
        ));
        output.metrics.push(MetricEntry::new(
            "function_coverage",
            function_coverage,
            "percent",
        ));
        output.metrics.push(MetricEntry::new(
            "branch_coverage",
            branch_coverage,
            "percent",
        ));
        output.metrics.push(MetricEntry::new(
            "files_covered",
            file_count as f64,
            "count",
        ));

        output
            .summary
            .insert("line_coverage".to_string(), line_coverage);
        output
            .summary
            .insert("function_coverage".to_string(), function_coverage);
        output
            .summary
            .insert("branch_coverage".to_string(), branch_coverage);
        output
            .summary
            .insert("total_lines_found".to_string(), total_lines_found as f64);
        output
            .summary
            .insert("total_lines_hit".to_string(), total_lines_hit as f64);
        output
            .summary
            .insert("files_covered".to_string(), file_count as f64);

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LCOV_FIXTURE: &str = r#"TN:
SF:src/app.ts
FN:1,main
FN:10,helper
FNDA:1,main
FNDA:0,helper
FNF:2
FNH:1
DA:1,1
DA:2,1
DA:3,1
DA:10,0
DA:11,0
LF:5
LH:3
BRF:2
BRH:1
end_of_record
SF:src/utils.ts
FN:1,add
FNDA:1,add
FNF:1
FNH:1
DA:1,1
DA:2,1
LF:2
LH:2
BRF:0
BRH:0
end_of_record"#;

    #[test]
    fn test_coverage_parser_basic() {
        let parser = CoverageParser;
        assert_eq!(parser.tool_name(), "coverage");

        let output = parser.parse(LCOV_FIXTURE).unwrap();
        assert_eq!(output.tool_name, "coverage");
    }

    #[test]
    fn test_coverage_parser_line_coverage() {
        let parser = CoverageParser;
        let output = parser.parse(LCOV_FIXTURE).unwrap();

        // 5 lines hit out of 7 total = ~71.4%
        let line_cov = output.summary.get("line_coverage").unwrap();
        assert!((line_cov - 71.42857).abs() < 0.01);

        assert_eq!(output.summary.get("total_lines_found"), Some(&7.0));
        assert_eq!(output.summary.get("total_lines_hit"), Some(&5.0));
    }

    #[test]
    fn test_coverage_parser_function_coverage() {
        let parser = CoverageParser;
        let output = parser.parse(LCOV_FIXTURE).unwrap();

        // 2 functions hit out of 3 total = ~66.7%
        let func_cov = output.summary.get("function_coverage").unwrap();
        assert!((func_cov - 66.6666).abs() < 0.01);
    }

    #[test]
    fn test_coverage_parser_branch_coverage() {
        let parser = CoverageParser;
        let output = parser.parse(LCOV_FIXTURE).unwrap();

        // 1 branch hit out of 2 total = 50%
        let branch_cov = output.summary.get("branch_coverage").unwrap();
        assert!((branch_cov - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_coverage_parser_file_count() {
        let parser = CoverageParser;
        let output = parser.parse(LCOV_FIXTURE).unwrap();

        assert_eq!(output.summary.get("files_covered"), Some(&2.0));
    }

    #[test]
    fn test_coverage_parser_empty() {
        let parser = CoverageParser;
        let err = parser.parse("").unwrap_err();
        assert!(matches!(err, ParserError::InvalidFormat(_)));
    }

    #[test]
    fn test_coverage_parser_no_data() {
        let parser = CoverageParser;
        let output = parser.parse("TN:\nend_of_record").unwrap();

        assert_eq!(output.summary.get("line_coverage"), Some(&0.0));
    }
}

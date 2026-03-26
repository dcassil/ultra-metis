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
    fn tool_name(&self) -> &'static str {
        "coverage"
    }

    fn parse(&self, input: &str) -> Result<ParsedToolOutput, ParserError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(ParserError::InvalidFormat("Empty input".to_string()));
        }

        let mut output = ParsedToolOutput::new("coverage");
        let totals = Self::parse_lcov_lines(trimmed, &mut output);
        Self::compute_and_insert_coverage(&mut output, &totals);

        Ok(output)
    }
}

/// Accumulated LCOV counters.
struct LcovTotals {
    lines_found: u64,
    lines_hit: u64,
    functions_found: u64,
    functions_hit: u64,
    branches_found: u64,
    branches_hit: u64,
    file_count: u32,
}

impl CoverageParser {
    fn parse_lcov_lines(input: &str, output: &mut ParsedToolOutput) -> LcovTotals {
        let mut totals = LcovTotals {
            lines_found: 0, lines_hit: 0, functions_found: 0,
            functions_hit: 0, branches_found: 0, branches_hit: 0, file_count: 0,
        };
        let mut current_file: Option<String> = None;

        for line in input.lines() {
            let line = line.trim();

            if let Some(file) = line.strip_prefix("SF:") {
                current_file = Some(file.to_string());
                totals.file_count += 1;
            } else if let Some(val) = line.strip_prefix("LF:") {
                if let Ok(n) = val.parse::<u64>() {
                    totals.lines_found += n;
                    if let Some(ref file) = current_file {
                        output.metrics.push(
                            MetricEntry::new("lines_found", n as f64, "count").with_file(file),
                        );
                    }
                }
            } else if let Some(val) = line.strip_prefix("LH:") {
                if let Ok(n) = val.parse::<u64>() {
                    totals.lines_hit += n;
                    if let Some(ref file) = current_file {
                        output.metrics.push(
                            MetricEntry::new("lines_hit", n as f64, "count").with_file(file),
                        );
                    }
                }
            } else if let Some(val) = line.strip_prefix("FNF:") {
                if let Ok(n) = val.parse::<u64>() { totals.functions_found += n; }
            } else if let Some(val) = line.strip_prefix("FNH:") {
                if let Ok(n) = val.parse::<u64>() { totals.functions_hit += n; }
            } else if let Some(val) = line.strip_prefix("BRF:") {
                if let Ok(n) = val.parse::<u64>() { totals.branches_found += n; }
            } else if let Some(val) = line.strip_prefix("BRH:") {
                if let Ok(n) = val.parse::<u64>() { totals.branches_hit += n; }
            } else if line == "end_of_record" {
                current_file = None;
            }
        }

        totals
    }

    fn compute_and_insert_coverage(output: &mut ParsedToolOutput, t: &LcovTotals) {
        let pct = |hit: u64, found: u64| -> f64 {
            if found > 0 { (hit as f64 / found as f64) * 100.0 } else { 0.0 }
        };

        let line_cov = pct(t.lines_hit, t.lines_found);
        let func_cov = pct(t.functions_hit, t.functions_found);
        let branch_cov = pct(t.branches_hit, t.branches_found);

        output.metrics.push(MetricEntry::new("line_coverage", line_cov, "percent"));
        output.metrics.push(MetricEntry::new("function_coverage", func_cov, "percent"));
        output.metrics.push(MetricEntry::new("branch_coverage", branch_cov, "percent"));
        output.metrics.push(MetricEntry::new("files_covered", f64::from(t.file_count), "count"));

        output.summary.insert("line_coverage".to_string(), line_cov);
        output.summary.insert("function_coverage".to_string(), func_cov);
        output.summary.insert("branch_coverage".to_string(), branch_cov);
        output.summary.insert("total_lines_found".to_string(), t.lines_found as f64);
        output.summary.insert("total_lines_hit".to_string(), t.lines_hit as f64);
        output.summary.insert("files_covered".to_string(), f64::from(t.file_count));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LCOV_FIXTURE: &str = r"TN:
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
end_of_record";

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

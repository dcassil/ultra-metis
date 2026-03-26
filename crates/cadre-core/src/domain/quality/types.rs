use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// Severity level for findings from static analysis tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
            Self::Info => write!(f, "info"),
        }
    }
}

impl FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "error" | "err" => Ok(Self::Error),
            "warning" | "warn" => Ok(Self::Warning),
            "info" | "note" | "suggestion" => Ok(Self::Info),
            _ => Err(format!("Unknown severity: {s}")),
        }
    }
}

/// A single numeric metric from tool output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricEntry {
    pub metric_name: String,
    pub value: f64,
    pub unit: String,
    pub file_path: Option<String>,
}

impl MetricEntry {
    pub fn new(metric_name: &str, value: f64, unit: &str) -> Self {
        Self {
            metric_name: metric_name.to_string(),
            value,
            unit: unit.to_string(),
            file_path: None,
        }
    }

    pub fn with_file(mut self, file_path: &str) -> Self {
        self.file_path = Some(file_path.to_string());
        self
    }
}

/// A single finding (issue/diagnostic) from tool output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingEntry {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub file_path: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

impl FindingEntry {
    pub fn new(rule_id: &str, severity: Severity, message: &str, file_path: &str) -> Self {
        Self {
            rule_id: rule_id.to_string(),
            severity,
            message: message.to_string(),
            file_path: file_path.to_string(),
            line: None,
            column: None,
        }
    }

    pub fn with_location(mut self, line: u32, column: u32) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    /// Unique key for deduplication/comparison (rule + file + line).
    pub fn finding_key(&self) -> String {
        format!(
            "{}:{}:{}",
            self.rule_id,
            self.file_path,
            self.line.unwrap_or(0)
        )
    }
}

/// Direction of a metric change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Improved,
    Regressed,
    Unchanged,
}

impl fmt::Display for TrendDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Improved => write!(f, "improved"),
            Self::Regressed => write!(f, "regressed"),
            Self::Unchanged => write!(f, "unchanged"),
        }
    }
}

/// Delta for a single metric between two baselines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDelta {
    pub metric_name: String,
    pub before: f64,
    pub after: f64,
    pub delta: f64,
    pub direction: TrendDirection,
}

/// Structured output from parsing a tool's results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedToolOutput {
    pub tool_name: String,
    pub timestamp: DateTime<Utc>,
    pub metrics: Vec<MetricEntry>,
    pub findings: Vec<FindingEntry>,
    pub summary: HashMap<String, f64>,
}

impl ParsedToolOutput {
    pub fn new(tool_name: &str) -> Self {
        Self {
            tool_name: tool_name.to_string(),
            timestamp: Utc::now(),
            metrics: Vec::new(),
            findings: Vec::new(),
            summary: HashMap::new(),
        }
    }

    pub fn error_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Warning)
            .count()
    }

    pub fn info_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Info)
            .count()
    }

    pub fn total_findings(&self) -> usize {
        self.findings.len()
    }

    /// Group findings by file path.
    pub fn findings_by_file(&self) -> HashMap<&str, Vec<&FindingEntry>> {
        let mut map: HashMap<&str, Vec<&FindingEntry>> = HashMap::new();
        for finding in &self.findings {
            map.entry(&finding.file_path).or_default().push(finding);
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_parsing() {
        assert_eq!("error".parse::<Severity>().unwrap(), Severity::Error);
        assert_eq!("warn".parse::<Severity>().unwrap(), Severity::Warning);
        assert_eq!("warning".parse::<Severity>().unwrap(), Severity::Warning);
        assert_eq!("info".parse::<Severity>().unwrap(), Severity::Info);
        assert_eq!("note".parse::<Severity>().unwrap(), Severity::Info);
        assert_eq!("suggestion".parse::<Severity>().unwrap(), Severity::Info);
        assert!("unknown".parse::<Severity>().is_err());
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(Severity::Error.to_string(), "error");
        assert_eq!(Severity::Warning.to_string(), "warning");
        assert_eq!(Severity::Info.to_string(), "info");
    }

    #[test]
    fn test_metric_entry() {
        let metric = MetricEntry::new("error_count", 5.0, "count");
        assert_eq!(metric.metric_name, "error_count");
        assert_eq!(metric.value, 5.0);
        assert!(metric.file_path.is_none());

        let with_file = metric.with_file("src/main.rs");
        assert_eq!(with_file.file_path.as_deref(), Some("src/main.rs"));
    }

    #[test]
    fn test_finding_entry() {
        let finding = FindingEntry::new(
            "no-unused-vars",
            Severity::Warning,
            "unused var",
            "src/app.js",
        )
        .with_location(10, 5);

        assert_eq!(finding.rule_id, "no-unused-vars");
        assert_eq!(finding.severity, Severity::Warning);
        assert_eq!(finding.line, Some(10));
        assert_eq!(finding.column, Some(5));
        assert_eq!(finding.finding_key(), "no-unused-vars:src/app.js:10");
    }

    #[test]
    fn test_parsed_tool_output_counts() {
        let mut output = ParsedToolOutput::new("eslint");
        output
            .findings
            .push(FindingEntry::new("rule1", Severity::Error, "err", "a.js"));
        output
            .findings
            .push(FindingEntry::new("rule2", Severity::Error, "err", "b.js"));
        output.findings.push(FindingEntry::new(
            "rule3",
            Severity::Warning,
            "warn",
            "a.js",
        ));
        output
            .findings
            .push(FindingEntry::new("rule4", Severity::Info, "info", "c.js"));

        assert_eq!(output.error_count(), 2);
        assert_eq!(output.warning_count(), 1);
        assert_eq!(output.info_count(), 1);
        assert_eq!(output.total_findings(), 4);
    }

    #[test]
    fn test_findings_by_file() {
        let mut output = ParsedToolOutput::new("eslint");
        output
            .findings
            .push(FindingEntry::new("r1", Severity::Error, "e1", "a.js"));
        output
            .findings
            .push(FindingEntry::new("r2", Severity::Warning, "w1", "a.js"));
        output
            .findings
            .push(FindingEntry::new("r3", Severity::Error, "e2", "b.js"));

        let by_file = output.findings_by_file();
        assert_eq!(by_file.get("a.js").unwrap().len(), 2);
        assert_eq!(by_file.get("b.js").unwrap().len(), 1);
    }

    #[test]
    fn test_trend_direction_display() {
        assert_eq!(TrendDirection::Improved.to_string(), "improved");
        assert_eq!(TrendDirection::Regressed.to_string(), "regressed");
        assert_eq!(TrendDirection::Unchanged.to_string(), "unchanged");
    }
}

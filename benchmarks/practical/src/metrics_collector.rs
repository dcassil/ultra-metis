use crate::types::CodeMetrics;
use std::path::Path;

pub struct MetricsCollector {
    code_metrics: CodeMetrics,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self {
            code_metrics: CodeMetrics {
                lines_of_code: 0,
                test_coverage_percent: 0.0,
                cyclomatic_complexity: 0.0,
                doc_accuracy_percent: 0.0,
                instruction_adherence_percent: 0.0,
            },
        }
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Count lines of code in generated Rust file
    pub fn collect_code_metrics(&mut self, code_path: &Path) -> anyhow::Result<()> {
        let content = std::fs::read_to_string(code_path)?;
        let lines: Vec<&str> = content.lines().collect();

        // Count non-empty, non-comment lines
        let mut code_lines = 0;
        for line in &lines {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with("//") {
                code_lines += 1;
            }
        }

        self.code_metrics.lines_of_code = code_lines as u32;
        Ok(())
    }

    /// Parse test output to extract coverage
    pub fn collect_test_coverage(&mut self, coverage_output: &str) -> anyhow::Result<()> {
        // Parse output like "Coverage: 85.5%"
        if let Some(line) = coverage_output.lines().find(|l| l.contains("Coverage:")) {
            if let Some(percent_str) = line.split_whitespace().find(|s| s.contains('%')) {
                let percent = percent_str.trim_end_matches('%').parse::<f32>()?;
                self.code_metrics.test_coverage_percent = percent;
            }
        }
        Ok(())
    }

    /// Manual review of doc accuracy (0-100%)
    pub fn set_doc_accuracy(&mut self, percent: f32) {
        self.code_metrics.doc_accuracy_percent = percent.clamp(0.0, 100.0);
    }

    /// Manual review of instruction adherence (0-100%)
    pub fn set_instruction_adherence(&mut self, percent: f32) {
        self.code_metrics.instruction_adherence_percent = percent.clamp(0.0, 100.0);
    }

    pub fn metrics(&self) -> &CodeMetrics {
        &self.code_metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_lines_counting() {
        let collector = MetricsCollector::new();
        // Would test with temp file
        assert_eq!(collector.code_metrics.lines_of_code, 0);
    }

    #[test]
    fn test_coverage_parsing() {
        let mut collector = MetricsCollector::new();
        let output = "Test results: PASS\nCoverage: 92.3%\n";
        collector.collect_test_coverage(output).unwrap();
        assert_eq!(collector.code_metrics.test_coverage_percent, 92.3);
    }
}

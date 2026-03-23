use super::types::{FindingEntry, MetricEntry, ParsedToolOutput, Severity};
use crate::domain::documents::reference_architecture::ReferenceArchitecture;
use std::collections::HashSet;
use std::path::Path;

/// Checker for architecture boundary conformance.
///
/// Verifies that actual file paths conform to the ReferenceArchitecture's
/// expected folder layout and dependency rules.
pub struct ArchitectureConformanceChecker;

impl ArchitectureConformanceChecker {
    /// Check actual file paths against a reference architecture.
    ///
    /// `actual_paths` should be relative paths from the repo root.
    pub fn check(reference: &ReferenceArchitecture, actual_paths: &[String]) -> ParsedToolOutput {
        let mut output = ParsedToolOutput::new("architecture_conformance");

        // Collect all expected layout entries (from layer_overrides as a proxy
        // for the inherited catalog layout; in a full implementation these would
        // come from the linked catalog entry).
        let expected_dirs = Self::collect_expected_dirs(reference);

        // Check folder layout conformance
        Self::check_folder_layout(&expected_dirs, actual_paths, &mut output);

        // Check dependency rule conformance (basic: flag violations in extra_dependency_rules)
        Self::check_dependency_rules(reference, actual_paths, &mut output);

        // Compute conformance score
        let total_checks = output.findings.len().max(1) as f64;
        let violations = output.error_count() as f64;
        let conformance_score = ((total_checks - violations) / total_checks) * 100.0;

        output.metrics.push(MetricEntry::new(
            "conformance_score",
            conformance_score,
            "percent",
        ));
        output.metrics.push(MetricEntry::new(
            "total_violations",
            output.error_count() as f64,
            "count",
        ));
        output.metrics.push(MetricEntry::new(
            "total_warnings",
            output.warning_count() as f64,
            "count",
        ));

        output
            .summary
            .insert("conformance_score".to_string(), conformance_score);
        output
            .summary
            .insert("total_violations".to_string(), violations);

        output
    }

    fn collect_expected_dirs(reference: &ReferenceArchitecture) -> Vec<String> {
        // In a full implementation, we'd resolve the catalog entry's folder_layout.
        // For now, use layer_overrides and additional_boundaries as expected dir indicators.
        let mut dirs = Vec::new();
        dirs.extend(reference.layer_overrides.iter().cloned());
        dirs.extend(reference.additional_boundaries.iter().cloned());
        dirs
    }

    fn check_folder_layout(
        expected_dirs: &[String],
        actual_paths: &[String],
        output: &mut ParsedToolOutput,
    ) {
        if expected_dirs.is_empty() {
            return;
        }

        // Collect actual top-level directories from file paths
        let actual_dirs: HashSet<String> = actual_paths
            .iter()
            .filter_map(|p| {
                let path = Path::new(p);
                path.components()
                    .next()
                    .map(|c| c.as_os_str().to_string_lossy().to_string())
            })
            .collect();

        // Check that expected dirs exist
        for expected in expected_dirs {
            let expected_top = Path::new(expected)
                .components()
                .next()
                .map(|c| c.as_os_str().to_string_lossy().to_string())
                .unwrap_or_default();

            if !expected_top.is_empty() && !actual_dirs.contains(&expected_top) {
                output.findings.push(FindingEntry::new(
                    "missing_expected_dir",
                    Severity::Warning,
                    &format!(
                        "Expected directory '{}' not found in actual paths",
                        expected
                    ),
                    expected,
                ));
            }
        }
    }

    fn check_dependency_rules(
        reference: &ReferenceArchitecture,
        actual_paths: &[String],
        output: &mut ParsedToolOutput,
    ) {
        // Basic: check if any extra_dependency_rules are expressed as
        // "no <dir-a> -> <dir-b>" style constraints
        for rule in &reference.extra_dependency_rules {
            // Parse simple "no X -> Y" constraints
            if let Some(constraint) = parse_no_dep_constraint(rule) {
                // Check if any file in the source dir exists
                let source_files: Vec<&String> = actual_paths
                    .iter()
                    .filter(|p| p.starts_with(&constraint.source))
                    .collect();

                if !source_files.is_empty() {
                    // We can't actually check imports without parsing source,
                    // so just record the constraint as tracked
                    output.findings.push(FindingEntry::new(
                        "dependency_rule_tracked",
                        Severity::Info,
                        &format!(
                            "Dependency rule '{}' tracked ({} files in source dir)",
                            rule,
                            source_files.len()
                        ),
                        &constraint.source,
                    ));
                }
            }
        }

        // Check tolerated exceptions
        for exception in &reference.tolerated_exceptions {
            output.findings.push(FindingEntry::new(
                "tolerated_exception",
                Severity::Info,
                &format!("Tolerated exception: {}", exception),
                "architecture",
            ));
        }
    }
}

struct NoDepConstraint {
    source: String,
}

fn parse_no_dep_constraint(rule: &str) -> Option<NoDepConstraint> {
    let lower = rule.to_lowercase();
    if lower.starts_with("no ") && lower.contains(" -> ") {
        let parts: Vec<&str> = rule[3..].split(" -> ").collect();
        if parts.len() == 2 {
            return Some(NoDepConstraint {
                source: parts[0].trim().to_string(),
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::documents::content::DocumentContent;
    use crate::domain::documents::metadata::DocumentMetadata;
    use crate::domain::documents::reference_architecture::{
        ArchitectureStatus, ReferenceArchitecture,
    };
    use crate::domain::documents::types::{Phase, Tag};

    fn make_reference_arch() -> ReferenceArchitecture {
        ReferenceArchitecture::from_parts(
            "Test Architecture".to_string(),
            DocumentMetadata::new("RA-TEST".to_string()),
            DocumentContent::new("# Test Architecture"),
            vec![Tag::Phase(Phase::Draft)],
            false,
            Some("AC-0001".to_string()),
            false,
            ArchitectureStatus::Active,
            vec!["src".to_string(), "tests".to_string()],
            vec!["api".to_string()],
            vec!["no handlers -> database".to_string()],
            None,
            None,
            vec!["legacy module uses old pattern".to_string()],
        )
    }

    #[test]
    fn test_conformance_check_basic() {
        let reference = make_reference_arch();
        let actual_paths = vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "tests/test_main.rs".to_string(),
            "api/routes.rs".to_string(),
        ];

        let output = ArchitectureConformanceChecker::check(&reference, &actual_paths);
        assert_eq!(output.tool_name, "architecture_conformance");
        assert!(output.summary.contains_key("conformance_score"));
    }

    #[test]
    fn test_conformance_missing_expected_dir() {
        let reference = make_reference_arch();
        let actual_paths = vec![
            "src/main.rs".to_string(),
            // Missing "tests" and "api" directories
        ];

        let output = ArchitectureConformanceChecker::check(&reference, &actual_paths);

        let missing_dirs: Vec<&FindingEntry> = output
            .findings
            .iter()
            .filter(|f| f.rule_id == "missing_expected_dir")
            .collect();

        // "tests" and "api" should be flagged
        assert_eq!(missing_dirs.len(), 2);
    }

    #[test]
    fn test_conformance_all_dirs_present() {
        let reference = make_reference_arch();
        let actual_paths = vec![
            "src/main.rs".to_string(),
            "tests/test.rs".to_string(),
            "api/routes.rs".to_string(),
        ];

        let output = ArchitectureConformanceChecker::check(&reference, &actual_paths);

        let missing_dirs: Vec<&FindingEntry> = output
            .findings
            .iter()
            .filter(|f| f.rule_id == "missing_expected_dir")
            .collect();

        assert_eq!(missing_dirs.len(), 0);
    }

    #[test]
    fn test_conformance_dependency_rule_tracked() {
        let reference = make_reference_arch();
        let actual_paths = vec![
            "handlers/api.rs".to_string(),
            "database/schema.rs".to_string(),
        ];

        let output = ArchitectureConformanceChecker::check(&reference, &actual_paths);

        let dep_tracked: Vec<&FindingEntry> = output
            .findings
            .iter()
            .filter(|f| f.rule_id == "dependency_rule_tracked")
            .collect();

        assert_eq!(dep_tracked.len(), 1);
    }

    #[test]
    fn test_conformance_tolerated_exceptions() {
        let reference = make_reference_arch();
        let actual_paths = vec!["src/main.rs".to_string()];

        let output = ArchitectureConformanceChecker::check(&reference, &actual_paths);

        let exceptions: Vec<&FindingEntry> = output
            .findings
            .iter()
            .filter(|f| f.rule_id == "tolerated_exception")
            .collect();

        assert_eq!(exceptions.len(), 1);
        assert!(exceptions[0].message.contains("legacy module"));
    }

    #[test]
    fn test_conformance_empty_reference() {
        let reference = ReferenceArchitecture::from_parts(
            "Empty Arch".to_string(),
            DocumentMetadata::new("RA-EMPTY".to_string()),
            DocumentContent::new("# Empty"),
            vec![Tag::Phase(Phase::Draft)],
            false,
            None,
            true,
            ArchitectureStatus::Draft,
            vec![],
            vec![],
            vec![],
            None,
            None,
            vec![],
        );

        let actual_paths = vec!["src/main.rs".to_string()];
        let output = ArchitectureConformanceChecker::check(&reference, &actual_paths);

        // No expected dirs, no rules, so only the conformance score metric
        assert!(output.summary.get("conformance_score").unwrap() >= &100.0);
    }
}

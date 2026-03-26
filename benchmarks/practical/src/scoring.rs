use crate::scenario_pack::{ExpectedOutputs, LoadedScenarioPack, VerificationCheck};
use crate::types::{BenchmarkRun, DocumentArtifact};
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Complete scoring breakdown for a benchmark run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub document_generation: TrackScore,
    pub decomposition: TrackScore,
    pub build_outcome: TrackScore,
    pub architecture_conformance: f32,
    pub static_tool_utilization: f32,
    pub tokens_total: u64,
    pub time_total_ms: u64,
}

/// Score for a single benchmark track with itemized checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackScore {
    pub score: f32,
    pub max_score: f32,
    pub checks: Vec<ScoringCheck>,
}

impl TrackScore {
    pub fn percent(&self) -> f32 {
        if self.max_score == 0.0 {
            0.0
        } else {
            (self.score / self.max_score) * 100.0
        }
    }
}

/// A single deterministic check with pass/fail and weight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringCheck {
    pub name: String,
    pub passed: bool,
    pub weight: f32,
    pub detail: String,
}

/// Score a benchmark run against scenario expectations.
pub fn score_run(run: &BenchmarkRun, scenario: &LoadedScenarioPack) -> ScoreBreakdown {
    let expected = &scenario.manifest.expected_outputs;

    let document_generation = score_document_generation(run, expected);
    let decomposition = score_decomposition(run, expected);
    let build_outcome = score_build_outcome(run, &scenario.manifest.verification);
    let architecture_conformance = score_architecture_conformance(run, expected);
    let static_tool_utilization = score_static_tool_utilization(run);

    ScoreBreakdown {
        document_generation,
        decomposition,
        build_outcome,
        architecture_conformance,
        static_tool_utilization,
        tokens_total: run.total_metrics.total_tokens,
        time_total_ms: run.total_metrics.total_time.as_millis() as u64,
    }
}

/// Score document generation: required docs exist, sections filled, no placeholders, hierarchy correct.
fn score_document_generation(run: &BenchmarkRun, expected: &ExpectedOutputs) -> TrackScore {
    let mut checks = Vec::new();

    // Check required document roles
    for req in &expected.required_docs {
        let matching_docs: Vec<_> = run
            .artifacts
            .documents
            .iter()
            .filter(|d| doc_matches_role(d, &req.role))
            .collect();

        let count = matching_docs.len() as u32;
        let min = req.min_count.unwrap_or(1);
        checks.push(ScoringCheck {
            name: format!("{}_count", req.role),
            passed: count >= min,
            weight: 2.0,
            detail: format!("Found {} '{}' documents (need >= {})", count, req.role, min),
        });

        // Check required sections in matching docs
        for section in &req.required_sections {
            let has_section = matching_docs
                .iter()
                .any(|d| d.excerpt.to_lowercase().contains(&section.to_lowercase()));
            checks.push(ScoringCheck {
                name: format!(
                    "{}_{}_section",
                    req.role,
                    section.to_lowercase().replace(' ', "_")
                ),
                passed: has_section,
                weight: 1.0,
                detail: format!(
                    "'{}' section in {} documents: {}",
                    section,
                    req.role,
                    if has_section { "found" } else { "missing" }
                ),
            });
        }
    }

    // Placeholder leakage check across all documents
    let placeholder_count: usize = run
        .artifacts
        .documents
        .iter()
        .map(|d| count_placeholders(&d.excerpt))
        .sum();
    checks.push(ScoringCheck {
        name: "no_placeholder_leakage".to_string(),
        passed: placeholder_count == 0,
        weight: 3.0,
        detail: format!(
            "{} placeholder patterns found in document excerpts",
            placeholder_count
        ),
    });

    build_track_score(checks)
}

/// Score decomposition: coverage, granularity, traceability, acceptance criteria.
fn score_decomposition(run: &BenchmarkRun, expected: &ExpectedOutputs) -> TrackScore {
    let mut checks = Vec::new();

    let total_tasks: usize = run.initiatives.iter().map(|i| i.tasks.len()).sum();
    let total_initiatives = run.initiatives.len();

    // Check hierarchy links
    for link in &expected.expected_hierarchy {
        let (parent_count, child_count) =
            match (link.parent_role.as_str(), link.child_role.as_str()) {
                ("vision", "initiative") => (1usize, total_initiatives),
                ("initiative", "task") => (total_initiatives, total_tasks),
                _ => continue,
            };
        let min = link.min_children.unwrap_or(1) as usize;
        let avg_children = if parent_count > 0 {
            child_count / parent_count
        } else {
            0
        };
        checks.push(ScoringCheck {
            name: format!("{}_per_{}", link.child_role, link.parent_role),
            passed: avg_children >= min,
            weight: 2.0,
            detail: format!(
                "Average {:.1} {} per {} (need >= {})",
                child_count as f32 / parent_count.max(1) as f32,
                link.child_role,
                link.parent_role,
                min
            ),
        });
    }

    // Task granularity: each task should have a non-empty, non-placeholder title
    let well_titled = run
        .initiatives
        .iter()
        .flat_map(|i| &i.tasks)
        .filter(|t| !t.task_title.is_empty() && !t.task_title.contains('{'))
        .count();
    checks.push(ScoringCheck {
        name: "task_title_quality".to_string(),
        passed: well_titled == total_tasks && total_tasks > 0,
        weight: 1.0,
        detail: format!(
            "{}/{} tasks have well-formed titles",
            well_titled, total_tasks
        ),
    });

    // Acceptance criteria: tasks should have doc accuracy above threshold
    let high_quality_tasks = run
        .initiatives
        .iter()
        .flat_map(|i| &i.tasks)
        .filter(|t| t.code_metrics.doc_accuracy_percent >= 60.0)
        .count();
    checks.push(ScoringCheck {
        name: "task_doc_quality".to_string(),
        passed: high_quality_tasks >= total_tasks.saturating_sub(1),
        weight: 2.0,
        detail: format!(
            "{}/{} tasks have doc accuracy >= 60%",
            high_quality_tasks, total_tasks
        ),
    });

    // Initiative coverage: at least min_count tasks from expected
    for req in &expected.required_docs {
        if req.role == "task" {
            let min = req.min_count.unwrap_or(1);
            checks.push(ScoringCheck {
                name: "total_task_count".to_string(),
                passed: total_tasks as u32 >= min,
                weight: 2.0,
                detail: format!("{} total tasks (need >= {})", total_tasks, min),
            });
        }
    }

    build_track_score(checks)
}

/// Score build outcome by running verification checks.
fn score_build_outcome(run: &BenchmarkRun, verification: &[VerificationCheck]) -> TrackScore {
    let mut checks = Vec::new();

    if verification.is_empty() {
        // No verification commands defined — score based on available metrics
        checks.push(ScoringCheck {
            name: "has_code_artifacts".to_string(),
            passed: !run.artifacts.code_files.is_empty(),
            weight: 2.0,
            detail: format!("{} code artifacts produced", run.artifacts.code_files.len()),
        });
        return build_track_score(checks);
    }

    for check in verification {
        let result = run_verification_command(&check.command);
        let exit_ok = match (result.as_ref().map(|r| r.0), check.expected_exit_code) {
            (Ok(actual), Some(expected)) => actual == expected,
            (Ok(actual), None) => actual == 0,
            (Err(_), _) => false,
        };
        let stdout_ok = check.expected_stdout_contains.iter().all(|pattern| {
            result
                .as_ref()
                .map(|r| r.1.contains(pattern))
                .unwrap_or(false)
        });

        checks.push(ScoringCheck {
            name: check.name.clone(),
            passed: exit_ok && stdout_ok,
            weight: 3.0,
            detail: format!(
                "Command '{}': exit={}, stdout_match={}",
                check.command,
                if exit_ok { "ok" } else { "fail" },
                if stdout_ok { "ok" } else { "fail" }
            ),
        });
    }

    build_track_score(checks)
}

/// Score architecture conformance based on boundary patterns.
fn score_architecture_conformance(run: &BenchmarkRun, expected: &ExpectedOutputs) -> f32 {
    if expected.architecture_constraints.is_empty() {
        return 100.0; // No constraints to violate
    }

    let mut passed = 0usize;
    let total = expected.architecture_constraints.len();

    for constraint in &expected.architecture_constraints {
        let all_patterns_ok = constraint.boundary_patterns.iter().all(|bp| {
            let found = run.artifacts.code_files.iter().any(|f| {
                // Check if the pattern exists in the code excerpt
                let matches_glob = bp
                    .file_glob
                    .as_ref()
                    .map(|g| glob_matches(&f.path, g))
                    .unwrap_or(true);
                matches_glob && f.excerpt.contains(&bp.pattern)
            });
            match bp.kind.as_str() {
                "must_exist" => found,
                "must_not_exist" => !found,
                _ => true,
            }
        });
        if all_patterns_ok {
            passed += 1;
        }
    }

    (passed as f32 / total as f32) * 100.0
}

/// Score static tool utilization: did the run use verification tools (test, lint, typecheck)?
fn score_static_tool_utilization(run: &BenchmarkRun) -> f32 {
    let tool_keywords = [
        "test",
        "lint",
        "clippy",
        "typecheck",
        "check",
        "verify",
        "fmt",
    ];
    let total_cli_events = run.trace.cli_events.len();
    if total_cli_events == 0 {
        return 0.0;
    }

    let tool_events = run
        .trace
        .cli_events
        .iter()
        .filter(|e| {
            let cmd_lower = e.command.to_lowercase();
            tool_keywords.iter().any(|kw| cmd_lower.contains(kw))
        })
        .count();

    // Score: percentage of CLI events that are verification-related, capped at 100
    ((tool_events as f32 / total_cli_events as f32) * 100.0).min(100.0)
}

fn count_placeholders(text: &str) -> usize {
    let mut count = 0usize;
    let mut depth = 0u32;
    for ch in text.chars() {
        match ch {
            '{' => depth += 1,
            '}' if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    count += 1;
                }
            }
            _ => {}
        }
    }
    count
}

fn doc_matches_role(doc: &DocumentArtifact, role: &str) -> bool {
    let path_lower = doc.path.to_lowercase();
    let title_lower = doc.title.to_lowercase();
    let role_lower = role.to_lowercase();

    path_lower.contains(&role_lower)
        || title_lower.contains(&role_lower)
        || doc
            .short_code
            .as_ref()
            .map(|sc| {
                let sc_upper = sc.to_uppercase();
                match role_lower.as_str() {
                    "vision" => sc_upper.contains("-V-"),
                    "initiative" => sc_upper.contains("-I-"),
                    "task" => sc_upper.contains("-T-"),
                    _ => false,
                }
            })
            .unwrap_or(false)
}

fn glob_matches(path: &str, pattern: &str) -> bool {
    // Simple glob matching: just check if the pattern's non-glob parts match
    let parts: Vec<&str> = pattern.split('*').collect();
    parts.iter().all(|part| {
        if part.is_empty() {
            true
        } else {
            path.contains(part)
        }
    })
}

fn run_verification_command(command: &str) -> Result<(i32, String), String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|e| format!("Failed to execute '{}': {}", command, e))?;

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok((exit_code, stdout))
}

fn build_track_score(checks: Vec<ScoringCheck>) -> TrackScore {
    let max_score: f32 = checks.iter().map(|c| c.weight).sum();
    let score: f32 = checks.iter().filter(|c| c.passed).map(|c| c.weight).sum();

    TrackScore {
        score,
        max_score,
        checks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario_pack::*;
    use crate::types::{
        CliTraceEvent, CodeArtifact, CodeMetrics, ExecutionMode, InitiativeResult, RunArtifacts,
        RunManifest, RunMetrics, RunTrace, ScenarioSummary, TaskResult, TaskStatus,
    };
    use std::time::Duration;

    fn make_test_run() -> BenchmarkRun {
        BenchmarkRun {
            run_id: "test".to_string(),
            timestamp: chrono::Utc::now(),
            manifest: RunManifest::default(),
            scenario: ScenarioSummary {
                id: "test".to_string(),
                title: "Test".to_string(),
                root: ".".to_string(),
            },
            execution_mode: ExecutionMode::Autonomous,
            phases: vec![],
            trace: RunTrace {
                prompt_events: vec![],
                cli_events: vec![
                    CliTraceEvent {
                        label: "test".to_string(),
                        command: "cargo test".to_string(),
                        exit_code: 0,
                        duration: Duration::from_secs(5),
                        approx_tokens: 100,
                        stdout_excerpt: "test result: ok".to_string(),
                        stderr_excerpt: String::new(),
                    },
                    CliTraceEvent {
                        label: "create".to_string(),
                        command: "cadre create".to_string(),
                        exit_code: 0,
                        duration: Duration::from_secs(1),
                        approx_tokens: 50,
                        stdout_excerpt: String::new(),
                        stderr_excerpt: String::new(),
                    },
                ],
            },
            artifacts: RunArtifacts {
                documents: vec![
                    DocumentArtifact {
                        path: "vision/BENCH-V-0001.md".to_string(),
                        title: "Test Vision".to_string(),
                        short_code: Some("BENCH-V-0001".to_string()),
                        excerpt: "## Context\nBuilding a file processor".to_string(),
                    },
                    DocumentArtifact {
                        path: "initiatives/BENCH-I-0001.md".to_string(),
                        title: "Parse Initiative".to_string(),
                        short_code: Some("BENCH-I-0001".to_string()),
                        excerpt: "## Context\nParsing module\n## Goals\nParse CSV".to_string(),
                    },
                    DocumentArtifact {
                        path: "initiatives/BENCH-I-0002.md".to_string(),
                        title: "Transform Initiative".to_string(),
                        short_code: Some("BENCH-I-0002".to_string()),
                        excerpt: "## Context\nTransform module\n## Goals\nTransform data"
                            .to_string(),
                    },
                ],
                code_files: vec![CodeArtifact {
                    path: "src/lib.rs".to_string(),
                    language: "Rust".to_string(),
                    line_count: 100,
                    excerpt: "mod parse;\nmod transform;\nstruct Record {}".to_string(),
                }],
            },
            initiatives: vec![InitiativeResult {
                initiative_id: "parse".to_string(),
                initiative_title: "Parse Module".to_string(),
                tasks: vec![
                    TaskResult {
                        task_id: "t1".to_string(),
                        task_title: "Implement CSV parser".to_string(),
                        status: TaskStatus::Completed,
                        tokens_used: 1000,
                        time_elapsed: Duration::from_secs(30),
                        code_metrics: CodeMetrics {
                            lines_of_code: 50,
                            test_coverage_percent: 80.0,
                            cyclomatic_complexity: 3.0,
                            doc_accuracy_percent: 85.0,
                            instruction_adherence_percent: 90.0,
                        },
                        validation_gate: None,
                    },
                    TaskResult {
                        task_id: "t2".to_string(),
                        task_title: "Add CSV tests".to_string(),
                        status: TaskStatus::Completed,
                        tokens_used: 800,
                        time_elapsed: Duration::from_secs(20),
                        code_metrics: CodeMetrics {
                            lines_of_code: 30,
                            test_coverage_percent: 90.0,
                            cyclomatic_complexity: 2.0,
                            doc_accuracy_percent: 75.0,
                            instruction_adherence_percent: 85.0,
                        },
                        validation_gate: None,
                    },
                ],
                total_tokens: 1800,
                total_time: Duration::from_secs(50),
            }],
            total_metrics: RunMetrics {
                total_tokens: 1800,
                total_time: Duration::from_secs(50),
                avg_code_quality: 80.0,
                avg_test_coverage: 85.0,
                avg_doc_accuracy: 80.0,
                avg_instruction_adherence: 87.5,
                gate_effectiveness: None,
            },
        }
    }

    fn make_expected_outputs() -> ExpectedOutputs {
        ExpectedOutputs {
            required_docs: vec![
                RequiredDoc {
                    role: "initiative".to_string(),
                    min_count: Some(2),
                    required_sections: vec!["Context".to_string(), "Goals".to_string()],
                },
                RequiredDoc {
                    role: "task".to_string(),
                    min_count: Some(2),
                    required_sections: vec![],
                },
            ],
            expected_hierarchy: vec![HierarchyLink {
                parent_role: "initiative".to_string(),
                child_role: "task".to_string(),
                min_children: Some(2),
            }],
            architecture_constraints: vec![],
        }
    }

    #[test]
    fn test_document_generation_scoring() {
        let run = make_test_run();
        let expected = make_expected_outputs();
        let score = score_document_generation(&run, &expected);

        assert!(score.percent() > 50.0, "Well-formed run should score > 50%");
        assert!(
            score
                .checks
                .iter()
                .any(|c| c.name == "no_placeholder_leakage" && c.passed),
            "No placeholders in test run"
        );
    }

    #[test]
    fn test_decomposition_scoring() {
        let run = make_test_run();
        let expected = make_expected_outputs();
        let score = score_decomposition(&run, &expected);

        assert!(
            score.percent() > 50.0,
            "Well-decomposed run should score > 50%"
        );
    }

    #[test]
    fn test_static_tool_utilization() {
        let run = make_test_run();
        let util = score_static_tool_utilization(&run);
        assert!(
            util > 0.0,
            "Run with 'cargo test' event should have > 0% utilization"
        );
    }

    #[test]
    fn test_architecture_conformance_no_constraints() {
        let run = make_test_run();
        let expected = ExpectedOutputs::default();
        let score = score_architecture_conformance(&run, &expected);
        assert_eq!(score, 100.0, "No constraints = perfect score");
    }

    #[test]
    fn test_doc_matches_role() {
        let doc = DocumentArtifact {
            path: "initiatives/BENCH-I-0001.md".to_string(),
            title: "Parse Initiative".to_string(),
            short_code: Some("BENCH-I-0001".to_string()),
            excerpt: String::new(),
        };
        assert!(doc_matches_role(&doc, "initiative"));
        assert!(!doc_matches_role(&doc, "vision"));
    }

    #[test]
    fn test_glob_matches() {
        assert!(glob_matches("src/lib.rs", "src/**/*.rs"));
        assert!(glob_matches("src/parse/mod.rs", "src/*/mod.rs"));
        assert!(!glob_matches("tests/test.rs", "src/**/*.rs"));
    }

    #[test]
    fn test_build_track_score() {
        let checks = vec![
            ScoringCheck {
                name: "a".to_string(),
                passed: true,
                weight: 2.0,
                detail: String::new(),
            },
            ScoringCheck {
                name: "b".to_string(),
                passed: false,
                weight: 3.0,
                detail: String::new(),
            },
        ];
        let score = build_track_score(checks);
        assert_eq!(score.score, 2.0);
        assert_eq!(score.max_score, 5.0);
        assert!((score.percent() - 40.0).abs() < 0.01);
    }

    #[test]
    fn test_count_placeholders() {
        assert_eq!(count_placeholders("{one} and {two}"), 2);
        assert_eq!(count_placeholders("no placeholders"), 0);
        assert_eq!(count_placeholders(""), 0);
    }
}

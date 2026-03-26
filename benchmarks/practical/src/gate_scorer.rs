use crate::types::{GateDecision, InitiativeResult, ValidationGateResult};
use std::path::Path;

const REWORK_TOKENS_PER_ISSUE: u64 = 500;

/// Deterministic quality gate scorer.
///
/// Evaluates an `InitiativeResult` against structural quality checks without
/// making API calls. Checks: doc accuracy (no placeholders, required fields),
/// instruction adherence (task count, adherence metrics), and artifact sanity.
pub struct GateScorer {
    /// Expected task count range per initiative (inclusive).
    pub expected_task_range: (usize, usize),
    /// Minimum doc accuracy % to pass without rework.
    pub doc_accuracy_threshold: f32,
    /// Minimum instruction adherence % to pass without rework.
    pub adherence_threshold: f32,
}

impl Default for GateScorer {
    fn default() -> Self {
        Self {
            expected_task_range: (1, 6),
            doc_accuracy_threshold: 60.0,
            adherence_threshold: 70.0,
        }
    }
}

impl GateScorer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Score an initiative result and return a `ValidationGateResult`.
    ///
    /// `artifact_dir` is optional — if provided, checks that the directory
    /// exists (confirming CLI operations ran successfully).
    pub fn score_initiative(
        &self,
        initiative: &InitiativeResult,
        artifact_dir: Option<&Path>,
    ) -> ValidationGateResult {
        let mut issues: Vec<String> = vec![];
        let mut blocking_failures: usize = 0;

        self.check_token_sanity(initiative, &mut issues, &mut blocking_failures);
        self.check_task_count(initiative, &mut issues, &mut blocking_failures);
        self.check_per_task_quality(initiative, &mut issues, &mut blocking_failures);
        check_initiative_title(initiative, &mut issues, &mut blocking_failures);
        check_artifact_dir(artifact_dir, &mut issues, &mut blocking_failures);

        let rework_tokens = issues.len() as u64 * REWORK_TOKENS_PER_ISSUE;
        let gate_decision = if blocking_failures > 0 {
            GateDecision::Rejected
        } else if !issues.is_empty() {
            GateDecision::RequiresRework
        } else {
            GateDecision::Approved
        };

        ValidationGateResult {
            gate_decision,
            issues_found: issues,
            rework_tokens,
            rework_time: std::time::Duration::from_millis(0),
        }
    }

    fn check_token_sanity(
        &self,
        initiative: &InitiativeResult,
        issues: &mut Vec<String>,
        blocking_failures: &mut usize,
    ) {
        if initiative.total_tokens == 0 {
            issues.push("No tokens recorded — initiative may not have executed".to_string());
            *blocking_failures += 1;
        }
    }

    fn check_task_count(
        &self,
        initiative: &InitiativeResult,
        issues: &mut Vec<String>,
        blocking_failures: &mut usize,
    ) {
        let (min_tasks, max_tasks) = self.expected_task_range;
        if initiative.tasks.is_empty() {
            issues.push("Initiative has no tasks".to_string());
            *blocking_failures += 1;
        } else {
            if initiative.tasks.len() < min_tasks {
                issues.push(format!(
                    "Too few tasks ({}) — expected at least {}",
                    initiative.tasks.len(),
                    min_tasks
                ));
            }
            if initiative.tasks.len() > max_tasks {
                issues.push(format!(
                    "Too many tasks ({}) — expected at most {}",
                    initiative.tasks.len(),
                    max_tasks
                ));
            }
        }
    }

    fn check_per_task_quality(
        &self,
        initiative: &InitiativeResult,
        issues: &mut Vec<String>,
        blocking_failures: &mut usize,
    ) {
        for task in &initiative.tasks {
            if task.task_title.trim().is_empty() {
                issues.push(format!("Task '{}' has empty title", task.task_id));
                *blocking_failures += 1;
            } else if task.task_title.contains('{') || task.task_title.contains('}') {
                issues.push(format!(
                    "Task '{}' contains unfilled placeholder text",
                    task.task_id
                ));
            }

            if task.code_metrics.doc_accuracy_percent < self.doc_accuracy_threshold {
                issues.push(format!(
                    "Task '{}' has low doc accuracy ({:.0}% < {:.0}% threshold)",
                    task.task_title,
                    task.code_metrics.doc_accuracy_percent,
                    self.doc_accuracy_threshold,
                ));
            }

            if task.code_metrics.instruction_adherence_percent < self.adherence_threshold {
                issues.push(format!(
                    "Task '{}' has low instruction adherence ({:.0}% < {:.0}% threshold) — AI may not have followed prompt format",
                    task.task_title,
                    task.code_metrics.instruction_adherence_percent,
                    self.adherence_threshold,
                ));
            }
        }
    }
}

fn check_initiative_title(
    initiative: &InitiativeResult,
    issues: &mut Vec<String>,
    blocking_failures: &mut usize,
) {
    if initiative.initiative_title.trim().is_empty() {
        issues.push("Initiative has empty title".to_string());
        *blocking_failures += 1;
    } else if initiative.initiative_title.contains('{')
        || initiative.initiative_title.contains('}')
    {
        issues.push(format!(
            "Initiative title '{}' contains unfilled placeholders",
            initiative.initiative_title
        ));
    }
}

fn check_artifact_dir(
    artifact_dir: Option<&Path>,
    issues: &mut Vec<String>,
    blocking_failures: &mut usize,
) {
    if let Some(dir) = artifact_dir {
        if !dir.exists() {
            issues.push(format!(
                "Artifact directory '{}' does not exist — CLI operations may have failed",
                dir.display()
            ));
            *blocking_failures += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CodeMetrics, TaskResult, TaskStatus};
    use std::time::Duration;

    fn make_good_task(id: &str) -> TaskResult {
        TaskResult {
            task_id: id.to_string(),
            task_title: format!("Design and implement {id}"),
            status: TaskStatus::Completed,
            tokens_used: 1500,
            time_elapsed: Duration::from_secs(30),
            code_metrics: CodeMetrics {
                lines_of_code: 50,
                test_coverage_percent: 80.0,
                cyclomatic_complexity: 3.0,
                doc_accuracy_percent: 90.0,
                instruction_adherence_percent: 95.0,
            },
            validation_gate: None,
        }
    }

    fn make_initiative(id: &str, tasks: Vec<TaskResult>, tokens: u64) -> InitiativeResult {
        let total_time = tasks.iter().map(|t| t.time_elapsed).sum();
        InitiativeResult {
            initiative_id: id.to_string(),
            initiative_title: format!("{id} Module"),
            tasks,
            total_tokens: tokens,
            total_time,
        }
    }

    #[test]
    fn test_all_pass_returns_approved() {
        let scorer = GateScorer::new();
        let initiative = make_initiative(
            "output",
            vec![make_good_task("t1"), make_good_task("t2")],
            2000,
        );
        let result = scorer.score_initiative(&initiative, None);
        assert!(matches!(result.gate_decision, GateDecision::Approved));
        assert!(result.issues_found.is_empty());
        assert_eq!(result.rework_tokens, 0);
    }

    #[test]
    fn test_zero_tokens_is_blocking_rejection() {
        let scorer = GateScorer::new();
        let initiative = make_initiative("output", vec![make_good_task("t1")], 0);
        let result = scorer.score_initiative(&initiative, None);
        assert!(matches!(result.gate_decision, GateDecision::Rejected));
        assert!(!result.issues_found.is_empty());
    }

    #[test]
    fn test_no_tasks_is_blocking_rejection() {
        let scorer = GateScorer::new();
        let initiative = make_initiative("output", vec![], 1000);
        let result = scorer.score_initiative(&initiative, None);
        assert!(matches!(result.gate_decision, GateDecision::Rejected));
    }

    #[test]
    fn test_low_doc_accuracy_requires_rework() {
        let scorer = GateScorer::new();
        let mut task = make_good_task("t1");
        task.code_metrics.doc_accuracy_percent = 40.0;
        let initiative = make_initiative("output", vec![task], 1500);
        let result = scorer.score_initiative(&initiative, None);
        assert!(matches!(result.gate_decision, GateDecision::RequiresRework));
        assert!(result
            .issues_found
            .iter()
            .any(|i| i.contains("doc accuracy")));
    }

    #[test]
    fn test_low_instruction_adherence_requires_rework() {
        let scorer = GateScorer::new();
        let mut task = make_good_task("t1");
        task.code_metrics.instruction_adherence_percent = 50.0;
        let initiative = make_initiative("output", vec![task], 1500);
        let result = scorer.score_initiative(&initiative, None);
        assert!(matches!(result.gate_decision, GateDecision::RequiresRework));
        assert!(result.issues_found.iter().any(|i| i.contains("adherence")));
    }

    #[test]
    fn test_placeholder_in_task_title_requires_rework() {
        let scorer = GateScorer::new();
        let mut task = make_good_task("t1");
        task.task_title = "Implement {module_name}".to_string();
        let initiative = make_initiative("output", vec![task], 1500);
        let result = scorer.score_initiative(&initiative, None);
        assert!(matches!(result.gate_decision, GateDecision::RequiresRework));
        assert!(result
            .issues_found
            .iter()
            .any(|i| i.contains("placeholder")));
    }

    #[test]
    fn test_rework_tokens_estimated_per_issue() {
        let scorer = GateScorer::new();
        let mut task = make_good_task("t1");
        task.code_metrics.doc_accuracy_percent = 40.0;
        task.code_metrics.instruction_adherence_percent = 50.0;
        let initiative = make_initiative("output", vec![task], 1500);
        let result = scorer.score_initiative(&initiative, None);
        assert_eq!(result.rework_tokens, result.issues_found.len() as u64 * 500);
    }

    #[test]
    fn test_missing_artifact_dir_is_blocking() {
        let scorer = GateScorer::new();
        let initiative = make_initiative("output", vec![make_good_task("t1")], 1500);
        let nonexistent = std::path::Path::new("/nonexistent/path/xyz");
        let result = scorer.score_initiative(&initiative, Some(nonexistent));
        assert!(matches!(result.gate_decision, GateDecision::Rejected));
        assert!(result
            .issues_found
            .iter()
            .any(|i| i.contains("Artifact directory")));
    }
}

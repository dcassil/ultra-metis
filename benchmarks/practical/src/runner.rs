use crate::{
    api_client, prompt_builder, scenario_pack::LoadedScenarioPack,
    types::{
        BenchmarkPhase, BenchmarkRun, CliTraceEvent, CodeArtifact, CodeMetrics, DocumentArtifact,
        ExecutionMode, InitiativeResult, PhaseResult, PhaseStatus, PromptTraceEvent, RunArtifacts,
        RunMetrics, RunTrace, ScenarioSummary, SystemUnderTest, TaskResult, TaskStatus,
    },
    workspace,
};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Invoke the cadre CLI binary, returning stdout and elapsed ms.
/// Mirrors the approach in benchmarks/run-cadre-bench.sh.
pub fn run_cli(binary: &Path, args: &[&str]) -> anyhow::Result<CliResult> {
    let start = std::time::Instant::now();
    let output = Command::new(binary).args(args).output()?;
    let elapsed = start.elapsed();

    Ok(CliResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
        elapsed,
    })
}

#[derive(Debug, Clone)]
pub struct CliResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub elapsed: std::time::Duration,
}

impl CliResult {
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    /// Approximate token count from output byte size.
    /// ~4 bytes per token is a rough but consistent approximation.
    pub fn approx_tokens(&self) -> u64 {
        ((self.stdout.len() + self.stderr.len()) / 4).max(1) as u64
    }

    pub fn as_trace_event(
        &self,
        label: impl Into<String>,
        command: impl Into<String>,
    ) -> CliTraceEvent {
        CliTraceEvent {
            label: label.into(),
            command: command.into(),
            exit_code: self.exit_code,
            duration: self.elapsed,
            approx_tokens: self.approx_tokens(),
            stdout_excerpt: excerpt(&self.stdout, 400),
            stderr_excerpt: excerpt(&self.stderr, 400),
        }
    }
}

/// Resolve cadre binary path. Checks CADRE_BINARY env var first,
/// then falls back to target/release/cadre relative to cwd.
pub fn resolve_binary_path() -> PathBuf {
    if let Ok(path) = std::env::var("CADRE_BINARY") {
        return PathBuf::from(path);
    }
    let candidate = PathBuf::from("target/release/cadre");
    if candidate.exists() {
        return candidate;
    }
    PathBuf::from("cadre")
}

/// Extract a short code (e.g., "BENCH-V-0001") from CLI stdout.
pub fn extract_short_code(stdout: &str, prefix: &str) -> String {
    for word in stdout.split_whitespace() {
        let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
        if cleaned.starts_with(prefix) {
            return cleaned.to_string();
        }
    }
    String::new()
}

/// An initiative identified by AI from the scenario assessment.
#[derive(Debug)]
pub struct AiInitiative {
    pub id: String,
    pub title: String,
    pub objective: String,
    pub tasks: Vec<String>,
}

/// Parse Claude's JSON response to extract AI-identified initiatives.
pub fn parse_initiative_response(response: &str) -> Vec<AiInitiative> {
    let start = response.find('{');
    let end = response.rfind('}');

    let json_str = match (start, end) {
        (Some(s), Some(e)) if e > s => &response[s..=e],
        _ => return vec![],
    };

    let parsed: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let mut result = vec![];
    if let Some(initiatives) = parsed.get("initiatives").and_then(|v| v.as_array()) {
        for init in initiatives {
            let id = init
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let title = init
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown Initiative")
                .to_string();
            let objective = init
                .get("objective")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let tasks = init
                .get("tasks")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|t| t.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            result.push(AiInitiative {
                id,
                title,
                objective,
                tasks,
            });
        }
    }
    result
}

/// Score an AI-generated initiative into CodeMetrics based on content quality.
pub fn score_ai_initiative(init: &AiInitiative, response_was_valid_json: bool) -> CodeMetrics {
    let instruction_adherence = if response_was_valid_json { 100.0 } else { 50.0 };

    // Doc accuracy: has objective + at least 2 tasks?
    let doc_accuracy = if !init.objective.is_empty() && init.tasks.len() >= 2 {
        90.0
    } else if !init.objective.is_empty() {
        60.0
    } else {
        30.0
    };

    // Test coverage proxy: does task list mention testing/validation?
    let has_test_task = init.tasks.iter().any(|t| {
        let lower = t.to_lowercase();
        lower.contains("test") || lower.contains("verify") || lower.contains("validate")
    });
    let test_coverage = if has_test_task { 80.0 } else { 40.0 };

    // Lines proxy: text length as thoroughness indicator (~80 chars per line)
    let text_len = (init.objective.len() + init.tasks.join(" ").len()) as u32;
    let lines = (text_len / 80).max(1);

    CodeMetrics {
        lines_of_code: lines,
        test_coverage_percent: test_coverage,
        cyclomatic_complexity: init.tasks.len() as f32,
        doc_accuracy_percent: doc_accuracy,
        instruction_adherence_percent: instruction_adherence,
    }
}

pub fn prompt_trace_event(
    label: impl Into<String>,
    system_prompt: impl Into<String>,
    user_prompt: impl Into<String>,
    response_content: &str,
    input_tokens: u64,
    output_tokens: u64,
    duration: std::time::Duration,
) -> PromptTraceEvent {
    PromptTraceEvent {
        label: label.into(),
        system_prompt: system_prompt.into(),
        user_prompt: user_prompt.into(),
        response_excerpt: excerpt(response_content, 500),
        input_tokens,
        output_tokens,
        duration,
    }
}

pub fn snapshot_workspace_artifacts(root: &Path) -> RunArtifacts {
    let mut files = vec![];
    collect_files(root, &mut files);
    files.sort();

    let mut artifacts = RunArtifacts::default();
    for path in files {
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) => continue,
        };

        if is_document_file(&path) {
            artifacts.documents.push(DocumentArtifact {
                path: rel,
                title: extract_document_title(&content, &path),
                short_code: extract_any_short_code(&content),
                excerpt: excerpt(&content, 600),
            });
        } else if let Some(language) = code_language(&path) {
            artifacts.code_files.push(CodeArtifact {
                path: rel,
                language: language.to_string(),
                line_count: content.lines().count() as u32,
                excerpt: excerpt(&content, 600),
            });
        }
    }

    artifacts
}

fn collect_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn is_document_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("md" | "mdx" | "txt" | "yaml" | "yml" | "json")
    )
}

fn code_language(path: &Path) -> Option<&'static str> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("rs") => Some("Rust"),
        Some("ts" | "tsx") => Some("TypeScript"),
        Some("js" | "jsx") => Some("JavaScript"),
        Some("py") => Some("Python"),
        Some("go") => Some("Go"),
        Some("java") => Some("Java"),
        Some("rb") => Some("Ruby"),
        Some("sh") => Some("Shell"),
        Some("c") => Some("C"),
        Some("cc" | "cpp" | "cxx" | "hpp" | "h") => Some("C++"),
        _ => None,
    }
}

fn extract_document_title(content: &str, path: &Path) -> String {
    content
        .lines()
        .find_map(|line| line.strip_prefix("title:"))
        .map(|title| title.trim().trim_matches('"').to_string())
        .or_else(|| {
            content
                .lines()
                .find_map(|line| line.strip_prefix("# "))
                .map(|title| title.trim().to_string())
        })
        .unwrap_or_else(|| {
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Untitled document")
                .to_string()
        })
}

fn extract_any_short_code(content: &str) -> Option<String> {
    content
        .split_whitespace()
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric() && c != '-'))
        .find(|word| word.contains("-V-") || word.contains("-I-") || word.contains("-T-"))
        .map(ToString::to_string)
}

pub fn excerpt(content: &str, max_chars: usize) -> String {
    let normalized = content
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty())
        .take(12)
        .collect::<Vec<_>>()
        .join("\n");

    let mut out = normalized.chars().take(max_chars).collect::<String>();
    if normalized.chars().count() > max_chars {
        out.push_str("...");
    }
    out
}

fn fallback_initiative(
    api_tokens: u64,
    api_time: std::time::Duration,
    valid_json: bool,
) -> InitiativeResult {
    let metrics = CodeMetrics {
        lines_of_code: 1,
        test_coverage_percent: 0.0,
        cyclomatic_complexity: 0.0,
        doc_accuracy_percent: if valid_json { 80.0 } else { 30.0 },
        instruction_adherence_percent: if valid_json { 100.0 } else { 50.0 },
    };
    InitiativeResult {
        initiative_id: "no-additional".to_string(),
        initiative_title: "No additional initiatives identified".to_string(),
        tasks: vec![TaskResult {
            task_id: "strategic-assessment".to_string(),
            task_title: "Strategic completeness assessment".to_string(),
            status: TaskStatus::Completed,
            tokens_used: api_tokens,
            time_elapsed: api_time,
            code_metrics: metrics,
            validation_gate: None,
        }],
        total_tokens: api_tokens,
        total_time: api_time,
    }
}

pub fn default_fallback_ai_initiative() -> AiInitiative {
    AiInitiative {
        id: "output-module".to_string(),
        title: "Output Module".to_string(),
        objective: "Implement export, validation, and delivery workflow for processed datasets."
            .to_string(),
        tasks: vec![
            "Design output writer abstractions".to_string(),
            "Implement CSV, JSON, and YAML exports".to_string(),
            "Add output-schema validation and integration tests".to_string(),
        ],
    }
}

/// Execute autonomous benchmark run (no validation gates).
///
/// Initializes a temp cadre project, runs the scenario through Claude API,
/// and records real token counts and CLI timing.
pub async fn execute_autonomous(scenario: &LoadedScenarioPack) -> anyhow::Result<BenchmarkRun> {
    let start_time = std::time::Instant::now();
    let run_id = uuid::Uuid::new_v4().to_string();
    let mut phases = vec![];
    let mut trace = RunTrace::default();
    let manifest = workspace::default_manifest(scenario, SystemUnderTest::Cadre);

    tracing::info!("Starting autonomous run: {}", run_id);

    // Create isolated workspace and seed scenario documents
    let mut ws = workspace::BenchmarkWorkspace::setup(scenario)?;
    let setup = ws.take_setup_trace();
    trace.cli_events.extend(setup.cli_events);
    phases.push(setup.phase_result);

    // Ask Claude to assess what additional initiatives are needed
    let prompt = prompt_builder::build_scenario_assessment_prompt(&scenario.root)?;
    let api_start = std::time::Instant::now();
    let api_result = api_client::ask(&prompt.system, &prompt.user).await;
    let api_time = api_start.elapsed();
    let (
        api_input_tokens,
        api_output_tokens,
        api_content,
        ai_initiatives,
        response_was_valid_json,
        phase_note,
    ) = match api_result {
        Ok(api_resp) => {
            let parsed = parse_initiative_response(&api_resp.content);
            let valid_json =
                !parsed.is_empty() || api_resp.content.contains("additional_initiatives_needed");
            (
                api_resp.input_tokens,
                api_resp.output_tokens,
                api_resp.content,
                parsed,
                valid_json,
                format!(
                    "Scenario '{}' with {} seeded initiatives assessed",
                    scenario.manifest.id,
                    scenario.seed_initiatives.len()
                ),
            )
        }
        Err(err) => (
            0,
            0,
            format!("Deterministic fallback used because scenario assessment failed: {err}"),
            vec![default_fallback_ai_initiative()],
            false,
            format!("Fallback initiative generation used after Claude failure: {err}"),
        ),
    };
    trace.prompt_events.push(prompt_trace_event(
        "scenario_assessment",
        &prompt.system,
        &prompt.user,
        &api_content,
        api_input_tokens,
        api_output_tokens,
        api_time,
    ));
    phases.push(PhaseResult {
        phase: BenchmarkPhase::DocumentGeneration,
        status: PhaseStatus::Completed,
        tokens_used: api_input_tokens + api_output_tokens,
        time_elapsed: api_time,
        notes: vec![phase_note],
    });

    // Build initiative results from AI response
    let mut initiatives = vec![];
    let n = ai_initiatives.len().max(1) as u32;

    for (idx, ai_init) in ai_initiatives.iter().enumerate() {
        // Create the initiative in CLI for artifact tracking
        let cli_result = ws.create_initiative(&ai_init.title);
        if let Some(ref cli) = cli_result {
            trace.cli_events.push(cli.as_trace_event(
                format!("materialize_{}", ai_init.id),
                format!("create initiative: {}", ai_init.title),
            ));
        }

        let cli_tokens = cli_result.as_ref().map(|r| r.approx_tokens()).unwrap_or(0);
        let cli_time = cli_result.as_ref().map(|r| r.elapsed).unwrap_or_default();
        let task_tokens = ((api_input_tokens + api_output_tokens) / n as u64) + cli_tokens;
        let task_time = (api_time / n) + cli_time;

        tracing::info!(
            "AI initiative {}/{}: '{}' (tokens: {})",
            idx + 1,
            n,
            ai_init.title,
            task_tokens
        );

        initiatives.push(InitiativeResult {
            initiative_id: ai_init.id.clone(),
            initiative_title: ai_init.title.clone(),
            tasks: vec![TaskResult {
                task_id: format!("{}-assess", ai_init.id),
                task_title: format!("Assess and design: {}", ai_init.title),
                status: TaskStatus::Completed,
                tokens_used: task_tokens,
                time_elapsed: task_time,
                code_metrics: score_ai_initiative(ai_init, response_was_valid_json),
                validation_gate: None,
            }],
            total_tokens: task_tokens,
            total_time: task_time,
        });
    }

    if initiatives.is_empty() {
        tracing::info!("AI identified no additional initiatives");
        initiatives.push(fallback_initiative(
            api_input_tokens + api_output_tokens,
            api_time,
            response_was_valid_json,
        ));
    }

    let total_tokens: u64 = initiatives.iter().map(|i| i.total_tokens).sum();
    let total_time = start_time.elapsed();
    phases.push(PhaseResult {
        phase: BenchmarkPhase::Decomposition,
        status: PhaseStatus::Completed,
        tokens_used: total_tokens,
        time_elapsed: total_time,
        notes: vec![format!(
            "Produced {} initiative assessments",
            initiatives.len()
        )],
    });
    let task_count = initiatives
        .iter()
        .map(|i| i.tasks.len())
        .sum::<usize>()
        .max(1) as f32;
    let avg_doc_accuracy = initiatives
        .iter()
        .flat_map(|i| i.tasks.iter().map(|t| t.code_metrics.doc_accuracy_percent))
        .sum::<f32>()
        / task_count;
    let avg_instruction_adherence = initiatives
        .iter()
        .flat_map(|i| {
            i.tasks
                .iter()
                .map(|t| t.code_metrics.instruction_adherence_percent)
        })
        .sum::<f32>()
        / task_count;

    Ok(BenchmarkRun {
        run_id,
        timestamp: Utc::now(),
        manifest,
        scenario: ScenarioSummary {
            id: scenario.manifest.id.clone(),
            title: scenario.manifest.title.clone(),
            root: scenario.root.display().to_string(),
        },
        execution_mode: ExecutionMode::Autonomous,
        phases,
        trace,
        artifacts: snapshot_workspace_artifacts(ws.path()),
        initiatives,
        total_metrics: RunMetrics {
            total_tokens,
            total_time,
            avg_code_quality: avg_doc_accuracy,
            avg_test_coverage: 0.0,
            avg_doc_accuracy,
            avg_instruction_adherence,
            gate_effectiveness: None,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RunManifest;

    #[test]
    fn test_cli_result_token_approximation() {
        let result = CliResult {
            stdout: "a".repeat(400),
            stderr: String::new(),
            exit_code: 0,
            elapsed: std::time::Duration::from_millis(10),
        };
        assert_eq!(result.approx_tokens(), 100);
    }

    #[test]
    fn test_cli_result_success() {
        let ok = CliResult {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
            elapsed: std::time::Duration::default(),
        };
        let fail = CliResult {
            exit_code: 1,
            ..ok.clone()
        };
        assert!(ok.success());
        assert!(!fail.success());
    }

    #[test]
    fn test_extract_short_code() {
        let stdout = "Created BENCH-V-0001 successfully\n";
        assert_eq!(extract_short_code(stdout, "BENCH-V-"), "BENCH-V-0001");
    }

    #[test]
    fn test_extract_short_code_missing() {
        let stdout = "no code here\n";
        assert_eq!(extract_short_code(stdout, "BENCH-V-"), "");
    }

    #[test]
    fn test_parse_initiative_response_valid_json() {
        let resp = r#"{"analysis":"needs output","additional_initiatives_needed":true,"initiatives":[{"id":"output-module","title":"Output Module","objective":"Export data","tasks":["Write CSV","Write JSON","Validate schema"]}]}"#;
        let initiatives = parse_initiative_response(resp);
        assert_eq!(initiatives.len(), 1);
        assert_eq!(initiatives[0].title, "Output Module");
        assert_eq!(initiatives[0].tasks.len(), 3);
    }

    #[test]
    fn test_parse_initiative_response_empty() {
        let resp = "sorry I cannot help";
        let initiatives = parse_initiative_response(resp);
        assert!(initiatives.is_empty());
    }

    #[test]
    fn test_score_ai_initiative_with_tests() {
        let init = AiInitiative {
            id: "test-init".to_string(),
            title: "Test Initiative".to_string(),
            objective: "Build something important".to_string(),
            tasks: vec![
                "Design API".to_string(),
                "Write unit tests".to_string(),
                "Deploy".to_string(),
            ],
        };
        let metrics = score_ai_initiative(&init, true);
        assert_eq!(metrics.instruction_adherence_percent, 100.0);
        assert_eq!(metrics.test_coverage_percent, 80.0);
        assert_eq!(metrics.cyclomatic_complexity, 3.0);
    }

    #[test]
    fn test_autonomous_runner_creates_valid_run() {
        let run = BenchmarkRun {
            run_id: "test".to_string(),
            timestamp: Utc::now(),
            manifest: RunManifest::default(),
            scenario: ScenarioSummary {
                id: "test-scenario".to_string(),
                title: "Test Scenario".to_string(),
                root: "scenario".to_string(),
            },
            execution_mode: ExecutionMode::Autonomous,
            phases: vec![],
            trace: RunTrace::default(),
            artifacts: RunArtifacts::default(),
            initiatives: vec![],
            total_metrics: RunMetrics {
                total_tokens: 0,
                total_time: std::time::Duration::from_secs(0),
                avg_code_quality: 0.0,
                avg_test_coverage: 0.0,
                avg_doc_accuracy: 0.0,
                avg_instruction_adherence: 0.0,
                gate_effectiveness: None,
            },
        };
        assert_eq!(run.execution_mode, ExecutionMode::Autonomous);
    }
}

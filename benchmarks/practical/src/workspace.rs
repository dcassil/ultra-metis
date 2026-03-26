use crate::runner::{self, CliResult};
use crate::scenario_pack::LoadedScenarioPack;
use crate::types::*;
use std::path::Path;

/// An isolated workspace for a benchmark run with seeded scenario artifacts.
pub struct BenchmarkWorkspace {
    pub temp_dir: tempfile::TempDir,
    pub project_path: String,
    pub vision_code: String,
    pub setup_trace: SetupTrace,
}

/// Trace data from workspace setup, to be merged into the run trace.
pub struct SetupTrace {
    pub cli_events: Vec<CliTraceEvent>,
    pub phase_result: PhaseResult,
}

impl BenchmarkWorkspace {
    /// Take the setup trace, consuming it. Call once to merge into run trace.
    pub fn take_setup_trace(&mut self) -> SetupTrace {
        std::mem::replace(
            &mut self.setup_trace,
            SetupTrace {
                cli_events: Vec::new(),
                phase_result: PhaseResult {
                    phase: BenchmarkPhase::ScenarioSetup,
                    status: PhaseStatus::Completed,
                    tokens_used: 0,
                    time_elapsed: std::time::Duration::ZERO,
                    notes: Vec::new(),
                },
            },
        )
    }
}

impl BenchmarkWorkspace {
    /// Create an isolated workspace, initialize the project, and seed
    /// scenario documents (vision + seed initiatives).
    pub fn setup(scenario: &LoadedScenarioPack) -> anyhow::Result<Self> {
        let binary = runner::resolve_binary_path();
        let temp_dir = tempfile::TempDir::new()?;
        let proj_str = temp_dir.path().to_str().unwrap_or("/tmp/bench").to_string();
        let mut cli_events = Vec::new();

        // Initialize project
        if let Ok(result) =
            runner::run_cli(&binary, &["init", "--path", &proj_str, "--prefix", "BENCH"])
        {
            cli_events.push(result.as_trace_event(
                "workspace_init",
                format!(
                    "{} init --path {} --prefix BENCH",
                    binary.display(),
                    proj_str
                ),
            ));
        }

        // Create vision from scenario
        let vision_title = extract_title(&scenario.vision).unwrap_or("Benchmark Vision");
        let vision_result = runner::run_cli(
            &binary,
            &[
                "create",
                "--type",
                "vision",
                "--path",
                &proj_str,
                vision_title,
            ],
        );
        if let Ok(result) = &vision_result {
            cli_events.push(result.as_trace_event(
                "seed_vision",
                format!(
                    "{} create --type vision --path {} {}",
                    binary.display(),
                    proj_str,
                    vision_title
                ),
            ));
        }
        let vision_code = vision_result
            .as_ref()
            .map(|r| runner::extract_short_code(&r.stdout, "BENCH-V-"))
            .unwrap_or_default();

        // Seed initiatives from scenario pack
        if !vision_code.is_empty() {
            for (i, seed) in scenario.seed_initiatives.iter().enumerate() {
                let title = extract_title(&seed.content).unwrap_or("Seed Initiative");
                if let Ok(result) = runner::run_cli(
                    &binary,
                    &[
                        "create",
                        "--type",
                        "initiative",
                        "--path",
                        &proj_str,
                        "--parent",
                        &vision_code,
                        title,
                    ],
                ) {
                    cli_events.push(result.as_trace_event(
                        format!("seed_initiative_{}", i),
                        format!(
                            "{} create --type initiative --path {} --parent {} {}",
                            binary.display(),
                            proj_str,
                            vision_code,
                            title
                        ),
                    ));
                }
            }
        }

        let phase_result = PhaseResult {
            phase: BenchmarkPhase::ScenarioSetup,
            status: PhaseStatus::Completed,
            tokens_used: cli_events.iter().map(|e| e.approx_tokens).sum(),
            time_elapsed: cli_events.iter().map(|e| e.duration).sum(),
            notes: vec![format!(
                "Scenario '{}' materialized in {}",
                scenario.manifest.id,
                temp_dir.path().display()
            )],
        };

        Ok(Self {
            temp_dir,
            project_path: proj_str,
            vision_code,
            setup_trace: SetupTrace {
                cli_events,
                phase_result,
            },
        })
    }

    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Create an initiative in this workspace under the seeded vision.
    pub fn create_initiative(&self, title: &str) -> Option<CliResult> {
        if self.vision_code.is_empty() {
            return None;
        }
        let binary = runner::resolve_binary_path();
        runner::run_cli(
            &binary,
            &[
                "create",
                "--type",
                "initiative",
                "--path",
                &self.project_path,
                "--parent",
                &self.vision_code,
                title,
            ],
        )
        .ok()
    }
}

/// Extract a title from markdown content (first `# ` heading or first non-empty line).
fn extract_title(content: &str) -> Option<&str> {
    content
        .lines()
        .find_map(|line| line.strip_prefix("# "))
        .or_else(|| content.lines().find(|line| !line.trim().is_empty()))
        .map(|s| s.trim())
}

/// Build a default RunManifest for an autonomous or validated run.
pub fn default_manifest(scenario: &LoadedScenarioPack, system: SystemUnderTest) -> RunManifest {
    let git_commit = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        });

    RunManifest {
        system_under_test: system,
        tool_surface: ToolSurface::Cli,
        model_id: "claude-haiku-4-5-20251001".to_string(),
        token_budget: scenario.manifest.run_rules.token_budget,
        time_budget_secs: scenario.manifest.run_rules.time_budget_secs,
        git_commit,
        environment: RunEnvironment::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_title_from_heading() {
        let content = "# My Title\n\nSome content";
        assert_eq!(extract_title(content), Some("My Title"));
    }

    #[test]
    fn extract_title_from_first_line() {
        let content = "My Title\nSome content";
        assert_eq!(extract_title(content), Some("My Title"));
    }

    #[test]
    fn extract_title_empty() {
        assert_eq!(extract_title(""), None);
    }

    #[test]
    fn default_manifest_uses_scenario_budgets() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(
            temp.path().join("scenario.json"),
            r#"{"id":"test","title":"Test","documents":{"vision":"v.md"},"run_rules":{"token_budget":100000,"time_budget_secs":120}}"#,
        ).unwrap();
        std::fs::write(temp.path().join("v.md"), "# Vision").unwrap();
        let pack = LoadedScenarioPack::load(temp.path()).unwrap();

        let manifest = default_manifest(&pack, SystemUnderTest::Cadre);
        assert_eq!(manifest.token_budget, 100_000);
        assert_eq!(manifest.time_budget_secs, 120);
        assert_eq!(manifest.system_under_test, SystemUnderTest::Cadre);
    }
}

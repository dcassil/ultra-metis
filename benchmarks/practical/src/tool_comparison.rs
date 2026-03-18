/// Tool comparison benchmark: ultra-metis vs original metis template quality.
///
/// Measures how well AI fills in initiative documents created by each tool's
/// template. Better templates guide AI toward more complete content.
use crate::doc_quality::{score_content, DocQualityScore};
use crate::runner::{resolve_binary_path, run_cli, extract_short_code};
use anyhow::Context;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Result of running the scenario with one tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRunResult {
    pub tool_name: String,
    pub templates_tested: u32,
    pub avg_completeness_percent: f32,
    pub avg_placeholder_count: f32,
    pub total_filled_sections: u32,
    pub total_empty_sections: u32,
    pub tokens_used: u64,
    pub time_elapsed: std::time::Duration,
}

/// Comparison of ultra-metis vs original metis template quality impact on AI output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolComparisonResult {
    pub timestamp: chrono::DateTime<Utc>,
    pub ultra_metis: ToolRunResult,
    pub original_metis: ToolRunResult,
    /// ultra_metis.avg_completeness - original_metis.avg_completeness
    /// Positive = ultra-metis templates produce more complete documents
    pub completeness_delta: f32,
    /// original_metis.avg_placeholder_count - ultra_metis.avg_placeholder_count
    /// Positive = original-metis templates leave more unfilled placeholders
    pub placeholder_delta: f32,
}

/// Three module specs used to fill initiative templates.
const MODULES: &[(&str, &str)] = &[
    (
        "CSV Parser Module",
        "Parse CSV files with automatic delimiter detection (comma, tab, semicolon), \
         header inference, type coercion (string/int/float/date), and error recovery \
         for malformed rows. Expose a streaming iterator API for large files.",
    ),
    (
        "JSON Transformer",
        "Transform and reshape JSON documents using a declarative mapping language. \
         Support field renaming, nesting/flattening, value transformation functions, \
         and schema validation. Must handle deeply nested structures and arrays.",
    ),
    (
        "Output Formatter",
        "Format processed pipeline data to multiple output targets: CSV, JSON, \
         Parquet, and pretty console output. Support streaming writes, configurable \
         column ordering, and pluggable compression (gzip, zstd).",
    ),
];

/// Get the blank initiative template from ultra-metis by creating a real project.
fn get_ultra_metis_template() -> anyhow::Result<String> {
    let binary = resolve_binary_path();
    let temp_dir = tempfile::tempdir()?;
    let proj = temp_dir.path().to_str().unwrap_or("/tmp/tcomp-ultra");

    // Init project
    run_cli(&binary, &["init", "--path", proj, "--prefix", "TC"])?;

    // Create + publish vision
    let v = run_cli(&binary, &[
        "create", "--type", "vision", "--path", proj, "File Processing Toolkit",
    ])?;
    let vision_code = extract_short_code(&v.stdout, "TC-V-");
    if vision_code.is_empty() {
        anyhow::bail!("ultra-metis: could not extract vision short code");
    }

    // Transition vision: draft → review → published
    run_cli(&binary, &["transition", "--path", proj, &vision_code])?;
    run_cli(&binary, &["transition", "--path", proj, &vision_code])?;

    // Create initiative (generates the template)
    let init = run_cli(&binary, &[
        "create", "--type", "initiative", "--path", proj,
        "--parent", &vision_code, "CSV Parser Module",
    ])?;
    let init_code = extract_short_code(&init.stdout, "TC-I-");
    if init_code.is_empty() {
        anyhow::bail!("ultra-metis: could not extract initiative short code");
    }

    // ultra-metis stores docs as .ultra-metis/docs/<SHORT_CODE>.md (flat structure)
    let doc_path = temp_dir.path()
        .join(".ultra-metis")
        .join("docs")
        .join(format!("{init_code}.md"));
    std::fs::read_to_string(&doc_path)
        .with_context(|| format!("ultra-metis: could not read {:?}", doc_path))
}

/// Get the blank initiative template from original metis.
///
/// Original metis CLI differences from ultra-metis:
/// - No `--path` flag — must `cd` into project directory
/// - Vision is created during `init` (no separate `create vision`)
/// - Requires `--preset full` and a strategy before creating initiatives
/// - Uses subcommand syntax: `metis create initiative -s <STRATEGY> <TITLE>`
fn get_original_metis_template() -> anyhow::Result<String> {
    let metis = find_original_metis_binary()
        .ok_or_else(|| anyhow::anyhow!("original metis binary not found"))?;

    let temp_dir = tempfile::tempdir()?;
    let proj = temp_dir.path();

    // Init project (creates vision automatically as TC-V-0001)
    let init_out = std::process::Command::new(&metis)
        .current_dir(proj)
        .args(["init", "--prefix", "TC", "--preset", "full"])
        .output()
        .context("metis init failed")?;
    if !init_out.status.success() {
        let stderr = String::from_utf8_lossy(&init_out.stderr);
        anyhow::bail!("metis init failed: {}", stderr);
    }

    // Transition vision: draft → review → published
    std::process::Command::new(&metis)
        .current_dir(proj)
        .args(["transition", "TC-V-0001"])
        .output()
        .context("metis transition vision to review failed")?;
    std::process::Command::new(&metis)
        .current_dir(proj)
        .args(["transition", "TC-V-0001"])
        .output()
        .context("metis transition vision to published failed")?;

    // Create strategy (required before initiative in original metis)
    let strat_out = std::process::Command::new(&metis)
        .current_dir(proj)
        .args(["create", "strategy", "Data Processing Strategy"])
        .output()
        .context("metis create strategy failed")?;
    let strat_stdout = String::from_utf8_lossy(&strat_out.stdout).to_string();
    let strat_code = extract_short_code(&strat_stdout, "TC-S-");
    if strat_code.is_empty() {
        anyhow::bail!(
            "original metis: could not extract strategy short code from: {}",
            strat_stdout
        );
    }

    // Create initiative under the strategy
    let init_result = std::process::Command::new(&metis)
        .current_dir(proj)
        .args(["create", "initiative", "-s", &strat_code, "CSV Parser Module"])
        .output()
        .context("metis create initiative failed")?;
    if !init_result.status.success() {
        let stderr = String::from_utf8_lossy(&init_result.stderr);
        anyhow::bail!("metis create initiative failed: {}", stderr);
    }

    // Original metis stores as .metis/strategies/<id>/initiatives/<id>/initiative.md
    find_initiative_file(proj)
        .and_then(|p| std::fs::read_to_string(&p).ok())
        .ok_or_else(|| anyhow::anyhow!("original metis: initiative.md not found after create"))
}

/// Try to find the original metis binary (CLI, then plugin cache).
pub fn find_original_metis_binary() -> Option<String> {
    // Check if metis is already on PATH
    if std::process::Command::new("metis")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some("metis".to_string());
    }

    let home = std::env::var("HOME").ok()?;

    // Common plugin cache paths
    let base = format!("{home}/.claude/plugins/cache/colliery-io-metis");
    if let Ok(entries) = std::fs::read_dir(&base) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            // Try direct bin dir (e.g. metis/bin/metis)
            let bin = path.join("bin").join("metis");
            if bin.exists() {
                return bin.to_str().map(|s| s.to_string());
            }
            // Try versioned subdirs (e.g. metis/1.1.0/bin/metis)
            if let Ok(versions) = std::fs::read_dir(&path) {
                for v in versions.flatten() {
                    let vbin = v.path().join("bin").join("metis");
                    if vbin.exists() {
                        return vbin.to_str().map(|s| s.to_string());
                    }
                }
            }
        }
    }

    None
}

/// Ask Claude to fill an initiative template for a given module.
/// Returns (filled_content, tokens_used).
fn fill_template_with_claude(
    module_name: &str,
    module_description: &str,
    template: &str,
) -> anyhow::Result<(String, u64)> {
    let system = "You are a software architect filling in a Metis initiative document template. \
        Replace ALL placeholder text (anything in {curly braces} or marked as needing content) \
        with real, substantive content appropriate for the described module. \
        Do NOT leave any template instructions, {placeholder} text, or empty sections. \
        Output ONLY the filled markdown document — no commentary.";

    let user = format!(
        "Fill in this Metis initiative template for the '{module_name}' module.\n\n\
         Module description: {module_description}\n\n\
         Template to fill:\n\n{template}"
    );

    let output = std::process::Command::new("claude")
        .args([
            "-p", &user,
            "--system-prompt", system,
            "--output-format", "json",
            "--model", "haiku",
            "--no-session-persistence",
        ])
        .output()
        .context("Failed to invoke claude CLI")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("claude CLI error for '{}': {}", module_name, stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|_| serde_json::json!({"result": stdout.as_ref(), "usage": {}}));

    if parsed["is_error"].as_bool().unwrap_or(false) {
        anyhow::bail!(
            "claude CLI returned error: {}",
            parsed["result"].as_str().unwrap_or("unknown")
        );
    }

    let content = parsed["result"].as_str().unwrap_or("").to_string();
    let input_tokens = parsed["usage"]["input_tokens"].as_u64().unwrap_or(0);
    let output_tokens = parsed["usage"]["output_tokens"].as_u64().unwrap_or(0);

    Ok((content, input_tokens + output_tokens))
}

/// Score a template filled for all 3 modules.
fn score_template_runs(
    tool_name: &str,
    template: &str,
) -> anyhow::Result<ToolRunResult> {
    let start = Instant::now();
    let mut scores: Vec<DocQualityScore> = Vec::new();
    let mut total_tokens = 0u64;

    for (module_name, description) in MODULES {
        match fill_template_with_claude(module_name, description, template) {
            Ok((filled, tokens)) => {
                let score = score_content(&filled);
                tracing::info!(
                    "{} / {}: completeness={:.1}%, placeholders={}",
                    tool_name, module_name, score.completeness_percent, score.placeholder_count
                );
                scores.push(score);
                total_tokens += tokens;
            }
            Err(e) => {
                tracing::warn!("{} / {} failed: {}", tool_name, module_name, e);
                // Count as zero-score so the comparison reflects the failure
                scores.push(DocQualityScore::default());
            }
        }
    }

    let n = scores.len().max(1) as f32;
    let avg_completeness = scores.iter().map(|s| s.completeness_percent).sum::<f32>() / n;
    let avg_placeholders = scores.iter().map(|s| s.placeholder_count as f32).sum::<f32>() / n;
    let total_filled: u32 = scores.iter().map(|s| s.filled_sections.len() as u32).sum();
    let total_empty: u32 = scores.iter().map(|s| s.empty_sections.len() as u32).sum();

    Ok(ToolRunResult {
        tool_name: tool_name.to_string(),
        templates_tested: scores.len() as u32,
        avg_completeness_percent: avg_completeness,
        avg_placeholder_count: avg_placeholders,
        total_filled_sections: total_filled,
        total_empty_sections: total_empty,
        tokens_used: total_tokens,
        time_elapsed: start.elapsed(),
    })
}

/// Run the full tool comparison benchmark.
pub fn run_comparison() -> anyhow::Result<ToolComparisonResult> {
    tracing::info!("Starting tool comparison: ultra-metis vs original-metis");

    // Get ultra-metis template
    tracing::info!("Extracting ultra-metis initiative template...");
    let ultra_template = get_ultra_metis_template()
        .context("Failed to get ultra-metis template")?;
    tracing::info!("ultra-metis template: {} chars", ultra_template.len());

    // Get original-metis template
    tracing::info!("Extracting original-metis initiative template...");
    let orig_template = get_original_metis_template()
        .context("Failed to get original-metis template")?;
    tracing::info!("original-metis template: {} chars", orig_template.len());

    // Fill and score each template
    tracing::info!("Filling ultra-metis templates with Claude...");
    let ultra_result = score_template_runs("ultra-metis", &ultra_template)?;

    tracing::info!("Filling original-metis templates with Claude...");
    let orig_result = score_template_runs("original-metis", &orig_template)?;

    let completeness_delta = ultra_result.avg_completeness_percent - orig_result.avg_completeness_percent;
    let placeholder_delta = orig_result.avg_placeholder_count - ultra_result.avg_placeholder_count;

    Ok(ToolComparisonResult {
        timestamp: Utc::now(),
        ultra_metis: ultra_result,
        original_metis: orig_result,
        completeness_delta,
        placeholder_delta,
    })
}

/// Generate a markdown report from the comparison result.
pub fn format_comparison_report(result: &ToolComparisonResult) -> String {
    let mut out = String::new();

    out.push_str("# Tool Comparison Report: ultra-metis vs original-metis\n\n");
    out.push_str(&format!(
        "**Date**: {}  \n**Scenario**: File Processing Toolkit (3 modules)\n\n",
        result.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
    ));

    // Executive summary
    out.push_str("## Executive Summary\n\n");
    let (winner, loser) = if result.completeness_delta > 5.0 {
        ("**ultra-metis** templates", "original-metis")
    } else if result.completeness_delta < -5.0 {
        ("**original-metis** templates", "ultra-metis")
    } else {
        ("Both tool", "")
    };
    let summary = if result.completeness_delta.abs() < 5.0 {
        format!("{winner}s produce similarly complete documents (delta < 5%)")
    } else {
        format!("{winner} produce more complete initiative documents than {loser}")
    };
    out.push_str(&format!("- {summary}\n"));
    out.push_str(&format!(
        "- Completeness delta: **{:+.1}%** (ultra-metis − original-metis)\n",
        result.completeness_delta,
    ));
    out.push_str(&format!(
        "- Placeholder reduction: **{:+.1}** placeholders/doc (original − ultra)\n\n",
        result.placeholder_delta,
    ));

    // Per-tool table
    out.push_str("## Per-Tool Results\n\n");
    out.push_str("| Metric | ultra-metis | original-metis | Delta |\n");
    out.push_str("|--------|-------------|----------------|-------|\n");

    let u = &result.ultra_metis;
    let o = &result.original_metis;
    out.push_str(&format!(
        "| Templates tested | {} | {} | — |\n\
         | Avg completeness | {:.1}% | {:.1}% | {:+.1}% |\n\
         | Avg placeholder count | {:.1} | {:.1} | {:+.1} |\n\
         | Total filled sections | {} | {} | {:+} |\n\
         | Total empty sections | {} | {} | {:+} |\n\
         | Tokens used | {} | {} | {:+} |\n\
         | Time (s) | {:.1} | {:.1} | — |\n\n",
        u.templates_tested, o.templates_tested,
        u.avg_completeness_percent, o.avg_completeness_percent, result.completeness_delta,
        u.avg_placeholder_count, o.avg_placeholder_count,
        u.avg_placeholder_count - o.avg_placeholder_count,
        u.total_filled_sections, o.total_filled_sections,
        u.total_filled_sections as i32 - o.total_filled_sections as i32,
        u.total_empty_sections, o.total_empty_sections,
        u.total_empty_sections as i32 - o.total_empty_sections as i32,
        u.tokens_used, o.tokens_used,
        u.tokens_used as i64 - o.tokens_used as i64,
        u.time_elapsed.as_secs_f32(), o.time_elapsed.as_secs_f32(),
    ));

    // Interpretation
    out.push_str("## Interpretation\n\n");
    if result.completeness_delta.abs() < 5.0 {
        out.push_str("- Templates perform similarly — AI fills both equally well\n");
        out.push_str("- Template structure may not be the dominant factor in output quality\n");
    } else if result.completeness_delta > 0.0 {
        out.push_str("- ultra-metis simpler templates reduce AI confusion about what to fill\n");
        out.push_str("- Fewer structural elements → more direct guidance toward content\n");
    } else {
        out.push_str("- original-metis structured [CONDITIONAL] sections guide AI toward completeness\n");
        out.push_str("- Richer template scaffolding helps AI understand expected scope\n");
    }

    if result.placeholder_delta > 1.0 {
        out.push_str(&format!(
            "- original-metis templates leave {:.1}x more unfilled placeholders on average\n",
            if result.placeholder_delta > 0.0 { result.placeholder_delta } else { 0.0 },
        ));
    } else if result.placeholder_delta < -1.0 {
        out.push_str(&format!(
            "- ultra-metis templates leave {:.1} more unfilled placeholders on average\n",
            -result.placeholder_delta,
        ));
    } else {
        out.push_str("- Both tools produce similar placeholder completion rates\n");
    }

    out
}

/// Find the first `initiative.md` file under a directory (recursive walk).
fn find_initiative_file(root: &Path) -> Option<PathBuf> {
    find_initiative_recursive(root, 0)
}

fn find_initiative_recursive(dir: &Path, depth: usize) -> Option<PathBuf> {
    if depth > 8 {
        return None;
    }
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_initiative_recursive(&path, depth + 1) {
                return Some(found);
            }
        } else if path.file_name().map(|n| n == "initiative.md").unwrap_or(false) {
            return Some(path);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_initiative_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(find_initiative_file(dir.path()).is_none());
    }

    #[test]
    fn test_find_initiative_file_present() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("a").join("b");
        std::fs::create_dir_all(&nested).unwrap();
        let file = nested.join("initiative.md");
        std::fs::write(&file, "# Test").unwrap();
        let found = find_initiative_file(dir.path());
        assert!(found.is_some());
        assert_eq!(found.unwrap(), file);
    }

    #[test]
    fn test_find_original_metis_binary_does_not_panic() {
        // Just verify the function runs without panicking
        // Result depends on system state
        let _ = find_original_metis_binary();
    }

    #[test]
    fn test_tool_comparison_result_deltas() {
        let result = ToolComparisonResult {
            timestamp: Utc::now(),
            ultra_metis: ToolRunResult {
                tool_name: "ultra-metis".to_string(),
                templates_tested: 3,
                avg_completeness_percent: 80.0,
                avg_placeholder_count: 0.5,
                total_filled_sections: 9,
                total_empty_sections: 3,
                tokens_used: 1000,
                time_elapsed: std::time::Duration::from_secs(10),
            },
            original_metis: ToolRunResult {
                tool_name: "original-metis".to_string(),
                templates_tested: 3,
                avg_completeness_percent: 60.0,
                avg_placeholder_count: 3.0,
                total_filled_sections: 6,
                total_empty_sections: 6,
                tokens_used: 1200,
                time_elapsed: std::time::Duration::from_secs(12),
            },
            completeness_delta: 20.0,
            placeholder_delta: 2.5,
        };

        assert!(result.completeness_delta > 0.0);
        assert!(result.placeholder_delta > 0.0);

        let report = format_comparison_report(&result);
        assert!(report.contains("ultra-metis"));
        assert!(report.contains("original-metis"));
        assert!(report.contains("Executive Summary"));
        assert!(report.contains("Per-Tool Results"));
    }
}

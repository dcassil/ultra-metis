use std::path::Path;
use anyhow::Context;

pub struct ScenarioPrompt {
    pub system: String,
    pub user: String,
}

/// Build the scenario assessment prompt: given vision + 2 initiatives, what else is needed?
pub fn build_scenario_assessment_prompt(scenario_path: &Path) -> anyhow::Result<ScenarioPrompt> {
    let vision = std::fs::read_to_string(scenario_path.join("vision.md"))
        .context("Failed to read vision.md")?;
    let parse_init = std::fs::read_to_string(scenario_path.join("parse-initiative.md"))
        .context("Failed to read parse-initiative.md")?;
    let transform_init = std::fs::read_to_string(scenario_path.join("transform-initiative.md"))
        .context("Failed to read transform-initiative.md")?;

    let system = r#"You are a software architect reviewing a project plan.
Analyze the vision and existing initiatives, then assess what additional initiatives (if any) are needed to fully deliver the vision.
Return ONLY valid JSON with this exact structure:
{"analysis":"string","additional_initiatives_needed":true,"initiatives":[{"id":"slug","title":"Title","objective":"string","tasks":["task1","task2"]}]}
No markdown formatting, no code blocks, no text outside the JSON object."#.to_string();

    let user = format!(
        "## Project Vision\n\n{}\n\n## Existing Initiatives\n\n### Initiative 1\n{}\n\n### Initiative 2\n{}\n\nAnalyze these documents. What additional initiatives are needed to fully deliver the vision? Consider: output/delivery mechanisms, validation, integration testing, and any missing functionality.",
        vision.trim(), parse_init.trim(), transform_init.trim()
    );

    Ok(ScenarioPrompt { system, user })
}

/// Build a quality gate check prompt for reviewing an AI-generated initiative plan
pub fn build_gate_check_prompt(initiative_title: &str, initiative_content: &str) -> ScenarioPrompt {
    let system = r#"You are a quality gate reviewer for software development plans.
Evaluate whether the given initiative plan is complete and well-structured.
Return ONLY valid JSON: {"approved":true,"score":0.9,"issues":["issue description"]}
Score is 0.0 to 1.0. Approve (true) if score >= 0.7. No text outside JSON."#.to_string();

    let user = format!(
        "Review this initiative plan for '{}':\n\n{}\n\nCheck for: clear objective, specific actionable tasks, identified risks, defined acceptance criteria. Score 0.0-1.0.",
        initiative_title, initiative_content
    );

    ScenarioPrompt { system, user }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_gate_check_prompt_has_title() {
        let prompt = build_gate_check_prompt("Test Initiative", "some content");
        assert!(prompt.user.contains("Test Initiative"));
        assert!(prompt.system.contains("quality gate"));
    }

    #[test]
    fn test_scenario_prompt_missing_file() {
        let result = build_scenario_assessment_prompt(&PathBuf::from("/nonexistent/path"));
        assert!(result.is_err());
    }
}

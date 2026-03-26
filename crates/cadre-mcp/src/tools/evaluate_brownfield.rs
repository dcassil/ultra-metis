use cadre_core::{BrownfieldEvaluator, CatalogQueryEngine};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "evaluate_brownfield",
    description = "Evaluate how well the current repo matches a catalog architecture entry.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EvaluateBrownfieldTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Language to match against
    pub language: String,
    /// Project type to match against
    pub project_type: String,
}

impl EvaluateBrownfieldTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let engine = CatalogQueryEngine::with_builtins();
        let all_entries = engine.all_entries();

        if engine
            .find_exact(&self.language, &self.project_type)
            .is_none()
        {
            let text = format!(
                "No catalog entry found for language='{}', project_type='{}'. \
                Use `list_catalog_languages` to see available options.",
                self.language, self.project_type
            );
            return Ok(CallToolResult {
                content: vec![TextContent::new(text, None, None).into()],
                is_error: None,
                meta: None,
                structured_content: None,
            });
        }

        let mut file_paths = Vec::new();
        let project = Path::new(&self.project_path);
        let pattern = project.join("**/*").display().to_string();
        if let Ok(entries) = glob::glob(&pattern) {
            for entry in entries.flatten() {
                if entry.is_file() {
                    if let Ok(relative) = entry.strip_prefix(project) {
                        let path_str = relative.display().to_string();
                        if !path_str.starts_with('.')
                            && !path_str.contains("/node_modules/")
                            && !path_str.contains("/target/")
                        {
                            file_paths.push(path_str);
                        }
                    }
                }
            }
        }

        let evaluator = BrownfieldEvaluator::new();
        let result = evaluator.evaluate(
            &file_paths,
            all_entries,
            format!("eval-{}-{}", self.language, self.project_type),
        );

        let outcome_desc = match &result.outcome {
            cadre_core::EvaluationOutcome::CatalogMatch {
                catalog_entry_id, ..
            } => format!("Catalog Match ({catalog_entry_id})"),
            cadre_core::EvaluationOutcome::DerivedArchitecture { .. } => {
                "Derived Architecture (no catalog match, good quality)".to_string()
            }
            cadre_core::EvaluationOutcome::RecommendCatalogPattern {
                recommended_entry_id,
                ..
            } => format!("Recommend Catalog Pattern ({recommended_entry_id})"),
            cadre_core::EvaluationOutcome::RecordAsIs { .. } => "Record As-Is".to_string(),
        };

        let text = format!(
            "## Brownfield Evaluation\n\n\
            | Field | Value |\n\
            | ----- | ----- |\n\
            | Language | {} |\n\
            | Project Type | {} |\n\
            | Outcome | {} |\n\
            | Quality Score | {:.1}% |\n\
            | Files Analyzed | {} |",
            self.language,
            self.project_type,
            outcome_desc,
            result.quality_score,
            file_paths.len()
        );
        Ok(CallToolResult {
            content: vec![TextContent::new(text, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}

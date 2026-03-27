use cadre_core::BootstrapFlow;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::helpers::{format_bootstrap_result, store_for, tool_error};

#[mcp_tool(
    name = "initialize_project",
    description = "Initialize a new Cadre project directory with a .cadre folder structure.",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct InitializeProjectTool {
    /// Path to the project root directory
    pub project_path: String,
    /// Short code prefix (e.g., 'PROJ')
    pub prefix: String,
}

impl InitializeProjectTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let store = store_for(&self.project_path);
        store
            .initialize(&self.prefix)
            .map_err(|e| tool_error(e.user_message()))?;

        let file_paths = collect_source_files(&self.project_path);
        let is_brownfield = !file_paths.is_empty();
        let classification = if is_brownfield {
            "Brownfield"
        } else {
            "Greenfield"
        };

        let mut text = format!(
            "## Project Initialized\n\n\
            | Field | Value |\n\
            | ----- | ----- |\n\
            | Path | {} |\n\
            | Prefix | {} |\n\
            | Docs Dir | {}/.cadre/docs/ |\n\
            | Classification | {} |",
            self.project_path, self.prefix, self.project_path, classification
        );

        if is_brownfield {
            let result = BootstrapFlow::analyze(&file_paths);
            let analysis = format_bootstrap_result(&result);
            text.push_str("\n\n");
            text.push_str(&analysis);
        } else {
            text.push_str(
                "\n\n## Suggested Next Steps\n\
                1. Create a ProductDoc to define the product vision\n\
                2. Browse the architecture catalog to select a reference architecture\n\
                3. Create your first Epic to begin planning work",
            );
        }

        Ok(CallToolResult {
            content: vec![TextContent::new(text, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}

fn collect_source_files(project_path: &str) -> Vec<String> {
    let project = Path::new(project_path);
    let pattern = project.join("**/*").display().to_string();

    let mut file_paths = Vec::new();
    if let Ok(entries) = glob::glob(&pattern) {
        for entry in entries.flatten() {
            if entry.is_file() {
                if let Ok(relative) = entry.strip_prefix(project) {
                    let path_str = relative.display().to_string();
                    if !path_str.starts_with('.')
                        && !path_str.contains("/.")
                        && !path_str.contains("/target/")
                        && !path_str.contains("/node_modules/")
                        && !path_str.contains("/.cadre/")
                    {
                        file_paths.push(path_str);
                    }
                }
            }
        }
    }
    file_paths
}

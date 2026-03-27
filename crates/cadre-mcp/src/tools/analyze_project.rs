use cadre_core::BootstrapFlow;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::helpers::format_bootstrap_result;

#[mcp_tool(
    name = "analyze_project",
    description = "Analyze a project directory to detect languages, build tools, project type, and monorepo structure. Returns a comprehensive bootstrap analysis.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AnalyzeProjectTool {
    /// Path to the project root directory
    pub project_path: String,
}

impl AnalyzeProjectTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let project = Path::new(&self.project_path);
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

        let result = BootstrapFlow::analyze(&file_paths);
        let text = format_bootstrap_result(&result);

        Ok(CallToolResult {
            content: vec![TextContent::new(text, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}

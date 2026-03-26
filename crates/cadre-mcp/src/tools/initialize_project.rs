use super::helpers::{store_for, tool_error};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

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

        let text = format!(
            "## Project Initialized\n\n\
            | Field | Value |\n\
            | ----- | ----- |\n\
            | Path | {} |\n\
            | Prefix | {} |\n\
            | Docs Dir | {}/.cadre/docs/ |",
            self.project_path, self.prefix, self.project_path
        );
        Ok(CallToolResult {
            content: vec![TextContent::new(text, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}

use super::helpers::{store_for, tool_error};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "reassign_parent",
    description = "Reassign a task to a different parent initiative or to/from the backlog.",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReassignParentTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Task short code to reassign
    pub short_code: String,
    /// Target parent short code. Omit to move to backlog.
    pub new_parent_id: Option<String>,
    /// Category when moving to backlog: bug, feature, tech-debt
    pub backlog_category: Option<String>,
}

impl ReassignParentTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let store = store_for(&self.project_path);
        let result = store
            .reassign_parent(
                &self.short_code,
                self.new_parent_id.as_deref(),
                self.backlog_category.as_deref(),
            )
            .map_err(|e| tool_error(e.user_message()))?;

        let text = format!("## Reassignment\n\n{}", result);
        Ok(CallToolResult {
            content: vec![TextContent::new(text, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}

use super::helpers::{store_for, tool_error};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "edit_document",
    description = "Edit document content using search-and-replace. Use short codes (e.g., PROJ-V-0001) to identify documents.",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EditDocumentTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Document short code (e.g., PROJ-V-0001) to identify the document
    pub short_code: String,
    /// Text to search for (will be replaced)
    pub search: String,
    /// Text to replace the search text with
    pub replace: String,
    /// Whether to replace all occurrences (default: false, only first match)
    #[serde(default)]
    pub replace_all: Option<bool>,
}

impl EditDocumentTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let store = store_for(&self.project_path);
        let replace_all = self.replace_all.unwrap_or(false);
        store
            .edit_document_with_options(&self.short_code, &self.search, &self.replace, replace_all)
            .map_err(|e| tool_error(e.user_message()))?;

        let mode = if replace_all {
            "all occurrences"
        } else {
            "first occurrence"
        };
        let search_preview = if self.search.len() > 80 {
            format!("{}...", &self.search[..77])
        } else {
            self.search.clone()
        };
        let replace_preview = if self.replace.len() > 80 {
            format!("{}...", &self.replace[..77])
        } else {
            self.replace.clone()
        };

        let text = format!(
            "## Edit: {}\n\nReplaced {} of:\n```diff\n- {}\n+ {}\n```",
            self.short_code, mode, search_preview, replace_preview
        );
        Ok(CallToolResult {
            content: vec![TextContent::new(text, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}

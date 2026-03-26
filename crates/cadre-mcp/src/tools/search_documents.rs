use super::helpers::{store_for, tool_error};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "search_documents",
    description = "Search documents by content with optional filtering. Returns matching documents with their unique short codes (format: PREFIX-TYPE-NNNN).",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchDocumentsTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Search query to match against document content
    pub query: String,
    /// Filter by document type (vision, initiative, task, etc.)
    pub document_type: Option<String>,
    /// Maximum number of results to return
    pub limit: Option<u64>,
    /// Include archived documents in results (defaults to false)
    #[serde(default)]
    pub include_archived: Option<bool>,
}

impl SearchDocumentsTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let store = store_for(&self.project_path);
        let include_archived = self.include_archived.unwrap_or(false);
        let limit = self.limit.map(|v| v as usize);
        let docs = store
            .search_documents_with_options(
                &self.query,
                self.document_type.as_deref(),
                limit,
                include_archived,
            )
            .map_err(|e| tool_error(e.user_message()))?;

        if docs.is_empty() {
            let text = format!("No documents matching '{}'", self.query);
            return Ok(CallToolResult {
                content: vec![TextContent::new(text, None, None).into()],
                is_error: None,
                meta: None,
                structured_content: None,
            });
        }

        let mut output = format!(
            "## Search Results for '{}'\n\n\
            | Short Code | Title | Type | Phase |\n\
            | ---------- | ----- | ---- | ----- |\n",
            self.query
        );
        for doc in &docs {
            output.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                doc.short_code, doc.title, doc.document_type, doc.phase
            ));
        }

        Ok(CallToolResult {
            content: vec![TextContent::new(output, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}

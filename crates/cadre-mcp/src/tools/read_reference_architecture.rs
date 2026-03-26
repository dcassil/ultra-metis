use super::helpers::{store_for, tool_error};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "read_reference_architecture",
    description = "Read the project's selected reference architecture document.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReadReferenceArchitectureTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Short code of the ReferenceArchitecture (optional, finds active one if omitted)
    pub short_code: Option<String>,
}

impl ReadReferenceArchitectureTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let store = store_for(&self.project_path);

        let sc = if let Some(sc) = &self.short_code {
            sc.clone()
        } else {
            let all_docs = store
                .list_documents(false)
                .map_err(|e| tool_error(e.user_message()))?;
            let ra = all_docs
                .iter()
                .find(|d| d.document_type == "reference_architecture");
            match ra {
                Some(d) => d.short_code.clone(),
                None => {
                    let text = "No reference architecture found. Create one first.".to_string();
                    return Ok(CallToolResult {
                        content: vec![TextContent::new(text, None, None).into()],
                        is_error: None,
                        meta: None,
                        structured_content: None,
                    });
                }
            }
        };

        let raw = store
            .read_document_raw(&sc)
            .map_err(|e| tool_error(e.user_message()))?;
        let text = format!("## Reference Architecture: {}\n\n{}", sc, raw);
        Ok(CallToolResult {
            content: vec![TextContent::new(text, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}

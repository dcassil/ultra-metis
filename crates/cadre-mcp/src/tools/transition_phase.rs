use super::helpers::{store_for, tool_error};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "transition_phase",
    description = "Transition a document to a new phase using its short code (e.g., PROJ-V-0001). If phase is not provided, transitions to the next valid phase automatically. IMPORTANT: You can only transition to adjacent phases - you cannot skip phases (e.g., todo->completed is invalid; must go todo->active->completed).",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TransitionPhaseTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Document short code (e.g., PROJ-V-0001) to identify the document
    pub short_code: String,
    /// Phase to transition to (optional - if not provided, transitions to next phase)
    pub phase: Option<String>,
    /// Force transition even if exit criteria aren't met
    #[serde(default)]
    pub force: Option<bool>,
}

impl TransitionPhaseTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let store = store_for(&self.project_path);
        let force = self.force.unwrap_or(false);
        let result = store
            .transition_phase_with_options(&self.short_code, self.phase.as_deref(), force)
            .map_err(|e| tool_error(e.user_message()))?;

        // Build progress visualization
        let doc = store
            .read_document(&self.short_code)
            .map_err(|e| tool_error(e.user_message()))?;
        let doc_type = doc.document_type();
        let current_phase = doc.phase().map_err(|e| tool_error(e.to_string()))?;
        let sequence = doc_type.phase_sequence();
        let progress: String = sequence
            .iter()
            .map(|p| {
                let idx_current = sequence.iter().position(|s| s == &current_phase);
                let idx_this = sequence.iter().position(|s| s == p);
                if idx_this <= idx_current {
                    format!("\u{25cf} {p}")
                } else {
                    format!("\u{25cb} {p}")
                }
            })
            .collect::<Vec<_>>()
            .join(" -> ");

        let force_note = if force { " (forced)" } else { "" };
        let text = format!(
            "## Phase Transition{}\n\n{}: {}\n\n{}",
            force_note, self.short_code, result, progress
        );
        Ok(CallToolResult {
            content: vec![TextContent::new(text, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}

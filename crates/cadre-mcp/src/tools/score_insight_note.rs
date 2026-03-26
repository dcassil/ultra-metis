use super::helpers::{store_for, tool_error};
use cadre_core::{DurableInsightNote, FeedbackSignal};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "score_insight_note",
    description = "Record feedback on an insight note after using it (helpful/meh/harmful). Auto-detects prune candidates.",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ScoreInsightNoteTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Short code of the insight note
    pub short_code: String,
    /// Feedback signal: helpful, meh, or harmful
    pub signal: String,
}

impl ScoreInsightNoteTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let signal: FeedbackSignal = self.signal.parse().map_err(|e: String| tool_error(e))?;

        let store = store_for(&self.project_path);
        let raw = store
            .read_document_raw(&self.short_code)
            .map_err(|e| tool_error(e.user_message()))?;
        let mut din = DurableInsightNote::from_content(&raw).map_err(tool_error)?;

        din.record_feedback(signal);

        let was_pruned = din.should_be_prune_candidate(30, 0.5, 3, 5, 0.3);
        if was_pruned {
            din.mark_prune_candidate();
        }

        let content = din.to_content().map_err(tool_error)?;
        let doc_path = Path::new(&self.project_path)
            .join(".cadre")
            .join("docs")
            .join(format!("{}.md", self.short_code));
        std::fs::write(&doc_path, content).map_err(tool_error)?;

        let status_change = if was_pruned {
            " (marked as prune candidate)"
        } else {
            ""
        };

        let text = format!(
            "## Feedback Recorded{}\n\n\
            | Field | Value |\n\
            | ----- | ----- |\n\
            | Note | {} |\n\
            | Signal | {} |\n\
            | Total Helpful | {} |\n\
            | Total Meh | {} |\n\
            | Total Harmful | {} |\n\
            | Status | {} |",
            status_change,
            self.short_code,
            self.signal,
            din.thumbs_up_count,
            din.meh_count,
            din.thumbs_down_count,
            din.status
        );
        Ok(CallToolResult {
            content: vec![TextContent::new(text, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}

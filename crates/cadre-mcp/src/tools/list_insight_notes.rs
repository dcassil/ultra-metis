use super::helpers::{store_for, tool_error};
use cadre_core::DurableInsightNote;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "list_insight_notes",
    description = "List insight notes with optional status and category filtering.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListInsightNotesTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Filter by status: active, prune_candidate, needs_human_review, archived (optional)
    pub status: Option<String>,
    /// Filter by category (optional)
    pub category: Option<String>,
    /// Include archived notes (default: false)
    #[serde(default)]
    pub include_archived: Option<bool>,
}

impl ListInsightNotesTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let include_archived = self.include_archived.unwrap_or(false);
        let store = store_for(&self.project_path);
        let all_docs = store
            .list_documents(include_archived)
            .map_err(|e| tool_error(e.user_message()))?;

        let mut notes = Vec::new();
        for doc in &all_docs {
            if doc.document_type != "durable_insight_note" {
                continue;
            }
            let raw = match store.read_document_raw(&doc.short_code) {
                Ok(r) => r,
                Err(_) => continue,
            };
            if let Ok(din) = DurableInsightNote::from_content(&raw) {
                if let Some(sf) = &self.status {
                    if din.status.to_string() != *sf {
                        continue;
                    }
                }
                if let Some(cf) = &self.category {
                    if din.category.to_string() != *cf {
                        continue;
                    }
                }
                notes.push((doc.short_code.clone(), din));
            }
        }

        if notes.is_empty() {
            let text = "No insight notes found.".to_string();
            return Ok(CallToolResult {
                content: vec![TextContent::new(text, None, None).into()],
                is_error: None,
                meta: None,
                structured_content: None,
            });
        }

        let mut output = format!(
            "## Insight Notes ({})\n\n\
            | Short Code | Title | Category | Status | Fetches | Helpful% |\n\
            | ---------- | ----- | -------- | ------ | ------- | -------- |\n",
            notes.len()
        );
        for (sc, din) in &notes {
            let helpful_pct = if din.total_feedback() > 0 {
                format!("{:.0}%", din.helpful_ratio() * 100.0)
            } else {
                "-".to_string()
            };
            output.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                sc,
                din.title(),
                din.category,
                din.status,
                din.fetch_count,
                helpful_pct
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

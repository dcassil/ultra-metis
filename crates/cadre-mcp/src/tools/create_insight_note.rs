use super::helpers::{build_scope_from_args, store_for, tool_error};
use cadre_core::{DurableInsightNote, InsightCategory, Phase, Tag};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "create_insight_note",
    description = "Create a durable insight note capturing reusable local knowledge.",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateInsightNoteTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Short descriptive title for the insight
    pub title: String,
    /// The insight text - what the agent should know
    pub note: String,
    /// Category: hotspot_warning, recurring_failure, misleading_name, validation_hint, local_exception, boundary_warning, subsystem_quirk
    pub category: String,
    /// Repository name (optional)
    pub scope_repo: Option<String>,
    /// Package/crate name (optional)
    pub scope_package: Option<String>,
    /// Logical subsystem label (optional)
    pub scope_subsystem: Option<String>,
    /// File paths the insight applies to (optional)
    pub scope_paths: Option<Vec<String>>,
    /// Symbol names the insight applies to (optional)
    pub scope_symbols: Option<Vec<String>>,
}

impl CreateInsightNoteTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let category: InsightCategory = self.category.parse().map_err(|e: String| tool_error(e))?;

        let scope = build_scope_from_args(
            &self.scope_repo,
            &self.scope_package,
            &self.scope_subsystem,
            &self.scope_paths,
            &self.scope_symbols,
        );

        let store = store_for(&self.project_path);
        let short_code = store
            .create_document("durable_insight_note", &self.title, None)
            .map_err(|e| tool_error(e.user_message()))?;

        let din = DurableInsightNote::new(
            self.title.clone(),
            self.note.clone(),
            category,
            scope.clone(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            short_code.clone(),
        )
        .map_err(tool_error)?;

        let content = din.to_content().map_err(tool_error)?;
        let doc_path = Path::new(&self.project_path)
            .join(".cadre")
            .join("docs")
            .join(format!("{short_code}.md"));
        std::fs::write(&doc_path, content).map_err(tool_error)?;

        let scope_desc = [
            scope.repo.as_deref().unwrap_or(""),
            scope.package.as_deref().unwrap_or(""),
            scope.subsystem.as_deref().unwrap_or(""),
        ]
        .iter()
        .filter(|s| !s.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(", ");

        let text = format!(
            "## Insight Note Created\n\n\
            | Field | Value |\n\
            | ----- | ----- |\n\
            | Short Code | {} |\n\
            | Title | {} |\n\
            | Category | {} |\n\
            | Scope | {} |",
            short_code,
            self.title,
            self.category,
            if scope_desc.is_empty() {
                "global"
            } else {
                &scope_desc
            }
        );
        Ok(CallToolResult {
            content: vec![TextContent::new(text, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}

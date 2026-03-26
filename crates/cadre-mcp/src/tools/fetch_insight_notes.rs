use super::helpers::{build_scope_from_args, store_for, tool_error};
use cadre_core::DurableInsightNote;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "fetch_insight_notes",
    description = "Fetch insight notes relevant to a given scope (call at task start to load contextual knowledge). Increments fetch count.",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct FetchInsightNotesTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Repository name (optional)
    pub scope_repo: Option<String>,
    /// Package/crate name (optional)
    pub scope_package: Option<String>,
    /// Logical subsystem label (optional)
    pub scope_subsystem: Option<String>,
    /// File paths to match (optional)
    pub scope_paths: Option<Vec<String>>,
    /// Symbol names to match (optional)
    pub scope_symbols: Option<Vec<String>>,
    /// Max notes to return (default: 10)
    pub limit: Option<u64>,
}

impl FetchInsightNotesTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let limit = self.limit.unwrap_or(10) as usize;
        let query_scope = build_scope_from_args(
            &self.scope_repo,
            &self.scope_package,
            &self.scope_subsystem,
            &self.scope_paths,
            &self.scope_symbols,
        );
        let store = store_for(&self.project_path);

        let all_docs = store
            .list_documents(false)
            .map_err(|e| tool_error(e.user_message()))?;

        let mut matched = Vec::new();
        for doc in &all_docs {
            if doc.document_type != "durable_insight_note" {
                continue;
            }
            let raw = match store.read_document_raw(&doc.short_code) {
                Ok(r) => r,
                Err(_) => continue,
            };
            if let Ok(din) = DurableInsightNote::from_content(&raw) {
                if din.status != cadre_core::NoteStatus::Active {
                    continue;
                }
                let has_query = query_scope.repo.is_some()
                    || query_scope.package.is_some()
                    || query_scope.subsystem.is_some()
                    || !query_scope.paths.is_empty()
                    || !query_scope.symbols.is_empty();

                if !has_query || din.scope.matches(&query_scope) {
                    matched.push((doc.short_code.clone(), din));
                }
            }
            if matched.len() >= limit {
                break;
            }
        }

        // Record fetch on each matched note and save back
        for (sc, _) in &matched {
            let mut note =
                DurableInsightNote::from_content(&store.read_document_raw(sc).unwrap_or_default())
                    .ok();
            if let Some(ref mut n) = note {
                n.record_fetch();
                if let Ok(content) = n.to_content() {
                    let doc_path = Path::new(&self.project_path)
                        .join(".cadre")
                        .join("docs")
                        .join(format!("{sc}.md"));
                    let _ = std::fs::write(&doc_path, content);
                }
            }
        }

        if matched.is_empty() {
            let text = "No insight notes found for the given scope.".to_string();
            return Ok(CallToolResult {
                content: vec![TextContent::new(text, None, None).into()],
                is_error: None,
                meta: None,
                structured_content: None,
            });
        }

        let mut output = format!("## Insight Notes ({})\n\n", matched.len());
        for (sc, din) in &matched {
            output.push_str(&format!(
                "### {} — {} [{}]\n\n{}\n\n---\n\n",
                sc,
                din.title(),
                din.category,
                din.note
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

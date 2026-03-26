use super::helpers::{load_all_rules_configs, store_for, tool_error};
use cadre_core::domain::rules::query::{RuleQuery, RuleQueryEngine};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "query_rules",
    description = "Query engineering rules by scope, category, and protection level.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct QueryRulesTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Filter by scope: platform, org, repo, package, component, task (optional)
    pub scope: Option<String>,
    /// Filter by protection level: standard or protected (optional)
    pub protection_level: Option<String>,
    /// Filter by source architecture short code (optional)
    pub source_architecture_ref: Option<String>,
    /// Include archived rules (default: false)
    #[serde(default)]
    pub include_archived: Option<bool>,
}

impl QueryRulesTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let store = store_for(&self.project_path);
        let rules_configs = load_all_rules_configs(&store)?;
        let refs: Vec<_> = rules_configs.iter().collect();
        let engine = RuleQueryEngine::new(&refs);

        let mut query = RuleQuery::new();
        if let Some(s) = &self.scope {
            let scope = s
                .parse()
                .map_err(|e: cadre_core::DocumentValidationError| tool_error(e))?;
            query = query.with_scope(scope);
        }
        if let Some(p) = &self.protection_level {
            let level = p
                .parse()
                .map_err(|e: cadre_core::DocumentValidationError| tool_error(e))?;
            query = query.with_protection_level(level);
        }
        if let Some(a) = &self.source_architecture_ref {
            query = query.with_source_architecture_ref(a);
        }
        if self.include_archived.unwrap_or(false) {
            query = query.including_archived();
        }

        let results = engine.query(&query);

        if results.is_empty() {
            let text = "No rules matching the query.".to_string();
            return Ok(CallToolResult {
                content: vec![TextContent::new(text, None, None).into()],
                is_error: None,
                meta: None,
                structured_content: None,
            });
        }

        let mut output = format!(
            "## Rules Query Results ({})\n\n\
            | Short Code | Title | Scope | Protection | Phase |\n\
            | ---------- | ----- | ----- | ---------- | ----- |\n",
            results.len()
        );
        for rule in &results {
            let phase = rule
                .phase()
                .map(|p| p.to_string())
                .unwrap_or_else(|_| "?".to_string());
            output.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                rule.metadata().short_code,
                rule.title(),
                rule.scope,
                rule.protection_level,
                phase
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

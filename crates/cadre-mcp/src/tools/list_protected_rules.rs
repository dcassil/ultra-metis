use super::helpers::{load_all_rules_configs, store_for};
use cadre_core::domain::rules::query::RuleQueryEngine;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "list_protected_rules",
    description = "List all protected rules for governance auditing.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListProtectedRulesTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
}

impl ListProtectedRulesTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let store = store_for(&self.project_path);
        let rules_configs = load_all_rules_configs(&store)?;
        let refs: Vec<_> = rules_configs.iter().collect();
        let engine = RuleQueryEngine::new(&refs);

        let results = engine.protected_rules();

        if results.is_empty() {
            let text = "No protected rules found.".to_string();
            return Ok(CallToolResult {
                content: vec![TextContent::new(text, None, None).into()],
                is_error: None,
                meta: None,
                structured_content: None,
            });
        }

        let mut output = format!(
            "## Protected Rules ({})\n\n\
            | Short Code | Title | Scope | Source Architecture |\n\
            | ---------- | ----- | ----- | ------------------- |\n",
            results.len()
        );
        for rule in &results {
            let arch_ref = rule.source_architecture_ref.as_deref().unwrap_or("-");
            output.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                rule.metadata().short_code,
                rule.title(),
                rule.scope,
                arch_ref
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

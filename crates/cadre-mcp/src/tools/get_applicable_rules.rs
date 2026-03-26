use super::helpers::{load_all_rules_configs, store_for, tool_error};
use cadre_core::domain::rules::query::{scope_hierarchy, scope_rank, RuleQueryEngine};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "get_applicable_rules",
    description = "Get all rules that apply at a given scope via inheritance (broadest to narrowest).",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetApplicableRulesTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Target scope level: platform, org, repo, package, component, task
    pub target_scope: String,
}

impl GetApplicableRulesTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let target_scope: cadre_core::domain::documents::rules_config::RuleScope = self
            .target_scope
            .parse()
            .map_err(|e: cadre_core::DocumentValidationError| tool_error(e))?;

        let store = store_for(&self.project_path);
        let rules_configs = load_all_rules_configs(&store)?;
        let refs: Vec<_> = rules_configs.iter().collect();
        let engine = RuleQueryEngine::new(&refs);

        let results = engine.applicable_at_scope(target_scope);

        if results.is_empty() {
            let text = format!("No rules applicable at scope '{}'.", self.target_scope);
            return Ok(CallToolResult {
                content: vec![TextContent::new(text, None, None).into()],
                is_error: None,
                meta: None,
                structured_content: None,
            });
        }

        let hierarchy: Vec<&str> = scope_hierarchy()
            .iter()
            .take_while(|s| scope_rank(**s) <= scope_rank(target_scope))
            .map(|s| {
                use cadre_core::domain::documents::rules_config::RuleScope;
                match s {
                    RuleScope::Platform => "platform",
                    RuleScope::Organization => "org",
                    RuleScope::Repository => "repo",
                    RuleScope::Package => "package",
                    RuleScope::Component => "component",
                    RuleScope::Task => "task",
                }
            })
            .collect();

        let mut output = format!(
            "## Applicable Rules at '{}' Scope ({} rules)\n\n\
            Inheritance chain: {}\n\n\
            | Short Code | Title | Scope | Protection |\n\
            | ---------- | ----- | ----- | ---------- |\n",
            self.target_scope,
            results.len(),
            hierarchy.join(" > ")
        );
        for rule in &results {
            output.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                rule.metadata().short_code,
                rule.title(),
                rule.scope,
                rule.protection_level
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

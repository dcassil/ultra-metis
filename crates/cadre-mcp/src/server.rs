use crate::log;
use crate::tools::{
    ArchiveDocumentTool, CadreTools, CaptureQualityBaselineTool, CheckArchitectureConformanceTool,
    CompareQualityBaselinesTool, CreateCrossReferenceTool, CreateDocumentTool,
    CreateInsightNoteTool, EditDocumentTool, EvaluateBrownfieldTool, FetchInsightNotesTool,
    GetApplicableRulesTool, IndexCodeTool, InitializeProjectTool, ListCatalogLanguagesTool,
    ListCrossReferencesTool, ListDocumentsTool, ListInsightNotesTool, ListProtectedRulesTool,
    ListQualityRecordsTool, QueryArchitectureCatalogTool, QueryRelationshipsTool, QueryRulesTool,
    ReadDocumentTool, ReadReferenceArchitectureTool, ReassignParentTool, ScoreInsightNoteTool,
    SearchDocumentsTool, TraceAncestryTool, TransitionPhaseTool,
};
use async_trait::async_trait;
use rust_mcp_sdk::{
    mcp_server::ServerHandler,
    schema::{
        schema_utils::CallToolError, CallToolRequestParams, CallToolResult, ListToolsResult,
        PaginatedRequestParams, RpcError,
    },
    McpServer,
};
use std::sync::Arc;

/// Deserialize JSON args into a tool struct and invoke it.
///
/// Each match arm follows the same pattern: deserialize, then call.
/// This macro eliminates that repetition.
macro_rules! dispatch_tool {
    ($tool_name:expr, $tool_args:expr, $($name:literal => $tool_ty:ty),+ $(,)?) => {
        match $tool_name.as_str() {
            $(
                $name => {
                    let tool: $tool_ty = serde_json::from_value($tool_args)
                        .map_err(CallToolError::new)?;
                    tool.call_tool().await
                }
            )+
            _ => {
                log(&format!("unknown tool: {}", $tool_name));
                Err(CallToolError::unknown_tool($tool_name))
            }
        }
    };
}

pub struct CadreServerHandler;

impl CadreServerHandler {
    #[allow(clippy::new_without_default)] // new() has logging side-effects
    pub fn new() -> Self {
        log("CadreServerHandler::new() called");
        Self
    }
}

#[async_trait]
impl ServerHandler for CadreServerHandler {
    async fn handle_list_tools_request(
        &self,
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> Result<ListToolsResult, RpcError> {
        let tools = CadreTools::tools();
        log(&format!(
            "handle_list_tools_request: returning {} tools",
            tools.len()
        ));
        Ok(ListToolsResult {
            tools,
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> Result<CallToolResult, CallToolError> {
        let args = serde_json::Value::Object(params.arguments.unwrap_or_default());
        log(&format!(
            "handle_call_tool_request: tool='{}' args_keys={:?}",
            params.name,
            args.as_object().map(|o| o.keys().collect::<Vec<_>>())
        ));

        let result = dispatch_tool!(
            params.name, args,
            "initialize_project"           => InitializeProjectTool,
            "create_document"              => CreateDocumentTool,
            "read_document"                => ReadDocumentTool,
            "list_documents"               => ListDocumentsTool,
            "edit_document"                => EditDocumentTool,
            "transition_phase"             => TransitionPhaseTool,
            "search_documents"             => SearchDocumentsTool,
            "archive_document"             => ArchiveDocumentTool,
            "reassign_parent"              => ReassignParentTool,
            "index_code"                   => IndexCodeTool,
            "capture_quality_baseline"     => CaptureQualityBaselineTool,
            "compare_quality_baselines"    => CompareQualityBaselinesTool,
            "list_quality_records"         => ListQualityRecordsTool,
            "check_architecture_conformance" => CheckArchitectureConformanceTool,
            "query_rules"                  => QueryRulesTool,
            "get_applicable_rules"         => GetApplicableRulesTool,
            "list_protected_rules"         => ListProtectedRulesTool,
            "create_insight_note"          => CreateInsightNoteTool,
            "fetch_insight_notes"          => FetchInsightNotesTool,
            "score_insight_note"           => ScoreInsightNoteTool,
            "list_insight_notes"           => ListInsightNotesTool,
            "create_cross_reference"       => CreateCrossReferenceTool,
            "query_relationships"          => QueryRelationshipsTool,
            "trace_ancestry"               => TraceAncestryTool,
            "list_cross_references"        => ListCrossReferencesTool,
            "query_architecture_catalog"   => QueryArchitectureCatalogTool,
            "list_catalog_languages"       => ListCatalogLanguagesTool,
            "read_reference_architecture"  => ReadReferenceArchitectureTool,
            "evaluate_brownfield"          => EvaluateBrownfieldTool,
        );

        match &result {
            Ok(_) => log("tool call succeeded"),
            Err(e) => log(&format!("tool call failed: {e:?}")),
        }

        result
    }
}

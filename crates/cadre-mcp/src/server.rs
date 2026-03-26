use crate::tools::{
    ArchiveDocumentTool, CadreTools, CaptureQualityBaselineTool, CheckArchitectureConformanceTool,
    CompareQualityBaselinesTool, CreateCrossReferenceTool, CreateDocumentTool,
    CreateInsightNoteTool, EditDocumentTool, EvaluateBrownfieldTool, FetchInsightNotesTool,
    GetApplicableRulesTool, IndexCodeTool, InitializeProjectTool, ListCatalogLanguagesTool,
    ListCrossReferencesTool, ListDocumentsTool, ListInsightNotesTool, ListProtectedRulesTool,
    ListQualityRecordsTool, QueryArchitectureCatalogTool, QueryRelationshipsTool,
    QueryRulesTool, ReadReferenceArchitectureTool, ReassignParentTool, ReadDocumentTool,
    ScoreInsightNoteTool, SearchDocumentsTool, TraceAncestryTool, TransitionPhaseTool,
};
use crate::log;
use async_trait::async_trait;
use rust_mcp_sdk::{
    mcp_server::ServerHandler,
    schema::{
        CallToolRequestParams, CallToolResult, ListToolsResult, PaginatedRequestParams, RpcError,
    },
    McpServer,
};
use std::sync::Arc;

pub struct CadreServerHandler;

impl CadreServerHandler {
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
        log(&format!("handle_list_tools_request: returning {} tools", tools.len()));
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
    ) -> Result<CallToolResult, rust_mcp_sdk::schema::schema_utils::CallToolError> {
        let args = serde_json::Value::Object(params.arguments.unwrap_or_default());
        log(&format!("handle_call_tool_request: tool='{}' args_keys={:?}", params.name, args.as_object().map(|o| o.keys().collect::<Vec<_>>())));

        let result = match params.name.as_str() {
            "initialize_project" => {
                let tool: InitializeProjectTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "create_document" => {
                let tool: CreateDocumentTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "read_document" => {
                let tool: ReadDocumentTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "list_documents" => {
                let tool: ListDocumentsTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "edit_document" => {
                let tool: EditDocumentTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "transition_phase" => {
                let tool: TransitionPhaseTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "search_documents" => {
                let tool: SearchDocumentsTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "archive_document" => {
                let tool: ArchiveDocumentTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "reassign_parent" => {
                let tool: ReassignParentTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "index_code" => {
                let tool: IndexCodeTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "capture_quality_baseline" => {
                let tool: CaptureQualityBaselineTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "compare_quality_baselines" => {
                let tool: CompareQualityBaselinesTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "list_quality_records" => {
                let tool: ListQualityRecordsTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "check_architecture_conformance" => {
                let tool: CheckArchitectureConformanceTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "query_rules" => {
                let tool: QueryRulesTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "get_applicable_rules" => {
                let tool: GetApplicableRulesTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "list_protected_rules" => {
                let tool: ListProtectedRulesTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "create_insight_note" => {
                let tool: CreateInsightNoteTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "fetch_insight_notes" => {
                let tool: FetchInsightNotesTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "score_insight_note" => {
                let tool: ScoreInsightNoteTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "list_insight_notes" => {
                let tool: ListInsightNotesTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "create_cross_reference" => {
                let tool: CreateCrossReferenceTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "query_relationships" => {
                let tool: QueryRelationshipsTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "trace_ancestry" => {
                let tool: TraceAncestryTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "list_cross_references" => {
                let tool: ListCrossReferencesTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "query_architecture_catalog" => {
                let tool: QueryArchitectureCatalogTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "list_catalog_languages" => {
                let tool: ListCatalogLanguagesTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "read_reference_architecture" => {
                let tool: ReadReferenceArchitectureTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "evaluate_brownfield" => {
                let tool: EvaluateBrownfieldTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            _ => {
                log(&format!("unknown tool: {}", params.name));
                Err(
                    rust_mcp_sdk::schema::schema_utils::CallToolError::unknown_tool(params.name),
                )
            }
        };

        match &result {
            Ok(_) => log(&format!("tool call succeeded")),
            Err(e) => log(&format!("tool call failed: {:?}", e)),
        }

        result
    }
}

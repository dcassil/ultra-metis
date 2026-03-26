#![allow(clippy::redundant_closure)]
#![allow(clippy::io_other_error)]

pub mod server;
pub mod system_prompt;
pub mod tools;

pub use server::CadreServerHandler;

use anyhow::Result;
use rust_mcp_sdk::{
    mcp_server::{server_runtime, McpServerOptions},
    schema::{
        Implementation, InitializeResult, ServerCapabilities, ServerCapabilitiesTools,
        LATEST_PROTOCOL_VERSION,
    },
    McpServer, StdioTransport, ToMcpServerHandler, TransportOptions,
};
use tracing::info;

/// Run the MCP server
pub async fn run() -> Result<()> {
    // Initialize logging to stderr (stdout is reserved for MCP protocol)
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .with_max_level(tracing::Level::WARN)
        .try_init();

    info!("Starting Cadre MCP Server");

    let server_details = InitializeResult {
        server_info: Implementation {
            name: "Cadre Document Management System".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            title: Some("Cadre MCP Server".to_string()),
            description: Some(
                "MCP server for repo-native AI engineering orchestration".to_string(),
            ),
            icons: vec![],
            website_url: None,
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        meta: None,
        instructions: Some(system_prompt::SYSTEM_PROMPT.to_string()),
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
    };

    let transport = StdioTransport::new(TransportOptions::default())
        .map_err(|e| anyhow::anyhow!("Failed to create transport: {}", e))?;

    let handler = CadreServerHandler::new().to_mcp_server_handler();

    let server = server_runtime::create_server(McpServerOptions {
        server_details,
        transport,
        handler,
        task_store: None,
        client_task_store: None,
    });

    info!("MCP Server starting on stdio transport");
    server
        .start()
        .await
        .map_err(|e| anyhow::anyhow!("MCP server failed to start: {}", e))?;

    Ok(())
}

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
use std::io::Write;

pub fn log(msg: &str) {
    let log_path = std::env::temp_dir().join("cadre-mcp-debug.log");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let _ = writeln!(f, "[{}] {}", now, msg);
    }
}

/// Run the MCP server
pub async fn run() -> Result<()> {
    log("=== cadre-mcp starting ===");
    log(&format!("PID: {}", std::process::id()));
    log(&format!("version: {}", env!("CARGO_PKG_VERSION")));
    log(&format!("args: {:?}", std::env::args().collect::<Vec<_>>()));

    // Initialize logging to a file (stderr may not be visible, stdout is MCP protocol)
    let log_path = std::env::temp_dir().join("cadre-mcp-tracing.log");
    log(&format!("tracing log path: {}", log_path.display()));

    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path);

    match log_file {
        Ok(file) => {
            let _ = tracing_subscriber::fmt()
                .with_writer(std::sync::Mutex::new(file))
                .with_ansi(false)
                .with_max_level(tracing::Level::DEBUG)
                .try_init();
            log("tracing initialized to file");
        }
        Err(e) => {
            log(&format!("failed to open tracing log: {}", e));
            let _ = tracing_subscriber::fmt()
                .with_writer(std::io::stderr)
                .with_ansi(false)
                .with_max_level(tracing::Level::DEBUG)
                .try_init();
        }
    }

    tracing::info!("Starting Cadre MCP Server");
    log("building server_details (InitializeResult)");

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

    log(&format!(
        "protocol_version: {}",
        server_details.protocol_version
    ));
    log(&format!(
        "instructions length: {} chars",
        server_details
            .instructions
            .as_ref()
            .map(|s| s.len())
            .unwrap_or(0)
    ));
    log(&format!(
        "capabilities: tools={:?}",
        server_details.capabilities.tools.is_some()
    ));

    log("creating StdioTransport");
    let transport = match StdioTransport::new(TransportOptions::default()) {
        Ok(t) => {
            log("StdioTransport created successfully");
            t
        }
        Err(e) => {
            log(&format!("FATAL: Failed to create StdioTransport: {}", e));
            return Err(anyhow::anyhow!("Failed to create transport: {}", e));
        }
    };

    log("creating CadreServerHandler");
    let handler = CadreServerHandler::new().to_mcp_server_handler();
    log("handler created and wrapped");

    log("creating server runtime");
    let server = server_runtime::create_server(McpServerOptions {
        server_details,
        transport,
        handler,
        task_store: None,
        client_task_store: None,
    });

    log("calling server.start().await — entering MCP event loop");
    tracing::info!("MCP Server starting on stdio transport");

    match server.start().await {
        Ok(()) => {
            log("server.start() returned Ok — clean shutdown");
            Ok(())
        }
        Err(e) => {
            log(&format!("server.start() returned Err: {}", e));
            Err(anyhow::anyhow!("MCP server failed: {}", e))
        }
    }
}

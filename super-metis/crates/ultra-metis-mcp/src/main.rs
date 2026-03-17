//! Ultra-Metis MCP Server
//!
//! A Model Context Protocol server that exposes Ultra-Metis document
//! management operations as tools. Communicates via JSON-RPC 2.0 over stdio.

mod protocol;
mod tools;

use protocol::McpServer;
use std::io::{self, BufRead, Write};
use tracing_subscriber::EnvFilter;

fn main() {
    // Initialize logging to stderr (stdout is reserved for JSON-RPC)
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_writer(io::stderr)
        .init();

    let mut server = McpServer::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Failed to read stdin: {}", e);
                break;
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let response = server.handle_message(trimmed);

        if let Some(resp) = response {
            let json = serde_json::to_string(&resp).unwrap();
            writeln!(stdout, "{}", json).unwrap();
            stdout.flush().unwrap();
        }
    }
}

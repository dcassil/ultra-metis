---
id: replace-hand-rolled-mcp-server
level: initiative
title: "Replace Hand-Rolled MCP Server with rust-mcp-sdk Based Implementation"
short_code: "SMET-I-0083"
created_at: 2026-03-26T17:05:38.583370+00:00
updated_at: 2026-03-26T17:30:28.511408+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
strategy_id: SMET-S-0001
initiative_id: replace-hand-rolled-mcp-server
---

# Replace Hand-Rolled MCP Server with rust-mcp-sdk Based Implementation Initiative

## Context

The current `cadre-mcp` crate implements a hand-rolled MCP server that speaks raw JSON-RPC 2.0 over bare stdio lines. It reads newline-delimited JSON from stdin and writes newline-delimited JSON to stdout. **This does not work** because Claude Code's MCP client expects the proper MCP stdio transport protocol as implemented by the `rust-mcp-sdk` library, which uses content-length framed messages (similar to LSP).

The reference implementation (`metis-docs-mcp` in `super-metis/metis/crates/cadre-docs-mcp/`) uses `rust-mcp-sdk v0.8.0` with `StdioTransport`, an async tokio runtime, the `ServerHandler` trait, and proper `schemars`-derived JSON schemas for tool input validation. The cadre-mcp binary starts, reads stdin, but never produces any output because the framing protocols are incompatible — the client sends MCP-framed messages that the bare `serde_json::from_str` parser cannot parse.

### Reference implementation (working): `super-metis/metis/crates/cadre-docs-mcp/`
### Current broken implementation: `crates/cadre-mcp/`

## Goals & Non-Goals

**Goals:**
- Make cadre-mcp a fully functional MCP server that Claude Code can communicate with
- Achieve feature parity with the reference metis-docs-mcp transport/protocol layer
- Preserve all existing tool definitions and their business logic
- Support the `instructions` field for dynamic system prompt injection

**Non-Goals:**
- Changing the tool business logic in cadre-core or cadre-store
- Adding new tools beyond what already exists
- Supporting HTTP/SSE transport (stdio only for now)
- Changing the plugin directory structure or .mcp.json config

## Diagnosis: What's Wrong

### 1. REMOVE: Hand-rolled protocol layer (`protocol.rs`)
The entire `protocol.rs` file implements manual JSON-RPC structs (`JsonRpcRequest`, `JsonRpcResponse`, `JsonRpcError`, `ToolDefinition`) and a synchronous `McpServer` struct with `handle_message()`. This must be **replaced entirely** with the `rust-mcp-sdk` `ServerHandler` trait implementation.

**Files to remove:**
- `crates/cadre-mcp/src/protocol.rs` — the entire hand-rolled JSON-RPC layer

### 2. REWRITE: Entry point (`main.rs`)
Current `main.rs` is a synchronous `fn main()` that does `stdin.lock().lines()` in a blocking loop. Must become `#[tokio::main] async fn main()` that creates a `StdioTransport`, a `ServerHandler`, and calls `server_runtime::create_server().start().await`.

**Changes needed:**
- Add `#[tokio::main]` async entry point
- Create `StdioTransport` from `rust-mcp-sdk`
- Create `InitializeResult` with server info, capabilities, and dynamic instructions
- Wire up the handler and start the server runtime

### 3. ADD: `lib.rs` with `run()` function
The reference has a `lib.rs` that exports a `pub async fn run()` containing all server setup. This is cleaner than putting everything in main and enables testing. Should also contain the dynamic instructions generation logic (currently in `system_prompt.rs`).

**New file:** `crates/cadre-mcp/src/lib.rs`

### 4. ADD: `server.rs` with `ServerHandler` implementation
Need a proper handler struct implementing `rust_mcp_sdk::mcp_server::ServerHandler` trait with:
- `handle_list_tools_request()` — return tool definitions
- `handle_call_tool_request()` — dispatch to tool implementations

**New file:** `crates/cadre-mcp/src/server.rs`

### 5. REWRITE: Tool definitions (`tools.rs`)
Current `tools.rs` defines tools using hand-rolled `ToolDefinition` structs with manual JSON schemas and a monolithic `call_tool()` function. Must be rewritten to use `schemars::JsonSchema` derive macros on per-tool structs that implement an async `call_tool()` method, matching the reference pattern.

**Current pattern (broken):**
```rust
pub fn get_tool_definitions() -> Vec<ToolDefinition> { /* manual JSON */ }
pub fn call_tool(name: &str, args: &Value) -> Result<String> { /* big match */ }
```

**Target pattern (reference):**
```rust
#[derive(Deserialize, JsonSchema)]
struct ListDocumentsTool { project_path: String, ... }
impl ListDocumentsTool {
    async fn call_tool(&self) -> Result<CallToolResult, CallToolError> { ... }
}
```

### 6. ADD: Missing Cargo.toml dependencies
Current dependencies are missing critical crates:

| Dependency | Purpose | Status |
|-----------|---------|--------|
| `rust-mcp-sdk` | MCP protocol, transport, server runtime | **MISSING** |
| `tokio` | Async runtime | **MISSING** |
| `async-trait` | Async trait methods for ServerHandler | **MISSING** |
| `schemars` | JSON Schema generation for tool inputs | **MISSING** |
| `clap` | CLI argument parsing (optional) | **MISSING** |
| `futures` | Async stream utilities | **MISSING** |
| `thiserror` | Error type derivation | **MISSING** |

### 7. ADAPT: `system_prompt.rs` → dynamic instructions
The `system_prompt.rs` content should be moved into the `instructions` field of `InitializeResult` in `lib.rs`, following the reference pattern of `generate_dynamic_instructions()`. The current constants-based approach can be adapted but must feed into the SDK's instructions mechanism instead of MCP prompts.

### 8. REMOVE: MCP prompts handling
The current `handle_prompts_list` and `handle_prompts_get` in `protocol.rs` expose a system prompt as an MCP prompt resource. The reference does not do this — it uses the `instructions` field on `InitializeResult` instead. The prompts capability should be removed.

## Detailed Design

The implementation follows the reference architecture exactly:

```
crates/cadre-mcp/
├── Cargo.toml          # Updated with rust-mcp-sdk, tokio, schemars, etc.
├── src/
│   ├── main.rs         # Minimal: #[tokio::main] → lib::run()
│   ├── lib.rs          # Server setup, transport, dynamic instructions
│   ├── server.rs       # ServerHandler trait impl (list_tools, call_tool dispatch)
│   ├── tools/
│   │   ├── mod.rs      # Tool registry, MetisTools::tools() equivalent
│   │   ├── list_documents.rs
│   │   ├── create_document.rs
│   │   ├── read_document.rs
│   │   ├── edit_document.rs
│   │   ├── transition_phase.rs
│   │   ├── search_documents.rs
│   │   ├── archive_document.rs
│   │   ├── reassign_parent.rs
│   │   ├── initialize_project.rs
│   │   ├── index_code.rs
│   │   └── all_tools.rs    # Aggregates all tool definitions
│   └── (protocol.rs DELETED)
│   └── (system_prompt.rs content moved to lib.rs)
```

Each tool becomes its own file with a `#[derive(Deserialize, JsonSchema)]` struct and an `async fn call_tool(&self)` method. The server handler dispatches by tool name to the appropriate struct.

## Alternatives Considered

**1. Fix the framing in the hand-rolled server** — Could add content-length header framing to the existing synchronous implementation. Rejected because: the MCP protocol is more than just framing; the SDK handles lifecycle, capabilities negotiation, error formatting, and future protocol changes. Hand-rolling this is fragile and maintenance-heavy.

**2. Keep synchronous, just add the SDK** — The rust-mcp-sdk requires async. Not viable without tokio.

**3. Use a different MCP library** — rust-mcp-sdk is the one used by the working reference and is the most mature Rust MCP implementation. No reason to diverge.

## Implementation Plan

1. Update `Cargo.toml` with all missing dependencies
2. Create `lib.rs` with server setup and dynamic instructions
3. Create `server.rs` with `ServerHandler` trait implementation
4. Split `tools.rs` into per-tool modules with schemars-derived schemas
5. Rewrite `main.rs` to minimal async entry point
6. Delete `protocol.rs`
7. Migrate `system_prompt.rs` content into instructions generation
8. Build, install, and verify Claude Code can communicate with the server
---
id: cadre-mcp-tool-dispatcher
level: initiative
title: "cadre-mcp: Tool Dispatcher Decomposition and Schema Generation"
short_code: "SMET-I-0088"
created_at: 2026-03-26T17:21:38.278504+00:00
updated_at: 2026-03-26T17:21:38.278504+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: SMET-S-0001
initiative_id: cadre-mcp-tool-dispatcher
---

# cadre-mcp: Tool Dispatcher Decomposition and Schema Generation Initiative

## Context

**UPDATE (2026-03-26): This initiative is largely already completed by SMET-I-0083.** The old monolithic `tools.rs` (2,505 LOC of hand-written JSON schemas) has been deleted and replaced with the `rust-mcp-sdk` pattern. The exploration agents reported stale data from before SMET-I-0083 finished.

### Current State (Post SMET-I-0083)

cadre-mcp is now **well-structured** at ~2,993 total LOC across 33 files:

```
cadre-mcp/src/
├── lib.rs              (72 LOC)
├── main.rs             (6 LOC)
├── server.rs           (203 LOC - dispatch match + handler)
├── system_prompt.rs    (117 LOC)
├── tools/
│   ├── mod.rs          (62 LOC - pub use * re-exports)
│   ├── all_tools.rs    (67 LOC - tool_box! macro registration)
│   ├── helpers.rs      (105 LOC)
│   └── 28 tool files   (38-126 LOC each, all using #[mcp_tool] + #[derive(JsonSchema)])
```

Each tool already uses:
- `#[mcp_tool(name = "...", description = "...")]` proc macro for metadata + schema generation
- `#[derive(JsonSchema)]` for automatic JSON schema from Rust types
- `tool_box!` macro for centralized registration
- Self-contained file with args struct + handler

### Remaining Minor Issues

1. **`tools/mod.rs` uses `pub use *` wildcard re-exports** for all 28 tool modules - should use explicit exports
2. **`server.rs` has a 203-line manual match dispatch** (28 arms) - could potentially use `tool_box!` dispatch instead of hand-written match, but this is idiomatic and readable
3. **A few tool files exceed 100 LOC** (6 files at 102-126 LOC) - borderline, mostly due to complex output formatting

## Goals & Non-Goals

**Goals:**
- Replace `pub use *` wildcard re-exports in `tools/mod.rs` with explicit exports
- Evaluate whether `server.rs` dispatch can be simplified via the `tool_box!` macro (may already be possible)
- Split the 6 tool files over 100 LOC if there's a clean separation point (output formatting vs logic)

**Non-Goals:**
- Major restructuring (already well-organized)
- Schema generation changes (already using schemars + mcp_tool proc macro)
- Adding new tools or changing behavior

## Detailed Design

### 1. Fix Wildcard Re-exports in tools/mod.rs

Replace:
```rust
pub use create_document::*;
```
With explicit:
```rust
pub use create_document::CreateDocumentTool;
```

### 2. Evaluate server.rs Dispatch

The current 28-arm match in `handle_call_tool_request` repeats the same 4-line pattern for each tool. The `tool_box!` macro may already generate a dispatch method that could replace this. If so, `server.rs` shrinks significantly. If not, a simple declarative macro could eliminate the repetition.

### 3. Split Borderline Files (Optional)

Files at 102-126 LOC are barely over the threshold. Only split if output formatting logic can cleanly separate from tool logic.

## Implementation Plan

This is now an **S-complexity** initiative (was L when the old tools.rs existed):

1. **Replace wildcard re-exports** (1 task): Update tools/mod.rs
2. **Evaluate and simplify dispatch** (1 task): Check if tool_box! provides dispatch, or add a macro
3. **Verification**: `cargo test --workspace` + manual MCP tool test
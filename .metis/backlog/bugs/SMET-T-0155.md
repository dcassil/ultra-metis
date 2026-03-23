---
id: fix-cargo-test-hang-shared-tool
level: task
title: "Fix cargo test hang: shared_tool_surface_smoke_test spawns MCP servers with no overall timeout"
short_code: "SMET-T-0155"
created_at: 2026-03-23T17:11:08.509942+00:00
updated_at: 2026-03-23T17:23:42.227415+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Fix cargo test hang: shared_tool_surface_smoke_test spawns MCP servers with no overall timeout

## Objective

Fix the `shared_tool_surface_smoke_test` in `benchmarks/practical/src/mcp_adapter.rs` which spawns real MCP server processes and can hang indefinitely, blocking `cargo test` for the entire workspace.

## Bug Details

### Type
- [x] Bug

### Priority
- [x] P1 - High (blocks all workspace test runs)

### Root Cause

`benchmarks/practical/src/mcp_adapter.rs:334` — `shared_tool_surface_smoke_test`:

1. **Spawns real processes**: Calls `adapter.start()` which runs `Command::new("metis")` and `Command::new("cadre-mcp")` as child processes with piped stdin/stdout
2. **No overall timeout on `send_request`**: The `send_request()` method (line 188-196) has an infinite `loop` that reads responses until it finds a matching ID. The per-line `read_next_json_line` has a 15s timeout, but if the server keeps sending non-matching output, the loop never terminates.
3. **Binaries may not exist**: If `metis` or `cadre-mcp` aren't on PATH, the process may spawn but never produce valid JSON-RPC output, causing the read loop to block.
4. **Resource contention**: When running alongside 850+ other tests in `cargo test`, spawning server processes competes for resources.

### Key File
`benchmarks/practical/src/mcp_adapter.rs`
- `MCP_RESPONSE_TIMEOUT`: 15s per read (line 10)
- `send_request()`: infinite loop at line 188-196
- `start()`: spawns child process at line 82-95

### Reproduction
Run `cargo test` for the full workspace. The test hangs during the benchmark crate's test execution.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `send_request()` has an overall timeout (not just per-line)
- [ ] Test is marked `#[ignore]` or gated behind a feature flag so `cargo test` doesn't spawn servers by default
- [ ] `cargo test` for the full workspace completes without hanging
- [ ] The smoke test can still be run explicitly (e.g., `cargo test --ignored` or `cargo test --features smoke-test`)

## Implementation Notes

### Recommended Fix
1. Add `#[ignore]` attribute to `shared_tool_surface_smoke_test` so it doesn't run during normal `cargo test`
2. Add an overall timeout to `send_request()` (e.g., 30s max total, not just per-line)
3. Consider using `tokio::time::timeout` or a simple elapsed check in the loop

## Status Updates

*Discovered 2026-03-23 during parallel agent execution — subagents kept hanging on cargo test runs.*

### 2026-03-23 — Fixed
- Added `MCP_REQUEST_OVERALL_TIMEOUT` (30s) constant
- Added `Instant`-based deadline check in `send_request()` loop — now errors instead of hanging forever
- Added `#[ignore]` to `shared_tool_surface_smoke_test` so it doesn't run during normal `cargo test`
- Test can still be run explicitly: `cargo test --ignored -p practical-benchmark`
- Verified: 80 passed, 1 ignored, 0 failed in 0.03s
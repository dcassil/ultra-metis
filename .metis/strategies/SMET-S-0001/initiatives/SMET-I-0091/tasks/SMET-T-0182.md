---
id: cadre-mcp-refactor-functions
level: task
title: "cadre-mcp: Refactor Functions Exceeding Structural Clippy Thresholds"
short_code: "SMET-T-0182"
created_at: 2026-03-26T19:20:10.186584+00:00
updated_at: 2026-03-26T20:50:00.789218+00:00
parent: SMET-I-0091
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0091
---

# cadre-mcp: Refactor Functions Exceeding Structural Clippy Thresholds

## Parent Initiative

[[SMET-I-0091]]

## Objective

Refactor all functions in the `cadre-mcp` crate that violate the structural clippy thresholds: `too_many_lines` (80), `too_many_arguments` (7), and `cognitive_complexity` (25). This crate has 2 violations total.

## Violations (2 total)

### too_many_lines (2 violations)

| File | Line | Function | Lines |
|------|------|----------|-------|
| `src/server.rs` | 31 | `handle_call_tool` (via `#[async_trait]`) | 162/80 |
| `src/lib.rs` | 31 | `run` | 101/80 |

### too_many_arguments (0 violations)

None.

### cognitive_complexity (0 violations)

None.

## Acceptance Criteria

## Acceptance Criteria

- [ ] `cargo clippy -p cadre-mcp --all-targets -- -D clippy::too_many_lines -D clippy::too_many_arguments -D clippy::cognitive_complexity` passes with zero warnings
- [ ] All existing tests pass (`cargo test -p cadre-mcp`)
- [ ] No public API changes

## Implementation Notes

### Technical Approach

- **server.rs `handle_call_tool`** (162 lines): This is the main MCP tool dispatch function. Extract each tool handler into its own private function, keeping the match arms as thin dispatchers.
- **lib.rs `run`** (101 lines): Extract setup/configuration steps into helper functions.

### Dependencies
Depends on cadre-core and cadre-store. No blocking dependency on other tasks in this initiative.

## Status Updates

*To be added during implementation*
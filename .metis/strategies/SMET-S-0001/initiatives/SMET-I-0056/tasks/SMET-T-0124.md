---
id: add-reassign-command-mirroring-mcp
level: task
title: "Add reassign Command Mirroring MCP Tool"
short_code: "SMET-T-0124"
created_at: 2026-03-18T04:31:32.850586+00:00
updated_at: 2026-03-18T04:38:44.255987+00:00
parent: SMET-I-0056
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0056
---

# Add reassign Command Mirroring MCP Tool

## Parent Initiative

[[SMET-I-0056]] - CLI Architecture: Add Missing Commands and Parameter Parity

## Objective

Add a `reassign` CLI command that mirrors the MCP `reassign_parent` tool added in SMET-I-0055. Allows moving tasks between initiatives or to/from the backlog from the command line.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `cadre reassign <short-code> --parent <id>` moves task to new parent
- [ ] `cadre reassign <short-code> --backlog <category>` moves task to backlog
- [ ] Validates task type, parent phase, and parent type (same rules as MCP)
- [ ] Prints success message with old/new parent
- [ ] Error messages use user_message() for consistency
- [ ] Rejects if neither --parent nor --backlog is provided

## Implementation Notes

### Technical Approach
1. Add `Reassign` variant to CLI Commands enum with args: short_code, --parent, --backlog
2. Call `store.reassign_parent(short_code, parent, backlog_category)`
3. Print result string from store method
4. Simple — just wiring the existing store method to a CLI command

### Key Files
- `crates/cadre-cli/src/main.rs` — add Reassign command

## Status Updates

*To be added during implementation*
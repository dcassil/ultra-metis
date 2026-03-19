---
id: implement-reassign-parent-tool
level: task
title: "Implement reassign_parent Tool"
short_code: "SMET-T-0117"
created_at: 2026-03-18T04:10:22.090513+00:00
updated_at: 2026-03-18T04:17:59.279368+00:00
parent: SMET-I-0055
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0055
---

# Implement reassign_parent Tool

## Parent Initiative

[[SMET-I-0055]] - Tool Integration: Add Missing Tools and Parameter Support

## Objective

Implement a new `reassign_parent` MCP tool that moves tasks between initiatives or to/from the backlog. This is the critical missing tool that blocks task reorganization workflows.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `reassign_parent` method added to DocumentStore in `crates/ultra-metis-store/src/store.rs`
- [ ] Tool validates only tasks can be reassigned (rejects initiatives/visions/ADRs)
- [ ] Can move a task from one initiative to another (validates target initiative is in decompose/active phase)
- [ ] Can move a task to backlog with required category (bug/feature/tech-debt)
- [ ] Can move a backlog task into an initiative
- [ ] Updates parent_id in task YAML frontmatter
- [ ] Moves task file from old parent directory to new parent directory
- [ ] Detects and reports file path conflicts at destination
- [ ] Tool registered in MCP tool definitions (`crates/ultra-metis-mcp/src/tools.rs`)
- [ ] JSON schema includes: project_path, short_code, new_parent_id (optional), backlog_category (optional)
- [ ] Unit tests for: type validation, parent phase validation, file conflict detection, successful moves
- [ ] Integration tests for: initiative-to-initiative, initiative-to-backlog, backlog-to-initiative workflows

## Implementation Notes

### Technical Approach
1. Add `reassign_parent(&self, short_code, new_parent_id, backlog_category)` to DocumentStore
2. Load task document, verify it's a Task type (not Initiative/Vision/ADR)
3. If `new_parent_id` provided: load parent initiative, verify phase is `decompose` or `active`
4. If `new_parent_id` is None: require `backlog_category` (bug/feature/tech-debt)
5. Compute new file path under target parent directory (or backlog directory)
6. Check for file conflicts at destination path
7. Move file via `fs::rename` (or copy+delete for cross-device)
8. Update `parent` field in task YAML frontmatter
9. Return success message with old/new parent info

### Key Files
- `crates/ultra-metis-store/src/store.rs` - DocumentStore: add reassign_parent method
- `crates/ultra-metis-mcp/src/tools.rs` - Tool definition and dispatch
- `crates/ultra-metis-store/src/error.rs` - Add error variants if needed

### Dependencies
- None (uses existing DocumentStore infrastructure)

## Status Updates

*To be added during implementation*
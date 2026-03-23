---
id: add-parameter-enhancements-to-edit
level: task
title: "Add Parameter Enhancements to edit, transition, and create Tools"
short_code: "SMET-T-0119"
created_at: 2026-03-18T04:10:24.050458+00:00
updated_at: 2026-03-18T04:18:04.900179+00:00
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

# Add Parameter Enhancements to edit, transition, and create Tools

## Parent Initiative

[[SMET-I-0055]] - Tool Integration: Add Missing Tools and Parameter Support

## Objective

Add missing parameters to three existing tools: `replace_all` for edit_document, `force` for transition_phase, and `decision_maker` for create_document (ADRs). These are small, independent enhancements bundled together.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

### edit_document — replace_all
- [ ] Accepts optional `replace_all` boolean parameter (defaults to false)
- [ ] When false (default), replaces only first occurrence (current behavior)
- [ ] When true, replaces all occurrences of search text
- [ ] MCP schema updated with replace_all parameter
- [ ] Tests for: single replace (default), replace_all with multiple matches

### transition_phase — force
- [ ] Accepts optional `force` boolean parameter (defaults to false)
- [ ] When false (default), validates exit criteria before transition (current behavior)
- [ ] When true, bypasses exit criteria validation and forces the transition
- [ ] Still validates phase sequence (force cannot skip phases, only bypass criteria checks)
- [ ] MCP schema updated with force parameter
- [ ] Tests for: normal transition, forced transition bypassing criteria

### create_document — decision_maker
- [ ] Accepts optional `decision_maker` string parameter
- [ ] When provided with document_type=adr, includes decision_maker in ADR frontmatter/content
- [ ] Ignored for non-ADR document types
- [ ] MCP schema updated with decision_maker parameter
- [ ] Test for: ADR creation with decision_maker field

## Implementation Notes

### Technical Approach
1. **edit_document**: Change `replacen(search, replace, 1)` to use `replace()` when replace_all is true
2. **transition_phase**: Add force flag that skips `exit_criteria_met` check in transition logic
3. **create_document**: Pass decision_maker string into ADR template during creation
4. All changes are additive — existing tool calls without new params work identically

### Key Files
- `crates/cadre-store/src/store.rs` - edit_document, transition_phase, create_document methods
- `crates/cadre-mcp/src/tools.rs` - Tool schemas and dispatch

### Dependencies
- None

## Status Updates

*To be added during implementation*
---
id: enhance-tool-output-formatting
level: task
title: "Enhance Tool Output Formatting"
short_code: "SMET-T-0121"
created_at: 2026-03-18T04:10:25.763841+00:00
updated_at: 2026-03-18T04:23:00.895561+00:00
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

# Enhance Tool Output Formatting

## Parent Initiative

[[SMET-I-0055]] - Tool Integration: Add Missing Tools and Parameter Support

## Objective

Upgrade all MCP tool output from minimal strings to rich markdown formatting with tables, diff visualizations, and contextual error messages. This improves UX and audit trail visibility.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `list_documents` returns markdown table format (Code | Title | Type | Phase | Parent)
- [ ] `create_document` returns formatted summary table (already partially done — verify and enhance)
- [ ] `edit_document` shows diff visualization: matched text with before/after context
- [ ] `search_documents` returns results with context snippets (surrounding text around match)
- [ ] `transition_phase` returns formatted progress bar (already partially done — verify and enhance)
- [ ] Error messages include: error type, message, actionable suggestion for next steps
- [ ] `archive_document` shows tree of archived documents
- [ ] Output formatting is consistent across all tools (shared formatting helpers)
- [ ] Unit tests for formatting helpers (table generation, diff rendering)
- [ ] All existing tool functionality unchanged (formatting only)

## Implementation Notes

### Technical Approach
1. Create output formatting module with helper functions:
   - `format_table(headers, rows)` — generates markdown table
   - `format_diff(old_text, new_text)` — generates before/after diff
   - `format_error(error_type, message, suggestion)` — structured error output
   - `format_snippet(text, match_start, match_end, context_lines)` — search context
2. Update each tool's return value to use formatting helpers
3. Keep raw data available (formatting is presentation layer only)

### Key Files
- `crates/cadre-mcp/src/tools.rs` - Tool output generation
- `crates/cadre-mcp/src/format.rs` - New formatting module (or inline in tools.rs)

### Dependencies
- Depends on all other tasks completing first (format new tools too)
- Can be started in parallel but finalized last

## Status Updates

*To be added during implementation*
---
id: liveoutput-claude-json-event
level: task
title: "LiveOutput Claude JSON Event Parsing and Semantic Rendering"
short_code: "SMET-T-0285"
created_at: 2026-03-29T01:29:12.233425+00:00
updated_at: 2026-03-29T02:19:00.180903+00:00
parent: SMET-I-0102
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0102
---

# LiveOutput Claude JSON Event Parsing and Semantic Rendering

## Parent Initiative

[[SMET-I-0102]]

## Objective

Replace raw JSON string rendering in the LiveOutput component with semantic parsing of Claude CLI JSON event types. Currently, all non-result JSON events display as raw JSON strings. After this task, assistant messages show extracted text, tool uses show tool name and summary, and unknown types fall back to formatted display.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Assistant message events (`type: "assistant"`) render extracted text content, not raw JSON
- [ ] Tool use events (`type: "tool_use"`) render tool name and a brief input summary
- [ ] Tool result events (`type: "tool_result"`) render with appropriate styling
- [ ] System events (`type: "system"`) render as system messages
- [ ] Result events (`type: "result"`) continue to render as existing card (no regression)
- [ ] Unknown/unparseable content falls back to plain text rendering (current behavior)
- [ ] No regressions in existing event rendering (guidance_injected, suppressed lines, etc.)

## Implementation Notes

### Technical Approach

In `apps/control-dashboard/src/components/LiveOutput.tsx`:

1. Add a `parseClaudeEvent` function that attempts to parse `evt.content` as Claude CLI JSON
2. Define TypeScript interfaces for Claude CLI event types (assistant, tool_use, tool_result, system, result)
3. Add rendering branches for each event type with appropriate styling:
   - Assistant: speech-bubble style with extracted text, agent icon
   - Tool use: compact line with tool name badge and truncated input
   - Tool result: indented result block
   - System: italic system message style
4. Keep `parseClaudeResult` as-is for the result card rendering
5. Fallback: if content doesn't parse as any known Claude event, render as plain text (current behavior)

### Key File
- `apps/control-dashboard/src/components/LiveOutput.tsx`

## Status Updates

*To be added during implementation*
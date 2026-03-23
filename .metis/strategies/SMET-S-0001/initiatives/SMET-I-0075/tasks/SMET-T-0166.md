---
id: add-stop-hook-to-enforce-todowrite
level: task
title: "Add Stop hook to enforce TodoWrite replacement with Cadre document updates"
short_code: "SMET-T-0166"
created_at: 2026-03-23T20:50:33.921919+00:00
updated_at: 2026-03-23T20:59:47.186683+00:00
parent: SMET-I-0075
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0075
---

# Add Stop hook to enforce TodoWrite replacement with Cadre document updates

## Parent Initiative
[[SMET-I-0075]]

## Objective
Add a `Stop` hook (specifically a `SubagentStop` hook) that detects when a subagent used TodoWrite and provides feedback telling it to use Cadre document updates instead. Also add a `PreToolUse` hook on `TodoWrite` that blocks the tool with an explanation of the alternative.

## Scope
- Add `PreToolUse` matcher for `TodoWrite` tool in hooks.json — prompt-based hook that returns `{"decision": "block", "reason": "..."}` explaining to use Cadre MCP tools instead
- This is a prompt-based hook (type: "prompt"), not a command hook — it uses the Claude model to evaluate and block
- Alternatively: a simpler command-based hook that always blocks TodoWrite with a static message

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [ ] TodoWrite tool is blocked in Cadre projects with a clear explanation
- [ ] Block message tells the agent to use `mcp__metis__edit_document` or `mcp__metis__create_document` instead
- [ ] Hook only activates in Cadre projects (has `.metis` directory)
- [ ] Non-Cadre projects are unaffected

## Implementation Notes
- PreToolUse command hooks receive tool input on stdin and can return `{"decision": "block", "reason": "Use Cadre MCP tools instead"}`
- Simpler than a Stop hook — blocks TodoWrite at the source rather than after the fact
- The session-start and subagent-start hooks already tell agents not to use TodoWrite; this enforces it

## Status Updates
---
id: implement-subagentstart-hook-for
level: task
title: "Implement SubagentStart hook for Cadre project context injection"
short_code: "SMET-T-0164"
created_at: 2026-03-23T20:50:31.403063+00:00
updated_at: 2026-03-23T20:56:55.250946+00:00
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

# Implement SubagentStart hook for Cadre project context injection

## Parent Initiative
[[SMET-I-0075]]

## Objective
Create a `SubagentStart` hook script and register it in hooks.json. When any subagent spawns in a Cadre project, the hook injects concise context (~500 tokens) telling the subagent about: the Cadre MCP tools available, the active task it should update, the no-TodoWrite rule, and how to record progress.

## Scope
- Create `plugins/cadre/hooks/subagent-start-hook.sh`
- Register `SubagentStart` event in `plugins/cadre/hooks/hooks.json`
- Hook detects `.metis` dir, reads active work items via `cadre status`, injects compact context
- Context must be concise (subagents have limited context) — no tutorial material, just tool names + rules + active task
- Hook reads stdin JSON for `agent_name` and `task_description` to tailor context

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [ ] SubagentStart hook registered in hooks.json
- [ ] Hook script detects Cadre project and injects context
- [ ] Context includes: MCP tool names, no-TodoWrite rule, active work items
- [ ] Context is under 500 tokens
- [ ] Hook exits silently (no output) for non-Cadre projects
- [ ] Hook handles missing `cadre` binary gracefully

## Implementation Notes
- Follow the same pattern as session-start-hook.sh but much more concise
- SubagentStart hook output format: `{"hookSpecificOutput": {"hookEventName": "SubagentStart", "additionalSystemPrompt": "..."}}`
- The `additionalSystemPrompt` field is what gets injected into the subagent

## Status Updates
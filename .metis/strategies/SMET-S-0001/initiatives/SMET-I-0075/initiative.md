---
id: subagent-awareness-subagentstart
level: initiative
title: "Subagent Awareness: SubagentStart Hook, Vendored Superpowers, and TodoWrite Replacement"
short_code: "SMET-I-0075"
created_at: 2026-03-23T17:28:02.729048+00:00
updated_at: 2026-03-23T20:59:55.486323+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0001
initiative_id: subagent-awareness-subagentstart
---

# Subagent Awareness: SubagentStart Hook, Vendored Superpowers, and TodoWrite Replacement Initiative

## Context

When Claude spawns subagents via the Agent tool in a Cadre project, those subagents have zero awareness of the work management system. They don't know about Cadre documents, MCP tools, phase lifecycles, or workflow rules. This was demonstrated when parallel agents were dispatched for SMET-I-0068 work — neither agent used Cadre tools, updated documents, or followed the prescribed workflow.

This is Phase 1 of the Cadre Execution Architecture (ADR SMET-A-0001).

## Goals & Non-Goals

**Goals:**
- Implement SubagentStart hook that injects Cadre project context into every spawned subagent
- Vendor the entire superpowers plugin (all 50+ files, v5.0.5) into vendor/superpowers/ with fallback resolution
- Ensure all orchestration flows use Cadre documents instead of TodoWrite

**Non-Goals:**
- Orchestrated multi-task execution (SMET-I-0076)
- Parallel dispatch or worktree isolation (SMET-I-0077)
- Two-stage review (SMET-I-0076)

## Detailed Design

### SubagentStart Hook
- Register `SubagentStart` event in hooks.json
- Hook checks for `.cadre` directory, injects: project awareness, active work items, MCP tool names, no-TodoWrite rule
- Context kept under 500 tokens to preserve subagent context window

### Vendored Superpowers
- Copy full superpowers v5.0.5 to vendor/superpowers/
- VERSION file tracks pinned version
- Setup scripts try installed plugin first, fall back to vendor
- Updated deliberately, not automatically

### TodoWrite Replacement
- All orchestration flows use Cadre document updates instead of TodoWrite
- SubagentStart hook explicitly tells subagents not to use TodoWrite

## Alternatives Considered

1. **Vendor only 7 mapped skills**: Rejected — full plugin is small, selective vendoring adds maintenance
2. **Reuse SessionStart hook for subagents**: Rejected — subagents need concise context, not tutorial material
3. **Skip vendoring, just document dependency**: Rejected — hard dependency makes Cadre fragile

## Implementation Plan

1. SubagentStart hook script and registration
2. Vendor superpowers plugin with fallback resolution
3. TodoWrite elimination in orchestration flows

## Dependencies

- **Blocked by**: SMET-I-0074 (rename)
- **Blocks**: SMET-I-0076 (orchestrated execution)
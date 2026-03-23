---
id: orchestrated-execution-sdd-style
level: initiative
title: "Orchestrated Execution: SDD-Style Subagent Dispatch with Review Modes and A/B Testing"
short_code: "SMET-I-0076"
created_at: 2026-03-23T17:28:03.669888+00:00
updated_at: 2026-03-23T21:14:13.254811+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
strategy_id: SMET-S-0001
initiative_id: orchestrated-execution-sdd-style
---

# Orchestrated Execution: Fresh-Subagent Dispatch for Multi-Task Initiatives

## Context

The current `/cadre-ralph-epic` runs all tasks in a single session — context pollutes over time. ADR SMET-A-0001 identified this as a key problem. Superpowers' SDD pattern (fresh subagent per task + review) solves it, but SDD uses TodoWrite for tracking. We want SDD's execution quality with Cadre's persistent document tracking.

**Slimmed scope**: Rather than building a new `/cadre-execute` command, enhance the existing `/cadre-ralph-epic` (now `/cadre-ralph-initiative`) to dispatch fresh subagents per task. Each subagent gets the task's Cadre doc content, mapped superpowers skills, and updates the Cadre doc with progress. An optional review subagent runs between tasks.

## Goals & Non-Goals

**Goals:**
- Enhance `/cadre-ralph-epic` → `/cadre-ralph-initiative` to dispatch fresh Agent per task
- Each subagent gets: task doc content, mapped superpowers skills, Cadre MCP tool instructions
- Subagents invoke superpowers skills (TDD, debugging, etc.) as part of execution
- Optional code review subagent between tasks via `--review` flag
- Cadre documents remain source of truth (not TodoWrite)
- Existing story-type-to-skill mapping carried forward from setup-cadre-ralph.sh

**Non-Goals:**
- New command (reuse existing command structure)
- Parallel execution (SMET-I-0077)
- Git worktrees (SMET-I-0077)
- A/B testing infrastructure (unnecessary complexity)
- Model selection (let Claude choose for now)
- Spec reviewer separate from code reviewer (one review pass is enough for MVP)

## Detailed Design

### Orchestrator Flow (cadre-ralph-initiative command)
1. Read initiative doc, list all child tasks
2. Filter to tasks in `todo` phase
3. For each task:
   a. Read task doc content
   b. Determine story type → map to superpowers skills
   c. Build agent prompt with: task content + skill instructions + Cadre MCP tool instructions
   d. Dispatch fresh Agent (via Agent tool) with that prompt
   e. Agent executes, invokes superpowers skills, updates Cadre doc
   f. If `--review`: dispatch code-review Agent to check the work
   g. Transition task to completed
   h. Update initiative doc with progress
4. When all tasks done, summarize and signal completion

### Subagent Context (per task)
- Full task document content (objective, acceptance criteria, implementation notes)
- Mapped superpowers skills: "Invoke skill: superpowers:test-driven-development" etc.
- Cadre MCP tool instructions: read/edit/transition
- SubagentStart hook automatically adds project awareness on top

### Review (optional, --review flag)
- Single code-review Agent dispatched after each task
- Uses `superpowers:requesting-code-review` pattern
- Reviews the git diff from the task's work
- Reports issues → orchestrator decides whether to re-dispatch or escalate

## Alternatives Considered

1. **Build new `/cadre-execute` command**: Rejected — unnecessary when enhancing existing command achieves the same thing
2. **Use SDD directly**: Rejected — uses TodoWrite, doesn't know about Cadre docs
3. **Keep single-session ralph loop for multi-task**: Rejected — context pollution is a real problem

## Implementation Plan

1. Rename command: cadre-ralph-epic → cadre-ralph-initiative
2. Build new setup script that dispatches Agent tool per task instead of inline ralph loop
3. Add --review flag for optional inter-task code review
4. Update command markdown with new dispatch instructions

## Dependencies

- **Blocked by**: SMET-I-0075 (completed)
- **Blocks**: SMET-I-0077
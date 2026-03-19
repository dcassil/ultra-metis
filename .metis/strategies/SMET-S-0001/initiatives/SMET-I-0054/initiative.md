---
id: ralph-loop-implement-autonomous
level: initiative
title: "Ralph Loop: Implement Autonomous Task Execution Framework"
short_code: "SMET-I-0054"
created_at: 2026-03-17T22:42:04.016037+00:00
updated_at: 2026-03-17T22:42:04.016037+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: SMET-S-0001
initiative_id: ralph-loop-implement-autonomous
---

# Ralph Loop: Autonomous Task Execution via Plugin Skills

## Strategy Update (2026-03-18)

**Revised approach**: Instead of building a full Rust execution engine in ultra-metis-core, we use the existing ralph-loop plugin pattern and port it as ultra-metis plugin skills/commands. Ultra-metis provides the document state and MCP tools; the plugin provides the execution workflow.

**Key decisions:**
- No new Rust crate or RalphLoopEngine — execution is plugin-level, not core-level
- Port `/metis-ralph` and `/metis-ralph-initiative` as ultra-metis plugin skills in `.claude-plugin/skills/`
- Skills use ultra-metis MCP tools (`read_document`, `edit_document`, `transition_phase`, `list_documents`) for all state management
- Iteration tracking persisted via `edit_document` updates to task progress sections (no new frontmatter schema needed)
- Leverage the existing ralph-loop plugin's patterns for iteration counting, promise signals, and max-iteration bounds
- Cancel/resume behavior handled at the plugin skill level

## Context

The original Metis ralph-loop plugin provides autonomous task execution through plugin skills that wrap MCP document operations. Ultra-Metis needs equivalent execution commands that use ultra-metis MCP tools instead of metis tools.

The ralph-loop plugin pattern has proven effective: skills provide the execution orchestration, MCP tools provide the state operations, and document progress sections serve as persistent working memory. No custom Rust engine is needed.

## Goals & Non-Goals

**Goals:**
- Create `/ultra-metis-ralph <task-short-code>` plugin skill for single-task autonomous execution
- Create `/ultra-metis-ralph-initiative <initiative-short-code>` plugin skill for multi-task orchestration
- Port iteration tracking, promise mechanism, and max-iteration safety bounds from ralph-loop plugin
- Use ultra-metis MCP tools for all document state operations
- Track execution progress in task document progress sections via `edit_document`
- Support cancel flow via `/cancel-ultra-metis-ralph` skill

**Non-Goals:**
- Building a Rust execution engine (RalphLoopEngine) — use plugin skills instead
- Adding new frontmatter schema (loop_metadata) — use document content sections
- Custom state file format — document progress sections are sufficient
- Replacing the existing ralph-loop plugin — this is an ultra-metis-specific equivalent

## Detailed Design

### Plugin Skill Architecture
```
.claude-plugin/
  skills/
    ultra-metis-ralph.md          # Single-task execution skill
    ultra-metis-ralph-initiative.md  # Initiative-level multi-task skill
    cancel-ultra-metis-ralph.md   # Cancel active loop
```

### Single-Task Skill (`/ultra-metis-ralph`)
1. Read task via `read_document` MCP tool
2. Transition task to active via `transition_phase`
3. Execute task work, updating progress via `edit_document` after each step
4. Track iteration count in progress section
5. Check max-iterations (configurable, default 4-5) before each iteration
6. Emit `<promise>` before final completion transition
7. Transition to completed via `transition_phase`

### Initiative-Level Skill (`/ultra-metis-ralph-initiative`)
1. Read initiative and list child tasks via `list_documents`
2. For each todo task in order, invoke single-task execution pattern
3. Track overall progress (N/total) in initiative document via `edit_document`
4. On max-iterations breach in any task, escalate with context
5. Update initiative on completion

### What to Port from ralph-loop Plugin
- Iteration counting and max-iteration enforcement logic
- Promise signal pattern before completion
- Cancel mechanism (skill checks for cancel signal)
- Progress update frequency and format
- Error escalation message patterns

## Implementation Plan

1. Study existing ralph-loop plugin skill files for patterns and structure
2. Create `/ultra-metis-ralph` skill with ultra-metis MCP tool references
3. Create `/ultra-metis-ralph-initiative` skill with task discovery and sequential execution
4. Create `/cancel-ultra-metis-ralph` skill
5. Test execution flow end-to-end with a real task

## Exit Criteria
- All three skills functional in `.claude-plugin/skills/`
- Single-task execution works: todo → active → completed with progress tracking
- Initiative-level execution works: discovers tasks, executes sequentially
- Cancel flow works mid-execution
- Iteration tracking and max-iteration safety bounds enforced
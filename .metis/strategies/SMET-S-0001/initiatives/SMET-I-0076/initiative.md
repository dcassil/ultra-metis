---
id: orchestrated-execution-sdd-style
level: initiative
title: "Orchestrated Execution: SDD-Style Subagent Dispatch with Review Modes and A/B Testing"
short_code: "SMET-I-0076"
created_at: 2026-03-23T17:28:03.669888+00:00
updated_at: 2026-03-23T17:28:03.669888+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: SMET-S-0001
initiative_id: orchestrated-execution-sdd-style
---

# Orchestrated Execution: SDD-Style Subagent Dispatch with Review Modes and A/B Testing Initiative

## Context

The current execution model runs a single ralph loop sequentially through all stories. ADR SMET-A-0001 identified three problems: context pollution, no review gates, and no model selection. This initiative implements Phase 2: `/cadre-execute` with SDD-style fresh-subagent-per-task dispatch.

## Goals & Non-Goals

**Goals:**
- Build `/cadre-execute` command dispatching fresh subagent per task with curated context
- Two-stage review (spec compliance + code quality) via `--review-mode full|light|none`
- Deterministic story-type-to-skill mapping
- Model selection guidance (cheap for mechanical, capable for judgment)
- A/B testing infrastructure: ralph loop vs cadre-execute
- Cadre document updates after each task

**Non-Goals:**
- Parallel execution (SMET-I-0077)
- Git worktrees (SMET-I-0077)
- Task claiming (SMET-I-0077)
- Replacing ralph loop entirely (coexists for single-task work)

## Detailed Design

### Agent Roles
- **Orchestrator**: Reads initiative, curates context, dispatches agents, manages flow
- **Implementer**: Fresh subagent per task with curated context and mapped skills
- **Spec Reviewer**: Validates against acceptance criteria, distrusts implementer
- **Quality Reviewer**: Checks code quality, patterns, coverage

### Review Modes
- `full` (default): both reviewers after every task
- `light`: quality review only
- `none`: no inter-task review

### Story-Type-to-Skill Mapping
- feature → brainstorming → writing-plans → TDD → verification
- bugfix → systematic-debugging → verification
- refactor → writing-plans → verification
- investigation → brainstorming
- (and others per ADR table)

### Model Selection
- Mechanical (1-2 files, clear spec): cheap model
- Judgment (multi-file, design decisions): capable model
- Debugging (unknown scope): capable model

### Escalation Policy
Review failure escalates to user — no auto-retry.

## Alternatives Considered

1. **Add review to existing ralph loop**: Doesn't solve context pollution
2. **Full parallel from start**: Conflates independent capabilities
3. **LLM-as-judge scoring**: Less actionable than structured reviewer subagents

## Implementation Plan

1. Core command and dispatch loop
2. Review subagents and modes
3. Model selection and skill mapping
4. A/B testing infrastructure

## Dependencies

- **Blocked by**: SMET-I-0075
- **Blocks**: SMET-I-0077
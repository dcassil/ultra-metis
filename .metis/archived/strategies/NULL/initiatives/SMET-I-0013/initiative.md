---
id: add-orchestrator-and-runner
level: initiative
title: "Add Orchestrator and Runner Execution Support"
short_code: "SMET-I-0013"
created_at: 2026-03-11T20:00:07.600996+00:00
updated_at: 2026-03-11T20:00:07.600996+00:00
parent: SMET-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: XL
strategy_id: NULL
initiative_id: add-orchestrator-and-runner
---

# Add Orchestrator and Runner Execution Support

## Context

Super-Metis should support two execution modes: single-agent mode (one agent works on one story at a time) and orchestrated mode (a coordinator agent assigns stories to multiple worker agents running in parallel). Metis currently has a "Ralph" loop concept for single-task execution, but no multi-agent orchestration.

The orchestrator is the component that reads the planning hierarchy, identifies ready work, acquires leases, dispatches work to runner agents, monitors progress, and handles completion/failure. This is the "execution engine" of Super-Metis.

## Governing Commitments

This initiative directly serves:
- **Single-agent and orchestrated modes share one governance model.** Both the single-agent runner and the multi-agent orchestrator operate on the same planning hierarchy, the same lease mechanism, the same quality gates, and the same rule checks. Governance semantics are consistent regardless of execution scale.
- **Controlled parallel execution** (Vision #8). The orchestrator enables safe parallel work through lease-based ownership and isolated worktrees. Coordination is structural, not informal.
- **Evidence-based workflow progression** (Vision #7). Runners verify quality gates before completing stories. Context assembly ensures agents work with the right design references, rules, and quality requirements — not guesswork.
- **All durable project memory lives in the repo.** Execution logs, orchestration decisions, and outcomes are persisted artifacts. The history of what was dispatched, what succeeded, and what failed is part of the repo's durable record.
- **The system is built around intentional, durable structure.** The orchestrator reads the planning hierarchy to select work — it doesn't improvise priorities. Execution follows the structure that was planned.

## Goals & Non-Goals

**Goals:**
- Build a single-agent runner mode that executes one Story at a time with full context from the planning hierarchy
- Build an orchestrator that can coordinate multiple runner agents working in parallel
- Implement work selection logic: pick the highest-priority ready Story
- Implement dispatch: assign a Story to a runner agent with proper context (design refs, rules, quality requirements)
- Implement progress monitoring: track runner status, detect stalls, handle failures
- Support configurable parallelism (how many runners at once)

**Non-Goals:**
- Distributed orchestration across multiple machines — this is single-machine, local execution
- Building a general-purpose task scheduler — this is specifically for Super-Metis story execution
- Replacing the existing Metis Ralph loop — extend and evolve it

## Detailed Design

### What to Reuse from `metis/`
- The existing Ralph loop concept as the foundation for single-agent runner mode
- MCP tool access patterns (runners interact with Super-Metis through MCP)
- Document reading and context gathering patterns
- Phase transition commands

### What to Change from `metis/`
- Evolve the Ralph loop to work with Stories instead of Tasks
- Add context enrichment: before executing, gather design refs, rules, quality requirements
- Add pre-execution quality gate checks
- Add post-execution quality verification

### What is Net New
- Orchestrator component: reads planning state, selects work, dispatches to runners
- Runner protocol: how the orchestrator communicates with runner agents
- Work selection algorithm: priority-based selection from ready Stories
- Context assembly: gather all relevant context (design, rules, quality) for a Story before execution
- Progress monitoring: periodic check on runner status with timeout handling
- Failure handling: what happens when a runner fails (release lease, mark blocked, retry?)
- Completion handling: verify quality, release lease, transition Story to completed
- Configurable parallelism: control how many runners execute simultaneously
- Execution log: persistent record of orchestration decisions and outcomes

## Alternatives Considered

1. **Use external orchestration (Kubernetes, etc.)**: Rejected — must be repo-native and local.
2. **Agents self-select work without orchestrator**: Rejected because it leads to race conditions and poor prioritization.
3. **Human dispatches all work manually**: Supported as a mode, but orchestrated mode adds significant value.
4. **Build on Claude Code's existing agent/worktree pattern**: Yes — leverage this directly where possible.

## Implementation Plan

Phase 1: Evolve Ralph loop into single-agent runner with context enrichment
Phase 2: Build work selection algorithm
Phase 3: Build context assembly (design refs, rules, quality requirements)
Phase 4: Build orchestrator dispatch logic
Phase 5: Implement runner protocol (orchestrator ↔ runner communication)
Phase 6: Implement progress monitoring and failure handling
Phase 7: Implement completion verification (quality gate check)
Phase 8: Add configurable parallelism
Phase 9: Build execution log

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- Single-agent runner can execute a Story end-to-end with proper context
- Orchestrator can select, dispatch, and monitor multiple Stories in parallel
- Context assembly includes design references, applicable rules, and quality requirements
- Failed runners release their leases and Stories are marked appropriately
- Completed Stories pass quality gate verification before being marked complete
- Execution log records all orchestration decisions and outcomes
- Parallelism is configurable and respects system resource limits
- The system degrades gracefully when runners fail or stall

## Risks / Dependencies

- Depends on SMET-I-0012 for work leasing
- Depends on SMET-I-0005 for quality gate checks
- Depends on SMET-I-0004 for rule queries
- Depends on SMET-I-0003 for design context access
- Depends on SMET-I-0002 for the Story lifecycle
- This is the most complex initiative — scope management is critical
- Runner agent reliability is unpredictable — need robust failure handling
- Must coordinate with SMET-I-0009 for MCP tools the orchestrator uses

## Codebase Areas to Inspect

- `metis/src/ralph/` or equivalent — existing Ralph loop implementation
- `metis/src/mcp/` — MCP tool patterns for runner interaction
- `metis/src/commands/` — command patterns for orchestration commands
- Any existing agent/subprocess management code

## Suggested Tasks for Decomposition

1. Audit existing Ralph loop and document its execution model
2. Evolve Ralph into single-agent Story runner with context enrichment
3. Build work selection algorithm (priority-based)
4. Build context assembly module (design, rules, quality)
5. Design orchestrator ↔ runner protocol
6. Build orchestrator dispatch logic
7. Implement progress monitoring with timeout detection
8. Implement failure handling (lease release, Story blocking)
9. Implement completion verification (quality gate check)
10. Add configurable parallelism controls
11. Build persistent execution log
12. Integration test full orchestration cycle
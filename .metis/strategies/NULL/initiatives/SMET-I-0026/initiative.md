---
id: multi-agent-orchestrator
level: initiative
title: "Multi-Agent Orchestrator"
short_code: "SMET-I-0026"
created_at: 2026-03-11T21:52:30.754481+00:00
updated_at: 2026-03-11T21:52:30.754481+00:00
parent: SMET-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: multi-agent-orchestrator
---

# Multi-Agent Orchestrator

> **STATUS: POST-MVP / DEFERRED** — The product spec explicitly defers full multi-agent orchestration to later phases, unless existing tools already cover it well enough. Multi-agent orchestration should come after the repo-native state layer is mature. When implemented, it should add work decomposition, scoped subtask dispatch, context packaging, conflict detection, execution log merge, completion verification, and durable traceability.

## Context

The single-agent runner (SMET-I-0025) executes one Story at a time. The orchestrator coordinates multiple runners working in parallel — it reads the planning hierarchy, identifies ready work, acquires leases, dispatches work to runners, monitors progress, and handles completion/failure.

This is the coordination layer that makes parallel AI-driven execution possible. It builds on leasing (SMET-I-0023), worktrees (SMET-I-0024), and the single-agent runner (SMET-I-0025).

Split from the original SMET-I-0013 (now archived).

## Goals & Non-Goals

**Goals:**
- Build work selection algorithm: pick highest-priority ready Stories from the planning hierarchy
- Build dispatch logic: acquire lease, set up worktree, launch runner with assembled context
- Implement progress monitoring: track runner status, detect stalls via lease expiration
- Implement failure handling: release lease, mark Story blocked, optionally retry
- Implement completion handling: verify quality, merge worktree, release lease, transition Story
- Support configurable parallelism (how many runners simultaneously)
- Build persistent execution log: record all orchestration decisions and outcomes

**Non-Goals:**
- The runner itself — covered by SMET-I-0025
- The leasing mechanism — covered by SMET-I-0023
- Git worktree management — covered by SMET-I-0024
- Distributed orchestration across machines — single-machine only
- Replacing human decision-making on priorities — orchestrator executes the plan, doesn't create it

## Detailed Design

### Work Selection
- Query planning hierarchy for Stories in "ready" phase
- Priority ordering: respect explicit priority if set, otherwise use hierarchy order (top of Epic first)
- Dependency awareness: skip Stories blocked by incomplete dependencies
- Capacity check: only select if runner slots are available

### Dispatch Protocol
- For each selected Story:
  1. Acquire lease (via SMET-I-0023)
  2. Create worktree (via SMET-I-0024, if enabled)
  3. Launch runner subprocess with context (via SMET-I-0025)
  4. Record dispatch in execution log

### Progress Monitoring
- Periodic check on runner status via lease metadata and Story document updates
- Stall detection: if Story hasn't been updated within configurable window, flag as potentially stalled
- Lease expiration serves as hard timeout — orchestrator detects expired leases and cleans up

### Failure Handling
- Runner process dies: detect via process monitoring, release lease, mark Story blocked
- Quality gate failure: runner reports failure, orchestrator decides retry or block based on config
- Stall timeout: release lease, mark Story blocked with "stall" reason

### Completion Flow
- Runner signals completion
- Orchestrator verifies quality gates passed
- Merge worktree changes back to main branch
- Release lease, transition Story to completed
- Record outcome in execution log

### Execution Log
- Persistent document tracking: dispatches, completions, failures, retries
- Per-Story: who ran it, when, duration, outcome, quality gate results
- Aggregate: throughput, failure rate, average execution time

## Alternatives Considered

1. **Agents self-select work**: Rejected — leads to race conditions and poor prioritization.
2. **External orchestration (Kubernetes, Temporal)**: Rejected — must be repo-native and local.
3. **Human dispatches all work manually**: Supported as a mode, but orchestrated mode adds significant value for parallelism.

## Implementation Plan

Phase 1: Build work selection algorithm
Phase 2: Build dispatch logic (lease + worktree + runner launch)
Phase 3: Implement progress monitoring and stall detection
Phase 4: Implement failure handling
Phase 5: Implement completion flow (quality verify + merge + release)
Phase 6: Add configurable parallelism
Phase 7: Build execution log
Phase 8: Integration test full orchestration cycle

## Acceptance Criteria

- Orchestrator can select, dispatch, and monitor multiple Stories in parallel
- Failed runners release leases and Stories are marked appropriately
- Completed Stories pass quality verification before being marked complete
- Execution log records all decisions and outcomes
- Parallelism is configurable and respects limits
- System degrades gracefully when runners fail or stall

## Risks / Dependencies

- Depends on SMET-I-0023 for leasing
- Depends on SMET-I-0024 for worktree isolation
- Depends on SMET-I-0025 for single-agent runner
- Depends on SMET-I-0022 for quality gate verification
- Runner reliability is unpredictable — robust failure handling is critical
- This is complex coordination logic — scope management matters
- Must coordinate with SMET-I-0009 for MCP tools the orchestrator exposes
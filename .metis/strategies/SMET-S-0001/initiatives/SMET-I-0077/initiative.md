---
id: parallel-execution-git-worktrees
level: initiative
title: "Parallel Execution: Git Worktrees, Multi-Story Dispatch, and Task Claiming"
short_code: "SMET-I-0077"
created_at: 2026-03-23T17:28:05.014149+00:00
updated_at: 2026-03-23T17:28:05.014149+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: SMET-S-0001
initiative_id: parallel-execution-git-worktrees
---

# Parallel Execution: Git Worktrees, Multi-Story Dispatch, and Task Claiming Initiative

## Context

SMET-I-0076 introduces SDD-style dispatch but processes tasks sequentially. When independent stories touch disjoint files, serializing them wastes time. This is Phase 3 of ADR SMET-A-0001.

## Goals & Non-Goals

**Goals:**
- Extend `/cadre-execute` to identify independent stories via file/dependency analysis
- Dispatch independent stories to parallel subagents in separate git worktrees
- Delegate worktree management to `superpowers:using-git-worktrees`
- Simple file-based task claiming via `.cadre/claims/` lock files
- Sequential fallback for dependent stories
- Full test suite after integrating parallel results

**Non-Goals:**
- Full work leasing with heartbeats (SMET-I-0023, future)
- Distributed multi-machine coordination
- Automatic conflict resolution on merge

## Detailed Design

### Dependency Analysis
- Build file touchpoint sets per story from content and code index
- Stories with disjoint file sets → independent (parallelizable)
- Overlapping files → dependent (sequential)
- Unknown touchpoints → conservative sequential fallback

### Worktree Management (Delegated)
- `superpowers:using-git-worktrees` handles creation, setup, cleanup
- Branch naming: `cadre/SMET-T-XXXX`
- `.worktrees/` directory, gitignored

### Task Claiming (MVP)
- `.cadre/claims/PROJ-T-XXXX.lock` with session_id and timestamp
- Atomic acquire via temp file + rename
- Release on completion or session end
- No heartbeats or expiration for MVP

### Integration
- Merge worktree branches back to working branch
- Conflict → flag for human resolution
- Full test suite on integrated result

## Alternatives Considered

1. **Reimplement worktrees**: Rejected — superpowers already handles this correctly
2. **Full leasing from day one**: Rejected — overkill for MVP
3. **Same-directory parallelism**: Rejected — git operations are workspace-global
4. **Container isolation**: Rejected — heavyweight, loses git integration

## Implementation Plan

1. Worktree integration (delegate to superpowers)
2. Dependency analysis
3. Task claiming
4. Parallel dispatch and integration

## Dependencies

- **Blocked by**: SMET-I-0076
- **Future**: SMET-I-0023 replaces simple claiming with full leasing
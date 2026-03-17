---
id: git-worktree-isolation-for-leased
level: initiative
title: "Git Worktree Isolation for Leased Work"
short_code: "SMET-I-0024"
created_at: 2026-03-11T21:52:28.960038+00:00
updated_at: 2026-03-11T21:52:28.960038+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: S
strategy_id: ultra-metis-core-engine-repo
initiative_id: git-worktree-isolation-for-leased
---

# Git Worktree Isolation for Leased Work

> **STATUS: POST-MVP / DEFERRED** — The product spec explicitly defers automated worktree lifecycle to later phases. For MVP, existing worktree plugins and manual worktree management suffice. This initiative is preserved for post-MVP planning.

## Context

Work leasing (SMET-I-0023) prevents conflicts at the document level. Git worktree isolation prevents conflicts at the code level — each leased Story gets its own isolated copy of the repo via `git worktree`, so multiple agents can write code simultaneously without stepping on each other.

This is an optional enhancement on top of leasing. Leasing works without worktrees (agents work in the main tree), but worktrees make parallel execution safe.

Split from the original SMET-I-0012 (now archived).

## Goals & Non-Goals

**Goals:**
- Create a git worktree when a lease is acquired (optional, configurable)
- Track worktree path in lease metadata (`lease_worktree_path` field)
- Create a feature branch per worktree (naming convention: `smet/{short-code}`)
- Clean up worktree when lease is released
- Handle worktree creation failures gracefully (lease still works, just without isolation)
- Support merging worktree changes back to the main branch on completion

**Non-Goals:**
- The leasing mechanism itself — covered by SMET-I-0023
- Solving merge conflicts — worktrees prevent them; if they occur on merge-back, that's a manual resolution
- Distributed worktrees across machines

## Detailed Design

### Worktree Lifecycle
- On lease acquire (if worktree isolation enabled): `git worktree add .smet-worktrees/{short-code} -b smet/{short-code}`
- Runner/agent executes in the worktree directory
- On lease release with completion: merge branch back to main (or create PR)
- On lease release without completion (abandoned/expired): clean up worktree, optionally preserve branch
- On cleanup: `git worktree remove .smet-worktrees/{short-code}`

### Configuration
- Project-level setting: `worktree_isolation: true/false` (default: true when orchestrator is active)
- Worktree base directory: configurable (default: `.smet-worktrees/`)
- Merge strategy on completion: `merge` (default), `pr`, or `manual`

### Error Handling
- Worktree creation failure (dirty state, disk space, etc.): log warning, proceed without isolation
- Worktree cleanup failure: log error, leave for manual cleanup, mark in lease metadata
- Branch already exists: append timestamp suffix

## Alternatives Considered

1. **Separate git clones per agent**: Rejected — too expensive in disk space and setup time.
2. **Stacked branches without worktrees**: Rejected — agents would still conflict on the working directory.
3. **Container-based isolation**: Over-engineered for single-machine use.

## Implementation Plan

Phase 1: Implement worktree creation on lease acquire
Phase 2: Implement worktree path tracking in lease metadata
Phase 3: Implement worktree cleanup on lease release
Phase 4: Implement merge-back on completion
Phase 5: Add configuration options
Phase 6: Error handling and edge cases
Phase 7: Integration tests

## Acceptance Criteria

- Worktrees are created on lease acquisition when enabled
- Each worktree has its own feature branch
- Worktrees are cleaned up on lease release
- Completed work can be merged back to main
- Worktree failures don't break the lease mechanism
- Configuration allows enabling/disabling per project

## Risks / Dependencies

- Depends on SMET-I-0023 for the lease lifecycle hooks
- Git worktree operations can fail in various ways — need robust error handling
- Disk space: many concurrent worktrees consume space — may need limits
- Must coordinate with SMET-I-0026 (orchestrator triggers worktree creation via leases)
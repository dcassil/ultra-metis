---
id: work-leasing-and-ownership
level: initiative
title: "Work Leasing and Ownership"
short_code: "SMET-I-0023"
created_at: 2026-03-11T21:52:27.616048+00:00
updated_at: 2026-03-11T21:52:27.616048+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: ultra-metis-core-engine-repo
initiative_id: work-leasing-and-ownership
---

# Work Leasing and Ownership

> **STATUS: POST-MVP / DEFERRED** — The product spec explicitly defers formal work leasing to later phases unless there is strong adoption pressure. For MVP, work coordination should leverage existing plugin capabilities. This initiative is preserved for post-MVP planning.

## Context

When multiple AI agents (or humans) work on a monorepo simultaneously, they need a way to claim exclusive ownership of work items to avoid conflicts. Current Metis has no concept of work ownership — any agent can work on any task.

This initiative introduces work leases: an agent acquires exclusive ownership of a Story or Task, works on it, and releases the lease when done. This prevents duplicate work and merge conflicts. Git worktree integration is handled separately in SMET-I-0024.

Split from the original SMET-I-0012 (now archived).

## Goals & Non-Goals

**Goals:**
- Define lease data model: lease_holder, lease_acquired_at, lease_expires_at as frontmatter fields on Stories and Tasks
- Implement atomic lease acquisition (acquire-or-fail to prevent races)
- Implement lease renewal (extend before expiration)
- Implement lease release (explicit release on completion or abandonment)
- Implement lease expiration (automatic release after configurable timeout)
- Add "leased" phase to Story lifecycle (between "ready" and "active")
- Implement lease conflict detection: prevent edits to leased documents by non-holders
- Build lease query APIs: who holds what, what's available, what's expired
- Storage: frontmatter fields + SQLite indexing for lease state
- CLI and MCP commands for lease operations

**Non-Goals:**
- Git worktree creation/cleanup — covered by SMET-I-0024
- Orchestrator logic that assigns leases — covered by SMET-I-0026
- File-level locking — leases are at document level
- Distributed consensus — single-machine, single-repo

## Detailed Design

### Lease Data Model
- Frontmatter fields on Story/Task: `lease_holder` (agent/human ID), `lease_acquired_at` (timestamp), `lease_expires_at` (timestamp), `lease_worktree_path` (optional, set by I-0024)
- SQLite index on lease fields for efficient queries

### Atomic Acquisition
- SQLite transaction: check availability + set lease holder in one atomic operation
- Returns success (lease granted) or failure (already leased) — no partial states
- Configurable default expiration (e.g., 2 hours)

### Lease Lifecycle
- Acquire → (optionally renew) → Release or Expire
- Renewal extends `lease_expires_at` without releasing
- Expiration checked on access (check-on-access pattern, no background timer needed)
- Release clears all lease fields and transitions document back to "ready" or forward to "active"

### Phase Integration
- Story phases: ... → ready → **leased** → active → completed
- A Story must be leased before it can become active
- Transition to "leased" requires a lease holder identity
- Transition from "leased" to "active" only allowed by the lease holder

### Conflict Detection
- Edit operations on leased documents check the current user against lease_holder
- Non-holders get a clear error: "Document is leased by {holder} until {expires_at}"

## Alternatives Considered

1. **Git branch conventions**: Rejected — unenforced, agents don't reliably follow conventions.
2. **Optimistic concurrency (merge on conflict)**: Rejected — AI agents handle merge conflicts poorly. Prevention > resolution.
3. **External lock service**: Rejected — must be repo-native.

## Implementation Plan

Phase 1: Define lease data model and add frontmatter fields
Phase 2: Implement atomic lease acquisition in SQLite
Phase 3: Implement renewal and expiration
Phase 4: Implement release with cleanup
Phase 5: Add "leased" phase to Story lifecycle
Phase 6: Implement conflict detection on edits
Phase 7: Build lease query APIs
Phase 8: Add CLI and MCP commands
Phase 9: Integration test concurrent acquisition

## Acceptance Criteria

- Leases can be acquired atomically (no race conditions on concurrent attempts)
- Only lease holders can transition or edit leased documents
- Lease expiration automatically releases after timeout
- Lease queries accurately show current state
- "Leased" phase works correctly in Story lifecycle
- CLI and MCP tools expose all lease operations

## Risks / Dependencies

- Depends on SMET-I-0018 for domain model (lease fields on Story/Task)
- Atomic SQLite transactions need careful handling
- Lease expiration via check-on-access means stale leases persist until someone queries them
- Must coordinate with SMET-I-0024 (worktree integration adds worktree_path field)
- Must coordinate with SMET-I-0026 (orchestrator consumes lease APIs)
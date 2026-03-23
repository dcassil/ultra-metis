---
id: add-work-leasing-and-isolated
level: initiative
title: "Add Work Leasing and Isolated Execution Ownership"
short_code: "SMET-I-0012"
created_at: 2026-03-11T20:00:03.503842+00:00
updated_at: 2026-03-11T20:00:03.503842+00:00
parent: SMET-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: XL
strategy_id: NULL
initiative_id: add-work-leasing-and-isolated
---

# Add Work Leasing and Isolated Execution Ownership

## Context

When multiple AI agents (or humans) work on a monorepo simultaneously, they need a way to claim ownership of specific work items to avoid conflicts. Current Metis has no concept of work ownership or isolation — any agent can work on any task.

Cadre should introduce work leases: a mechanism where an agent acquires exclusive ownership of a Story (or Task), works on it in isolation (potentially in a git worktree), and releases the lease when done. This prevents merge conflicts, duplicate work, and allows the orchestrator to coordinate parallel execution.

## Governing Commitments

This initiative directly serves:
- **Parallel work is enabled through explicit ownership and isolation.** Work leasing allows humans and agents to operate safely in parallel. Exclusive lease acquisition prevents conflicts structurally, not through convention or hope.
- **Single-agent and orchestrated modes share one governance model.** Leases work identically whether a human claims a story, a single agent takes a task, or an orchestrator assigns work to a fleet. One mechanism, consistent semantics.
- **All durable project memory lives in the repo.** Lease state is persisted in document frontmatter and the database — it's durable state, not ephemeral session context. Lease history is part of the repo's execution record.
- **Evidence-based workflow progression** (Vision #7). The "leased" phase is a structural gate — a Story cannot become active until someone has claimed ownership. Progression reflects real commitment, not just intent.
- **The system is built around intentional, durable structure.** Leases make ownership explicit and enforced. The system prevents conflicts structurally rather than relying on agents to coordinate informally.

## Goals & Non-Goals

**Goals:**
- Introduce a lease mechanism for Stories and Tasks that grants exclusive execution ownership
- Support lease acquisition, renewal, release, and expiration
- Track lease holder identity (agent ID, session ID, or human name)
- Integrate with git worktree creation for isolated execution environments
- Add a "leased" phase to the Story lifecycle
- Prevent multiple agents from working on the same Story simultaneously

**Non-Goals:**
- Implementing the orchestrator that assigns leases (that's SMET-I-0013)
- Solving git merge conflicts — leases prevent them by isolating work
- File-level locking — leases are at the document level, not file level
- Distributed consensus — this is single-machine, single-repo

## Detailed Design

### What to Reuse from `metis/`
- Document state management for tracking lease status
- Frontmatter fields for lease metadata
- Phase transition machinery (adding "leased" as a phase)
- Database for lease tracking and queries

### What to Change from `metis/`
- Add lease-related frontmatter fields to Story and Task types: lease_holder, lease_acquired_at, lease_expires_at, lease_worktree_path
- Add "leased" phase to Story lifecycle (between "ready" and "active")
- Modify transition rules: a Story must be leased before it can become active

### What is Net New
- Lease acquisition API: atomic acquire-or-fail operation to prevent races
- Lease renewal: extend a lease before it expires
- Lease release: explicit release when work is complete or abandoned
- Lease expiration: automatic release after timeout (configurable)
- Lease queries: who holds what, what's available, what's expired
- Git worktree integration: optionally create an isolated worktree when a lease is acquired
- Worktree cleanup: remove worktree when lease is released
- Lease conflict detection: prevent edits to leased documents by non-holders

## Alternatives Considered

1. **Git branch conventions instead of leases**: Rejected because branch conventions are unenforced and agents don't reliably follow conventions.
2. **File-level locking**: Rejected because it's too granular — we want story-level ownership, not file-level.
3. **Optimistic concurrency (merge conflicts as resolution)**: Rejected because prevention is better than resolution for AI agents, which handle merge conflicts poorly.
4. **External lock service**: Rejected — must be repo-native with no external dependencies.

## Implementation Plan

Phase 1: Define lease data model and frontmatter fields
Phase 2: Implement lease acquisition with atomic semantics
Phase 3: Implement lease renewal and expiration
Phase 4: Implement lease release and cleanup
Phase 5: Add "leased" phase to Story lifecycle
Phase 6: Integrate git worktree creation/cleanup with lease lifecycle
Phase 7: Implement lease conflict detection on edits
Phase 8: Add lease query APIs

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- Leases can be acquired atomically (no race conditions)
- Only the lease holder can transition a leased Story
- Lease expiration automatically releases after timeout
- Git worktrees are optionally created on lease acquisition and cleaned up on release
- Lease queries show current state accurately
- Non-holders cannot edit leased documents (or edits are blocked/warned)
- Lease metadata is visible in document frontmatter
- Multiple concurrent lease acquisitions for the same Story correctly resolve (one wins, others fail)

## Risks / Dependencies

- Depends on SMET-I-0001 for domain model (lease fields on Story/Task)
- Depends on SMET-I-0002 for "leased" phase in Story lifecycle
- Atomic lease acquisition in SQLite may need careful transaction handling
- Git worktree operations can fail — need robust error handling
- Lease expiration needs a background timer or check-on-access pattern
- Must coordinate with SMET-I-0013 (orchestrator) for lease assignment logic

## Codebase Areas to Inspect

- `metis/src/domain/` — document types to extend with lease fields
- `metis/src/db/` — SQLite transactions for atomic operations
- `metis/src/commands/` — where git worktree operations would live
- Any existing git integration code in Metis

## Suggested Tasks for Decomposition

1. Define lease data model and frontmatter fields
2. Implement atomic lease acquisition in SQLite
3. Implement lease renewal and expiration checking
4. Implement lease release with cleanup
5. Add "leased" phase to Story lifecycle
6. Integrate git worktree creation on lease acquire
7. Integrate git worktree cleanup on lease release
8. Implement lease conflict detection on document edits
9. Build lease query APIs (list, status, availability)
10. Integration test concurrent lease acquisition
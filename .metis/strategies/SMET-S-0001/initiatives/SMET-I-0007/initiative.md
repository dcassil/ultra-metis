---
id: strengthen-workflow-states
level: initiative
title: "Strengthen Workflow States, Traceability, and Transition Enforcement"
short_code: "SMET-I-0007"
created_at: 2026-03-11T19:59:39.960863+00:00
updated_at: 2026-03-17T01:43:26.036816+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
strategy_id: cadre-core-engine-repo
initiative_id: strengthen-workflow-states
---

# Strengthen Workflow States, Traceability, and Transition Enforcement

## Context

Metis already has a solid forward-only phase transition model with exit criteria validation. Cadre needs to strengthen this foundation with richer transition semantics: pre-transition hooks (quality gates, rule checks), post-transition side effects (notifications, child document creation), transition audit trails, and full traceability from any document to its ancestors and descendants.

This initiative is the workflow backbone that other initiatives depend on for enforcement behavior.

## Governing Commitments

This initiative directly serves:
- **Evidence-based workflow progression** (Vision #7). Workflow progression is backed by durable evidence. Pre-transition hooks enforce measurable criteria — quality gates, rule checks, exit conditions — before work can advance.
- **All durable project memory lives in the repo.** Transition audit trails, blocked reasons, and traceability chains are persistent artifacts. The history of how work progressed is part of the repo's durable memory.
- **Planning is durable and traceable from product intent to execution.** Traceability queries walk the full hierarchy from any task up to its product doc. Every transition, every blocked state, every gate check is recorded.
- **Protected governance** (Vision #4, #10). Pre-transition hooks give rules and quality gates structural teeth — governance checks run automatically as part of workflow, not as optional afterthoughts.
- **The system is built around intentional, durable structure.** Forward-only transitions, enforced hooks, and audit trails make it structurally difficult to bypass the intended workflow.

## Goals & Non-Goals

**Goals:**
- Extend the phase transition engine with pre-transition and post-transition hook infrastructure
- Implement a complete transition audit trail (who, when, from-phase, to-phase, checks passed/failed)
- Add blocked-reason tracking for documents in blocked state
- Build traceability queries: ancestors, descendants, siblings, cross-references
- Support conditional transitions (only allow if certain criteria are met beyond basic exit criteria)

**Non-Goals:**
- Defining specific phase flows for new document types (SMET-I-0002)
- Implementing specific quality gates (SMET-I-0005)
- Building UI for transition visualization (SMET-I-0011)

## Detailed Design

### What to Reuse from `metis/`
- The existing phase transition engine — it's well-structured with forward-only enforcement
- Exit criteria validation pattern
- The `blocked_by` field pattern for dependency tracking
- Database-backed state tracking

### What to Change from `metis/`
- Add pre-transition hook registration (pluggable checks that run before a transition is allowed)
- Add post-transition hook registration (side effects that fire after a successful transition)
- Enrich the transition record with more metadata (actor, timestamp, checks run, reason)
- Make transition history queryable (not just current state)

### What is Net New
- Pre-transition hook system: register checks that must pass before a transition proceeds
- Post-transition hook system: register actions that fire after successful transitions
- Transition audit log: persistent record of all transitions with full metadata
- Blocked-reason tracking: structured explanation of why a document is blocked
- Traceability query engine: walk the document hierarchy in any direction
- Sibling awareness: know what other documents share a parent
- Cross-reference graph: queryable graph of all document relationships

## Alternatives Considered

1. **Keep transitions simple, add hooks later**: Rejected because quality gates and rule checks need hooks from day one — bolting them on later creates inconsistency.
2. **Event sourcing for all state changes**: Deferred as over-engineering for now. Audit log captures the essential history without full event sourcing complexity.
3. **External workflow engine (Temporal, etc.)**: Rejected — violates repo-native principle and adds external dependency.

## Implementation Plan

Phase 1: Design hook registration API for pre- and post-transition hooks
Phase 2: Implement pre-transition hook execution in the transition engine
Phase 3: Implement post-transition hook execution
Phase 4: Build transition audit log with persistent storage
Phase 5: Add blocked-reason tracking
Phase 6: Implement traceability query engine
Phase 7: Add cross-reference graph queries

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- Pre-transition hooks can be registered and are executed before any transition
- Post-transition hooks fire reliably after successful transitions
- All transitions are recorded in an audit log with full metadata
- Blocked documents have structured reason tracking
- Traceability queries can walk up and down the hierarchy from any document
- Cross-reference relationships are queryable
- Hook failures prevent transitions and report clear error messages
- Audit log is queryable by document, date range, actor, and phase

## Risks / Dependencies

- Depends on SMET-I-0001 for the domain model updates
- Hook infrastructure must be performant — transitions should remain fast
- Risk of hook ordering issues — need clear execution semantics
- Must coordinate with SMET-I-0005 (quality gates register as pre-transition hooks)
- Must coordinate with SMET-I-0004 (rule checks register as pre-transition hooks)

## Codebase Areas to Inspect

- `metis/src/commands/transition.rs` or equivalent — current transition logic
- `metis/src/domain/phases.rs` — phase definitions and validation
- `metis/src/db/` — state storage and query patterns
- `metis/src/domain/` — exit criteria patterns

## Suggested Tasks for Decomposition

1. Design pre-transition and post-transition hook API
2. Implement hook registration and execution engine
3. Implement pre-transition hook execution in transition flow
4. Implement post-transition hook execution
5. Build transition audit log schema and storage
6. Implement blocked-reason tracking
7. Build ancestor/descendant traceability queries
8. Build cross-reference graph queries
9. Add hook failure reporting with clear error messages
10. Integration test hooks, audit log, and traceability together
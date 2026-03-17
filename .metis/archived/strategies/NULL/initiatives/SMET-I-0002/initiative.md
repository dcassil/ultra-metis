---
id: evolve-planning-hierarchy-beyond
level: initiative
title: "Evolve Planning Hierarchy Beyond Vision-Initiative-Task"
short_code: "SMET-I-0002"
created_at: 2026-03-11T19:59:18.127998+00:00
updated_at: 2026-03-11T19:59:18.127998+00:00
parent: SMET-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: XL
strategy_id: NULL
initiative_id: evolve-planning-hierarchy-beyond
---

# Evolve Planning Hierarchy Beyond Vision-Initiative-Task

## Context

Metis currently uses a flight-levels inspired hierarchy: Vision → Strategy → Initiative → Task (with Strategy being optional depending on preset). This works for general project management but is too flat for software engineering in a monorepo.

Super-Metis needs a richer hierarchy that connects product intent through design to implementation: Product Doc → Epic → Story → Task, with cross-cutting governance artifacts (Design Context, Rules Config, Analysis Baselines) that can be referenced from any level.

This initiative defines the new hierarchy semantics, phase flows, and transition rules. It works closely with SMET-I-0001 (domain model) which provides the types, and SMET-I-0007 (workflow states) which strengthens enforcement.

## Governing Commitments

This initiative directly serves:
- **Planning is durable and traceable from product intent to execution.** Product Doc → Epic → Story → Task creates clear traceability. This hierarchy is the structural backbone of how intent flows to work.
- **All durable project memory lives in the repo.** Every level in the hierarchy is a persisted artifact. Planning decisions, phase transitions, and governance references survive across sessions and context windows.
- **Evidence-based workflow progression** (Vision #7). Phase transitions are forward-only and backed by exit criteria — progression reflects demonstrated readiness, not ad-hoc status updates.
- **Single-agent and orchestrated modes share one governance model.** The planning hierarchy and its phase rules apply identically whether one human, one agent, or a coordinated fleet is doing the work.
- **The system is built around intentional, durable structure.** The hierarchy enforces valid relationships and progression. Structure makes it hard to skip steps or create orphaned work.

## Goals & Non-Goals

**Goals:**
- Define the complete planning hierarchy for Super-Metis
- Specify phase flows and valid transitions for each new document type
- Define parent-child rules and cross-reference semantics
- Ensure the hierarchy supports both human and AI-driven planning workflows
- Design for traceability from product intent through to task execution

**Non-Goals:**
- Implementing the domain types themselves (SMET-I-0001)
- Building UI for hierarchy visualization (SMET-I-0011)
- Defining template content for each level (SMET-I-0014)

## Detailed Design

### What to Reuse from `metis/`
- The phase/state machine infrastructure — it's well-designed and extensible
- Forward-only transition enforcement pattern
- Exit criteria validation pattern
- Parent-child relationship tracking in the database
- The concept of "flight levels" as inspiration, even though we're replacing the specific levels

### What to Change from `metis/`
- Replace Vision/Strategy/Initiative/Task levels with Product Doc/Epic/Story/Task
- Define new phase sequences for each document type
- Update parent requirement rules (e.g., Stories require an Epic parent, Tasks require a Story parent)
- Modify the "backlog" concept to work within the new hierarchy
- Update transition validation to handle new phase sequences

### What is Net New
- Phase definitions for Product Doc: draft → review → published
- Phase definitions for Epic: discovery → design → ready → active → completed
- Phase definitions for Story: draft → refined → ready → leased → active → completed (with blocked state)
- Phase definitions for Task: todo → active → completed (with blocked state)
- Cross-cutting artifact phases: Design Context (draft → approved), Rules Config (draft → active → superseded), Analysis Baseline (captured → active → archived)
- Governance artifact linking — any planning artifact can reference governance artifacts
- Traceability chain enforcement — ability to trace from any task up to its product doc

## Alternatives Considered

1. **Keep existing hierarchy and add layers**: Rejected because renaming creates confusion. A clean new hierarchy is clearer.
2. **Flat document model with tags**: Rejected because hierarchy enforcement is a key principle — structure should be enforced, not suggested.
3. **Deeper hierarchy (Product → Theme → Epic → Story → Task → Subtask)**: Rejected as over-engineered for monorepo use. Can be added later if needed.

## Implementation Plan

Phase 1: Document the complete hierarchy specification with all phase flows
Phase 2: Implement phase definitions for each new document type
Phase 3: Implement parent-child validation rules
Phase 4: Implement cross-reference validation for governance artifacts
Phase 5: Implement traceability chain queries
Phase 6: Integration testing of complete hierarchy flows

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- All document types have defined, enforced phase sequences
- Parent-child rules prevent invalid hierarchy construction
- Forward-only transitions are enforced for all types
- Cross-cutting governance artifacts can be referenced from any planning level
- Traceability queries can walk from any task up to its product doc
- Backlog items work correctly within the new hierarchy
- Phase transition hooks fire correctly for all types

## Risks / Dependencies

- Depends on SMET-I-0001 for the domain types
- Must coordinate with SMET-I-0007 for transition enforcement details
- Risk of over-specifying phases — keep it simple enough to be usable
- The Story "leased" phase is new and may need iteration (coordinate with SMET-I-0012)

## Codebase Areas to Inspect

- `metis/src/domain/phases.rs` or equivalent — phase definitions
- `metis/src/domain/transitions.rs` or equivalent — transition logic
- `metis/src/domain/hierarchy.rs` or equivalent — parent-child rules
- `metis/src/db/` — queries that enforce hierarchy

## Suggested Tasks for Decomposition

1. Write the complete hierarchy specification document
2. Implement Product Doc phase flow
3. Implement Epic phase flow
4. Implement Story phase flow (including leased state)
5. Implement governance artifact phase flows (Design Context, Rules Config, Analysis Baseline)
6. Implement parent-child validation rules
7. Implement cross-reference validation
8. Implement traceability chain queries
9. Update backlog concept for new hierarchy
10. Integration test full hierarchy lifecycle
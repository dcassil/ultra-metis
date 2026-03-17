---
id: add-remediation-and-investigation
level: initiative
title: "Add Remediation and Investigation Loops for Degraded Quality"
short_code: "SMET-I-0006"
created_at: 2026-03-11T19:59:35.285219+00:00
updated_at: 2026-03-17T01:59:41.514427+00:00
parent: SMET-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: add-remediation-and-investigation
---

# Add Remediation and Investigation Loops for Degraded Quality

## Context

When quality gates detect regression (SMET-I-0005), the system needs a structured response — not just a blocked transition, but a workflow for investigating the root cause, proposing remediation, executing fixes, and verifying resolution. Without this, quality gate violations become dead-ends that users work around rather than address.

Super-Metis should introduce Architecture Investigation documents for root-cause analysis and structured remediation loops that connect quality degradation to corrective action.

## Governing Commitments

This initiative directly serves:
- **Quality includes architectural integrity and is tracked over time.** Remediation loops close the feedback cycle — degradation is not just detected but investigated, corrected, and verified, creating a durable record of how quality was restored.
- **Evidence-based workflow progression** (Vision #7). Investigations are triggered by measurable quality evidence, not subjective concern. Resolution is verified by re-checking the same evidence.
- **All durable project memory lives in the repo.** Investigations, remediation plans, and resolution records are persisted artifacts. The history of why quality degraded and how it was fixed becomes part of the repo's institutional memory.
- **The system is built around intentional, durable structure.** Structured investigation workflows prevent quality gate violations from becoming dead-ends that users work around instead of address.

## Goals & Non-Goals

**Goals:**
- Introduce Architecture Investigation document type for structured root-cause analysis when quality degrades
- Define a remediation loop workflow: detect degradation → create investigation → propose fix → execute → verify
- Connect investigations to the quality baselines and records that triggered them
- Support both human-driven and agent-driven investigation workflows
- Track remediation history so patterns of recurring issues become visible

**Non-Goals:**
- Auto-fixing quality issues — investigations produce plans, not automatic code changes
- Replacing code review — investigations are complementary, not substitutes
- Handling non-quality concerns (security incidents, operational issues) — scope is code quality

## Detailed Design

### What to Reuse from `metis/`
- Document creation and workflow infrastructure
- Phase transition machinery for investigation lifecycle
- Cross-reference patterns for linking investigations to baselines
- Search and indexing for finding related investigations

### What to Change from `metis/`
- Extend phase transition failure handling to automatically suggest creating an investigation
- Add "triggered_by" relationship type for connecting investigations to quality records

### What is Net New
- Architecture Investigation document type with phases: opened → analyzing → proposal → remediating → verified → closed
- Remediation loop workflow engine: automatic investigation creation when quality gates fail
- Investigation templates that pre-populate with quality delta information
- Remediation task generation — investigations can spawn tasks for the fix work
- Verification step — after remediation, re-run quality check to confirm resolution
- Recurring issue detection — flag when similar quality issues recur across investigations

## Alternatives Considered

1. **Just block transitions and let users figure it out**: Rejected because unstructured responses lead to workarounds, not fixes.
2. **Automatic remediation by AI agents**: Deferred — start with structured investigation, add auto-remediation as a later enhancement.
3. **Integrate with external incident management**: Rejected for now — keep it repo-native. Can add integrations later.

## Implementation Plan

Phase 1: Define Architecture Investigation document schema and phase flow
Phase 2: Implement domain type (coordinate with SMET-I-0001)
Phase 3: Build automatic investigation creation on quality gate failure
Phase 4: Implement investigation templates with quality delta pre-population
Phase 5: Build remediation task generation from investigations
Phase 6: Implement verification step (re-check quality after remediation)
Phase 7: Add recurring issue detection

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- Architecture Investigations can be created manually or automatically on quality gate failure
- Investigations are linked to the quality records/baselines that triggered them
- Investigation templates include relevant quality delta information
- Remediation tasks can be spawned from investigations
- Verification re-checks quality after remediation completes
- Recurring issue patterns are detected and flagged
- Full audit trail from quality degradation through investigation to resolution

## Risks / Dependencies

- Depends on SMET-I-0001 for domain types
- Depends on SMET-I-0005 for quality baselines and gates
- Depends on SMET-I-0002 for phase flow definitions
- Risk of investigation fatigue if quality gates are too sensitive — thresholds matter
- Must coordinate with SMET-I-0013 (orchestrator) for agent-driven investigations

## Codebase Areas to Inspect

- `metis/src/domain/` — domain type patterns
- `metis/src/commands/transition.rs` — transition failure handling
- `metis/src/templates/` — template system for pre-populated investigations
- `metis/src/db/` — cross-reference queries

## Suggested Tasks for Decomposition

1. Define Architecture Investigation document schema and phase flow
2. Implement Architecture Investigation domain type
3. Build automatic investigation creation on quality gate failure
4. Create investigation templates with quality delta pre-population
5. Implement remediation task generation from investigations
6. Implement verification step (quality re-check)
7. Add recurring issue detection logic
8. Integration test full remediation loop end-to-end
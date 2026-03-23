---
id: governance-quality-domain-types
level: initiative
title: "Governance & Quality Domain Types"
short_code: "SMET-I-0019"
created_at: 2026-03-11T21:39:40.254555+00:00
updated_at: 2026-03-16T21:07:39.111099+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: cadre-core-engine-repo
initiative_id: governance-quality-domain-types
---

# Governance & Quality Domain Types

## Context

Cadre needs cross-cutting governance artifacts that enforce engineering quality alongside the planning hierarchy. These types — RulesConfig, AnalysisBaseline, QualityRecord, DesignChangeProposal, and ArchitectureInvestigation — sit orthogonal to the ProductDoc → Epic → Story → Task planning hierarchy and provide the quality enforcement layer.

This initiative depends on SMET-I-0018 (Core Planning Hierarchy) being complete, since governance types reference and cross-cut planning types. Split out from the original SMET-I-0001 (now archived) as a focused vertical slice.

## Governing Commitments

- **Reference architecture drives rules, structure, and analysis.** RulesConfig and AnalysisBaseline encode the enforcement relationships that flow from architecture decisions.
- **All durable project memory lives in the repo.** Quality records and governance artifacts are persisted alongside planning artifacts.
- **The system is built around intentional, durable structure.** Governance types have strong typing and validated relationships to planning artifacts.

## Goals & Non-Goals

**Goals:**
- Implement RulesConfig type with protection semantics (rules that cannot be casually overridden), layered scopes (platform → org → repo → package → component → task), and rule categories typed by purpose (behavioral, architectural, operational, information-handling, decision-making, validation/quality, approval/escalation, execution-safety)
- Implement AnalysisBaseline type (captures expected quality thresholds)
- Implement QualityRecord type (captures point-in-time quality measurements with baseline comparison, regressions, improvements, threshold breaches, blocked transitions, accepted overrides)
- Implement ValidationRecord type (captures validation type, inputs, result, failures, evidence links, related task/story, whether required or optional — critical for audits and autonomous modes)
- Implement RemediationRecord type (tracks detected problem, affected scope, required fixes, validation after fix, recurrence signals, whether systemic or local)
- Implement DesignChangeProposal type (formal proposals to change design decisions)
- Implement ArchitectureInvestigation type (structured investigation triggered by quality degradation, repeated failures, architecture drift, or enforcement/code divergence)
- Implement ApprovalRecord type (durable record of who approved what, when, and why)
- Implement ValidationPolicy type (configurable policies defining what validations are required for different work types)
- Implement OwnershipMap type (who is responsible for what scope)
- Implement ConstraintRecord type (explicit constraints that govern decision boundaries)
- Build cross-reference system between governance artifacts and planning artifacts
- Serialization/deserialization for all governance types (markdown + YAML frontmatter)
- SQLite schema and indexing for governance types
- Comprehensive unit tests

**Non-Goals:**
- Planning hierarchy types (ProductDoc, Epic, Story, DesignContext) — covered by SMET-I-0018
- Architecture catalog types — covered by SMET-I-0020
- Actual quality gate enforcement logic — covered by SMET-I-0022
- Remediation loops when quality degrades — covered by SMET-I-0006
- UI for governance types — covered by SMET-I-0011
- Execution/traceability record types (ExecutionRecord, TransitionRecord, DecisionRecord) — covered by SMET-I-0031
- Durable insight notes — covered by SMET-I-0030

## Detailed Design

### RulesConfig
- Stores engineering rules for a repo/project (linting, formatting, naming conventions, dependency restrictions)
- Protection semantics: rules marked as `protected` require a DesignChangeProposal to modify
- References a ReferenceArchitecture (from SMET-I-0020) as its seed source
- Fields: rule entries (name, severity, config, protected flag), source reference, override history

### AnalysisBaseline
- Captures expected quality thresholds at a point in time
- Linked to a RulesConfig — defines "what good looks like" for the rules in effect
- Fields: metric entries (name, threshold, direction), baseline date, linked rules config

### QualityRecord
- Point-in-time snapshot of actual quality measurements
- Compared against AnalysisBaseline to determine pass/fail
- Fields: metric entries (name, value, pass/fail), record date, linked baseline, linked planning artifacts

### DesignChangeProposal
- Formal proposal to change a protected rule or design decision
- Requires justification and approval before the change takes effect
- Fields: target artifact reference, proposed change, justification, status (proposed/approved/rejected), reviewer

### ArchitectureInvestigation
- Structured investigation triggered when quality degrades or architecture questions arise
- Links to QualityRecords that triggered it, and produces recommendations
- Fields: trigger references, investigation scope, findings, recommendations, status

### Cross-Reference System
- Governance artifacts reference planning artifacts by short code
- Planning artifacts can query "what governance artifacts apply to me?"
- Implemented as indexed reference fields in frontmatter, queryable via SQLite

## Alternatives Considered

1. **Embed governance fields directly in planning types**: Rejected — governance concerns are cross-cutting and evolve independently. Separate types are cleaner.
2. **Single "GovernanceArtifact" type with a subtype field**: Rejected — each governance type has distinct fields and semantics. Strong typing is better.
3. **Skip protection semantics on RulesConfig**: Rejected — the whole point is to prevent casual rule changes. Protection is a core requirement.

## Implementation Plan

Phase 1: Implement RulesConfig type with protection semantics
Phase 2: Implement AnalysisBaseline and QualityRecord types
Phase 3: Implement DesignChangeProposal and ArchitectureInvestigation types
Phase 4: Build cross-reference indexing between governance and planning artifacts
Phase 5: Update SQLite schema and indexing for all governance types
Phase 6: Comprehensive unit tests

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- All five governance types defined as Rust types with proper traits
- RulesConfig protection semantics enforced at the domain level
- Cross-references between governance and planning artifacts are validated and queryable
- All types round-trip through markdown+frontmatter without data loss
- SQLite schema updated and indexed for governance types
- Unit tests cover type creation, validation, serialization, cross-references, and protection semantics

## Risks / Dependencies

- Depends on SMET-I-0018 (planning hierarchy must exist for cross-references to work)
- SMET-I-0020 (architecture catalog) provides ReferenceArchitecture that seeds RulesConfig — can implement with a reference field initially and wire up later
- Cross-reference system design must be extensible for future artifact types

## Codebase Areas to Inspect

- `metis/src/domain/` — existing domain types to extend
- `metis/src/storage/` — serialization patterns to follow
- `metis/src/db/` — SQLite schema to extend
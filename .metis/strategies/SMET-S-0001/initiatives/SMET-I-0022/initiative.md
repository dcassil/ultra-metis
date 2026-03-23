---
id: quality-gates-and-phase-transition
level: initiative
title: "Quality Gates and Phase Transition Integration"
short_code: "SMET-I-0022"
created_at: 2026-03-11T21:52:26.799824+00:00
updated_at: 2026-03-17T00:31:04.323255+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: ultra-metis-core-engine-repo
initiative_id: quality-gates-and-phase-transition
---

# Quality Gates and Phase Transition Integration

## Context

Capturing quality data (SMET-I-0021) is only half the picture. The other half is enforcing quality standards: blocking phase transitions when quality regresses, configuring per-project thresholds, and integrating with the transition hook system (SMET-I-0007).

This initiative builds the enforcement layer that gives quality gates structural teeth — a Story can't move to "completed" if quality regressed below thresholds.

Split from the original SMET-I-0005 (now archived).

## Goals & Non-Goals

**Goals:**
- Define quality gate configuration format: per-project thresholds for different metrics
- Implement quality gate threshold checking: given a Quality Record, determine pass/fail per metric
- Register quality gates as pre-transition hooks (using SMET-I-0007's hook infrastructure)
- Block phase transitions when quality gate thresholds are violated
- Provide clear, actionable error messages when gates fail (which metrics, by how much)
- Support gate override with audit trail for emergencies
- CLI command: `check-quality-gate`
- MCP tool for same

**Non-Goals:**
- Capturing baselines or comparing them — covered by SMET-I-0021
- Remediation workflows when gates fail — covered by SMET-I-0006
- UI for quality gate status — covered by SMET-I-0011
- Running analysis tools — Super-Metis checks results, doesn't run tools

## Detailed Design

### Quality Gate Configuration
- Per-project YAML config defining thresholds per metric
- Configurable per document type and phase transition (e.g., stricter gates for "active → completed" than for "ready → active")
- Threshold types: absolute (must be below X), relative (must not regress by more than Y%), trend (must be improving)

### Gate Checking Engine
- Takes a Quality Record (from SMET-I-0021) and gate configuration
- Evaluates each metric against its threshold
- Returns pass/fail with details per metric

### Phase Transition Integration
- Registers as pre-transition hooks via SMET-I-0007's hook system
- Before allowing transitions, captures a fresh baseline and compares against the active baseline
- If any gate fails, blocks the transition with a detailed explanation
- Supports configurable gate strictness per transition type

### Emergency Override
- Force flag on transition that bypasses quality gates
- Creates audit trail entry documenting who overrode, when, and why
- Does not silently pass — the override is visible in transition history

## Alternatives Considered

1. **Advisory-only gates (warn but don't block)**: Supported as a configuration option, but blocking is the default. Advisory-only undermines the enforcement principle.
2. **Git hooks for quality gates**: Rejected — too granular (per-commit) and don't integrate with the planning workflow.
3. **Single pass/fail without details**: Rejected — actionable feedback is essential for agents and humans to fix issues.

## Implementation Plan

Phase 1: Define quality gate configuration format
Phase 2: Build gate checking engine
Phase 3: Register gates as pre-transition hooks
Phase 4: Implement gate failure messaging
Phase 5: Implement emergency override with audit trail
Phase 6: Add CLI and MCP commands
Phase 7: Integration tests with full transition flows

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- Quality gates can be configured with per-metric thresholds
- Phase transitions are blocked when gate thresholds are violated
- Gate failure messages clearly identify which metrics failed and by how much
- Emergency override works with audit trail
- Gates integrate correctly with SMET-I-0007's hook system
- CLI and MCP tools expose gate checking

## Risks / Dependencies

- Depends on SMET-I-0021 for baseline capture and Quality Records — **COMPLETED**
- Depends on SMET-I-0007 for pre-transition hook infrastructure — **NOT YET BUILT; hook integration deferred. Core gate types and checking engine are independent.**
- Gates that block too aggressively frustrate users — need good defaults and easy configuration
- Must coordinate with SMET-I-0006 (remediation loops trigger on gate failure)

## Design Decision: Scope for This Initiative

Since SMET-I-0007 (hook infrastructure) is not yet built, this initiative will focus on:
1. **Quality gate configuration format** — domain types for gate configs, thresholds, per-metric rules
2. **Gate checking engine** — takes QualityRecord + gate config, returns pass/fail with details
3. **Gate result types** — structured pass/fail results with actionable failure messages
4. **Emergency override model** — force flag + audit trail entry type

Hook registration and phase transition blocking will be implemented when I-0007 provides the infrastructure. The gate types and engine are fully usable standalone.
---
id: quality-integration-architecture
level: initiative
title: "Quality Integration: Architecture Hooks, Conformance Gates, and Baseline Comparison"
short_code: "SMET-I-0078"
created_at: 2026-03-23T17:28:06.415439+00:00
updated_at: 2026-03-23T17:28:06.415439+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: SMET-S-0001
initiative_id: quality-integration-architecture
---

# Quality Integration: Architecture Hooks, Conformance Gates, and Baseline Comparison Initiative

## Context

Phase 4 of ADR SMET-A-0001. The building blocks exist in cadre-core (Architecture document type, conformance checker, gate engine, baseline services) but nothing connects them to the execution lifecycle. This initiative wires architecture hooks into transitions and quality baselines into the review flow.

## Goals & Non-Goals

**Goals:**
- Wire architecture lifecycle hooks (SMET-I-0069) into HookRegistry
- Conformance gate on story completion blocks on drift beyond tolerance
- Quality baseline capture at story start, comparison after each task in `/cadre-execute`
- Surface quality regressions to code quality reviewer
- Auto-create investigation stories on conformance drift
- Record quality deltas per task in documents

**Non-Goals:**
- Modifying the Architecture document type (SMET-I-0068, completed)
- Implementing the HookContext trait refactor (SMET-I-0069)
- Implementing architecture MCP tools (SMET-I-0070)
- Modifying conformance checker or gate engine algorithms
- The `/cadre-execute` command itself (SMET-I-0076)

## Detailed Design

### Hook Registration
- `register_architecture_hooks()` at MCP server and CLI startup
- Hook 1: Story → design creates Architecture doc with checklist + baseline_score
- Hook 2: Task → active snapshots architecture context from parent Story
- Hook 3: Story → completed blocks on conformance drift, creates investigation story

### Quality Baseline Flow in /cadre-execute
- Story start: capture quality metrics via existing parsers
- After each task: compare via BaselineComparisonEngine
- Review stage: quality delta formatted as markdown for reviewer context
- Task document: quality delta recorded as audit trail

### Investigation Story Auto-Creation
- Title: "Investigate architecture drift from [story title]"
- Type: Investigation
- Pre-populated: violated rules, score delta, affected files

## Alternatives Considered

1. **Quality checks only at story completion**: Rejected — per-task tracking identifies which task introduced regression
2. **Advisory-only gates**: Rejected as default — enforcement is the point. Configurable via GateSeverity
3. **Inline quality checks in reviewer only**: Rejected — automated comparison provides objective deltas

## Implementation Plan

1. Hook registration
2. Conformance gate wiring and end-to-end test
3. Quality baseline capture integration in /cadre-execute
4. Review stage integration
5. Full lifecycle integration test

## Dependencies

- **Blocked by**: SMET-I-0068 (completed), SMET-I-0069, SMET-I-0070, SMET-I-0076
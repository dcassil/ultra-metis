---
id: create-deterministic-scoring
level: initiative
title: "Create Deterministic Scoring Rubric for Docs, Decomposition, and Build Quality"
short_code: "SMET-I-0061"
created_at: 2026-03-18T17:31:29.612954+00:00
updated_at: 2026-03-20T16:51:22.042921+00:00
parent: benchmarking-and-evaluation
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0003
initiative_id: create-deterministic-scoring
---

# Create Deterministic Scoring Rubric for Docs, Decomposition, and Build Quality Initiative

## Context

The benchmark only becomes useful when the scores reflect real engineering value. Right now several metrics in the repo are proxies that are too weak for this purpose. For example, “contains the word test” is not the same as a good decomposition, and output byte size is not the same as good document quality. The benchmark needs deterministic scoring that matches the product goals: architecture alignment, disciplined use of static tools, useful planning artifacts, and complete end-product delivery.

## Goals & Non-Goals

**Goals:**
- Define deterministic scoring for each benchmark track.
- Prefer computed evidence over subjective judgment wherever possible.
- Make architecture conformance and static-tool usage first-class metrics.
- Produce a score breakdown that explains tradeoffs rather than hiding them.

**Non-Goals:**
- Reducing the benchmark to a single magic number
- Fully removing all optional qualitative review
- Replacing engineering judgment for product decisions

## Requirements

### System Requirements
- REQ-001: Score document generation for required-doc coverage, section completeness, placeholder leakage, and hierarchy correctness.
- REQ-002: Score decomposition for coverage, granularity, traceability, dependency quality, and acceptance-criteria specificity.
- REQ-003: Score build outcome for implementation completeness, verification status, edge-case handling, and architecture conformance.
- REQ-004: Record token usage and time as separate cost metrics, not hidden inside quality metrics.
- REQ-005: Include a static-tool utilization score based on actual inspection and verification actions.

## Architecture

### Overview

Scoring should have three layers:

1. Deterministic file and structure checks
2. Deterministic verification command results
3. Optional secondary judge pass for nuanced qualitative comparison

The first two layers should carry most of the score. Judge-model or human review should be supplementary, not the primary signal.

## Detailed Design

The rubric should score the three tracks independently:

- `document_generation_score`
- `decomposition_score`
- `build_outcome_score`

It should also produce supporting metrics:

- `architecture_conformance_score`
- `static_tool_utilization_score`
- `tokens_total`
- `time_total`
- `rework_count`

Examples of deterministic checks:

- required docs exist
- required sections exist
- frontmatter and traceability links are valid
- expected architecture boundaries are preserved
- tests/lint/typecheck pass
- expected features and edge cases pass scenario verification

## Alternatives Considered

Relying mainly on an LLM judge would be faster to implement, but the score would drift over time and be difficult to trust. Purely deterministic scoring can miss nuance, but it provides the stable regression signal needed to tune Cadre. A deterministic-first approach with optional qualitative review is the right balance.

## Implementation Plan

1. Define score categories and supporting metrics for each benchmark track.
2. Map scenario expectations to deterministic checks.
3. Add architecture conformance and static-tool usage scoring.
4. Define a normalized score output schema with breakdowns instead of only totals.
5. Add a secondary optional judge phase only for qualities that are hard to compute mechanically.
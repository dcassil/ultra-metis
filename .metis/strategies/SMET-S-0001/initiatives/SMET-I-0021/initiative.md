---
id: baseline-capture-comparison-and
level: initiative
title: "Baseline Capture, Comparison, and Quality Records"
short_code: "SMET-I-0021"
created_at: 2026-03-11T21:52:25.384187+00:00
updated_at: 2026-03-16T21:28:17.274362+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: cadre-core-engine-repo
initiative_id: baseline-capture-comparison-and
---

# Baseline Capture, Comparison, and Quality Records

## Context

Code quality degrades gradually unless actively measured. Linters, type checkers, and static analysis tools produce results, but those results are typically ephemeral — run once and discarded. There's no durable record of quality over time and no way to detect regression trends.

This initiative introduces the data capture side of quality tracking: Analysis Baselines (point-in-time snapshots), Quality Records (comparison deltas), and the parsers/comparison engine needed to produce them. Quality gate enforcement is a separate initiative (SMET-I-0022).

Split from the original SMET-I-0005 (now archived). Domain types for AnalysisBaseline and QualityRecord come from SMET-I-0019.

## Goals & Non-Goals

**Goals:**
- Build tool output parsers for common analysis tools (ESLint, clippy, TypeScript, test coverage)
- Implement baseline capture: parse tool output → store as Analysis Baseline document
- Implement baseline comparison engine: diff two baselines → produce Quality Record with deltas
- Store baselines and quality records as durable, searchable markdown+frontmatter documents
- Support architecture-aware analysis: verify implementation respects Reference Architecture boundaries
- Update SQLite schema for efficient quality metric comparison queries
- CLI commands: `capture-baseline`, `compare-baselines`
- MCP tools for same operations

**Non-Goals:**
- Quality gate enforcement and phase transition blocking — covered by SMET-I-0022
- Remediation loops when quality degrades — covered by SMET-I-0006
- Running static analysis tools — Cadre captures results, it doesn't run tools
- Defining what "good quality" means — thresholds are configurable per-project

## Detailed Design

### Tool Output Parsers
- Pluggable parser architecture: each parser takes tool output (stdout/file) and produces structured metric entries
- Initial parsers: ESLint (JSON format), TypeScript compiler (diagnostic output), clippy (JSON format), test coverage (lcov/cobertura)
- Each parser produces: metric name, value, file-level breakdown, severity counts

### Analysis Baseline
- Document containing: tool name, timestamp, summary metrics, detailed findings, file-level breakdown
- Stored as markdown+frontmatter with structured YAML data sections
- Indexed in SQLite for efficient comparison queries

### Baseline Comparison Engine
- Takes two baselines (before/after) for the same tool
- Produces a Quality Record: metric deltas, new issues, resolved issues, trend direction
- Supports per-file drill-down (which files improved/regressed)

### Architecture Boundary Checks
- Verify module boundaries, dependency direction, folder structure conventions against the Reference Architecture (from SMET-I-0020)
- Produce architectural conformance metrics as part of baseline capture

## Alternatives Considered

1. **Store quality data in CI/CD only**: Rejected — CI data is ephemeral and not repo-native.
2. **External quality dashboard (SonarQube)**: Deferred — core tracking should be repo-native.
3. **Custom analysis engine**: Rejected — leverage existing tools, just capture and track results.

## Implementation Plan

Phase 1: Build pluggable parser architecture and ESLint parser
Phase 2: Build TypeScript, clippy, and coverage parsers
Phase 3: Implement baseline capture and storage
Phase 4: Build baseline comparison engine
Phase 5: Implement architecture boundary checking
Phase 6: Add CLI and MCP commands
Phase 7: Unit and integration tests

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- Baselines can be captured from ESLint, TypeScript, clippy, and coverage tool outputs
- Baselines are stored as durable, searchable documents
- Two baselines can be compared to produce a Quality Record with deltas
- Architecture boundary checks produce conformance metrics
- CLI and MCP tools expose all capture/compare operations
- All quality data round-trips through markdown+frontmatter without loss

## Risks / Dependencies

- Depends on SMET-I-0019 for AnalysisBaseline and QualityRecord domain types
- Depends on SMET-I-0020 for Reference Architecture (architecture boundary checks)
- Tool output format fragility — parsers need robust error handling
- Must coordinate with SMET-I-0022 (quality gates consume baselines)
---
id: add-static-analysis-baselines
level: initiative
title: "Add Static Analysis Baselines, Comparison, and Quality Gates"
short_code: "SMET-I-0005"
created_at: 2026-03-11T19:59:31.300041+00:00
updated_at: 2026-03-11T19:59:31.300041+00:00
parent: SMET-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: XL
strategy_id: NULL
initiative_id: add-static-analysis-baselines
---

# Add Static Analysis Baselines, Comparison, and Quality Gates

## Context

Code quality in a monorepo degrades gradually unless actively measured and enforced. Linters, type checkers, and static analysis tools produce results, but those results are typically ephemeral — run once and discarded. There's no durable record of quality over time, no way to detect regression trends, and no structured mechanism to block work when quality drops below acceptable thresholds.

Cadre should introduce Analysis Baselines as durable artifacts that capture point-in-time quality snapshots, Quality Records that track changes over time, and Quality Gates that can block workflow transitions when thresholds are violated.

## Governing Commitments

This initiative directly serves:
- **Quality includes architectural integrity and is tracked over time.** Quality is measured as an evolving historical signal. Snapshots, comparisons, and trends make quality progress visible — not just a pass/fail at a single moment.
- **Architecture-aware quality** (Vision #6). Boundary adherence, dependency direction, and architectural conformance are part of the quality model. Analysis expectations are derived from the Reference Architecture, not defined independently.
- **Evidence-based workflow progression** (Vision #7). Quality gates block phase transitions when standards degrade. Progression is backed by measurable evidence, not self-reported status.
- **All durable project memory lives in the repo.** Baselines and quality records are persisted as repo-native artifacts, creating a permanent quality history that survives sessions and context windows.
- **Reference architecture drives rules, structure, and analysis** (Vision #4). The Reference Architecture is a control artifact. Analysis expectations flow from it — structure enforcement is not a separate concern but an integral part of quality measurement.

## Goals & Non-Goals

**Goals:**
- Introduce Analysis Baseline document type for capturing point-in-time static analysis results
- Introduce Quality Record type for tracking quality metrics over time
- Implement quality gate checks that can block phase transitions (e.g., a story can't move to "completed" if quality regressed)
- Support multiple analysis tool outputs (ESLint, clippy, TypeScript, test coverage, etc.)
- Provide comparison tooling to show quality deltas between baselines
- Support architecture-aware analysis checks: verify that implementation respects the selected Reference Architecture's folder structure, module boundaries, dependency direction, and naming conventions
- Derive analysis expectations from the Reference Architecture pattern so that architecture enforcement is part of the quality baseline, not a separate concern

**Non-Goals:**
- Running static analysis tools — Cadre captures and tracks results, it doesn't run the tools
- Defining what "good quality" means for every project — thresholds are configurable per-project
- Real-time code quality monitoring — this is snapshot-based, not continuous

## Detailed Design

### What to Reuse from `metis/`
- Document storage infrastructure for baselines and quality records
- Markdown + frontmatter format for quality artifacts
- The existing code indexing infrastructure as a foundation for analysis result storage
- Phase transition hooks as the mechanism for quality gate enforcement

### What to Change from `metis/`
- Extend phase transition logic to support pre-transition quality gate checks
- Add structured data sections to documents (not just prose — quality data needs structured formats)
- Extend the database schema to store quality metrics for efficient comparison queries

### What is Net New
- Analysis Baseline document type: tool name, timestamp, summary metrics, detailed findings, file-level breakdown
- Quality Record document type: baseline comparison, delta calculations, trend data
- Quality gate configuration: per-project thresholds for different metrics
- Baseline comparison engine: diff two baselines and produce a Quality Record
- Phase transition gate hooks: check quality thresholds before allowing transitions
- Architecture boundary checks: verify module boundaries, dependency direction, folder structure conventions against the Reference Architecture
- Analysis expectation derivation: read the Reference Architecture's analysis expectations and include them in baseline capture and gate checks automatically
- CLI commands: capture-baseline, compare-baselines, check-quality-gate, check-architecture-boundaries
- MCP tools: same operations for programmatic/agent access

## Alternatives Considered

1. **Store quality data in CI/CD only**: Rejected because CI data is ephemeral and not repo-native. Quality history should be durable and local.
2. **Use git hooks for quality gates**: Rejected because git hooks are too granular (per-commit) and don't integrate with the planning workflow.
3. **External quality dashboard (SonarQube, etc.)**: Deferred — can integrate later, but the core tracking should be repo-native.

## Implementation Plan

Phase 1: Define Analysis Baseline and Quality Record schemas
Phase 2: Implement domain types (coordinate with SMET-I-0001)
Phase 3: Build baseline capture tooling (parse common analysis tool outputs)
Phase 4: Build baseline comparison engine
Phase 5: Implement quality gate configuration and threshold checking
Phase 6: Integrate quality gates with phase transition hooks
Phase 7: Add CLI and MCP commands

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- Analysis Baselines can be captured from common tool outputs (ESLint, clippy, tsc, coverage)
- Baselines are stored as durable, searchable documents
- Two baselines can be compared to produce a Quality Record showing deltas
- Quality gates can be configured with thresholds per metric
- Phase transitions are blocked when quality gate thresholds are violated
- Quality trends are visible over time through Quality Records
- CLI and MCP tools expose all quality operations

## Risks / Dependencies

- Depends on SMET-I-0001 for domain types
- Depends on SMET-I-0007 for phase transition hook infrastructure
- Risk of tool output format fragility — parsers need to be robust
- Quality gates that block too aggressively will frustrate users — need good defaults and easy override
- Must coordinate with SMET-I-0006 (remediation loops) for what happens when quality degrades
- Must coordinate with SMET-I-0016 (architecture catalog) for analysis expectation format and Reference Architecture access

## Codebase Areas to Inspect

- `metis/src/domain/` — domain type patterns
- `metis/src/code_index/` or equivalent — existing code analysis infrastructure
- `metis/src/commands/transition.rs` or equivalent — phase transition hooks
- `metis/src/db/` — database schema for structured quality data

## Suggested Tasks for Decomposition

1. Define Analysis Baseline document schema
2. Define Quality Record document schema
3. Build ESLint/TypeScript output parser
4. Build clippy/Rust output parser
5. Build test coverage output parser
6. Implement baseline comparison engine
7. Define quality gate configuration format
8. Implement quality gate threshold checking
9. Integrate quality gates with phase transitions
10. Add CLI commands for baseline and quality operations
11. Add MCP tools for baseline and quality operations
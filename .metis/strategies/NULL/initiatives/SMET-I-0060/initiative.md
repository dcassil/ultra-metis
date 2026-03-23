---
id: implement-agent-system-run-harness
level: initiative
title: "Implement Agent-System Run Harness and Trace Capture"
short_code: "SMET-I-0060"
created_at: 2026-03-18T17:31:29.484543+00:00
updated_at: 2026-03-20T16:48:30.267918+00:00
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
initiative_id: implement-agent-system-run-harness
---

# Implement Agent-System Run Harness and Trace Capture Initiative

## Context

The current benchmark code captures some useful measurements, but much of it is based on CLI output size, one-off prompt calls, and synthetic quality proxies. To compare the full agent-system fairly, the harness needs to capture the entire run lifecycle: prompts, tool interactions, tokens, elapsed time, generated artifacts, and verification outputs.

This initiative builds that execution substrate. It turns `benchmarks/` into a reusable runner that can execute a scenario pack, persist structured traces, and hand the results off to scoring and reporting.

## Goals & Non-Goals

**Goals:**
- Build a reusable benchmark harness that runs a scenario end to end.
- Capture enough run data to explain why one run was better or worse than another.
- Normalize outputs into a stable result schema for downstream scoring and reporting.
- Support phase-aware execution for docs, decomposition, and build steps.

**Non-Goals:**
- Defining the benchmark scenarios themselves
- Deciding the final scoring weights
- Providing a hosted or distributed benchmark service

## Requirements

### System Requirements
- REQ-001: The harness must create a fresh run workspace for every benchmark execution.
- REQ-002: The harness must capture wall-clock time, token usage, run phase boundaries, and produced artifacts.
- REQ-003: The harness must record prompt inputs, agent outputs, and tool interactions where available.
- REQ-004: The harness must support storing per-phase results for document generation, decomposition, and build execution.
- REQ-005: The harness must emit normalized JSON suitable for trend history and comparison.

## Architecture

### Overview

The harness should be broken into:

- `scenario loader`: reads the canonical scenario pack
- `runner`: executes the scenario under a selected mode
- `trace collector`: stores prompts, tool calls, token counts, timing, and artifacts
- `result writer`: saves raw traces and normalized summaries

### Sequence

1. Create isolated workspace
2. Seed scenario inputs
3. Run phase 1: document generation
4. Run phase 2: decomposition
5. Run phase 3: build outcome
6. Execute deterministic verification commands
7. Persist raw trace and normalized result bundle

## Detailed Design

The run schema should distinguish:

- `run_metadata`: scenario id, mode, model, timestamp, commit, environment
- `phase_results`: docs, decomposition, build
- `usage`: tokens, time, tool counts
- `artifacts`: files created, docs created, reports produced
- `verification`: test, lint, typecheck, architecture checks
- `trace`: prompt log and tool/event sequence where obtainable

The harness should prefer tool-derived evidence over guessed metrics. If a metric cannot be computed deterministically, it should be clearly labeled as inferred.

## Alternatives Considered

Using the current practical benchmark structure as-is would be faster, but it would keep the benchmark tied to one scenario and preserve weak proxies like “did the task list mention testing.” A trace-first harness is more work up front but creates a durable foundation for fair comparison.

## Implementation Plan

1. Define the normalized run schema for raw and summarized output.
2. Refactor the current practical harness around explicit benchmark phases.
3. Add isolated workspace setup and seeded scenario materialization.
4. Add structured trace capture and phase-level usage metrics.
5. Save run bundles in a stable location for comparison and historical analysis.
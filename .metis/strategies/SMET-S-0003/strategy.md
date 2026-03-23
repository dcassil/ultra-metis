---
id: benchmarking-and-evaluation
level: strategy
title: "Benchmarking and Evaluation Framework for Agentic Workflows"
short_code: "SMET-S-0003"
created_at: 2026-03-18T17:31:08.443464+00:00
updated_at: 2026-03-18T17:31:08.443464+00:00
parent: cadre-repo-native-ai
blocked_by: []
archived: false

tags:
  - "#strategy"
  - "#phase/shaping"
  - "#strategy"
  - "#phase/shaping"


exit_criteria_met: false
risk_level: medium
stakeholders: []
strategy_id: benchmarking-and-evaluation
initiative_id: NULL
---

# Benchmarking and Evaluation Framework for Agentic Workflows Strategy

## Problem Statement

Cadre needs a benchmark system that measures the thing that actually matters: how well an AI agent performs when using the tooling, prompts, MCP surface, and document model end to end. The current benchmark work in `benchmarks/` contains useful experiments, but it mixes transport timing, template-fill experiments, synthetic proxy scores, and single-scenario runs in a way that does not yet provide a stable engineering signal.

The benchmark must answer three practical questions repeatedly over time:

1. How well does the agent create the planning document set from a prompt and partial seed state?
2. How well does the agent decompose vision and initiative inputs into usable, traceable, architecture-aligned work?
3. How well does the agent build the final product from the prompt, seeded documents, and decomposition outputs?

The system must compare original Metis and Cadre fairly, reward stronger architecture alignment and static-tool usage, and produce repeatable outputs that can guide product decisions instead of one-off anecdotes.

## Success Metrics

- At least 3 benchmark tracks are implemented: document generation, decomposition, and build outcome.
- Each run records tokens, wall-clock time, prompt sequence, tool calls, artifacts produced, and verification results.
- The harness supports both original Metis and Cadre execution paths using the same scenario pack and evaluation pipeline.
- Deterministic scoring covers required document coverage, traceability, architecture conformance, static-tool usage, and final product completeness.
- Benchmark runs emit a normalized JSON result schema plus human-readable reports.
- At least 3 canonical scenarios exist, with room to grow into a scenario pack library.
- Re-running the same benchmark pack provides stable enough output to compare revisions over time.

## Solution Approach

Build the benchmark as a neutral harness in `benchmarks/` that evaluates complete agent-system behavior rather than only tool latency. The harness will:

1. Define canonical scenario packs with prompt, seed documents, expected hierarchy, architecture constraints, and verification rules.
2. Run the same scenario in multiple execution modes, including original Metis and Cadre.
3. Capture full run traces: prompts, tool interactions, timing, tokens, generated artifacts, and final validation outputs.
4. Score results with deterministic scripts first, and optionally a secondary judge pass for qualities that are hard to compute mechanically.
5. Produce machine-readable results for trend tracking and concise reports for comparison.

The benchmark will explicitly reward the qualities Cadre is meant to improve: architectural alignment, stronger implementation discipline, heavier use of static tools, and lower token cost for comparable or better outcomes.

## Scope

**In Scope:**
- A benchmark strategy for agent-system comparison rather than binary-only timing
- Scenario pack definitions and seed artifact conventions
- Run harness and trace capture for benchmark phases
- Deterministic scoring for documents, decomposition, and built output
- Comparative execution modes for original Metis and Cadre
- Reporting, trend tracking, and regression workflow

**Out of Scope:**
- Perfect determinism of all LLM behavior
- Replacing every existing ad hoc benchmark immediately
- Using benchmark scores as the only product decision input
- Supporting every model or every agent host in the first pass

## Risks & Unknowns

- LLM variance can make repeated runs noisy if the prompts, budgets, or stopping rules are not tightly defined.
- A benchmark that is too synthetic may be repeatable but miss the actual strengths of each system in real use.
- A benchmark that is too open-ended may be realistic but too noisy to track regressions reliably.
- Architecture conformance and decomposition quality need careful scoring or they will collapse into weak heuristics.
- Existing Metis workspace sync issues and legacy document inconsistencies may interfere with benchmark planning operations if not isolated.

## Implementation Dependencies

The strategy is split into focused initiatives with a clear dependency chain:

- `SMET-I-0059` defines the canonical inputs and expected outputs.
- `SMET-I-0060` builds the harness that executes and records runs.
- `SMET-I-0061` defines how runs are scored in a deterministic way.
- `SMET-I-0062` adds fair comparison modes across original Metis and Cadre.
- A final reporting initiative will turn raw run results into trend and regression outputs.

The practical critical path is:

1. Scenario pack structure and seed artifacts
2. Run harness and capture
3. Deterministic scoring
4. Comparative modes
5. Reporting and historical regression tracking

## Change Log

### 2026-03-18 - Initial Strategy
- **Change**: Created benchmark strategy for agent-system evaluation.
- **Rationale**: Existing benchmark work is promising but fragmented; the repo needs a reusable framework for comparing original Metis and Cadre on planning and delivery outcomes.
- **Impact**: Establishes a durable benchmark track in Metis and defines the initiative structure for the next implementation pass.

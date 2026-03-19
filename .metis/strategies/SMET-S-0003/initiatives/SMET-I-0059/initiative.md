---
id: build-canonical-scenario-packs-and
level: initiative
title: "Build Canonical Scenario Packs and Seed Artifacts"
short_code: "SMET-I-0059"
created_at: 2026-03-18T17:31:29.479674+00:00
updated_at: 2026-03-18T17:31:29.479674+00:00
parent: benchmarking-and-evaluation
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0003
initiative_id: build-canonical-scenario-packs-and
---

# Build Canonical Scenario Packs and Seed Artifacts Initiative

## Context

The benchmark is only as good as the scenarios it exercises. Right now the repo has one promising practical scenario and several ad hoc experiments, but no canonical scenario pack format that can be re-run, extended, and scored consistently across tools and revisions. To compare original Metis and Ultra-Metis meaningfully, each run needs a fixed set of inputs, seed artifacts, and expected outcomes.

This initiative creates those benchmark inputs. It defines what a scenario pack contains, how a run is seeded, and what artifacts are considered ground truth for evaluation.

## Goals & Non-Goals

**Goals:**
- Define a canonical scenario pack schema for benchmark inputs.
- Create initial benchmark scenarios that cover planning-heavy and build-heavy work.
- Package each scenario with seed docs, architecture expectations, and verification hints.
- Make scenarios reusable across both original Metis and Ultra-Metis execution modes.

**Non-Goals:**
- Building the full run harness
- Implementing scoring logic
- Solving every product domain in the first scenario pack release

## Requirements

### System Requirements
- REQ-001: Each scenario pack must include the original product prompt, seed documents, and expected planning hierarchy.
- REQ-002: Each scenario pack must define which docs are pre-seeded versus which docs the agent is expected to create.
- REQ-003: Each scenario pack must define architecture constraints and required boundaries for scoring.
- REQ-004: Each scenario pack must define completion checks for the final built project.
- REQ-005: Scenario packs must be stored in a repo-native format that is easy to diff and extend.

## Architecture

### Overview

Store scenario packs under `benchmarks/` with a normalized structure such as:

- `scenario.md` or `prompt.md`
- `seed/` documents
- `expectations/` for required docs and hierarchy
- `verification/` for deterministic checks
- `fixtures/` for test inputs and golden outputs where relevant

Each scenario should support three benchmark tracks:

1. Document generation
2. Decomposition
3. Build outcome

### Seed Scenario Set

The first benchmark pack should include at least:

- An app-generation scenario based on the defined vision plus 2 seeded initiatives/stories
- A data-pipeline or processing scenario
- An architecture-sensitive scenario that rewards strong boundary discipline

## Detailed Design

Each scenario pack should define:

- Prompt and user intent
- Seeded vision, initiatives, stories, and specs
- Documents the agent must create
- Expected hierarchy and traceability links
- Required architectural constraints
- Required implementation behaviors and edge cases
- Deterministic verification rules

The benchmark pack should also separate:

- `canonical_inputs`: things both systems receive
- `expected_outputs`: things both systems are judged against
- `run_rules`: budgets, stopping criteria, and allowed tools

## Alternatives Considered

Using one giant scenario would be faster to set up, but it would overfit the benchmark to a single style of work. Using many scenarios immediately would increase coverage but slow implementation and make early debugging painful. A small, curated starter pack with explicit structure is the right first step.

## Implementation Plan

1. Normalize the existing practical benchmark scenario into a formal scenario pack structure.
2. Promote the app-generation scenario in `benchmarks/` into the same structure.
3. Define a scenario manifest schema covering seeds, expectations, verification, and run rules.
4. Create at least one additional architecture-sensitive scenario.
5. Add documentation explaining how to add new scenario packs without changing harness code.

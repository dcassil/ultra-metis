---
id: add-comparative-execution-modes
level: initiative
title: "Add Comparative Execution Modes for Original Metis and Ultra-Metis"
short_code: "SMET-I-0062"
created_at: 2026-03-18T17:31:29.629094+00:00
updated_at: 2026-03-20T16:53:57.783381+00:00
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
initiative_id: add-comparative-execution-modes
---

# Add Comparative Execution Modes for Original Metis and Ultra-Metis Initiative

## Context

The benchmark must compare original Metis and Ultra-Metis fairly. Previous comparisons in the repo have mixed transport layers, compared CLI to MCP, or measured tool latency without measuring the full quality of the agent outcome. This initiative creates explicit execution modes so the benchmark can support both strict apples-to-apples comparison and realistic workflow comparison without confusing the two.

## Goals & Non-Goals

**Goals:**
- Add benchmark modes that compare original Metis and Ultra-Metis on the same scenario pack.
- Define the fairness rules for model, budget, stopping conditions, and allowed tools.
- Support both constrained and realistic comparison styles.
- Make the comparison output explain what differed, not just who “won.”

**Non-Goals:**
- Pretending original Metis and Ultra-Metis have identical workflow affordances
- Benchmarking every host application or every model family immediately
- Reducing everything to transport-level timing

## Requirements

### System Requirements
- REQ-001: Both systems must run against the same scenario inputs and deterministic scoring pipeline.
- REQ-002: Constrained mode must hold model, prompt framing, budget, and stopping rules as constant as possible.
- REQ-003: Realistic mode must allow each system to use its intended workflow while still producing comparable outputs.
- REQ-004: The benchmark must record which tool surface was used: CLI, MCP, plugin commands, or mixed.
- REQ-005: Comparison reports must clearly distinguish cost metrics from quality metrics.

## Architecture

### Overview

The comparison layer should support at least two modes:

- `constrained`: same model, same prompt scaffold, same token budget, same stopping criteria
- `realistic`: same scenario and evaluation, but each system can use its natural workflow

This avoids the common failure modes:

- unfairly giving one system a transport advantage
- unfairly stripping away the workflow that makes one system useful

## Detailed Design

Each benchmark run should include:

- `system_under_test`: original-metis or ultra-metis
- `execution_mode`: constrained or realistic
- `tool_surface`: CLI, MCP, plugin workflow, or mixed
- `model_config`: model name and budget
- `run_rules`: stopping criteria and retry behavior

The benchmark should also report:

- `tokens_to_green`
- `time_to_green`
- `doc_quality_breakdown`
- `decomposition_breakdown`
- `build_quality_breakdown`

## Alternatives Considered

A single comparison mode would be simpler, but it would either over-constrain the systems or make the benchmark too noisy to trust. Separate constrained and realistic modes preserve both fairness and product relevance.

## Implementation Plan

1. Define comparison mode semantics and fairness rules.
2. Add execution adapters for original Metis and Ultra-Metis.
3. Ensure both adapters emit the same normalized run schema.
4. Add comparison reporting that separates constrained and realistic outcomes.
5. Validate the comparison flow on the initial canonical scenario pack before expanding coverage.
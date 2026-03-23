---
id: cognitive-operation-kernel-and
level: initiative
title: "Cognitive Operation Kernel and Reusable Loops"
short_code: "SMET-I-0029"
created_at: 2026-03-16T20:06:10.876415+00:00
updated_at: 2026-03-17T01:14:59.705927+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
strategy_id: ultra-metis-core-engine-repo
initiative_id: cognitive-operation-kernel-and
---

# Cognitive Operation Kernel and Reusable Loops

## Context

Super-Metis's central design idea is that all higher-level workflows are composed from a fixed set of reusable engineering operations, not from brittle one-off workflow scripts. This avoids hardcoding separate large flows for every task type while still allowing structured execution.

The cognitive operation kernel defines 12 universal operations. These operations compose into reusable loops, and a "workflow" is simply a predefined composition of loops with entry conditions, required artifacts, required validations, escalation rules, and completion rules. This is the central abstraction of the entire system.

## Governing Commitments

- **All workflows are composed from a fixed set of reusable cognitive operations** (Vision #8). This initiative defines and implements the kernel itself.
- **The system is built around intentional, durable structure** (Vision #15). The operation kernel replaces ad-hoc reasoning with structured, auditable execution.
- **Static tools first, AI reasoning second** (Principle #8). Each operation can be backed by static tools, with AI reasoning selecting and interpreting.

## Goals & Non-Goals

**Goals:**
- Define and implement the 12 core cognitive operations as a formal Rust abstraction: frame objective, acquire context, build/refine model, locate focus, analyze structure/boundaries, trace flow/causality, assess impact/risk, shape/select solution, decompose/sequence work, create/modify artifact, validate against reality, reassess/adapt
- Define and implement the 11 reusable loops that compose these operations: objective framing, context sufficiency, model construction, focus narrowing, trace, risk/impact, solution shaping, decomposition, artifact production, validation, adaptation
- Define the workflow template format: a predefined composition of loops with entry conditions, required artifacts, required validations, escalation rules, and completion rules
- Create initial workflow templates for common work types: bugfix, feature slice, refactor, migration, architecture change, brownfield evaluation, remediation, investigation, greenfield bootstrap
- Each operation should be: invocable directly, combinable into loops, mappable to templates, backable by static tools, auditable through execution records
- Integrate with the execution record system (SMET-I-0031) so that operation/loop execution is traceable

**Non-Goals:**
- Building the execution engine that runs workflows (that's the runner, post-MVP)
- Implementing every possible workflow template — start with common patterns
- Defining the static tool bindings for every operation (tools vary per repo)

## Detailed Design

### Core Operations
Each operation is a typed Rust struct/trait with:
- Operation identifier and description
- Input requirements (what context/artifacts it needs)
- Output type (what it produces)
- Tool hints (what static tools could back this operation)
- Escalation conditions (when to escalate instead of proceeding)

### Reusable Loops
Each loop is a composition of operations with:
- Entry condition (when to enter the loop)
- Operations in sequence (which operations to execute)
- Exit condition (when the loop is satisfied)
- Max iterations (to prevent infinite loops)
- Escalation rules (when to break out and escalate)

### Workflow Templates
A workflow template defines:
- Work type (bugfix, feature, refactor, etc.)
- Required loops in sequence
- Per-loop configuration (which operations, entry/exit conditions)
- Required artifacts at each stage
- Required validations before completion
- Escalation rules
- Completion rules (what constitutes "done")

### Integration Points
- Operations produce entries for ExecutionRecord (SMET-I-0031)
- Loops can trigger gate checks (SMET-I-0032)
- Workflow templates reference planning artifacts (SMET-I-0018)
- Operations can fetch durable insight notes (SMET-I-0030)

## Alternatives Considered

1. **Hardcoded workflows per task type**: Rejected — leads to brittle, duplicated code. The operation kernel is more composable and maintainable.
2. **Fully dynamic AI-driven workflow**: Rejected — too unpredictable. The kernel provides structured reasoning while allowing AI to operate within each operation.
3. **External workflow engine (Temporal, etc.)**: Rejected — must be repo-native. The kernel is a design pattern, not a distributed orchestration system.

## Implementation Plan

Phase 1: Define the 12 core operation types as Rust traits/structs
Phase 2: Define the 11 reusable loop types with composition semantics
Phase 3: Define the workflow template format (YAML/markdown-based)
Phase 4: Implement loop execution engine (sequential operation composition)
Phase 5: Create initial workflow templates (bugfix, feature, refactor, investigation)
Phase 6: Create remaining workflow templates (migration, architecture change, brownfield eval, remediation, greenfield bootstrap)
Phase 7: Integrate with execution record system for auditability
Phase 8: Add MCP and CLI tools for querying available operations, loops, and templates

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- All 12 core operations are defined as typed Rust abstractions
- All 11 reusable loops are implemented with entry/exit conditions
- Workflow templates can be defined and stored as durable artifacts
- At least 5 workflow templates exist for common work types
- Operations are auditable through execution records
- Loops correctly escalate when conditions are met
- Workflow templates are queryable via MCP and CLI

## Risks / Dependencies

- This is a foundational initiative — many other initiatives depend on the operation/loop abstraction
- Depends on SMET-I-0018 for planning types that workflows reference
- Depends on SMET-I-0031 for execution record integration
- Depends on SMET-I-0032 for gate integration
- The abstraction must be practical, not academic — risk of over-engineering
- Must coordinate with SMET-I-0014 (templates) for workflow template format

## Suggested Tasks for Decomposition

1. Design core operation trait/struct in Rust
2. Implement all 12 operation types
3. Design loop composition semantics
4. Implement all 11 loop types
5. Design workflow template format
6. Implement workflow template parser and storage
7. Create bugfix and feature workflow templates
8. Create refactor, investigation, and migration workflow templates
9. Create architecture change, brownfield eval, remediation, and greenfield bootstrap templates
10. Integrate operations with execution record system
11. Add MCP and CLI tools for operation/loop/template queries
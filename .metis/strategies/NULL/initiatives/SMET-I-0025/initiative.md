---
id: single-agent-runner-with-context
level: initiative
title: "Single-Agent Runner with Context Enrichment"
short_code: "SMET-I-0025"
created_at: 2026-03-11T21:52:29.887488+00:00
updated_at: 2026-03-11T21:52:29.887488+00:00
parent: SMET-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: single-agent-runner-with-context
---

# Single-Agent Runner with Context Enrichment

> **STATUS: POST-MVP / DEFERRED** — The product spec defers a full internal single-agent runner to later phases. For MVP, existing structured-execution plugins (Ralph loop, etc.) should be leveraged, with Super-Metis enriching those runs with durable repo-native context and capturing results. The system's MVP role is to assemble correct context, retrieve relevant architecture/rules/notes/history, require validations, and store outputs — not to own the execution engine itself.

## Context

Metis has a "Ralph" loop concept for single-task execution — an agent picks up a task and works through it iteratively. Super-Metis needs to evolve this into a Story-level runner that gathers full context (design references, applicable rules, quality requirements) before executing, and verifies quality after completing.

This is the execution engine for single-agent mode. The multi-agent orchestrator (SMET-I-0026) builds on top of this runner.

Split from the original SMET-I-0013 (now archived).

## Goals & Non-Goals

**Goals:**
- Evolve the Ralph loop to work with Stories (not just Tasks)
- Build context assembly: before executing, gather design references, applicable rules, quality requirements, and Reference Architecture guidance for the Story
- Add pre-execution quality baseline capture
- Add post-execution quality verification (compare against baseline)
- Support iterative execution: plan → implement → test → verify quality → complete
- Track execution progress in the Story document (working memory)
- Handle execution failures gracefully (mark blocked, record reason)

**Non-Goals:**
- Multi-agent coordination — covered by SMET-I-0026
- Work selection/prioritization — the runner executes what it's given
- Lease management — covered by SMET-I-0023
- Git worktree management — covered by SMET-I-0024

## Detailed Design

### Context Assembly Module
- Given a Story short code, gather:
  - Story content and acceptance criteria
  - Parent Epic context and Product Doc intent
  - Linked Design Context documents
  - Applicable engineering rules (from RulesConfig)
  - Quality requirements (from quality gate configuration)
  - Reference Architecture guidance (relevant layers, boundaries, conventions)
- Package into a structured context object that the runner agent receives

### Execution Loop
1. Acquire lease (via SMET-I-0023)
2. Assemble context
3. Capture pre-execution quality baseline (via SMET-I-0021)
4. Execute: plan → implement → test (iterative, updates Story progress)
5. Capture post-execution quality baseline
6. Compare baselines — check quality gates (via SMET-I-0022)
7. If gates pass: transition Story to completed, release lease
8. If gates fail: record failure, optionally retry or mark blocked

### Progress Tracking
- During execution, regularly update the Story document with progress notes
- Serves as working memory that survives context compaction
- Records: what was done, what was found, decisions made, files modified

### Failure Handling
- Execution error: mark Story as blocked with reason, release lease
- Quality gate failure: record which gates failed, optionally trigger remediation (SMET-I-0006)
- Timeout: lease expires, Story returns to ready state

## Alternatives Considered

1. **Keep Ralph loop unchanged**: Rejected — Story-level execution with context enrichment is fundamentally more capable.
2. **Agent self-gathers context**: Rejected — structured context assembly ensures consistency and completeness. Agents shouldn't improvise what context they need.
3. **Skip quality verification**: Rejected — post-execution quality checks are core to evidence-based progression.

## Implementation Plan

Phase 1: Build context assembly module
Phase 2: Evolve Ralph loop into Story-level runner
Phase 3: Integrate pre/post quality baseline capture
Phase 4: Integrate quality gate checking on completion
Phase 5: Implement progress tracking in Story documents
Phase 6: Implement failure handling
Phase 7: Integration test full execution cycle

## Acceptance Criteria

- Runner can execute a Story end-to-end with full context
- Context assembly includes design refs, rules, quality reqs, and architecture guidance
- Pre/post quality baselines are captured and compared
- Quality gate failures prevent completion
- Progress is tracked in Story documents throughout execution
- Failures are handled gracefully with clear status updates

## Risks / Dependencies

- Depends on SMET-I-0018 for Story type
- Depends on SMET-I-0019 for RulesConfig/DesignContext types
- Depends on SMET-I-0021 for baseline capture
- Depends on SMET-I-0022 for quality gate checking
- Depends on SMET-I-0023 for leasing
- Context assembly quality directly affects execution quality — garbage in, garbage out
- Must coordinate with SMET-I-0026 (orchestrator dispatches to this runner)
---
id: investigate-and-strengthen-error
level: initiative
title: "Investigate and Strengthen Error Handling to Match or Exceed Original Metis"
short_code: "SMET-I-0036"
created_at: 2026-03-17T18:45:20.234215+00:00
updated_at: 2026-03-17T19:30:33.254846+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: cadre-core-engine-repo
initiative_id: investigate-and-strengthen-error
---

# Investigate and Strengthen Error Handling to Match or Exceed Original Metis Initiative

## Context

The SMET-I-0035 benchmark revealed that the original metis plugin catches 4/4 error scenarios while cadre only catches 2/4. The two known bugs (SMET-T-0078 orphan parents, SMET-T-0079 terminal phase transitions) account for the gap, but fixing those two bugs alone isn't sufficient — we need a systematic audit of ALL error paths to find other gaps we haven't discovered yet.

The original metis plugin (TypeScript) has mature error handling built over multiple iterations. Cadre (Rust) has strong type-level guarantees in the core library but the store/MCP/CLI layers were built quickly and likely have additional gaps in validation, error messaging, and edge case handling.

### Known Gaps from Benchmarks
1. **Parent validation**: Creating documents with non-existent parents succeeds (SMET-T-0078)
2. **Terminal phase transitions**: No error on transitioning completed/published docs (SMET-T-0079)
3. **Unknown gaps**: Need systematic investigation

## Goals & Non-Goals

**Goals:**
- Catalog every error path in the original metis plugin's MCP server (what does it validate, when does it error, what messages does it return)
- Catalog every error path in cadre store/MCP/CLI layers
- Produce a gap analysis: what does metis catch that cadre doesn't
- Fix all identified gaps in cadre
- Add a comprehensive negative-path test suite (invalid inputs, edge cases, constraint violations)
- Ensure cadre error messages are as good or better than metis (actionable, specific, suggest fixes)
- Re-run benchmark Scenario 6 (Error Handling) and achieve 4/4 or better

**Non-Goals:**
- Changing the core domain library error types (those are already well-typed with `DocumentValidationError`)
- Adding error handling for features metis doesn't have (cadre-only features)
- Internationalization of error messages

## Detailed Design

### Phase 1: Audit Original Metis Error Handling
- Read the metis MCP server source code (TypeScript, at `~/.claude/plugins/cache/colliery-io-metis/`)
- For each MCP tool, catalog: what validations run, what error messages return, what edge cases are handled
- Document in a matrix: Operation × Validation × Error Message

### Phase 2: Audit Cadre Error Handling
- Walk through each `cadre-store` operation and each MCP tool handler
- For each, catalog: what validations exist, what's missing compared to metis
- Identify gaps: validations metis does that cadre doesn't

### Phase 3: Gap Analysis and Prioritization
- Produce a ranked list of missing validations
- Categorize: data integrity (HIGH), user experience (MEDIUM), edge case (LOW)

### Phase 4: Fix Gaps
- Implement missing validations in `cadre-store`
- Ensure MCP server and CLI surface errors clearly
- Add unit tests for every new error path

### Phase 5: Negative-Path Test Suite
- Create comprehensive tests that exercise EVERY invalid operation
- Include: bad short codes, missing required fields, invalid phase transitions, duplicate short codes, empty strings, oversized inputs, special characters
- This becomes a regression test suite

### Phase 6: Re-benchmark
- Re-run benchmark Scenario 6 (Error Handling)
- Verify 4/4 or better on error catching
- Compare error message quality

## Alternatives Considered

1. **Just fix the two known bugs**: Rejected — the two bugs are symptoms, not the full picture. A systematic audit prevents future surprises.
2. **Copy metis error handling verbatim**: Rejected — different language, different architecture. Need to understand the intent and implement idiomatically in Rust.
3. **Add a generic error middleware**: Rejected — error handling should be specific to each operation, not generic.

## Implementation Plan

Phase 1: Audit original metis error paths (read source, build matrix)
Phase 2: Audit cadre error paths (same matrix format)
Phase 3: Gap analysis and fix prioritization
Phase 4: Implement fixes (includes SMET-T-0078 and SMET-T-0079)
Phase 5: Build negative-path test suite
Phase 6: Re-run error handling benchmark

## Final Results

**Error handling benchmark: 4/4 PASS** (up from 2/4)

### Changes Made (6 commits)
1. **Parent validation** — create_document now verifies parent exists and enforces hierarchy rules via HierarchyValidator
2. **Terminal phase errors** — transition_phase returns error for terminal phases instead of silently succeeding
3. **Archive improvements** — cascading archive for children, error on already-archived documents
4. **Edit validation** — post-edit frontmatter integrity check prevents document corruption
5. **Error messages** — user_message() on StoreError with actionable guidance, wired into MCP layer
6. **Test suite** — 40 store tests (up from 14), comprehensive negative-path coverage

### Discovered Gaps (for future work)
- Empty title validation not enforced during document creation (validate() not called in constructors)

## Decomposition Summary

7 tasks created, ordered by dependency:

| Task | Title | Dependencies |
|------|-------|-------------|
| SMET-T-0080 | Parent existence and hierarchy validation in create_document | None |
| SMET-T-0081 | Terminal phase transition and auto-advance error handling | None |
| SMET-T-0082 | Archive cascading and already-archived detection | None |
| SMET-T-0083 | Edit document frontmatter integrity validation | None |
| SMET-T-0084 | Error message quality improvements across all operations | T-0080 through T-0083 |
| SMET-T-0085 | Comprehensive negative-path test suite | T-0080 through T-0084 |
| SMET-T-0086 | Re-run error handling benchmark and verify 4/4 | T-0080 through T-0085 |

**Execution order**: T-0080 through T-0083 can run in parallel (independent fixes). T-0084 improves error messages after all paths exist. T-0085 builds the regression suite. T-0086 is the final validation.
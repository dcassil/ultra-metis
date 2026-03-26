---
id: fast-path-integration-into
level: task
title: "Fast-Path Integration into BrownfieldEvaluator"
short_code: "SMET-T-0175"
created_at: 2026-03-26T18:01:59.182746+00:00
updated_at: 2026-03-26T18:19:43.153064+00:00
parent: SMET-I-0090
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0090
---

# Fast-Path Integration into BrownfieldEvaluator

## Parent Initiative

[[SMET-I-0090]]

## Objective

Wire the `RulesConfigAnalyzer` into `BrownfieldEvaluator::evaluate()` as an early-exit fast path. Add the `FileContentReader` parameter to the evaluator's public API. Ensure the fast path produces the same `EvaluationResult` type and feeds into `PatternMatcher` correctly. Add integration tests covering fast-path triggers, fallback to full analysis, and equivalence of output types.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `BrownfieldEvaluator::evaluate()` signature updated to accept optional `&dyn FileContentReader` parameter (backward compatible — `None` skips fast path)
- [ ] Fast path runs before `StructureAnalyzer::analyze()` when reader is provided
- [ ] When fast path succeeds (both axes above threshold): uses config-inferred `StructureAnalysis` → `PatternMatcher::match_against()` → returns `EvaluationResult` with appropriate `EvaluationOutcome`
- [ ] When fast path fails (either axis below threshold): falls through to existing full analysis path unchanged
- [ ] `EvaluationResult` includes a new `fast_path_used: bool` field so callers can distinguish how the analysis was produced
- [ ] `EvaluationResult` includes optional `config_analysis: Option<RulesConfigAnalysisResult>` for transparency (quality/layering scores, signals)
- [ ] Existing tests continue to pass unchanged (no reader = no fast path = identical behavior)
- [ ] Integration test: JS/TS project with strict ESLint + dependency-cruiser → fast path triggers, returns CatalogMatch or DerivedArchitecture
- [ ] Integration test: Rust workspace with clippy::pedantic + 4 crates → fast path triggers
- [ ] Integration test: project with ESLint but no boundary enforcement → fast path skipped, falls through to full analysis
- [ ] Integration test: project with no config files → fast path skipped immediately
- [ ] `cargo test` passes for all existing and new tests

## Implementation Notes

### Technical Approach
- Use `Option<&dyn FileContentReader>` as the new parameter to keep backward compatibility — existing callers pass `None` and get identical behavior
- The fast path should be the first thing in `evaluate()`, before any `StructureAnalyzer` call
- For the `EvaluationOutcome` decision tree after fast path: reuse the same quality_threshold / match_threshold logic from the existing evaluator, just with the config-inferred `StructureAnalysis` instead of the file-scanned one
- Integration tests should use `MockContentReader` with realistic config file contents

### Dependencies
- SMET-T-0174 (RulesConfigAnalyzer with full analyze() method)

### Risk Considerations
- API change to `evaluate()` — ensure all call sites are updated (check MCP tools, CLI commands, and integration tests that call evaluate)
- The fast path must never produce a worse result than the slow path — conservative thresholds are key

## Status Updates

- 2026-03-26: Wired RulesConfigAnalyzer into BrownfieldEvaluator. Added evaluate_with_reader() method alongside backward-compatible evaluate(). Added fast_path_used and config_analysis fields to EvaluationResult. All 957 workspace tests pass (0 failures). MCP tool callers unaffected (use evaluate() without reader).
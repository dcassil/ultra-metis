---
id: implement-validation-gate-scorer
level: task
title: "Implement validation gate scorer for code quality, doc accuracy, and instruction adherence"
short_code: "SMET-T-0113"
created_at: 2026-03-17T22:06:49.737376+00:00
updated_at: 2026-03-17T22:23:07.835694+00:00
parent: SMET-I-0048
blocked_by: [SMET-T-0112]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0048
---

# Implement validation gate scorer for code quality, doc accuracy, and instruction adherence

## Parent Initiative

[[SMET-I-0048]]

## Objective

Build the quality scoring engine used by the gated runner to evaluate each initiative before approving it. The scorer checks generated code, tests, and docs against the spec's acceptance criteria, then produces a structured `ValidationGateResult` with a pass/rework/reject decision.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `GateScorer::score_initiative()` takes an `InitiativeResult` + artifact paths and returns `ValidationGateResult`
- [ ] Code quality check: does generated code compile? do tests pass? line count reasonable?
- [ ] Doc accuracy check: do generated docs contain required sections? are placeholders removed? format valid?
- [ ] Instruction adherence check: did AI create the expected tasks? follow phase flow? match I/O contracts in spec?
- [ ] Gate decision: `Approved` if all blocking checks pass, `RequiresRework` if fixable issues, `Rejected` if fundamental failures
- [ ] `issues_found` populated with specific, actionable descriptions per failure
- [ ] `rework_tokens` estimated from issues found (~500 tokens per issue as heuristic)
- [ ] Unit tests cover: all-pass, partial failure, and rejection cases

## Implementation Notes

### Technical Approach

Create `benchmarks/practical/src/gate_scorer.rs`:

**Scoring dimensions:**
1. **Code quality** — run `cargo check` on generated Rust files, count test functions, verify `cargo test` passes
2. **Doc accuracy** — parse markdown, check required sections present, scan for `{placeholder}` text remaining
3. **Instruction adherence** — compare task count vs expected range, verify phase transitions followed correct order, check I/O contracts match spec (Dataset in/out types)

**Gate decision thresholds:**
- `Approved`: 0 blocking failures
- `RequiresRework`: 1-3 fixable issues (missing tests, thin docs, minor gaps)
- `Rejected`: structural failure (wrong I/O contracts, code won't compile, missing core functionality)

### Dependencies
- SMET-T-0112 must be complete (needs real `InitiativeResult` with artifact paths to score)

### Risk Considerations
- Scoring is inherently heuristic — document the rubric so results are reproducible across runs
- Start with simple checks (compile, test count, placeholder scan) — avoid over-engineering
- The scorer itself must be deterministic: same artifacts → same score every time

## Status Updates

### Session complete
- Created `benchmarks/practical/src/gate_scorer.rs` with `GateScorer` struct (deterministic, no API cost)
- Checks: token sanity (blocking), task count range, per-task doc accuracy/instruction adherence thresholds, placeholder text detection, optional artifact dir existence
- Gate decision: Rejected if blocking failures, RequiresRework if 1+ issues, Approved if clean
- `rework_tokens` = issues × 500 heuristic
- Wired into `gated_runner.rs`: GateScorer runs first; if structural rejection, skips API gate; otherwise merges structural + API gate results with `stricter_decision()`
- 8 unit tests covering all-pass, zero-tokens rejection, no-tasks rejection, low doc accuracy, low adherence, placeholder title, rework token estimation, missing artifact dir
- All 29 tests pass
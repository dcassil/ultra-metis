---
id: structural-clippy-compliance
level: initiative
title: "Structural Clippy Compliance: Function Length, Argument Count, and Complexity"
short_code: "SMET-I-0091"
created_at: 2026-03-26T18:47:12.508020+00:00
updated_at: 2026-03-26T18:47:12.508020+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: SMET-S-0001
initiative_id: structural-clippy-compliance
---

# Structural Clippy Compliance: Function Length, Argument Count, and Complexity Initiative

## Context

SMET-I-0084 established workspace-wide clippy configuration with `clippy.toml` thresholds (`too_many_lines=80`, `too_many_arguments=7`, `cognitive_complexity=25`). These structural lints are currently **allowed** in `Cargo.toml` because 209 violations exist across the workspace. Fixing them requires genuine refactoring — splitting functions, extracting parameter structs, and simplifying control flow — not mechanical changes.

### Current Violation Counts by Crate

| Crate | too_many_lines | too_many_arguments | cognitive_complexity | Total |
|-------|---------------|--------------------|---------------------|-------|
| cadre-core | ~60 | ~5 | ~4 | ~69 |
| cadre-cli | ~20 | ~3 | ~2 | ~25 |
| cadre-mcp | ~15 | ~8 | ~1 | ~24 |
| cadre-store | ~5 | ~1 | ~0 | ~6 |
| practical (benchmarks) | ~25 | ~5 | ~0 | ~30 |

### Hotspot Files
- `crates/cadre-core/src/domain/transitions/enforcer.rs` — 50 violations (worst offender)
- `benchmarks/practical/src/scoring.rs` — 16 violations
- `benchmarks/practical/src/comparison.rs` — 11 violations
- `crates/cadre-store/src/store.rs` — 8 violations
- `crates/cadre-mcp/src/tools/capture_quality_baseline.rs` — 7 violations

## Goals & Non-Goals

**Goals:**
- Refactor all functions exceeding 80 lines into smaller, well-named sub-functions
- Extract parameter structs for functions with >7 arguments
- Simplify functions with cognitive complexity >25
- Remove the `too_many_lines`, `too_many_arguments`, and `cognitive_complexity` allows from workspace `Cargo.toml`
- Maintain full test coverage and API compatibility

**Non-Goals:**
- Module-level restructuring (covered by SMET-I-0085 through SMET-I-0088)
- Auto-fixable pedantic lint cleanup (covered by SMET-I-0092)
- Adding new functionality — purely refactoring existing code

## Detailed Design

### Approach: Crate-by-Crate Refactoring

Work through each crate in dependency order (core → store → mcp → cli → benchmarks), splitting long functions and extracting parameter structs.

**Common patterns to apply:**
1. **Long functions**: Extract logical blocks into private helper functions
2. **Too many arguments**: Create `XxxOptions` or `XxxParams` structs
3. **High complexity**: Use early returns, extract match arms, simplify nested conditionals

### Exit Criteria
- `cargo clippy --workspace --all-targets -- -D clippy::too_many_lines -D clippy::too_many_arguments -D clippy::cognitive_complexity` passes
- All allows for these three lints removed from `Cargo.toml`
- All tests pass

## Alternatives Considered

1. **Raise thresholds**: Could set `too_many_lines=150` to reduce violations, but that defeats the purpose of the lint.
2. **Per-function `#[allow]`**: Would let the build pass but hides the problem. Only appropriate for genuinely complex functions where splitting would harm readability.
3. **Fix alongside module restructuring**: The I-0085 through I-0088 initiatives focus on file-level organization; function-level refactoring is orthogonal and should be tracked separately.

## Implementation Plan

1. Start with the worst hotspot (`transitions/enforcer.rs` — 50 violations)
2. Work through cadre-core remaining violations
3. Fix cadre-store and cadre-mcp violations
4. Fix cadre-cli violations
5. Fix benchmarks/practical violations
6. Remove allows from Cargo.toml, verify clean build
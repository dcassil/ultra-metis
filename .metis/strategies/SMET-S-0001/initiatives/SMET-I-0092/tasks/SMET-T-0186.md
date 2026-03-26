---
id: auto-fixable-pedantic-lint-batch
level: task
title: "Auto-Fixable Pedantic Lint Batch"
short_code: "SMET-T-0186"
created_at: 2026-03-26T19:24:22.747754+00:00
updated_at: 2026-03-26T19:27:05.535225+00:00
parent: SMET-I-0092
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0092
---

# Auto-Fixable Pedantic Lint Batch

## Parent Initiative

[[SMET-I-0092]] — Bulk Pedantic Lint Cleanup

## Objective

Remove the `= "allow"` entries from `Cargo.toml` for all lints that `cargo clippy --fix` can auto-fix, then verify the build and tests pass. This batch covers ~1,094 total violations across 7 lints.

## Lints to Address

All lints in this batch are auto-fixable via `cargo clippy --fix`.

| Lint | Hits | What the fix does |
|------|------|-------------------|
| `use_self` | 560 | Replace explicit type name with `Self` inside impl blocks |
| `uninlined_format_args` | 373 | `format!("{}", x)` becomes `format!("{x}")` |
| `redundant_closure_for_method_calls` | 82 | `.map(\|x\| x.foo())` becomes `.map(Type::foo)` |
| `manual_let_else` | 49 | `if let Some(x) = expr { x } else { return }` becomes `let Some(x) = expr else { return }` |
| `cast_lossless` | 28 | `x as u64` becomes `u64::from(x)` for safe widening casts |
| `needless_raw_string_hashes` | 11 | Remove unnecessary `#` from raw string literals |
| `unnest_or_patterns` | 11 | Flatten `A \| (B \| C)` to `A \| B \| C` in match arms |

## Approach

For each lint (one at a time, in the order listed above):

1. Remove the `= "allow"` line from `[workspace.lints.clippy]` in `Cargo.toml`
2. Run `cargo clippy --fix --workspace --all-targets --allow-dirty --allow-staged`
3. Review the diff to confirm the fixes are correct
4. Run `cargo clippy --workspace --all-targets -- -D warnings` to verify no remaining violations
5. Run `cargo test --workspace` to verify no regressions
6. If any auto-fix produces incorrect code, manually fix or add a local `#[allow(clippy::lint_name)]` with a comment explaining why
7. Commit the changes for that lint

## Acceptance Criteria

## Acceptance Criteria

- [ ] `use_self` allow removed from `Cargo.toml` and all violations fixed
- [ ] `uninlined_format_args` allow removed and all violations fixed
- [ ] `redundant_closure_for_method_calls` allow removed and all violations fixed
- [ ] `manual_let_else` allow removed and all violations fixed
- [ ] `cast_lossless` allow removed and all violations fixed
- [ ] `needless_raw_string_hashes` allow removed and all violations fixed
- [ ] `unnest_or_patterns` allow removed and all violations fixed
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes with all 7 allows removed
- [ ] `cargo test --workspace` passes with no regressions

## Implementation Notes

### Technical Approach
Run `cargo clippy --fix` one lint at a time. The `--allow-dirty` and `--allow-staged` flags are needed since we edit `Cargo.toml` before running the fix. Process lints from highest hit count to lowest so the biggest wins come first.

### Risk Considerations
- `redundant_closure_for_method_calls` can sometimes produce incorrect fixes when closures capture variables — review the diff carefully
- `manual_let_else` auto-fix may produce code that does not compile if the diverging branch is complex — manual fixup may be needed
- `use_self` at 560 hits is the largest batch; review a sample of the changes to confirm correctness

## Status Updates

*To be added during implementation*
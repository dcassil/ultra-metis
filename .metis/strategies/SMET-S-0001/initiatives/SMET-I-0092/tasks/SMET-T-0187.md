---
id: mechanical-review-pedantic-lint
level: task
title: "Mechanical Review Pedantic Lint Batch"
short_code: "SMET-T-0187"
created_at: 2026-03-26T19:24:23.774931+00:00
updated_at: 2026-03-26T19:24:23.774931+00:00
parent: SMET-I-0092
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0092
---

# Mechanical Review Pedantic Lint Batch

## Parent Initiative

[[SMET-I-0092]] — Bulk Pedantic Lint Cleanup

## Objective

Address pedantic/nursery lints that are NOT auto-fixable and require per-instance judgment. For each violation, either apply the fix or add a local `#[allow]` with a justification comment. This batch covers ~694 total violations across 5 lints.

## Lints to Address

These lints require human judgment for each instance. They cannot be blindly auto-fixed.

| Lint | Hits | Fix approach |
|------|------|--------------|
| `missing_const_for_fn` | 207 | Review each function: mark `const` if all operations are const-compatible, otherwise add local `#[allow]` with reason (e.g., "calls non-const fn", "will change in future") |
| `must_use_candidate` | 457 | Add `#[must_use]` to pure functions that compute a value. Skip for functions with side effects or where ignoring the return value is common. Add local `#[allow]` with reason for skipped cases |
| `needless_pass_by_value` | 8 | Check if the function actually needs ownership. Convert to `&T` or `&str` where appropriate. Keep by-value if ownership transfer is intentional (e.g., builder pattern, Into impls) |
| `unnecessary_wraps` | 7 | Check if the `Option`/`Result` wrapper is needed for trait conformance or future-proofing. Unwrap where the wrapper is truly unnecessary |
| `derive_partial_eq_without_eq` | 15 | Add `Eq` derive where the type truly supports total equality. Keep without `Eq` for types containing floats or other non-Eq fields |

## Approach

For each lint (one at a time, in the order listed above):

1. Remove the `= "allow"` line from `[workspace.lints.clippy]` in `Cargo.toml`
2. Run `cargo clippy --workspace --all-targets 2>&1` to get the full list of violations
3. Review each violation individually:
   - If the fix is appropriate: apply it
   - If the lint is a false positive or the current code is intentional: add `#[allow(clippy::lint_name)]` with a `// Reason: ...` comment
4. Run `cargo clippy --workspace --all-targets -- -D warnings` to verify no remaining violations
5. Run `cargo test --workspace` to verify no regressions
6. Commit the changes for that lint

## Acceptance Criteria

- [ ] `missing_const_for_fn` allow removed from `Cargo.toml` and all 207 violations resolved (fixed or locally allowed with justification)
- [ ] `must_use_candidate` allow removed and all 457 violations resolved
- [ ] `needless_pass_by_value` allow removed and all 8 violations resolved
- [ ] `unnecessary_wraps` allow removed and all 7 violations resolved
- [ ] `derive_partial_eq_without_eq` allow removed and all 15 violations resolved
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes with all 5 allows removed
- [ ] `cargo test --workspace` passes with no regressions
- [ ] Every local `#[allow]` has an accompanying comment explaining why

## Implementation Notes

### Technical Approach
Work through lints from lowest hit count to highest to build momentum. For `must_use_candidate` (457 hits), consider using a heuristic: if the function name starts with `get_`, `is_`, `has_`, `to_`, `as_`, or `into_`, it is almost certainly a `#[must_use]` candidate. Functions that primarily perform side effects (write, log, emit) should get local `#[allow]`.

### Dependencies
Should be done after SMET-T-0186 (Auto-Fixable Batch) to avoid merge conflicts in `Cargo.toml`.

### Risk Considerations
- `missing_const_for_fn` is a nursery lint and can have false positives — some functions may not be const-eligible despite clippy's suggestion
- `must_use_candidate` at 457 hits is the largest batch and most judgment-intensive; consider doing it last within this task
- Adding `#[must_use]` to public API functions is a semver-compatible change but may produce warnings for downstream consumers

## Status Updates

*To be added during implementation*
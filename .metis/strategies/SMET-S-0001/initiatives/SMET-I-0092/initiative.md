---
id: bulk-pedantic-lint-cleanup-auto
level: initiative
title: "Bulk Pedantic Lint Cleanup: Auto-Fixable and Mechanical Violations"
short_code: "SMET-I-0092"
created_at: 2026-03-26T18:47:18.120902+00:00
updated_at: 2026-03-26T20:49:38.150160+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0001
initiative_id: bulk-pedantic-lint-cleanup-auto
---

# Bulk Pedantic Lint Cleanup: Auto-Fixable and Mechanical Violations Initiative

## Context

SMET-I-0084 enabled `clippy::pedantic` and `clippy::nursery` at the workspace level but had to **allow** many high-volume lints to keep the build green. Several of these are auto-fixable via `cargo clippy --fix` or require only mechanical edits (no judgment calls). This initiative systematically removes those allows one at a time, applying fixes and verifying the build.

### Allowed Lints by Fix Approach

**Auto-fixable (`cargo clippy --fix`):**
| Lint | Hits | Description |
|------|------|-------------|
| `use_self` | 560 | Use `Self` instead of explicit type name in impl blocks |
| `uninlined_format_args` | 373 | `format!("{}", x)` → `format!("{x}")` |
| `needless_raw_string_hashes` | 11 | Remove unnecessary `r#` hashes |
| `unnest_or_patterns` | 11 | Flatten nested or-patterns in match |
| `redundant_closure_for_method_calls` | 82 | `.map(|x| x.foo())` → `.map(Type::foo)` |
| `manual_let_else` | 49 | `if let` + return → `let ... else` |

**Case-by-case review (not auto-fixable):**
| Lint | Hits | Description |
|------|------|-------------|
| `must_use_candidate` | 457 | Add `#[must_use]` to pure functions |
| `missing_errors_doc` | 253 | Document `# Errors` section on fallible fns |
| `missing_panics_doc` | 51 | Document `# Panics` section |
| `doc_markdown` | 127 | Backtick-wrap identifiers in docs |
| `missing_const_for_fn` | 207 | Mark eligible functions as `const` |

## Goals & Non-Goals

**Goals:**
- Systematically enable allowed pedantic/nursery lints, starting with auto-fixable ones
- Remove each allow from `Cargo.toml` once all violations are fixed
- Reduce the allow list to only genuinely controversial style choices

**Non-Goals:**
- Structural refactoring (covered by SMET-I-0091)
- Module restructuring (covered by SMET-I-0085 through SMET-I-0088)
- Perfect docs (the doc lints can be a later phase if desired)

## Detailed Design

### Approach: One Lint at a Time

For each lint in the allow list:
1. Remove the `= "allow"` line from `Cargo.toml`
2. Run `cargo clippy --fix` if auto-fixable, or manually fix
3. Verify `cargo clippy --workspace --all-targets -- -D warnings` passes
4. Verify `cargo test --workspace` passes
5. Commit

### Recommended Order
1. **Auto-fixable first** (lowest risk, highest reward): `use_self` → `uninlined_format_args` → `needless_raw_string_hashes` → `unnest_or_patterns` → `redundant_closure_for_method_calls` → `manual_let_else`
2. **Mechanical review** (moderate effort): `missing_const_for_fn` → `must_use_candidate`
3. **Docs lints** (optional, high effort): `missing_errors_doc` → `missing_panics_doc` → `doc_markdown`

## Alternatives Considered

1. **Fix everything at once**: Too risky — a single bad autofix could break tests and be hard to bisect.
2. **Leave all allows permanently**: Defeats the purpose of enabling pedantic.
3. **Remove pedantic entirely**: Loses the value of the enforced lints we do want.

## Implementation Plan

1. Create one task per lint group (auto-fixable batch, mechanical review batch, docs batch)
2. Execute auto-fixable batch first — ~1100 changes but all mechanical
3. Execute mechanical review batch — requires judgment per function
4. Docs batch is optional/deferred based on priorities
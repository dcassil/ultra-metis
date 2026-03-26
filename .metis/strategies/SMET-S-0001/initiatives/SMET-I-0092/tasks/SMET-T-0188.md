---
id: style-and-docs-pedantic-lint-batch
level: task
title: "Style and Docs Pedantic Lint Batch"
short_code: "SMET-T-0188"
created_at: 2026-03-26T19:24:24.387137+00:00
updated_at: 2026-03-26T20:13:41.134817+00:00
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

# Style and Docs Pedantic Lint Batch

## Parent Initiative

[[SMET-I-0092]] — Bulk Pedantic Lint Cleanup

## Objective

Review all remaining pedantic/nursery allow entries in `Cargo.toml` that were not covered by the auto-fixable or mechanical review batches. For each lint, either fix the violations and remove the allow, or document why the allow should remain permanently. The goal is to reduce the allow list to a curated, justified set of permanent exceptions.

## Lints to Address

These are lower-priority style preferences, documentation lints, and nursery lints. Many are intentional style choices that may be kept as permanent allows.

### Documentation Lints
| Lint | Hits | Notes |
|------|------|-------|
| `missing_errors_doc` | 253 | Requires `# Errors` section on every fallible public fn |
| `doc_markdown` | 127 | Requires backtick-wrapping identifiers in doc comments |
| `missing_panics_doc` | 51 | Requires `# Panics` section on fns that can panic |
| `too_long_first_doc_paragraph` | 3 | First doc paragraph should be concise |

### Style Preference Lints
| Lint | Hits | Notes |
|------|------|-------|
| `module_name_repetitions` | 560 | e.g., `document::DocumentType` — common in domain-driven code |
| `items_after_statements` | - | Style preference: `let` bindings before items |
| `similar_names` | 31 | e.g., `node`/`nodes` — false positives on domain terms |
| `if_not_else` | - | `if !x` is often clearer than `if x {} else {}` |
| `redundant_else` | - | Style preference |
| `single_match_else` | - | `match` with one arm + else |
| `match_wildcard_for_single_variants` | - | Style preference |
| `range_plus_one` | - | `0..n+1` vs `0..=n` |
| `new_without_default` | - | Domain types where `Default` semantics don't apply |
| `manual_string_new` | - | `String::from("")` vs `String::new()` |
| `enum_glob_use` | 2 | `use Enum::*` in match arms |
| `match_same_arms` | 24 | Sometimes intentional for clarity |

### Cast and Numeric Lints
| Lint | Hits | Notes |
|------|------|-------|
| `cast_possible_truncation` | 38 | Needs case-by-case audit |
| `cast_sign_loss` | - | Safe casts in our context (lengths, counts) |
| `cast_precision_loss` | - | Safe casts in our context |
| `cast_possible_wrap` | 16 | Safe in our u32/i64 context |
| `unreadable_literal` | - | Numeric literals in thresholds/scores |
| `float_cmp` | 28 | `assert_eq!` on f64 in tests |
| `trivially_copy_pass_by_ref` | - | Small Copy types passed by ref |

### Nursery Lints
| Lint | Hits | Notes |
|------|------|-------|
| `option_if_let_else` | 56 | Sometimes less readable than if-let |
| `format_push_string` | 77 | `push_str` vs `write!` style preference |
| `unused_async` | 62 | Async fns needed for trait conformance |
| `stable_sort_primitive` | 6 | `sort()` vs `sort_unstable()` |
| `needless_collect` | 5 | Sometimes needed for borrow checker |
| `regex_creation_in_loops` | 4 | Fix by hoisting regex to lazy_static or once_cell |
| `unnecessary_literal_bound` | 4 | Nursery |
| `map_unwrap_or` | - | Sometimes less readable |
| `redundant_pub_crate` | - | Visibility is intentional |
| `significant_drop_tightening` | - | False positives |
| `suboptimal_flops` | - | Not applicable to our math |
| `useless_let_if_seq` | - | Sometimes clearer with let + if |
| `unnecessary_map_or` | - | Readability preference |
| `unnecessary_debug_formatting` | - | Nursery |
| `missing_fields_in_debug` | - | Nursery |
| `trim_split_whitespace` | - | Nursery |
| `future_not_send` | - | Async fns with non-Send refs in benchmarks |
| `if_same_then_else` | 2 | Sometimes intentional |

### Other
| Lint | Hits | Notes |
|------|------|-------|
| `return_self_not_must_use` | - | Builder pattern types |
| `struct_excessive_bools` | - | Domain types with boolean flags |
| `case_sensitive_file_extension_comparisons` | 5 | Known extensions only |
| `used_underscore_binding` | 6 | Intentional unused markers |
| `unused_self` | 6 | Methods that take `&self` for API consistency |
| `string_lit_as_bytes` | - | `b"..."` vs `"...".as_bytes()` |

## Approach

1. **Triage each lint** into one of three categories:
   - **Fix**: The lint catches real issues; fix violations and remove the allow
   - **Permanent allow with comment**: The lint conflicts with our style or domain conventions; keep the allow and add a comment like `# PERMANENT: reason`
   - **Defer**: The lint requires significant effort (e.g., docs lints with 250+ hits); leave for a future initiative

2. For lints categorized as "Fix":
   - Remove the allow, fix violations (auto-fix where possible, manual otherwise)
   - Verify clippy and tests pass
   - Commit

3. For lints categorized as "Permanent allow":
   - Update the comment in `Cargo.toml` to indicate the allow is intentional and permanent
   - Group permanent allows into a clearly labeled section

4. For "Defer" lints:
   - Leave the allow as-is but update the comment with context for the future

## Acceptance Criteria

## Acceptance Criteria

- [ ] Every remaining allow in `Cargo.toml` has been triaged as Fix, Permanent, or Defer
- [ ] All "Fix" lints have been resolved and their allows removed
- [ ] All "Permanent" allows have a `# PERMANENT:` comment explaining why
- [ ] All "Defer" allows have an updated comment with context
- [ ] `Cargo.toml` lint section is reorganized into clear groups: Enforced, Permanent Allows, Deferred
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes with no regressions

## Implementation Notes

### Technical Approach
Start by triaging all lints (a quick pass through the full list with categorization decisions). Then execute fixes for the "Fix" category. Finally, reorganize the `Cargo.toml` lint section for clarity.

### Dependencies
Should be done after SMET-T-0186 (Auto-Fixable Batch) and SMET-T-0187 (Mechanical Review Batch) to avoid merge conflicts.

### Risk Considerations
- This task involves many judgment calls about style — document the rationale for each decision
- The documentation lints (`missing_errors_doc`, `missing_panics_doc`, `doc_markdown`) represent 400+ violations and are likely "Defer" candidates unless docs cleanup is a priority
- Some nursery lints may graduate to pedantic in future clippy versions, changing the calculus

## Status Updates

*To be added during implementation*
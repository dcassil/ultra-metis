---
id: replace-metis-references-with
level: task
title: "Replace Metis references with Cadre in cadre-store/store.rs"
short_code: "SMET-T-0219"
created_at: 2026-03-27T19:43:37.842565+00:00
updated_at: 2026-03-27T19:43:37.842565+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Replace Metis references with Cadre in cadre-store/store.rs

## Objective

Audit `crates/cadre-store/src/store.rs` for any remaining Metis references (tool names, type names, comments, string literals, etc.) and replace them with the correct Cadre equivalents. Also compare against the architecture repo to check for other drift.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (nice to have)

### Technical Debt Impact
- **Current Problems**: Leftover Metis naming in Cadre codebase creates confusion and inconsistency after the rename (ADR SMET-A-0001)
- **Benefits of Fixing**: Clean, consistent naming throughout the codebase
- **Risk Assessment**: Low — straightforward find-and-replace, but must verify all callers and tests still pass

## Acceptance Criteria

- [ ] Identify all Metis references in `store.rs` (grep for metis, Metis, METIS)
- [ ] Replace with correct Cadre equivalents (cadre, Cadre, CADRE)
- [ ] Compare against architecture repo for additional drift
- [ ] All tests pass after changes
- [ ] No broken imports or references in dependent crates

## Implementation Notes

### Files to Investigate
- `crates/cadre-store/src/store.rs` (primary target)
- Any files that import from or reference `store.rs` types

### Approach
1. Grep for all Metis variants in the file
2. Replace with Cadre equivalents, preserving casing conventions
3. Check for downstream breakage in cadre-core, cadre-mcp, cadre-cli
4. Run `make test` to verify

## Status Updates

*To be added during implementation*
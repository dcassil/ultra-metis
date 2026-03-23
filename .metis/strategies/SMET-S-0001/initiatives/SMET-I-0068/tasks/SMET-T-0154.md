---
id: add-architecture-variant-to
level: task
title: "Add Architecture variant to DocumentType enum with AR prefix and empty transitions"
short_code: "SMET-T-0154"
created_at: 2026-03-23T15:53:04.143058+00:00
updated_at: 2026-03-23T16:40:44.934924+00:00
parent: SMET-I-0068
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0068
---

# Add Architecture variant to DocumentType enum with AR prefix and empty transitions

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[SMET-I-0068]]

## Objective

Add the `Architecture` document type variant to the `DocumentType` enum in cadre-core with "AR" short code prefix, empty transitions (always Published), and add unit tests to verify the behavior. This variant represents top-level architecture documents that are published once and not transitioned through phases.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `Architecture` variant exists in `DocumentType` enum in types.rs
- [ ] `short_code_prefix()` returns "AR" for `DocumentType::Architecture`
- [ ] `valid_transitions_from()` returns empty vec for all phases (no transitions)
- [ ] `phase_sequence()` returns `vec![Phase::Published]`
- [ ] `is_governance_type()` returns true for Architecture
- [ ] `is_cadre_type()` returns false for Architecture
- [ ] `is_legacy_type()` returns false for Architecture
- [ ] Display impl outputs "architecture"
- [ ] FromStr impl parses "architecture" correctly
- [ ] Unit tests cover all of the above behaviors
- [ ] `make test` passes with no failures



## Implementation Notes

### Technical Approach
The `Architecture` variant is added to the `DocumentType` enum in `crates/cadre-core/src/domain/documents/types.rs`. It follows the pattern of a governance type that is always in the Published phase with no transitions. The variant needs entries in all match arms: Display, FromStr, short_code_prefix ("AR"), valid_transitions_from (empty vec), phase_sequence ([Published]), and is_governance_type (true). Unit tests are added to verify all behaviors.

### File Changed
- `crates/cadre-core/src/domain/documents/types.rs` - enum variant, all impl blocks, and tests

## Status Updates **[REQUIRED]**

*To be added during implementation*
---
id: add-architecture-variant-to
level: task
title: "Add Architecture variant to DocumentType enum with AR prefix"
short_code: "SMET-T-0153"
created_at: 2026-03-23T15:31:25.250491+00:00
updated_at: 2026-03-23T15:31:58.511296+00:00
parent: SMET-I-0068
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0068
---

# Add Architecture variant to DocumentType enum with AR prefix

## Objective

Add the `Architecture` variant to the `DocumentType` enum in `crates/cadre-core/src/domain/documents/types.rs` so the type system recognizes Architecture as a first-class document type. This includes short_code_prefix ("AR"), valid_transitions_from (empty — Architecture documents are always Published), phase_sequence (vec![Phase::Published]), is_governance_type (true), and Display/FromStr implementations for "architecture".

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `DocumentType::Architecture` variant exists in the enum
- [ ] `short_code_prefix()` returns "AR"
- [ ] `valid_transitions_from()` returns empty vec for all phases
- [ ] `phase_sequence()` returns `vec![Phase::Published]`
- [ ] `is_governance_type()` returns true for Architecture
- [ ] Display formats as "architecture"
- [ ] FromStr parses "architecture" correctly
- [ ] All existing tests pass, new unit tests added

## Implementation Notes

### Technical Approach
- Add `Architecture` variant to the DocumentType enum
- Add match arm to every existing match block: Display, FromStr, short_code_prefix, valid_transitions_from, phase_sequence, is_governance_type
- Architecture is unique: it has NO valid transitions (always Published), so valid_transitions_from returns empty vec for all phases
- phase_sequence returns just vec![Phase::Published]
- No dependencies on other tasks

### File
`crates/cadre-core/src/domain/documents/types.rs`

## Status Updates

*Starting implementation*
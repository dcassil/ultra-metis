---
id: add-architecture-types-to
level: task
title: "Add architecture types to DocumentType enum, factory, exports, and tests"
short_code: "SMET-T-0015"
created_at: 2026-03-16T21:12:19.148926+00:00
updated_at: 2026-03-16T21:18:57.125278+00:00
parent: SMET-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: SMET-I-0020
---

# Add architecture types to DocumentType enum, factory, exports, and tests

## Objective

Wire up the two new architecture types into the broader type system: add DocumentType enum variants, update the factory for polymorphic loading, add lib.rs exports, update the documents mod.rs, and write comprehensive integration/unit tests for both types.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] DocumentType enum: add ArchitectureCatalogEntry and ReferenceArchitecture variants with short code prefixes (AC, RA)
- [ ] DocumentType methods updated: FromStr, Display, is_cadre_type, short_code_prefix, phase sequences
- [ ] documents/mod.rs: add module declarations for both types
- [ ] lib.rs: export both types and any new supporting enums (ArchitectureStatus, ArchitectureSource, LayerDefinition, ModuleBoundary)
- [ ] Comprehensive tests: type creation, round-trip serialization, field validation, catalog-linked vs derived modes, governance linkage
- [ ] All existing tests still pass (cargo test)

## Implementation Notes

### Files to Modify
- `src/domain/documents/types.rs` — add enum variants, FromStr/Display, phase config
- `src/domain/documents/mod.rs` — add module declarations
- `src/lib.rs` — add exports

### Test Coverage
- Construction with default/custom fields
- Markdown+frontmatter round-trip (to_content -> from_content)
- File round-trip (to_file -> from_file)
- All enum variants parse correctly
- Governance linkage fields serialize/deserialize
- Edge cases: empty arrays, None optional fields, "NULL" handling

## Progress

*Updated during implementation*
---
id: implement-referencearchitecture
level: task
title: "Implement ReferenceArchitecture domain type"
short_code: "SMET-T-0014"
created_at: 2026-03-16T21:12:18.196606+00:00
updated_at: 2026-03-16T21:18:56.382315+00:00
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

# Implement ReferenceArchitecture domain type

## Objective

Implement the ReferenceArchitecture domain type following the governance type pattern. This type represents the selected or derived architecture for a specific repo — one per repo (or per workspace in a monorepo). It links to a catalog entry (if matched) or contains a derived pattern, and holds references to RulesConfig and AnalysisBaseline that were seeded from it.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] ReferenceArchitecture struct with fields: source (linked catalog entry ref or "derived" flag), customizations (overrides on top of catalog entry), governance linkage (rules_config_ref, analysis_baseline_ref), status (ArchitectureStatus enum: Draft/Active/Superseded)
- [ ] ArchitectureStatus enum with Draft, Active, Superseded variants
- [ ] ArchitectureSource enum: CatalogLinked(String) vs Derived
- [ ] Customization fields: layer overrides, additional boundaries, extra rules
- [ ] Frontmatter YAML template (frontmatter.yaml)
- [ ] Content markdown template (content.md)
- [ ] from_file/to_file serialization matching governance type patterns
- [ ] from_content/to_content for in-memory round-trip
- [ ] Unit tests for creation, serialization round-trip, catalog-linked vs derived modes

## Implementation Notes

### Pattern to Follow
Same governance type pattern as SMET-T-0013:
- Module at `src/domain/documents/reference_architecture/mod.rs`
- Templates at `src/domain/documents/reference_architecture/frontmatter.yaml` and `content.md`
- Uses DocumentCore for shared fields

### Key Fields
- `core: DocumentCore`
- `source_catalog_ref: Option<String>` (short code of linked ArchitectureCatalogEntry, None if derived)
- `is_derived: bool` (true when no catalog match)
- `status: ArchitectureStatus` (Draft/Active/Superseded)
- `layer_overrides: Vec<String>` (customizations on top of catalog entry)
- `additional_boundaries: Vec<String>` (extra module boundaries)
- `extra_dependency_rules: Vec<String>` (additional dependency constraints)
- `rules_config_ref: Option<String>` (ref to seeded RulesConfig)
- `analysis_baseline_ref: Option<String>` (ref to seeded AnalysisBaseline)
- `tolerated_exceptions: Vec<String>` (known deviations from the pattern)

## Progress

*Updated during implementation*
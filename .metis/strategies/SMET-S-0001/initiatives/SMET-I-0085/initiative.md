---
id: cadre-core-file-size-reduction-and
level: initiative
title: "cadre-core: File Size Reduction and Module Restructuring"
short_code: "SMET-I-0085"
created_at: 2026-03-26T17:21:34.639989+00:00
updated_at: 2026-03-26T17:21:34.639989+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: XL
strategy_id: SMET-S-0001
initiative_id: cadre-core-file-size-reduction-and
---

# cadre-core: File Size Reduction and Module Restructuring Initiative

## Context

cadre-core is the largest crate in the workspace: **~25K LOC across 105+ Rust files**. It contains all domain logic including 30+ document types, governance, quality, transitions, bootstrap, catalog, remediation, rules, templates, and operations.

The crate has grown organically and now has significant structural debt:
- **60+ files exceed 100 LOC**, with many in the 500-1200 line range
- Each document type lives in a **monolithic `mod.rs`** combining struct definition, all impls, Display, serialization, template generation, and tests in a single file
- `templates/mod.rs` is **1,090 lines** of string templates for all document types mixed together
- `types.rs` is **919 lines** of shared document types and enums
- Tests are embedded inline rather than in separate files

### File Size Distribution (Top Offenders)

| File | LOC | Contents |
|------|-----|----------|
| `documents/durable_insight_note/mod.rs` | 1,214 | Struct + impl + tests |
| `templates/mod.rs` | 1,090 | Templates for ALL document types |
| `documents/cross_reference/mod.rs` | 929 | Struct + impl + tests |
| `documents/types.rs` | 919 | Shared enums, DocumentType, Phase types |
| `documents/architecture/mod.rs` | 904 | Struct + impl + tests |
| `documents/execution_record/mod.rs` | 890 | Struct + impl + tests |
| `transitions/audit.rs` | 828 | Audit log types + impl + tests |
| `documents/quality_gate_config/mod.rs` | 816 | Struct + impl + tests |
| `documents/rule_change_proposal/mod.rs` | 805 | Struct + impl + tests |
| `bootstrap/monorepo_detector.rs` | 799 | Detector + tests |
| `transitions/traceability.rs` | 768 | Query engine + tests |
| `documents/reference_architecture/mod.rs` | 733 | Struct + impl + tests |
| `bootstrap/repo_scanner.rs` | 723 | Scanner + tests |
| `catalog/brownfield_evaluator/evaluator.rs` | 700 | Evaluator + tests |
| `quality/gate_engine.rs` | 698 | Engine + tests |
| `documents/epic/mod.rs` | 665 | Struct + impl + tests |
| `governance/escalation.rs` | 650 | Types + impl + tests |
| `documents/gate_override/mod.rs` | 644 | Struct + impl + tests |
| Plus **40+ more files** over 100 LOC | | |

### Current Module Organization

```
domain/
├── bootstrap/         (4 files, ~2,732 LOC)
├── catalog/           (5 files + submodules, ~2,500 LOC)
├── documents/         (30+ subdirectories, each with mod.rs, ~17,000 LOC)
├── governance/        (4 files, ~2,474 LOC)
├── operations/        (4 files, ~1,500 LOC)
├── quality/           (8 files + parsers/, ~2,500 LOC)
├── remediation/       (5 files, ~1,500 LOC)
├── rules/             (4 files, ~1,400 LOC)
├── templates/         (1 file at 1,090 LOC)
└── transitions/       (6 files, ~3,200 LOC)
```

## Goals & Non-Goals

**Goals:**
- Split every file over 100 LOC into focused sub-files targeting ~100 LOC each
- Extract tests from domain types into separate `tests.rs` files
- Split `templates/mod.rs` (1,090 LOC) into per-domain template files
- Split `documents/types.rs` (919 LOC) into logical groupings
- Each file should have **one primary export** (struct, enum, or trait)
- Establish clear module boundaries within cadre-core's domain structure
- Maintain public API compatibility (lib.rs re-exports stay the same)

**Non-Goals:**
- Changing the domain model or adding features
- Moving types between crates (that's a separate architectural decision)
- Changing test behavior or adding new tests
- Modifying the public API surface

## Detailed Design

### Pattern: Splitting Monolithic Document Type mod.rs Files

Each document type (e.g., `documents/epic/mod.rs` at 665 LOC) currently contains:
1. Struct definition with field types
2. `impl` block with constructors, getters, and business logic
3. `Display` impl
4. Serialization/deserialization helpers
5. `#[cfg(test)] mod tests` with comprehensive unit tests

**Target structure for each document type:**

```
documents/epic/
├── mod.rs          (~20 LOC - module declarations and re-exports)
├── types.rs        (~50-80 LOC - struct definition, enums specific to this type)
├── impl.rs         (~80-120 LOC - business logic, constructors, Display)
├── serde.rs        (~30-50 LOC - serialization helpers, if complex enough)
└── tests.rs        (~150-300 LOC - all unit tests)
```

For simpler document types (under 200 LOC total), keep struct + impl together but still extract tests:

```
documents/adr/
├── mod.rs          (~15 LOC - module declarations)
├── adr.rs          (~100 LOC - struct + impl + Display)
└── tests.rs        (~150 LOC - tests)
```

### Splitting templates/mod.rs (1,090 LOC)

Currently one massive file with template strings for every document type. Split into:

```
templates/
├── mod.rs              (~30 LOC - re-exports and shared helpers)
├── vision.rs           (~30-50 LOC)
├── initiative.rs       (~80-100 LOC)
├── task.rs             (~40-60 LOC)
├── epic.rs             (~50-70 LOC)
├── story.rs            (~50-70 LOC)
├── adr.rs              (~30-50 LOC)
├── quality_types.rs    (~80-100 LOC - quality gate, record, baseline templates)
├── governance_types.rs (~80-100 LOC - rules config, gate override, etc.)
├── architecture.rs     (~80-100 LOC - catalog entry, reference arch, etc.)
└── operational.rs      (~60-80 LOC - execution record, transition record, etc.)
```

### Splitting documents/types.rs (919 LOC)

This file contains shared enums like `DocumentType`, `Phase`, `StoryType`, etc. Split by concern:

```
documents/
├── types/
│   ├── mod.rs           (~20 LOC - re-exports)
│   ├── document_type.rs (~100 LOC - DocumentType enum and conversions)
│   ├── phases.rs        (~150 LOC - Phase types for each document type)
│   ├── story_types.rs   (~60 LOC - StoryType, story-specific enums)
│   ├── metadata.rs      (~80 LOC - shared metadata types)
│   └── hierarchy.rs     (~100 LOC - parent-child relationship types)
```

### Splitting Large Non-Document Files

Apply same pattern to other oversized files:

| File | LOC | Split Strategy |
|------|-----|----------------|
| `transitions/audit.rs` | 828 | Split into `audit_types.rs` (~100), `audit_log.rs` (~200), `audit_queries.rs` (~200), `tests.rs` (~300) |
| `transitions/traceability.rs` | 768 | Split into `traceability_types.rs`, `query_engine.rs`, `tests.rs` |
| `bootstrap/monorepo_detector.rs` | 799 | Split into `detector.rs`, `patterns.rs`, `tests.rs` |
| `bootstrap/repo_scanner.rs` | 723 | Split into `scanner.rs`, `language_detection.rs`, `tests.rs` |
| `catalog/brownfield_evaluator/evaluator.rs` | 700 | Split into `evaluator.rs`, `scoring.rs`, `tests.rs` |
| `quality/gate_engine.rs` | 698 | Split into `engine.rs`, `checks.rs`, `tests.rs` |
| `governance/escalation.rs` | 650 | Split into `types.rs`, `detection.rs`, `tests.rs` |
| `governance/autonomy.rs` | 613 | Split into `config.rs`, `modes.rs`, `tests.rs` |

### Module Boundary Rules for cadre-core

1. **`domain/documents/`**: Each document type is a self-contained module. Cross-document-type imports go through `documents::types` or `documents::traits`.
2. **`domain/governance/`**: Consumes document types read-only. No direct dependency on templates or transitions.
3. **`domain/transitions/`**: Depends on documents and governance. Owns phase transition logic.
4. **`domain/quality/`**: Depends on documents. Parsers are leaf modules with no upward dependencies.
5. **`domain/catalog/`**: Depends on documents (ArchitectureCatalogEntry). Brownfield evaluator is self-contained.
6. **`domain/templates/`**: Pure functions returning strings. Depends on document types for parameter types only.
7. **`domain/bootstrap/`**: Depends on catalog for architecture selection. No dependency on transitions or governance.

## Testing Strategy

- **All existing tests must pass** after every split operation
- Tests are moved, not changed - same test names, same assertions
- Run `cargo test --workspace` after each file split
- Verify `lib.rs` re-exports still compile with downstream crates (cadre-store, cadre-cli, cadre-mcp)

## Alternatives Considered

1. **Leave tests inline**: Rejected. Inline tests inflate file size and make it harder to see the ratio of implementation to test code. Rust convention supports both patterns, but separate test files are preferred when files exceed ~100 LOC.
2. **Flatten document types instead of subdirectories**: Rejected. The current subdirectory-per-type pattern is good; it just needs files split within each subdirectory.
3. **Auto-generate document types with macros**: Possible future optimization but adds complexity. Manual splitting is safer and more transparent.

## Implementation Plan

This is an XL initiative due to the number of files (~60+) that need splitting. Recommended decomposition into tasks by domain area:

1. **Templates split** (1 task): Split templates/mod.rs into per-type files
2. **Document types.rs split** (1 task): Split the shared types file
3. **Document types - planning** (1 task per ~5 types): Split vision, initiative, task, epic, story, adr, product_doc, design_context
4. **Document types - governance** (1 task): Split quality_gate_config, rules_config, gate_override, rule_change_proposal, approval_record, constraint_record, validation_policy, ownership_map
5. **Document types - architecture** (1 task): Split architecture, architecture_catalog_entry, reference_architecture, architecture_investigation
6. **Document types - operational** (1 task): Split execution_record, transition_record, decision_record, cross_reference, durable_insight_note, analysis_baseline, quality_record, remediation_record, validation_record, design_change_proposal, specification
7. **Transitions domain** (1 task): Split audit.rs, traceability.rs, enforcer.rs, hooks.rs
8. **Bootstrap domain** (1 task): Split monorepo_detector.rs, repo_scanner.rs, tool_detector.rs, init_flow.rs
9. **Catalog domain** (1 task): Split brownfield_evaluator, builtin_entries, query_engine
10. **Governance domain** (1 task): Split escalation.rs, autonomy.rs, gates.rs
11. **Quality domain** (1 task): Split gate_engine.rs and related files
12. **Rules and remediation** (1 task): Split remaining oversized files
13. **Final verification** (1 task): Full build + test + downstream crate compilation check
---
id: core-planning-hierarchy-domain
level: initiative
title: "Core Planning Hierarchy & Domain Rework"
short_code: "SMET-I-0018"
created_at: 2026-03-11T21:39:39.914810+00:00
updated_at: 2026-03-16T20:52:21.215933+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"
  - "#feature-planning"
  - "#category-domain-model"


exit_criteria_met: false
estimated_complexity: L
strategy_id: cadre-core-engine-repo
initiative_id: core-planning-hierarchy-domain
---

# Core Planning Hierarchy & Domain Rework

## Context

The current Metis domain model uses a general-purpose project management hierarchy (Vision → Strategy → Initiative → Task) with ADRs as a separate concept. CADRE replaces this with an engineering-oriented hierarchy centered on product definition and design-linked implementation: ProductDoc → Epic → Story → Task, with DesignContext as a first-class cross-cutting artifact.

This initiative is the foundational vertical slice — it reworks the core `DocumentType` enum, introduces the new planning types, rewires hierarchy validation, and lands storage support for all of it. Other domain-model initiatives (SMET-I-0019 governance types, SMET-I-0020 architecture catalog types) depend on this being done first.

Split out from the original SMET-I-0001 (now archived) to keep scope manageable as a vertical slice.

## Governing Commitments

- **All durable project memory lives in the repo.** These types define the core planning artifacts that become repo-native persistent state.
- **Planning is durable and traceable from product intent to execution.** The ProductDoc → Epic → Story → Task hierarchy makes traceability a structural property.
- **The system is built around intentional, durable structure.** Strong Rust types with enforced hierarchy rules make it hard to create invalid document relationships.

## Goals & Non-Goals

**Goals:**
- Audit existing Metis domain types and map them to the new target types
- Rework `DocumentType` enum to replace Vision/Strategy/Initiative with ProductDoc/Epic/Story (Task remains)
- Implement ProductDoc, DesignContext, Epic, and Story as Rust domain types with proper traits
- Implement parent-child hierarchy validation for the new type relationships
- Implement Story typing by purpose: feature, bugfix, refactor, migration, architecture-change, investigation, remediation, setup/bootstrap. This allows workflow templates and gate behavior to vary by story type without changing the core hierarchy.
- Implement required planning fields on all artifacts: objective, scope, rationale, acceptance criteria, dependencies, architecture relevance, validation expectations, risk level, status/phase, related artifacts, notes/follow-ups
- Update frontmatter schema to support new fields (design references, story type, etc.)
- Update document ID generation for new type prefixes (PD, DC, E, S)
- Update serialization/deserialization for new types (markdown + YAML frontmatter)
- Update SQLite schema and indexing for new types
- Establish schema versioning foundation for document formats
- Comprehensive unit tests for all new types, validation, and round-trip serialization

**Non-Goals:**
- Governance/quality types (RulesConfig, AnalysisBaseline, etc.) — covered by SMET-I-0019
- Architecture catalog types — covered by SMET-I-0020
- UI for new types — covered by SMET-I-0011
- MCP tool extensions — covered by SMET-I-0009
- Templates for new types — covered by SMET-I-0014
- Migration of existing documents — covered by SMET-I-0015

## Detailed Design

### What to Reuse from `metis/`
- The existing document trait/type system in the Rust codebase
- Markdown + YAML frontmatter serialization/deserialization patterns
- SQLite-backed document indexing and search infrastructure
- File-system layout conventions (directories per document type)
- Tag system and metadata handling

### What to Change
- Extend/replace `DocumentType` enum: add ProductDoc, DesignContext, Epic, Story
- Rework the `level` concept from flight-levels to the new engineering-oriented hierarchy
- Update parent-child relationship validation for the new hierarchy:
  - ProductDoc: top-level (no parent required)
  - Epic: parent must be a ProductDoc
  - Story: parent must be an Epic
  - Task: parent must be a Story or Epic
  - DesignContext: cross-cutting, can be referenced by Epics/Stories
- Update frontmatter schema: add design_context references, story points, acceptance criteria fields
- Update document ID prefixes: PD (ProductDoc), DC (DesignContext), E (Epic), S (Story), T (Task)

### What is Net New
- ProductDoc type: defines product intent, scope, and success criteria
- DesignContext type: captures design decisions, constraints, and references linked to planning artifacts
- Epic type: groups related stories under a product doc
- Story type: implementable unit of work with design linkage
- Schema versioning mechanism for document format evolution
- Updated SQLite schema with columns/indexes for new type relationships

## Alternatives Considered

1. **Keep Vision/Initiative/Task and add new types alongside**: Rejected — creates confusing parallel hierarchies. Better to replace with a unified model.
2. **Generic "document" type with metadata-driven behavior**: Rejected — strong Rust typing gives compile-time guarantees and better maintainability.
3. **Do all domain types in one initiative**: Rejected — too large. Splitting into planning/governance/catalog vertical slices keeps each initiative focused and independently shippable.

## Implementation Plan

Phase 1: Audit existing domain types in `metis/` and map to target types
Phase 2: Define new Rust types and traits for ProductDoc, DesignContext, Epic, Story
Phase 3: Implement hierarchy validation rules
Phase 4: Implement serialization/deserialization for new types
Phase 5: Update SQLite schema and indexing
Phase 6: Add schema versioning foundation
Phase 7: Comprehensive unit tests

## Acceptance Criteria

- All four new planning types (ProductDoc, DesignContext, Epic, Story) defined as Rust types with proper traits
- Hierarchy rules enforced at the domain level (compile-time where possible, runtime where necessary)
- All types round-trip through markdown+frontmatter without data loss
- SQLite schema updated and indexed for new types
- Schema version field present on all documents
- Unit tests cover type creation, validation, serialization, and hierarchy enforcement

## Risks / Dependencies

- This is the foundational initiative; SMET-I-0019 and SMET-I-0020 depend on it
- Must coordinate with SMET-I-0015 (migration) on how existing documents map to new types
- Risk of scope creep into governance types — must stay focused on planning hierarchy only

## Codebase Audit Results

**Completed audit of `metis/crates/cadre-docs-core/src/`**

### Current Architecture (Key Files)
- `domain/documents/types.rs` — DocumentType enum (Vision, Initiative, Task, Adr, Specification), Phase enum (15 phases), DocumentId
- `domain/documents/traits.rs` — Document trait (polymorphic interface), DocumentCore struct
- `domain/documents/metadata.rs` — DocumentMetadata (timestamps, exit_criteria, short_code)
- `domain/documents/content.rs` — DocumentContent (body + acceptance_criteria)
- `domain/documents/factory.rs` — DocumentFactory for polymorphic creation
- `domain/documents/helpers.rs` — FrontmatterParser
- `domain/configuration.rs` — FlightLevelConfig (Streamlined vs Direct presets)
- `dal/database/schema.rs` — Diesel-generated SQLite schema
- `dal/database/models.rs` — Diesel ORM models

### Current Domain Model
- **5 types**: Vision, Initiative, Task, Adr, Specification
- **Short codes**: PREFIX-{V,I,T,A,S}-NNNN
- **Hierarchy**: Vision → Initiative → Task (streamlined) or Vision → Task (direct)
- **Serialization**: Markdown + YAML frontmatter via gray_matter + serde_yaml
- **Templates**: Embedded Tera templates per type (frontmatter.yaml, content.md, acceptance_criteria.md)
- **DB**: SQLite via Diesel with 9 migrations, FTS5 search

### Key Design Decisions from Audit
1. Each document type is a separate module with `from_content()`, `to_content()`, `from_file()`, `to_file()`
2. `DocumentCore` holds shared state; type-specific modules add custom fields
3. `FlightLevelConfig` controls hierarchy flexibility at runtime
4. `initiative_id` field enables lineage tracking independent of parent
5. Short code counters persisted in configuration table
6. Filesystem is source of truth; DB is index rebuilt from filesystem
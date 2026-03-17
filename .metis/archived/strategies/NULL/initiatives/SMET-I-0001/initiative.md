---
id: rework-metis-core-domain-model-for
level: initiative
title: "Rework Metis Core Domain Model for Repo-Native Software Engineering"
short_code: "SMET-I-0001"
created_at: 2026-03-11T19:59:13.998926+00:00
updated_at: 2026-03-11T19:59:13.998926+00:00
parent: SMET-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: XL
strategy_id: NULL
initiative_id: rework-metis-core-domain-model-for
---

# Rework Metis Core Domain Model for Repo-Native Software Engineering

## Context

The current Metis domain model is built around a general-purpose project management hierarchy (Vision → Strategy → Initiative → Task) with ADRs as a separate concept. For Super-Metis, the core domain model must be reworked to center on software engineering in a monorepo context, where the primary concerns are product definition, design-linked implementation, engineering execution, and quality governance.

This initiative is foundational — nearly every other initiative depends on the domain model being reworked first. The existing Metis Rust codebase already has well-structured domain types, document storage, and serialization. The goal is to evolve these, not replace them.

## Governing Commitments

This initiative directly serves:
- **All durable project memory lives in the repo.** The domain model defines every artifact type that becomes repo-native persistent state.
- **Every repo gets a persisted reference architecture.** ArchitectureCatalogEntry and ReferenceArchitecture are first-class domain types, not afterthoughts.
- **Reference architecture drives rules, structure, and analysis** (Vision #4). The Reference Architecture is a control artifact. The domain model must encode the relationships between Reference Architecture, Rules Config, and Analysis Baselines so that enforcement flows structurally from architecture.
- **Planning is durable and traceable from product intent to execution.** The type hierarchy (Product Doc → Epic → Story → Task) with cross-cutting governance artifacts makes traceability a structural property of the domain, not an ad-hoc convention.
- **The system is built around intentional, durable structure.** Strong Rust types with enforced hierarchy rules make it hard to create invalid document relationships — structure is enforced, not suggested.

## Goals & Non-Goals

**Goals:**
- Refactor the core domain types to support the richer Super-Metis document hierarchy (Product Doc, Epic, Story, Task, Design Context, Rules Config, Analysis Baseline, etc.)
- Preserve the file-based, markdown+frontmatter storage model
- Maintain backward compatibility with existing document serialization where possible
- Define clear relationships and hierarchy rules between all new document types
- Ensure the domain model supports cross-cutting governance artifacts (Rules Config, Analysis Baselines) alongside planning artifacts

**Non-Goals:**
- Building UI for new document types (covered by SMET-I-0011)
- Implementing MCP tool extensions (covered by SMET-I-0009)
- Defining specific templates for each type (covered by SMET-I-0014)
- Implementing quality gates or static analysis (covered by SMET-I-0005)

## Detailed Design

### What to Reuse from `metis/`
- The existing document trait/type system in the Rust codebase
- Markdown + YAML frontmatter serialization/deserialization
- SQLite-backed document indexing and search infrastructure
- File-system layout conventions (directories per document type)
- The existing `DocumentType` enum and phase/state machinery
- Tag system and metadata handling

### What to Change from `metis/`
- Extend `DocumentType` enum to include: ProductDoc, DesignContext, Epic, Story, RulesConfig, AnalysisBaseline, QualityRecord, DesignChangeProposal, ArchitectureInvestigation, ArchitectureCatalogEntry, ReferenceArchitecture
- Rework parent-child relationship validation to support the richer hierarchy
- Update the `level` concept from flight-levels (vision/strategy/initiative/task) to the new engineering-oriented hierarchy
- Modify frontmatter schema to support new fields: design references, rule references, quality thresholds, lease information
- Update document ID generation to handle new type prefixes

### What is Net New
- Domain types for: ProductDoc, DesignContext, Epic, Story, RulesConfig, AnalysisBaseline, QualityRecord, DesignChangeProposal, ArchitectureInvestigation, ArchitectureCatalogEntry, ReferenceArchitecture
- ArchitectureCatalogEntry type: stores a reusable architecture pattern with fields for language, project type, folder structure, layers, dependency rules, module boundaries, naming conventions, testing layout, anti-patterns, rules seed data, and analysis expectations
- ReferenceArchitecture type: stores the selected/derived architecture for a specific repo, linking to a catalog entry (if matched) or containing a derived pattern, persisted as a durable cross-cutting governance artifact
- Hierarchy validation rules for the new type relationships
- Cross-reference system between governance artifacts and planning artifacts (including Reference Architecture → Rules Config seeding, Reference Architecture → Analysis Baseline expectations)
- Schema versioning for document formats to support future evolution

## Alternatives Considered

1. **Complete rewrite of domain model**: Rejected because the existing Metis model is well-structured and the serialization/storage patterns are sound. Extending is less risky and faster.
2. **Use a generic "document" type with metadata-driven behavior**: Rejected because strong typing in Rust gives compile-time guarantees and makes the codebase more maintainable.
3. **Keep Vision/Initiative/Task and add new types alongside**: Rejected because it would create confusing parallel hierarchies. Better to replace with a unified model.

## Implementation Plan

Phase 1: Audit existing domain types in `metis/` and map them to target types
Phase 2: Define new Rust types and traits for all Super-Metis document types
Phase 3: Implement serialization/deserialization for new types
Phase 4: Update hierarchy validation and parent-child rules
Phase 5: Update SQLite schema and indexing for new types
Phase 6: Write migration logic for existing documents (coordinate with SMET-I-0015)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- All target document types are defined as Rust types with proper serialization
- Hierarchy rules are enforced at the domain level
- Existing Metis documents can be read and mapped to new types
- All domain types round-trip through markdown+frontmatter without data loss
- Cross-references between governance and planning artifacts are validated
- Unit tests cover all type creation, validation, and serialization paths

## Risks / Dependencies

- This is the foundational initiative; most others depend on it
- Risk of scope creep — must resist adding behavior beyond the domain model
- Must coordinate closely with SMET-I-0002 (planning hierarchy), SMET-I-0015 (migration), and SMET-I-0016 (architecture catalog)

## Codebase Areas to Inspect

- `metis/src/domain/` — existing domain types and traits
- `metis/src/storage/` — document serialization and file I/O
- `metis/src/db/` — SQLite schema and indexing
- `metis/src/models/` — data models if separate from domain
- `metis/Cargo.toml` — dependency structure

## Suggested Tasks for Decomposition

1. Audit and document all existing Metis domain types and their relationships
2. Design the new type hierarchy as Rust traits and structs
3. Implement ProductDoc and DesignContext types
4. Implement Epic and Story types with hierarchy rules
5. Implement RulesConfig type with protection semantics
6. Implement AnalysisBaseline and QualityRecord types
7. Implement DesignChangeProposal and ArchitectureInvestigation types
8. Implement ArchitectureCatalogEntry and ReferenceArchitecture types
9. Update SQLite schema for new document types
10. Update document indexing for new types
11. Write comprehensive unit tests for all new domain types
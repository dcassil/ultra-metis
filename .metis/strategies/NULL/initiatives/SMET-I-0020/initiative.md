---
id: architecture-catalog-domain-types
level: initiative
title: "Architecture Catalog Domain Types"
short_code: "SMET-I-0020"
created_at: 2026-03-11T21:39:40.959763+00:00
updated_at: 2026-03-16T21:19:01.673383+00:00
parent: SMET-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: architecture-catalog-domain-types
---

# Architecture Catalog Domain Types

## Context

Super-Metis gives every repo a persisted reference architecture that drives rules, structure, and analysis. This requires two domain types: ArchitectureCatalogEntry (reusable architecture patterns) and ReferenceArchitecture (the selected/derived architecture for a specific repo). These types are the bridge between architecture knowledge and enforcement — the ReferenceArchitecture seeds RulesConfig and sets AnalysisBaseline expectations.

This initiative depends on SMET-I-0018 (Core Planning Hierarchy) for the base domain model, and has a soft dependency on SMET-I-0019 (Governance Types) for the RulesConfig/AnalysisBaseline linkage — though the reference fields can be implemented before those types fully land. Split out from the original SMET-I-0001 (now archived) as a focused vertical slice.

## Governing Commitments

- **Every repo gets a persisted reference architecture.** ArchitectureCatalogEntry and ReferenceArchitecture are first-class domain types, not afterthoughts.
- **Reference architecture drives rules, structure, and analysis.** The ReferenceArchitecture is a control artifact that seeds enforcement.
- **All durable project memory lives in the repo.** Architecture decisions are persisted as queryable, structured documents.

## Goals & Non-Goals

**Goals:**
- Implement ArchitectureCatalogEntry type: reusable architecture patterns with fields for language, project type, folder structure, layers, dependency rules, module boundaries, naming conventions, testing layout, anti-patterns, rules seed data, and analysis expectations
- Implement ReferenceArchitecture type: the selected/derived architecture for a specific repo, linking to a catalog entry (if matched) or containing a derived pattern
- Define linkage fields from ReferenceArchitecture → RulesConfig seeding and → AnalysisBaseline expectations
- Serialization/deserialization for both types (markdown + YAML frontmatter)
- SQLite schema and indexing for catalog and reference architecture types
- Comprehensive unit tests

**Non-Goals:**
- Planning hierarchy types — covered by SMET-I-0018
- Governance types (RulesConfig, AnalysisBaseline) — covered by SMET-I-0019
- Bootstrap/setup flows that auto-detect architecture — covered by SMET-I-0008
- UI for architecture browsing — covered by SMET-I-0011
- Bundled catalog entries (actual architecture patterns) — separate future work

## Detailed Design

### ArchitectureCatalogEntry
- Represents a reusable, well-known architecture pattern (e.g., "Rust CLI with workspace", "Next.js monorepo with Turborepo")
- Fields:
  - Identity: name, description, tags, language(s), project type
  - Structure: expected folder layout, layer definitions, module boundaries
  - Rules: dependency rules, naming conventions, anti-patterns
  - Quality: expected analysis tool config, baseline thresholds
  - Seeding: rules seed data (what RulesConfig entries to create), analysis expectations (what AnalysisBaseline entries to create)
- Stored as markdown+frontmatter documents in a `catalog/` directory
- Can be bundled with Super-Metis or user-defined

### ReferenceArchitecture
- Represents the architecture selected or derived for a specific repo
- One per repo (or one per workspace in a monorepo)
- Fields:
  - Source: linked catalog entry short code (if matched), or "derived" flag
  - Customizations: overrides/additions on top of the catalog entry
  - Linkage: references to RulesConfig and AnalysisBaseline that were seeded from this architecture
  - Status: draft/active/superseded
- When created from a catalog entry, inherits structure/rules/quality fields
- When derived (no catalog match), fields are populated from repo analysis

### Linkage to Governance Types
- ReferenceArchitecture has `rules_config_ref` and `analysis_baseline_ref` fields
- These are short code references, validated when the referenced documents exist
- Seeding flow: ReferenceArchitecture → generates initial RulesConfig entries and AnalysisBaseline thresholds (the actual seeding logic is future work, but the reference fields and data structures are defined here)

## Alternatives Considered

1. **Store architecture patterns as JSON/TOML instead of markdown**: Rejected — consistency with all other Metis documents. Markdown+frontmatter is the universal format.
2. **Single type for both catalog and repo-specific architecture**: Rejected — catalog entries are reusable templates, reference architectures are instance-specific. Different lifecycles require different types.
3. **Skip catalog entries, only have ReferenceArchitecture**: Rejected — catalog entries enable pattern reuse across repos and provide the "known good" patterns that bootstrap relies on.

## Implementation Plan

Phase 1: Implement ArchitectureCatalogEntry type with all fields
Phase 2: Implement ReferenceArchitecture type with catalog linkage
Phase 3: Define governance linkage fields (rules_config_ref, analysis_baseline_ref)
Phase 4: Update SQLite schema and indexing for both types
Phase 5: Comprehensive unit tests

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- ArchitectureCatalogEntry type fully defined with all specified fields
- ReferenceArchitecture type supports both catalog-linked and derived modes
- Governance linkage fields present and validated when targets exist
- Both types round-trip through markdown+frontmatter without data loss
- SQLite schema updated and indexed for both types
- Unit tests cover type creation, validation, serialization, and linkage

## Risks / Dependencies

- Depends on SMET-I-0018 for the base domain model and type system
- Soft dependency on SMET-I-0019 for RulesConfig/AnalysisBaseline types (can implement reference fields before those land)
- Catalog entry schema needs to be rich enough to support bootstrap flows (SMET-I-0008) — coordinate on field requirements

## Codebase Areas to Inspect

- `metis/src/domain/` — existing domain types to extend
- `metis/src/storage/` — serialization patterns to follow
- `metis/src/db/` — SQLite schema to extend
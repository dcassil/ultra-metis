---
id: add-migration-and-versioning-path
level: initiative
title: "Add Migration and Versioning Path from Metis to Cadre"
short_code: "SMET-I-0015"
created_at: 2026-03-11T20:00:15.612457+00:00
updated_at: 2026-03-11T20:00:15.612457+00:00
parent: SMET-S-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: cadre-core-engine-repo
initiative_id: add-migration-and-versioning-path
---

# Add Migration and Versioning Path from Metis to Cadre

## Context

Existing Metis users have projects with Visions, Strategies, Initiatives, Tasks, and ADRs. Cadre introduces a fundamentally different document hierarchy (Product Doc, Epic, Story, Task) and new artifact types. There must be a clear, safe migration path from Metis documents to Cadre documents, as well as a schema versioning system that supports future evolution.

This initiative ensures that the transition from Metis to Cadre is smooth, data-preserving, and reversible where possible.

## Governing Commitments

This initiative directly serves:
- **Evolution from Metis foundations** (Principle #7, Constraint #1). The system builds forward from Metis rather than discarding its strengths. Existing durable artifacts and workflows are preserved and transformed, not abandoned.
- **All durable project memory lives in the repo.** Migration preserves all existing planning context as repo-native artifacts in the new format. No project knowledge is lost in the transition.
- **Planning is durable and traceable from product intent to execution.** Type and phase mapping rules ensure that the traceability chain from vision-level intent to task-level execution survives migration into the richer hierarchy.
- **The system is built around intentional, durable structure.** Schema versioning and explicit migration tooling ensure that format evolution is managed, predictable, and reversible — not ad-hoc.

## Goals & Non-Goals

**Goals:**
- Define mapping rules from Metis document types to Cadre types (Vision → Product Doc, Initiative → Epic, Task → Story or Task)
- Build a migration tool that converts existing Metis projects to Cadre format
- Implement document schema versioning so future changes are manageable
- Preserve all existing document content during migration (no data loss)
- Support incremental migration (not all-or-nothing)
- Provide a dry-run mode that shows what would change without applying changes

**Non-Goals:**
- Backward migration (Cadre → Metis) — this is a one-way upgrade
- Supporting Metis and Cadre formats simultaneously in production — after migration, the project is Cadre
- Migrating external integrations (CI/CD references to Metis short codes, etc.)

## Detailed Design

### What to Reuse from `metis/`
- Document reading/parsing infrastructure for reading existing Metis documents
- Frontmatter parsing for extracting metadata from current format
- Database migration patterns (if any exist)
- File I/O patterns for reading and writing documents

### What to Change from `metis/`
- Add schema version field to all document frontmatter
- Make the document parser version-aware (read old format, write new format)

### What is Net New
- Type mapping rules: Vision → Product Doc, Strategy → (absorbed into Epic or dropped), Initiative → Epic, Task → Story or Task (based on complexity)
- Migration tool: `cadre migrate` command
- Schema version tracking in frontmatter and database
- Dry-run mode: preview migration without applying changes
- Migration report: summary of what was migrated, what needs manual attention
- Incremental migration support: migrate document by document or type by type
- Rollback support: backup original files before migration
- Schema evolution framework: define migrations between schema versions

## Alternatives Considered

1. **No migration — start fresh**: Rejected because existing Metis projects have valuable planning context that shouldn't be lost.
2. **Automatic in-place migration on first access**: Rejected because silent data transformation is risky. Migration should be explicit.
3. **Side-by-side migration (keep both formats)**: Rejected as too complex. Clean migration with backup is safer.
4. **Manual migration**: Rejected because it's error-prone and tedious. Automated with human review is better.

## Implementation Plan

Phase 1: Define complete type mapping rules (Metis type → Cadre type)
Phase 2: Add schema version field to all Cadre document types
Phase 3: Build document format migration logic (read old, write new)
Phase 4: Build database schema migration
Phase 5: Implement dry-run mode
Phase 6: Implement backup and rollback
Phase 7: Build migration report generation
Phase 8: Implement incremental migration support
Phase 9: Build schema evolution framework for future versions
Phase 10: Test migration with real Metis projects

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- All Metis document types can be migrated to Cadre equivalents
- No content is lost during migration
- Dry-run mode accurately previews all changes
- Original files are backed up before migration
- Migration report clearly identifies what was migrated and what needs manual review
- Schema version is tracked in all document frontmatter
- Incremental migration works correctly (migrate some documents, leave others)
- The schema evolution framework supports future version upgrades
- Migration completes in under 60 seconds for projects with up to 500 documents

## Risks / Dependencies

- Depends on SMET-I-0001 for target document types
- Depends on SMET-I-0002 for target phase flows (need to map old phases to new)
- Type mapping is lossy in some cases (Strategy has no direct equivalent) — need clear documentation
- Must handle edge cases: partially complete documents, documents with broken references
- Must coordinate with SMET-I-0009 and I-0010 for CLI/MCP migration commands

## Codebase Areas to Inspect

- `metis/src/domain/` — current document type definitions (source format)
- `metis/src/storage/` — document reading/writing patterns
- `metis/src/db/` — database schema and migration patterns
- `metis/src/commands/` — command patterns for the migrate command

## Suggested Tasks for Decomposition

1. Document complete type mapping rules (Metis → Cadre)
2. Document phase mapping rules (old phases → new phases)
3. Add schema version field to all Cadre document types
4. Build Metis document reader (parse old format)
5. Build document format converter (old → new)
6. Build database schema migration
7. Implement dry-run mode with preview output
8. Implement backup before migration
9. Build migration report generator
10. Implement incremental migration support
11. Build schema evolution framework
12. Test with real Metis project data
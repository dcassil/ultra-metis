---
id: architecture-catalog-content-and
level: initiative
title: "Architecture Catalog Content and Selection Flow"
short_code: "SMET-I-0027"
created_at: 2026-03-11T21:52:31.983372+00:00
updated_at: 2026-03-17T01:07:48.812435+00:00
parent: SMET-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: architecture-catalog-content-and
---

# Architecture Catalog Content and Selection Flow

## Context

Super-Metis moves repos from improvised architecture to intentional architecture. This requires a curated catalog of known-good patterns and a selection flow that helps users choose the right one during setup.

The domain types for ArchitectureCatalogEntry and ReferenceArchitecture are defined in SMET-I-0020. This initiative focuses on the actual catalog content (the 5 initial JS/TS patterns) and the greenfield selection flow. Brownfield evaluation is a separate initiative (SMET-I-0028).

Split from the original SMET-I-0016 (now archived).

## Goals & Non-Goals

**Goals:**
- Write 5 initial JavaScript/TypeScript catalog entries: `javascript/server`, `javascript/react-app`, `javascript/component-lib`, `javascript/cli-tool`, `javascript/node-util`
- Each entry includes: use case, tradeoffs, folder structure, layers, dependency rules, module boundaries, naming conventions, testing layout, anti-patterns, rules seed data, analysis expectations
- Build catalog query engine: filter by language and project type
- Build greenfield selection flow: detect project type → query catalog → present options with tradeoffs → user chooses → persist as Reference Architecture
- Support catalog extensibility: projects can add custom entries
- Support Reference Architecture tailoring: users can adjust aspects of a selected pattern

**Non-Goals:**
- Brownfield evaluation and matching — covered by SMET-I-0028
- Domain types (ArchitectureCatalogEntry, ReferenceArchitecture) — covered by SMET-I-0020
- Rules seeding from architecture — covered by SMET-I-0004
- Analysis expectations enforcement — covered by SMET-I-0021/I-0022
- Catalog entries for non-JS/TS languages — future expansion

## Detailed Design

### Catalog Entries (Initial Set)
- **javascript/server**: Express/Fastify/Hono backend. Layered: routes → handlers → services → repositories. Clear HTTP/business separation.
- **javascript/react-app**: React SPA or Next.js. Feature-based folders. Component → hook → service layering. Co-located tests.
- **javascript/component-lib**: Shared UI library. Component-per-folder. Storybook-friendly. Explicit public API via index exports.
- **javascript/cli-tool**: Node CLI. Command-based structure. CLI parsing separated from core logic.
- **javascript/node-util**: Utility/library package. Flat or domain-grouped source. Comprehensive unit tests. Clean public API.

Each entry defines:
- Folder structure with explanations
- Layer definitions and dependency rules (which layers can import from which)
- Module boundary conventions
- File and export naming conventions
- Test placement and naming
- Common anti-patterns to avoid
- Rules seed data (structured data to auto-generate RulesConfig entries)
- Analysis expectations (what quality checks should enforce)

### Catalog Storage
- Built-in entries ship with Super-Metis binary (embedded or bundled)
- Custom entries stored in `.metis/catalog/` directory
- Both queried through the same interface

### Selection Flow
1. Detect language and project type (from SMET-I-0008's repo scanner)
2. Query catalog for matching entries
3. Present 1-3 matching options with use case descriptions and tradeoffs
4. User selects one (or skips for brownfield evaluation)
5. Optional: user tailors specific aspects (folder names, conventions)
6. Persist as Reference Architecture document

## Alternatives Considered

1. **AI-generated architecture each time**: Rejected — architecture should be stable and intentional, not regenerated.
2. **Massive catalog from day one**: Rejected — start curated and small, expand based on usage.
3. **No selection flow, just default**: Rejected — different project types need different patterns. One size doesn't fit.

## Implementation Plan

Phase 1: Write `javascript/server` catalog entry (full detail, serves as template for others)
Phase 2: Write remaining 4 JS/TS catalog entries
Phase 3: Build catalog storage and query engine
Phase 4: Build selection flow (query → present → choose → persist)
Phase 5: Implement Reference Architecture tailoring
Phase 6: Implement custom catalog entry support
Phase 7: Integration test selection flow with each pattern

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- Five JS/TS catalog entries exist and are complete with all sections
- Catalog query correctly filters by language and project type
- Selection flow presents matching patterns and persists the choice as a Reference Architecture
- Users can tailor selected patterns
- Projects can add custom catalog entries
- Each catalog entry's rules seed data and analysis expectations are structured and machine-readable

## Risks / Dependencies

- Depends on SMET-I-0020 for domain types
- Depends on SMET-I-0008 for integration with the initialization flow
- Catalog entry quality directly affects downstream rules and analysis — entries must be excellent
- Must coordinate with SMET-I-0004 for rules seed data format
- Must coordinate with SMET-I-0014 for catalog entry templates
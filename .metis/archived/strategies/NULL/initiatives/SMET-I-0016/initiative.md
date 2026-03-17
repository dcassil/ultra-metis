---
id: architecture-catalog-and-reference
level: initiative
title: "Architecture Catalog and Reference Architecture Selection"
short_code: "SMET-I-0016"
created_at: 2026-03-11T20:44:16.202930+00:00
updated_at: 2026-03-11T20:44:16.202930+00:00
parent: SMET-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: XL
strategy_id: NULL
initiative_id: architecture-catalog-and-reference
---

# Architecture Catalog and Reference Architecture Selection

## Context

Today, when an AI agent sets up a new project or works within an existing repo, it improvises the repo structure — folder layout, module boundaries, dependency direction, naming conventions, testing placement. This means every repo ends up with a slightly different, ad-hoc architecture that nobody explicitly chose and that drifts over time.

Super-Metis should move repos from improvised architecture to intentional architecture. This means:
1. A curated catalog of known-good architecture patterns organized by language and project type
2. A selection flow during setup that suggests strong options, explains tradeoffs, and lets the user choose
3. A brownfield evaluation flow for existing repos that matches current structure to a catalog pattern or derives a stable reference
4. A persisted Reference Architecture artifact that becomes the durable source of truth for the repo's intended structure

The chosen architecture should then drive downstream concerns: rules are seeded from it, analysis checks enforce it, planning and decomposition follow it, and AI agents consult it instead of improvising.

## Why It Matters

Without explicit architecture, every agent interaction is a coin flip on structure. Architecture drift is invisible until it's entrenched. By making architecture a first-class, persisted artifact:
- AI agents have a stable reference instead of guessing each time
- Rules can be automatically generated from the architecture pattern
- Analysis can enforce architectural boundaries and conventions
- Planning and decomposition can be architecture-aware (e.g., "this story touches the data layer")
- Code review and quality checks have a shared standard to measure against
- Brownfield repos get the same benefits as greenfield ones

## Governing Commitments

This initiative directly serves:
- **Every repo gets a persisted reference architecture.** This is the initiative that makes it happen. Whether greenfield selection, brownfield match/capture, or brownfield recommendation with user choice — every repo resolves to a durable architecture reference.
- **The user may keep the current architecture as the governing reference if they choose.** When brownfield evaluation finds weak architecture, the system recommends a stronger pattern but the user has final say. Declining records the current architecture as-is — the system guides, it does not force.
- **Reference architecture drives rules, structure, and analysis** (Vision #4). The Reference Architecture is not documentation — it is a living control artifact. Rules are seeded from it, structure is validated against it, analysis enforces it, planning consults it, and agents follow it instead of improvising.
- **Architecture-driven rules** (Vision #4). Each catalog entry includes rules seed data. When a Reference Architecture is established, engineering rules are automatically generated from the architecture pattern, keeping enforcement aligned to the actual repo model.
- **Brownfield repos are fully supported.** Existing systems are analyzed with a quality-first assessment (including static analysis), understood, and resolved into an explicit architecture reference. Brownfield repos are first-class participants in the governance model.
- **Architecture-aware quality** (Vision #6). Each catalog entry includes analysis expectations. Boundary adherence, dependency direction, and architectural conformance become part of the quality model — derived from the architecture, not defined independently.
- **Structural guidance over improvisation** (Principle #3). The entire purpose of this initiative is to replace improvised architecture with intentional, durable structure. The catalog provides curated options; the Reference Architecture makes the choice permanent and operational.

## Goals & Non-Goals

**Goals:**
- Define a structured format for architecture catalog entries that is both human-readable and machine-queryable
- Build an initial curated catalog of JavaScript/TypeScript architecture patterns: `javascript/server`, `javascript/react-app`, `javascript/component-lib`, `javascript/cli-tool`, `javascript/node-util`
- Each catalog entry defines: intended use case, tradeoffs, folder/package structure, layers, dependency rules, module boundaries, naming conventions, testing layout, anti-patterns, rules seed data, and analysis expectations
- Build a selection flow that presents matching patterns based on detected language/project type, explains tradeoffs, and persists the user's choice
- Build a brownfield evaluation engine that first assesses the quality of an existing repo's architecture (including static analysis), then either matches/captures a good architecture or recommends a catalog replacement for a bad one — with the user always having final say
- Define the Reference Architecture artifact format — the persisted, per-repo document that records the selected/derived architecture
- Ensure the Reference Architecture integrates with rules (seeding), analysis (boundary enforcement), and planning (architecture-aware decomposition)

**Non-Goals:**
- Building an exhaustive catalog of every possible architecture style — start curated and small
- Forcing architecture rewrites on brownfield repos — the system recommends but the user always decides. If the existing architecture is poor, the system explains the problems and suggests a catalog alternative, but the user can decline and keep their current structure
- Runtime architecture enforcement (e.g., import checking at build time) — Super-Metis defines and tracks the architecture; linters/build tools enforce it
- Supporting every language from day one — start with JavaScript/TypeScript, expand later
- Monorepo-root multi-architecture support — that's a future concern (see SMET-I-0017)

## Detailed Design

### What to Reuse from `metis/`
- Document storage infrastructure for catalog entries and reference architecture artifacts
- Markdown + frontmatter format — catalog entries and reference architectures are documents like everything else
- Search and indexing for catalog queries (find patterns by language, project type)
- Template system for generating Reference Architecture documents
- The existing code indexing infrastructure as a foundation for brownfield analysis

### What to Change from `metis/`
- Extend the document model with ArchitectureCatalogEntry and ReferenceArchitecture types (coordinate with SMET-I-0001)
- The initialization flow must include architecture selection as a core step (coordinate with SMET-I-0008)

### What is Net New

#### Architecture Catalog
- **Catalog format**: Each entry is a structured document with sections for:
  - `language`: e.g., "javascript", "typescript", "rust"
  - `project_type`: e.g., "server", "react-app", "component-lib", "cli-tool", "node-util"
  - `use_case`: When to use this pattern
  - `tradeoffs`: Strengths, weaknesses, when NOT to use it
  - `folder_structure`: Expected directory layout with explanations
  - `layers`: Architectural layers and their responsibilities (e.g., routes → controllers → services → repositories)
  - `dependency_rules`: Which layers can depend on which, import restrictions
  - `module_boundaries`: How code is organized into modules, what's public vs internal
  - `naming_conventions`: File naming, export naming, test file naming
  - `testing_layout`: Where tests live, test file naming, test types per layer
  - `anti_patterns`: Common mistakes and what to do instead
  - `rules_seed_data`: Structured data that can be used to auto-generate Rules Config documents
  - `analysis_expectations`: Structured data that defines what quality checks and boundary enforcement should look for
- **Catalog storage**: Shipped as built-in documents within Super-Metis (not per-project — they're system-level reference data)
- **Catalog extensibility**: Projects can add custom catalog entries for their own patterns

#### Initial Catalog Entries (JavaScript/TypeScript)
- `javascript/server`: Express/Fastify/Hono-style backend. Layered (routes → handlers → services → repositories). Clear separation of HTTP concerns from business logic.
- `javascript/react-app`: React SPA or Next.js app. Feature-based folder structure. Component → hook → service layering. Co-located tests.
- `javascript/component-lib`: Shared UI component library. Component-per-folder. Storybook-friendly. Explicit public API via index exports.
- `javascript/cli-tool`: Node CLI application. Command-based structure. Clear separation of CLI parsing from core logic.
- `javascript/node-util`: Utility/library package. Flat or domain-grouped source. Comprehensive unit tests. Clean public API.

#### Architecture Selection Flow
- Triggered during Super-Metis initialization (SMET-I-0008)
- After language/project type detection, query the catalog for matching patterns
- Present a small set (typically 1-3) of matching patterns with use case descriptions and tradeoffs
- User selects one (or can skip to derive from existing structure)
- Optional tailoring: user can adjust specific aspects of the selected pattern
- Persist as a Reference Architecture artifact

#### Brownfield Architecture Evaluation
- For existing repos with significant code already in place
- **Quality-first assessment**: Before matching or recommending, evaluate the quality of the existing architecture:
  - Analyze current folder structure, directory naming, file organization patterns
  - Analyze import/dependency graphs to detect layering and module boundaries
  - Analyze naming conventions in use (file names, export patterns, test file placement)
  - Run static analysis checks: circular dependencies, layering violations, inconsistent patterns, dead code clustering, test coverage gaps by module
  - Produce an architecture quality score with specific findings
- **Good architecture path** (quality score above threshold):
  - Score the existing structure against matching catalog patterns
  - If a catalog match is found: confirm the match and create Reference Architecture linking to that catalog entry, noting any deviations
  - If no catalog match but architecture is sound: capture the existing structure as a derived Reference Architecture — document the actual folder structure, actual layering, actual conventions
  - In both cases, the repo ends up with a durable architecture reference that reflects its existing (good) structure
- **Bad architecture path** (quality score below threshold):
  - Consider the project's language, project type, and what it's trying to do
  - Recommend a specific catalog pattern that would be a strong fit
  - Explain to the user: their current architecture has specific problems (cite findings), the recommended pattern would address them, but adopting it will require a refactor of specific files/modules
  - Present the user with a clear choice:
    - **Accept**: The recommended catalog pattern becomes the Reference Architecture. The system creates a refactoring plan as the first priority work item. Rules and analysis are seeded from the recommended pattern.
    - **Decline**: The current (bad) architecture is recorded as-is as the Reference Architecture, with quality findings noted. The system respects the user's choice but preserves the assessment for future reference.
  - The user always has final say — the system recommends but does not force

#### Reference Architecture Artifact
- Per-repo document persisted in the `.metis` (or `.super-metis`) directory
- Links to the source catalog entry (if selected or matched), or marked as "derived" if created from brownfield analysis
- Contains the canonical architecture definition for THIS repo:
  - folder structure
  - layers and their responsibilities
  - dependency rules
  - module boundaries
  - naming conventions
  - testing layout
  - any repo-specific tailoring or overrides
- Used as input by:
  - Rules Config seeding (SMET-I-0004)
  - Analysis baseline expectations (SMET-I-0005)
  - Planning and decomposition guidance (agents consult this when breaking down work)
  - Architecture Investigation triggers (SMET-I-0006) when boundaries are violated

## Alternatives Considered

1. **Let agents improvise architecture each time**: Rejected — this is the current state and the entire reason this initiative exists. Improvisation leads to drift and inconsistency.
2. **Use CLAUDE.md or .cursorrules for architecture**: Rejected because these have no structured format, no protection, no versioning, and can't be queried programmatically.
3. **Build a massive catalog from day one**: Rejected — start curated and small (5 JS/TS patterns), expand based on actual usage.
4. **Only support greenfield repos**: Rejected because most real repos are brownfield. Evaluation and matching is essential.
5. **Store catalog externally (npm package, remote API)**: Rejected — must be repo-native. Built-in catalog ships with Super-Metis; custom entries are repo-local.
6. **Generate architecture from AI each time**: Rejected — the whole point is that architecture should be stable and intentional, not regenerated.

## Implementation Plan

Phase 1: Define the Architecture Catalog Entry format (structured document schema)
Phase 2: Define the Reference Architecture artifact format
Phase 3: Implement domain types for both (coordinate with SMET-I-0001)
Phase 4: Write the initial 5 JavaScript/TypeScript catalog entries
Phase 5: Build the architecture selection flow (catalog query, presentation, user choice)
Phase 6: Build the brownfield evaluation engine (quality-first assessment with static analysis, good path: match/capture, bad path: recommend + user choice)
Phase 7: Build Reference Architecture persistence and tailoring
Phase 8: Build rules seed data integration (Reference Architecture → Rules Config seeding, coordinate with SMET-I-0004)
Phase 9: Build analysis expectations integration (Reference Architecture → Analysis Baseline expectations, coordinate with SMET-I-0005)
Phase 10: Add CLI and MCP support for catalog browsing, architecture selection, and reference viewing
Phase 11: Integration test with real greenfield and brownfield JavaScript repos

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- Architecture Catalog Entry format is defined and documented with all required sections
- Five initial JavaScript/TypeScript catalog entries exist and are complete (server, react-app, component-lib, cli-tool, node-util)
- Each catalog entry includes use case, tradeoffs, folder structure, layers, dependency rules, module boundaries, naming conventions, testing layout, anti-patterns, rules seed data, and analysis expectations
- Architecture selection flow works during initialization: detects project type, presents matching patterns, allows user to choose, persists the choice
- Brownfield evaluation performs quality-first assessment (including static analysis), then correctly handles good architecture (match/capture) and bad architecture (recommend catalog pattern, explain refactor, user chooses)
- Every initialized repo ends up with a persisted Reference Architecture artifact — no repo is left without one
- Reference Architecture drives rule seeding: architecture-derived rules are generated automatically
- Reference Architecture drives analysis: boundary checks and convention checks respect the architecture
- Catalog is extensible: projects can add custom catalog entries
- Reference Architecture is tailorable: users can adjust the selected pattern for their specific needs
- CLI and MCP tools allow browsing the catalog, viewing the reference architecture, and querying architecture details

## Risks / Dependencies

- Depends on SMET-I-0001 for ArchitectureCatalogEntry and ReferenceArchitecture domain types
- Depends on SMET-I-0008 for integration with the initialization/bootstrap flow
- Must coordinate with SMET-I-0004 for rules seed data format and seeding workflow
- Must coordinate with SMET-I-0005 for analysis expectations format and enforcement
- Must coordinate with SMET-I-0014 for catalog entry and reference architecture templates
- Must coordinate with SMET-I-0009 and SMET-I-0010 for MCP and CLI tool support
- Risk of catalog entries being too prescriptive — must balance opinionation with flexibility
- Brownfield evaluation heuristics are inherently imperfect — must be transparent about confidence and allow override
- Risk of scope creep into the catalog — resist adding too many patterns too early

## Codebase Areas to Inspect

- `metis/src/domain/` — domain type patterns for new catalog and reference architecture types
- `metis/src/storage/` — document storage for catalog entries (system-level vs project-level)
- `metis/src/code_index/` — existing code analysis infrastructure for brownfield evaluation
- `metis/src/commands/init.rs` — where architecture selection integrates with initialization
- `metis/src/templates/` — template system for catalog entries and reference architecture documents
- `metis/src/mcp/` — MCP tool patterns for catalog browsing and architecture queries

## Suggested Tasks for Decomposition

1. Define Architecture Catalog Entry document schema (all sections, frontmatter fields)
2. Define Reference Architecture artifact schema (per-repo format, linkage to catalog)
3. Implement ArchitectureCatalogEntry domain type (coordinate with SMET-I-0001)
4. Implement ReferenceArchitecture domain type (coordinate with SMET-I-0001)
5. Write `javascript/server` catalog entry
6. Write `javascript/react-app` catalog entry
7. Write `javascript/component-lib` catalog entry
8. Write `javascript/cli-tool` catalog entry
9. Write `javascript/node-util` catalog entry
10. Build catalog query engine (filter by language, project type)
11. Build architecture selection flow (present options, explain tradeoffs, persist choice)
12. Build brownfield structure analyzer (folder structure, import graph, naming convention detection, static analysis checks)
13. Build architecture quality scorer (circular deps, layering violations, inconsistent patterns, dead code, coverage gaps → quality score with findings)
14. Build good-architecture path: pattern matching/scoring against catalog, match confirmation, derived capture when no catalog match
15. Build bad-architecture path: recommend catalog pattern, explain problems and refactor scope, present accept/decline choice to user
16. Build declined-bad-architecture fallback: record current structure as-is with quality findings noted
17. Build Reference Architecture tailoring (allow user overrides of selected pattern)
18. Build rules seed data integration (generate Rules Config from architecture seed data)
19. Build analysis expectations integration (generate analysis checks from architecture expectations)
20. Add CLI commands for catalog and architecture operations
21. Add MCP tools for catalog and architecture operations
22. Integration test greenfield selection flow with each catalog pattern
23. Integration test brownfield evaluation with real repo structures (good and bad architecture cases)
---
id: build-repo-aware-setup-and
level: initiative
title: "Build Repo-Aware Setup and Bootstrap Flows for Monorepo Projects"
short_code: "SMET-I-0008"
created_at: 2026-03-11T19:59:45.102876+00:00
updated_at: 2026-03-17T02:25:03.521566+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: cadre-core-engine-repo
initiative_id: build-repo-aware-setup-and
---

# Build Repo-Aware Setup and Bootstrap Flows for Monorepo Projects

## Context

When Cadre is initialized in a new repo, it should understand the repo's structure — what languages are used, what build tools exist, what the directory layout looks like, where packages/apps/libraries live. This context informs everything from template generation to quality gate configuration to rule suggestions.

Current Metis initialization creates a generic workspace. Cadre should create a workspace that's aware of the software project it's managing.

## Governing Commitments

This initiative directly serves:
- **Every repo gets a persisted reference architecture.** Setup is where architecture resolution happens. No repo exits initialization without a durable Reference Architecture — whether selected from a catalog, matched from a strong existing codebase, or recorded as-is after user declines a recommendation.
- **The user may keep the current architecture as the governing reference if they choose.** The system supports guided architecture selection, not forced replacement. When the existing architecture is weak, the system recommends but the user decides.
- **Brownfield repos are fully supported.** Existing systems are analyzed, understood, and resolved into an explicit architecture reference during setup — brownfield repos are first-class participants in the governance model from day one.
- **Architecture-driven rules** (Vision #4). Starter rules are seeded from the selected Reference Architecture's seed data during setup, so governance is aligned to the actual repo model from initialization.
- **All durable project memory lives in the repo.** Setup creates the foundational persistent artifacts — Reference Architecture, Product Doc scaffold, starter Rules Config, quality tool configuration — that become the repo's long-term memory.
- **The system is built around intentional, durable structure.** Setup is the moment where improvisation ends and intentional structure begins.

## Goals & Non-Goals

**Goals:**
- Build a repo-aware initialization flow that detects languages, frameworks, build tools, and monorepo structure
- Integrate architecture selection into the setup flow: after detecting the repo's language and project type, suggest matching architecture patterns from the catalog, explain tradeoffs, and let the user choose
- For existing/brownfield repos: evaluate the current repo architecture quality (including static analysis results), and branch based on the assessment:
  - If the architecture is strong: match it to a catalog pattern or create a custom reference capturing the existing structure
  - If the architecture is weak: identify the project's type and intent, recommend a first-class catalog pattern, explain the refactor implications, and let the user choose whether to adopt the recommendation or keep the existing architecture as-is
- Persist the selected or derived architecture as a durable Reference Architecture artifact
- Auto-generate a Product Doc scaffold based on detected repo characteristics
- Pre-configure quality gate tools based on detected linters and analysis tools
- Generate starter Rules Config based on detected conventions AND the selected architecture pattern's seed data
- Support common monorepo patterns: Turborepo, Nx, Cargo workspaces, pnpm workspaces

**Non-Goals:**
- Supporting non-software projects — Cadre is purpose-built for software repos
- Auto-generating all planning artifacts — bootstrap gets you started, humans refine
- Deep framework-specific knowledge (e.g., understanding Next.js routing conventions)
- Forcing architecture rewrites on brownfield repos — the system recommends but the user always decides. If they decline a recommended architecture, the current structure is recorded as-is

## Detailed Design

### What to Reuse from `metis/`
- The existing `initialize_project` command as the starting point
- Project configuration file format
- Database initialization
- Template generation system

### What to Change from `metis/`
- Extend initialization to include repo scanning phase
- Add project type detection to configuration
- Generate richer initial artifacts beyond just a vision doc

### What is Net New
- Repo scanner: detect languages (Cargo.toml, package.json, go.mod, etc.), build tools, monorepo structure
- Package/app discovery: find all packages in a monorepo workspace
- Tool detection: find linters, formatters, test runners, CI configuration
- **Architecture selection flow**: after detecting language/project type, query the Architecture Catalog for matching patterns, present a small set of strong options with tradeoffs explained, let the user choose, optionally allow tailoring, and persist as a Reference Architecture artifact
- **Brownfield architecture evaluation**: for existing repos, analyze current folder structure, module boundaries, dependency patterns, naming conventions, and static analysis results to assess architecture quality. Two paths:
  - **Good architecture**: score against catalog patterns, match to the closest one, or create a custom reference capturing the existing structure faithfully
  - **Bad architecture**: identify project type/intent, recommend a first-class catalog pattern, explain what a refactor would involve (affected files/modules), present the recommendation to the user with clear tradeoffs, and let them accept (triggering refactor-first workflow) or decline (recording current architecture as-is)
- **Reference Architecture persistence**: store the selected/derived architecture as a durable artifact that becomes the source of truth for the repo
- Product Doc scaffold generator: create initial Product Doc based on repo analysis
- Quality tool configuration: pre-configure baseline capture for detected analysis tools
- Starter Rules Config generation: sensible defaults based on detected project conventions AND the selected architecture's rule seed data
- Interactive setup flow: guided prompts for information that can't be auto-detected, including architecture selection step

## Design Decision

**Chosen approach**: Build a `domain/bootstrap/` module with three sub-modules:

1. **`repo_scanner`** -- Detects languages, package managers, and build tools by looking for manifest files (Cargo.toml, package.json, go.mod, pyproject.toml, etc.). Lightweight, path-based only.

2. **`monorepo_detector`** -- Detects monorepo patterns (Cargo workspaces, pnpm workspaces, Turborepo, Nx, Lerna) and discovers packages/apps/libraries within them. Returns structured `MonorepoInfo`.

3. **`init_flow`** -- Orchestrates the bootstrap: scans repo, detects monorepo, detects tools (linters/formatters/test runners), and produces a `BootstrapResult` containing all detected information ready for downstream use (architecture selection, product doc scaffolding, rules config generation).

All modules operate on file path lists (no filesystem I/O in the domain layer), consistent with the existing `StructureAnalyzer` pattern.

## Alternatives Considered

1. **Keep initialization generic, let users configure manually**: Rejected because manual setup is tedious and error-prone. Auto-detection with override is better UX.
2. **Plugin system for language-specific detection**: Deferred — start with built-in support for common patterns, add plugin architecture later.
3. **Scan entire repo deeply on init**: Rejected as too slow. Do lightweight detection on init, deeper scanning on-demand.

## Implementation Plan

Phase 1: Build repo scanner (language, build tool, monorepo pattern detection)
Phase 2: Build package/app discovery for common monorepo patterns
Phase 3: Build tool detection (linters, formatters, test runners)
Phase 4: Build architecture selection flow (query catalog, present options, persist choice)
Phase 5: Build brownfield architecture evaluation (analyze existing structure, match or derive pattern)
Phase 6: Implement Product Doc scaffold generation
Phase 7: Implement starter Rules Config generation (including architecture-derived rule seeding)
Phase 8: Implement quality tool pre-configuration
Phase 9: Build interactive setup flow for CLI (including architecture selection step)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- Initialization detects primary languages and frameworks in the repo
- Monorepo structure (packages, apps, libraries) is correctly identified
- For greenfield repos: architecture selection flow presents matching catalog patterns, user can choose, and the selection is persisted as a Reference Architecture artifact
- For brownfield repos with strong architecture: existing structure is evaluated positively, matched to a catalog pattern or captured as a custom reference
- For brownfield repos with weak architecture: the system explains the issues, recommends a catalog pattern with refactor scope, and the user chooses to accept (refactor-first) or decline (record current architecture as-is)
- In all brownfield cases, the user has final say and the repo ends up with a persisted Reference Architecture
- A Product Doc scaffold is generated with detected repo information pre-filled
- Quality baselines are pre-configured for detected analysis tools
- Starter rules are generated based on detected conventions AND the selected architecture's seed data
- Users can override any auto-detected settings including architecture selection
- Initialization completes in under 10 seconds for repos up to 100k files

## Risks / Dependencies

- Depends on SMET-I-0001 for Product Doc, Rules Config, and Reference Architecture types
- Depends on SMET-I-0016 for the Architecture Catalog content and pattern format
- Detection heuristics will have false positives/negatives — need good defaults and easy override
- Brownfield evaluation heuristics may misclassify architecture — must allow manual override
- Must not be slow — repo scanning and architecture evaluation need to be lightweight
- Must coordinate with SMET-I-0014 (templates) for scaffold generation
- Must coordinate with SMET-I-0004 (rules) for architecture-derived rule seeding

## Codebase Areas to Inspect

- `metis/src/commands/init.rs` or equivalent — current initialization logic
- `metis/src/config/` — project configuration patterns
- `metis/src/code_index/` — existing repo scanning infrastructure
- `metis/src/templates/` — template generation system

## Suggested Tasks for Decomposition

1. Build language/framework detection (Cargo.toml, package.json, go.mod, etc.)
2. Build monorepo pattern detection (Turborepo, Nx, Cargo workspaces, pnpm)
3. Build package/app/library discovery
4. Build linter/formatter/test runner detection
5. Build architecture selection flow (query catalog, present options, persist choice)
6. Build brownfield architecture evaluator (analyze current structure, score against catalog patterns)
7. Build architecture pattern matching/derivation logic (match to catalog or derive stable reference)
8. Implement Product Doc scaffold generator
9. Implement starter Rules Config generator (including architecture-derived rule seeding)
10. Implement quality tool pre-configuration
11. Build interactive CLI setup flow (including architecture selection step)
12. Integration test with sample greenfield and brownfield repos
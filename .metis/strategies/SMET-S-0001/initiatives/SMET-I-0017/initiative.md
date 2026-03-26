---
id: future-monorepo-root-orchestration
level: initiative
title: "Future: Monorepo-Root Orchestration and Multi-Architecture Support"
short_code: "SMET-I-0017"
created_at: 2026-03-11T20:45:54.158770+00:00
updated_at: 2026-03-26T18:20:13.316634+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/decompose"


exit_criteria_met: false
estimated_complexity: XL
strategy_id: cadre-core-engine-repo
initiative_id: future-monorepo-root-orchestration
---

# Future: Monorepo-Root Orchestration and Multi-Architecture Support

> **STATUS: ACTIVE** — Scoped to Phase 1: per-member architecture support. Cross-package coordination, dashboards, and aggregate views deferred to a follow-up initiative.

## Context

The MVP of Cadre assumes initialization at the level of a single project or package — one repo, one architecture pattern, one set of rules, one planning hierarchy. But many real engineering teams work in monorepos where a single repository root contains multiple packages, apps, services, and libraries, each potentially requiring a different architecture pattern.

When Cadre is initialized at the root of a monorepo, the system should eventually understand that:
- There are multiple packages/apps/services within the repo
- Different packages may need different architecture patterns (e.g., the `web` package uses `javascript/react-app` while the `server` package uses `javascript/server`)
- Some concerns (product definition, shared design context, shared rules) belong at the root/shared level
- Some concerns (package-specific architecture, package-specific rules, package-specific tasks) belong at the package level
- Cross-package features require coordinated work across multiple package-level workspaces

## Why It Matters

Without monorepo-root awareness, teams must either:
- Initialize Cadre separately in each package (losing cross-package coordination)
- Initialize at the root and pretend the whole monorepo has one architecture (which is wrong)

Neither is satisfactory. The future system should support a hierarchical workspace model that reflects how monorepos actually work.

## Governing Commitments

This initiative directly serves (in future):
- **Monorepo-aware governance** (extends Vision #1, #2, #4). The model supports both root-level and package-level governance. Shared repo-wide guidance coexists with architectural specificity within packages.
- **Every repo gets a persisted reference architecture.** In a monorepo, every *package* gets its own Reference Architecture from the catalog — the `web` package can be `javascript/react-app` while `server` is `javascript/server`. Architecture specificity is preserved, not flattened.
- **Architecture-driven rules** (Vision #4). Rule inheritance flows from root to packages. Root-level rules apply everywhere; package-level rules add architecture-specific enforcement seeded from each package's Reference Architecture.
- **All durable project memory lives in the repo.** Root-level shared context (product doc, shared design, shared rules) and package-level specifics (architecture, package rules, package work) are all repo-native artifacts in a structured hierarchy.
- **Planning is durable and traceable from product intent to execution.** Cross-package features decompose from root-level intent to package-level work items, preserving traceability across the monorepo.
- **The system is built around intentional, durable structure.** The two-level workspace model makes monorepo governance explicit and structural, not improvised per-package.

## Goals & Non-Goals

**Goals (future):**
- Detect that Cadre is being initialized at a monorepo root (as opposed to within a single package)
- Support a two-level workspace model: root-level Cadre + package-level Cadre workspaces
- Root level holds: shared product doc, shared design context, shared rules, cross-package coordination
- Package level holds: package-specific architecture (each package has its own Reference Architecture), package-specific rules, package-specific initiative/task breakdowns
- Support cross-package feature planning: a feature request at the root can be decomposed into package-level work items in the relevant packages
- Support multiple architecture profiles within one monorepo (different catalog patterns for different packages)

**Non-Goals:**
- Building this in the MVP — this is explicitly post-MVP
- Supporting arbitrary nesting depth (root → package is sufficient; no root → group → package → subpackage)
- Distributed monorepo support (multiple repos linked together)
- Package-level autonomy (all packages share the same Cadre installation, just different workspaces within it)

## Detailed Design

### Conceptual Model

**Naming convention**: Use dot notation for member subdirectories. The first segment before `.` is the repo directory, the rest is the package name. E.g., `packages.api-core` → `packages/api-core`. This avoids ambiguity with repos that use `-` in directory names.

**Centralized storage**: All member data lives under the root `.cadre/members/` directory, NOT distributed `.cadre/` dirs inside each package.

```
monorepo-root/
  .cadre/                              ← root-level workspace
    product-doc.md                           ← shared product definition
    design-context/                          ← shared design references
    rules/                                   ← shared/root-level rules
    members/
      apps.dashboard/                  ← member workspace (from apps/dashboard)
        reference-arch.md                    ← javascript/react-app
        rules/                               ← member-specific rules (inherits from root)
      packages.api-core/               ← member workspace (from packages/api-core)
        reference-arch.md                    ← javascript/server
        rules/                               ← member-specific rules (inherits from root)
      packages.shared-ui/              ← member workspace (from packages/shared-ui)
        reference-arch.md                    ← javascript/component-lib
        rules/                               ← member-specific rules (inherits from root)
```

### Key Design Considerations

- **Root workspace is scaled down**: The root doesn't have its own architecture — it holds shared context and coordination. Architecture lives at the package level.
- **Rule inheritance**: Package-level rules inherit from root-level rules. Root rules apply everywhere; package rules add or override for their scope.
- **Cross-package features**: A user at the root asks for "add user onboarding flow" → the system identifies this requires work in both `web` and `server` → creates or directs package-level initiatives in each.
- **Architecture per package**: Each package has its own Reference Architecture from the catalog. The root knows about all package architectures but doesn't enforce a single one.
- **Quality gates per package**: Each package has its own analysis baselines and quality thresholds, while the root can enforce aggregate quality standards.

### What to Reuse from Prior Initiatives
- SMET-I-0008 (bootstrap) already detects monorepo structure and package layout — extend it
- SMET-I-0016 (architecture catalog) already supports per-repo Reference Architecture — extend to per-package
- SMET-I-0004 (rules) already supports rule inheritance with directory scoping — extend to package scoping
- SMET-I-0013 (orchestrator) already coordinates work — extend to cross-package coordination

### What is Net New
- Monorepo root detection during initialization
- Two-level workspace model (root + package workspaces)
- Cross-package feature decomposition (root-level feature → package-level work items)
- Package-level workspace initialization within a monorepo
- Root-level coordination views (see work across all packages)
- Aggregate quality dashboards (quality state across all packages)

## Example Future Workflow

1. User runs `cadre init` at monorepo root
2. System detects monorepo structure (Turborepo with `packages/web`, `packages/server`, `packages/shared-ui`)
3. System creates root-level workspace with shared Product Doc
4. System offers to initialize package-level workspaces for each detected package
5. For each package, runs architecture selection flow against that package's language/type
6. `packages/web` gets `javascript/react-app`, `packages/server` gets `javascript/server`, `packages/shared-ui` gets `javascript/component-lib`
7. Each package gets its own Reference Architecture, package-specific rules (seeded from architecture + inheriting root rules), and package-specific analysis baselines
8. Later, user asks: "We need to add user onboarding"
9. System creates a root-level cross-package initiative
10. System identifies this needs work in `web` (onboarding UI) and `server` (onboarding API endpoints)
11. System creates package-level stories in both `web` and `server` workspaces, linked to the root initiative

## Alternatives Considered

1. **Single flat workspace for entire monorepo**: Rejected because it forces one architecture on diverse packages and loses package-level specificity.
2. **Completely independent Cadre installations per package**: Rejected because it loses cross-package coordination, shared rules, and unified product context.
3. **Build this in the MVP**: Rejected because the complexity is substantial and the single-project model must work first.

## Implementation Plan

This is future/backlog work. Implementation should not begin until:
- Core Cadre is working for single-project repos
- Architecture Catalog (SMET-I-0016) is complete
- Rules inheritance (SMET-I-0004) is working
- Orchestrator (SMET-I-0013) is working

When the time comes, likely phases:
1. Design the two-level workspace data model
2. Implement monorepo root detection and multi-workspace initialization
3. Implement package-level workspace management
4. Implement cross-package feature decomposition
5. Implement root-level coordination views
6. Implement aggregate quality dashboards
7. Integration test with real monorepo structures

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria (future)

- Monorepo root initialization creates a root workspace and offers package-level workspace creation
- Each package can have its own Reference Architecture from the catalog
- Root-level rules are inherited by all packages
- Cross-package features can be decomposed into package-level work items
- Root-level views show work status across all packages
- The system gracefully handles packages being added or removed

## Risks / Dependencies

- Depends on all core Cadre initiatives being complete first
- The workspace data model must be designed carefully to avoid tight coupling between root and packages
- Cross-package feature decomposition requires sophisticated understanding of package responsibilities
- Performance: root-level queries that aggregate across many packages must remain fast
- Must not break the single-project model — monorepo support is additive

## Codebase Areas to Inspect (when the time comes)

- Whatever workspace initialization code exists after SMET-I-0008
- Rule inheritance implementation from SMET-I-0004
- Orchestrator coordination logic from SMET-I-0013
- Architecture catalog and reference architecture from SMET-I-0016

## Suggested Tasks for Decomposition (future)

1. Design two-level workspace data model
2. Implement monorepo root detection heuristics
3. Implement root-level workspace initialization
4. Implement package-level workspace initialization within monorepo
5. Implement per-package architecture selection
6. Implement root → package rule inheritance
7. Implement cross-package initiative creation
8. Implement cross-package feature decomposition logic
9. Implement root-level coordination dashboard
10. Implement aggregate quality views
11. Integration test with Turborepo monorepo
12. Integration test with pnpm workspaces monorepo
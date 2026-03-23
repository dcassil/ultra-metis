---
id: cadre-repo-native-ai
level: vision
title: "Cadre: Repo-Native AI Engineering Orchestration for Monorepo"
short_code: "SMET-V-0001"
created_at: 2026-03-11T19:55:32.128760+00:00
updated_at: 2026-03-11T19:59:08.237643+00:00
archived: false

tags:
  - "#vision"
  - "#phase/published"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Cadre: Repo-Native AI Engineering Orchestration for Monorepo

## Purpose

Cadre is a repo-native AI engineering operating system for software delivery. It is designed for a single repository or monorepo and persists the durable state of engineering work directly inside the repo: product intent, architecture, plans, rules, quality history, investigations, approvals, and reusable local insight.

It is not just an AI assistant, not just a planner, and not just a workflow engine. It is the system that lets AI behave like a disciplined software engineer working inside a governed delivery environment.

Its purpose is to support the full path from product intent to validated code change while preserving traceability, architecture alignment, quality context, and human trust. It should work for greenfield and brownfield repos, for single-agent and multi-agent execution, and for tight human collaboration or more autonomous operation. It should use deterministic static tools and existing plugins whenever possible, and only build custom capability where the repo-native, governed, durable layer is uniquely valuable.

The design is centered on three ideas:

1. **Developer work is built from reusable cognitive operations**, not from brittle one-off workflow scripts.
2. **Only durable, reusable, governing, or risk-relevant knowledge should be persisted**; moment-to-moment reasoning should remain internal unless promoted.
3. **The repo is the long-term engineering memory and governance surface**; chat context and transient agent state are not enough.

Build implementation lives in the `cadre/` folder using the original `metis/` repository as the starting point and reference implementation. The goal is not to rebuild Metis from scratch, but to evolve its existing architecture into this stronger operating system.

## Foundational Commitments

These commitments govern all design decisions across the system:

 1. **All durable project memory lives in the repo.** Architecture, planning, rules, quality history, governance artifacts, and reusable insight notes are persisted as repo-native files. The repo is the long-term memory — not chat context, not external tools.
 2. **Every repo gets a persisted reference architecture.** Whether selected from a catalog (greenfield), matched/captured from a strong existing codebase (brownfield-good), or recorded as-is after the user declines a recommendation (brownfield-bad) — no repo operates without an explicit architecture reference.
 3. **The user may keep the current architecture as the governing reference if they choose.** The system supports guided architecture selection, not forced replacement. When an existing architecture is weak, the system recommends a stronger target pattern, but the user may explicitly retain the current architecture and record it as the governing reference.
 4. **Reference architecture drives rules, structure, and analysis.** The reference architecture is a living control artifact. It defines expected structure, module boundaries, dependency direction, naming conventions, and testing shape. Engineering rules are seeded from it, and analysis enforces it.
 5. **Brownfield repos are fully supported.** Existing systems are analyzed, understood, and resolved into an explicit architecture reference so they can participate fully in the same governance model as greenfield repos.
 6. **Quality includes architectural integrity and historical baselines.** Quality is measured as an evolving historical signal — snapshots, comparisons, and trends make progress visible over time. Boundary adherence, dependency direction, and architectural conformance are part of the quality model.
 7. **Planning is durable and traceable from product intent to execution.** ProductDoc → Epic → Story → Task creates clear traceability. Workflow progression is backed by persisted artifacts, not temporary conclusions.
 8. **All workflows are composed from a fixed set of reusable cognitive operations.** The system uses a cognitive operation kernel — frame objective, acquire context, build model, locate focus, analyze boundaries, trace flow, assess impact, shape solution, decompose work, create artifact, validate, reassess — as the universal reasoning substrate for all work types.
 9. **Internal reasoning stays internal unless promoted.** Ephemeral reasoning (file guesses, temporary hypotheses, micro-sequencing) remains in the model. Only confirmed, reusable, governing, cross-agent relevant, or risk-relevant knowledge is promoted to durable state. This prevents transcript archives while preserving meaningful repo memory.
10. **Static tools are preferred over unconstrained reasoning.** When a question can be answered by a tool, prefer the tool. When a constraint can be enforced by a tool, prefer the tool. When a validation can be produced by a tool, require the tool result. AI reasoning selects tools, interprets outputs, synthesizes decisions, and creates durable records.
11. **Parallel execution is enabled through explicit ownership and isolation.** Work leasing allows humans and agents to operate safely in parallel while preserving coordination through durable state.
12. **Single-agent and orchestrated modes share one governance model.** Planning, governance, and quality semantics remain consistent regardless of execution scale.
13. **The system supports multiple autonomy modes.** Tight collaboration (frequent human approval), mixed mode (proceed within bounds, escalate on risk), and autonomous mode (proceed without routine approval, respecting gates and thresholds). Mode affects what can be changed directly, what requires approval, and what contradictions can be tolerated.
14. **Work is complete when required evidence exists, not when code changes.** Completion requires success criteria satisfied, validations passed, architecture conflicts resolved, rules complied with, artifacts updated, and approvals logged.
15. **The system is built around intentional, durable structure.** Its value comes from making the desired path clear, durable, and operationally grounded — structural guidance over improvisation.

## Product Overview

**Target audience:** Engineering teams and AI agents working within a single repo or monorepo, where planning, design, execution, and quality enforcement need to be tightly integrated and durable.

**Key benefits:**

- **Cognitive operation kernel** — all workflows composed from 12 reusable engineering operations, not brittle one-off scripts
- **Reusable loops** — operations compose into objective framing, context sufficiency, model construction, focus narrowing, trace, risk/impact, solution shaping, decomposition, artifact production, validation, and adaptation loops
- Curated architecture catalog with selection flow — repos get intentional, enforced architecture rather than improvised structure
- Richer planning hierarchy (ProductDoc → Epic → Story → Task) connecting product intent through design to implementation, with typed Stories (feature, bugfix, refactor, migration, investigation, etc.)
- Protected engineering rules that cannot be casually modified, with architecture-derived rule seeding and layered rule scopes (platform → org → repo → package → component → task)
- Static analysis baselines with quality gates, degradation investigations, and architecture-aware boundary enforcement
- **Durable insight note system** — lightweight, self-pruning repo memory for hotspots, recurring patterns, validation hints, and subsystem quirks
- **Execution records** as the audit spine — every work run traces intent, context, tools, validations, decisions, and disposition
- **Gates, escalation, and autonomy modes** — explicit control points with configurable human-in-the-loop from tight collaboration to autonomous operation
- **Static-tool-first execution** — deterministic tools preferred over unconstrained reasoning; AI selects, interprets, and records
- **Plugin leverage strategy** — aggressive use of existing tools for execution while owning the durable repo-native layer
- Brownfield-aware initialization that evaluates existing repos and maps them to stable architecture patterns
- Durable repo-local artifacts as the source of truth, not chat context

## Current State

The original Metis system (`metis/`) provides a solid foundation:

- File-based durable documents with markdown + frontmatter structure
- A three-level planning hierarchy: Vision → Initiative → Task
- Phase-based workflow with forward-only transitions
- MCP server integration for AI tool access
- CLI workflows for human interaction
- GUI foundations for visualization
- Template systems for document creation
- SQLite-backed indexing and search
- Code indexing capabilities

However, it lacks:

- Product-level definition and design reference handling
- Richer planning levels between strategic vision and execution tasks
- Architecture catalog and selection flow — no way to choose, persist, or enforce intentional repo architecture
- Engineering rule enforcement and protected configuration with layered scopes
- Architecture-derived rule seeding and boundary enforcement
- Static analysis baselines, quality records, and quality gates
- Investigation/remediation workflows for quality degradation
- Brownfield architecture evaluation — no way to analyze and normalize an existing repo's structure
- Cognitive operation kernel — no reusable operation/loop abstraction for composing workflows
- Durable insight note system — no lightweight repo memory layer for reusable local knowledge
- Execution records and audit spine — no durable traceability of what happened during work runs
- Gates, escalation, and autonomy model — no configurable control points or human-in-the-loop modes
- Internal cognition vs durable persistence design rule — no promotion mechanism for when to persist
- Work leasing and isolated execution ownership (post-MVP)
- Orchestrator/runner modes for multi-agent coordination (post-MVP)

## Future State

Cadre should be a complete repo-native AI engineering operating system that supports:

### Core Operating Model

- **Cognitive Operation Kernel**: All workflows composed from 12 reusable operations — frame objective, acquire context, build/refine model, locate focus, analyze structure/boundaries, trace flow/causality, assess impact/risk, shape/select solution, decompose/sequence work, create/modify artifact, validate against reality, reassess/adapt. These operations are the universal reasoning substrate for bug fixes, feature work, refactors, migrations, architecture changes, brownfield evaluation, greenfield setup, remediation, code review, and quality investigations.
- **Reusable Loops**: Operations compose into loops — objective framing, context sufficiency, model construction, focus narrowing, trace, risk/impact, solution shaping, decomposition, artifact production, validation, adaptation. A "workflow" is a predefined composition of loops with entry conditions, required artifacts, required validations, escalation rules, and completion rules. This is the central abstraction of the system.
- **Internal Cognition vs Durable Persistence**: First-class design rule with three state categories — (A) ephemeral internal reasoning kept inside the model, (B) durable lightweight insight stored as notes, (C) durable governed records stored as formal docs. Promotion rules define when internal information should be promoted: when it becomes confirmed, reusable, governing, cross-agent relevant, human-review relevant, risk-relevant, repeated enough to matter, or needed for auditability.

### Planning Artifacts

- **Product Doc**: A repo-level product definition that anchors all planning
- **Architecture Catalog**: A curated library of approved architecture patterns organized by language and project type (e.g., javascript/server, javascript/react-app, javascript/component-lib), each defining folder structure, layers, dependency rules, module boundaries, naming conventions, testing layout, anti-patterns, and seed data for rules and analysis expectations
- **Reference Architecture**: The selected (or derived) architecture pattern for this repo, persisted as a durable artifact and used as the stable source of truth for folder structure, package structure, layering, dependency direction, naming conventions, testing placement, rules generation, and static analysis expectations
- **Design Context**: References to approved UI patterns, design specs, and visual standards
- **Epics**: Major product capabilities or engineering initiatives that group related implementation work
- **Stories**: Implementation slices within epics, typed by purpose (feature, bugfix, refactor, migration, architecture-change, investigation, remediation, setup/bootstrap), sized for meaningful delivery
- **Tasks**: Execution-level work items with clear ownership

### Architecture and Design Artifacts

- **Architecture Catalog**: Reusable patterns with name, category, structure expectations, dependency rules, layering rules, testing expectations, common risks, validation patterns, rule seeding hints
- **Reference Architecture**: Per-repo (or per-package) persisted structural intent with layers, boundaries, dependency directions, ownership, naming conventions, integration seams, validation expectations, known tolerated exceptions
- **Design Context**: References to approved UI patterns, design specs, and visual standards
- **Design Change Proposals**: Structured proposals for design-level changes
- **Architecture Investigations**: Triggered when quality degrades, repeated failures occur, architecture drift appears, or enforcement and actual code shape diverge

### Governance Artifacts

- **Rules Config**: Protected engineering rules with controlled change workflows, layered scopes (platform → org → repo → package → component → task), typed by purpose (behavioral, architectural, operational, information-handling, decision-making, validation/quality, approval/escalation, execution-safety), partially seeded from the selected architecture pattern
- **Approval Records**: Durable records of who approved what, when, and why
- **Validation Policies**: Configurable policies defining what validations are required for different work types
- **Ownership Maps**: Who is responsible for what scope
- **Constraint Records**: Explicit constraints that govern decision boundaries

### Quality Artifacts

- **Analysis Baselines**: Point-in-time quality snapshots from deterministic tool outputs (lint, type-check, tests, coverage, dependency analysis, security, complexity, dead code)
- **Quality Records**: Baseline comparisons with regressions, improvements, threshold breaches, blocked transitions, accepted overrides
- **Validation Records**: Captures validation type, inputs, result, failures, evidence links, whether required or optional — critical for audits and autonomous modes
- **Remediation Records**: Tracks detected problems, affected scope, required fixes, validation after fix, recurrence signals

### Execution and Traceability Artifacts

- **Execution Records**: The audit spine — every meaningful work run records initiating artifact, execution mode, context sources, architecture/rules consulted, notes fetched, tools run, files touched, validations run, durable artifacts updated, decisions made, escalations/overrides, and final disposition
- **Transition Records**: Audit trail of all phase transitions with full metadata
- **Decision Records**: Durable records of significant decisions with rationale
- **Cross-Reference Index**: Queryable graph of all document relationships (parent/child, governs, references, derived-from, supersedes, conflicts-with, validates, blocks, approved-by)

### Durable Insight Note System

- **Durable Insight Notes**: Lightweight repo memory for compressed, local, reusable insight — hotspot warnings, misleading naming patterns, recurring bug signatures, validation hints, subsystem gotchas, local exception patterns. Scoped to repo/package/subsystem/path/symbol. Self-pruning through use-driven feedback: notes are scored on fetch (helpful/meh/harmful), marked as prune candidates when unused or harmful, flagged for human review on conflicts, and archived when superseded. Integrated into task start (fetch relevant notes) and task wrap-up (score and propose notes).

### Gates, Escalation, and Autonomy

- **Major Gates**: Entry gate, context sufficiency gate, solution gate, execution readiness gate, validation gate, completion gate, escalation gate — abstract control points attachable to different workflows
- **Escalation Triggers**: Insufficient evidence, unresolved contradiction, policy conflict, high-impact change, architecture mismatch, security/safety concern, failing required validation, uncertainty above threshold, business ambiguity with material impact
- **Autonomy Modes**: Tight collaboration (human approval required often), mixed mode (AI proceeds within bounds, escalates on risk — the default), autonomous mode (AI proceeds without routine approval, respects gates and thresholds). Mode affects what can be changed, what requires approval, evidence requirements, and whether work can be decomposed and dispatched automatically.

### Execution Model

- **Static-Tool-First**: Prefer deterministic tools (repo discovery, code search, dependency graphs, test runners, linters, type checkers, architecture validators, security analyzers, coverage tools, build tools, formatters, diff/review tools) over unconstrained reasoning
- **Plugin Leverage**: Use existing tools/plugins for structured execution, subagent dispatch, code review, PR review, git workflows, hooks/enforcement, security guidance, static analysis, repo structure detection, code search, live docs lookup, GUI prototyping, browser debugging, E2E testing
- **Work Leases**: Isolated execution ownership for work in progress (post-MVP)
- **Quality-Gated Execution**: Enforcement of quality standards before work can proceed

For **brownfield / existing repos**, the AI should evaluate the current repo architecture's quality — including running static analysis tools as part of the assessment. If the existing architecture is coherent and strong, the system matches it to a known catalog pattern or creates a custom reference to capture it faithfully. If the existing architecture is weak or incoherent, the system should identify the project's type and intent, recommend a first-class catalog pattern, and explain that a refactor would be needed — including which areas of the codebase would be affected. The user is given a clear choice: accept the recommended architecture (making refactoring the first order of work) or decline, in which case the current architecture is recorded as the reference despite its quality. No repo is left without a durable architecture reference.

For **monorepo roots** (future/non-MVP), the system should eventually support detecting monorepo root initialization, distinguishing root-level shared context from package-level workspaces, and allowing different packages to have different architecture profiles while coordinating work across them.

## Major Features

### MVP Features (What Cadre Must Uniquely Own)

- **Cognitive operation kernel and reusable loops**: The universal reasoning substrate — 12 operations composing into loops that define all workflows. This is the central abstraction replacing brittle one-off workflow scripts.
- **Core planning hierarchy**: ProductDoc → Epic → Story → Task with typed Stories (feature, bugfix, refactor, migration, architecture-change, investigation, remediation, setup/bootstrap) and required planning fields (objective, scope, rationale, acceptance criteria, dependencies, architecture relevance, validation expectations, risk level)
- **Durable state model and storage**: Repo-native persistence for all artifact classes — planning, architecture, governance, quality, execution, and memory artifacts. All durable objects cross-linkable and queryable.
- **Architecture catalog and selection**: A curated catalog of architecture patterns organized by language/project type. During setup, the system suggests strong options, explains tradeoffs, and lets the user choose. The selected architecture becomes a durable reference that guides all downstream planning, rules, and analysis.
- **Brownfield architecture evaluation**: For existing repos, analyze current structure, inspect import/dependency shape, naming/layout conventions, and quality signals. Determine whether architecture is strong/coherent, mixed/salvageable, or weak/drifting. Match to catalog, derive local reference, or capture current-state and target-state separately. System does not assume brownfield repos must be "fixed first" but makes drift visible and queryable.
- **Architecture-driven rules and analysis**: The selected architecture seeds engineering rules with layered scopes and defines analysis expectations. Rules typed by purpose with change proposal flow for protected rules.
- **Internal cognition vs durable persistence**: Formal promotion rules defining when internal reasoning becomes durable state. Three categories enforced throughout.
- **Durable insight note system**: Lightweight self-pruning repo memory with fetch tracking, use-driven feedback (helpful/meh/harmful), automatic prune candidate detection, human review flagging on conflicts, and archival. Integrated into task start and wrap-up.
- **Execution records and audit spine**: Every meaningful work run emits a durable record linking intent, context, tools, validations, decisions, and disposition. This is the traceability backbone.
- **Gates, escalation, and autonomy model**: Seven major gates as abstract control points, explicit escalation triggers, and three configurable autonomy modes.
- **Baseline capture/storage/comparison shell**: Ingest external tool outputs (lint, type-check, tests, coverage, dependency analysis, security, complexity, dead code) rather than building custom scanners.
- **Governance artifact types**: RulesConfig, ApprovalRecord, ValidationPolicy, OwnershipMap, ConstraintRecord with layered scoping and protection semantics.
- **Quality artifact types**: AnalysisBaseline, QualityRecord, ValidationRecord, RemediationRecord.
- **Workflow states and traceability backbone**: Transition records, decision records, cross-reference index with typed relationships.
- **Protected engineering rules**: Rules that require explicit change proposals and approval, not casual modification. Protected rules require propose → review → approve/reject → apply → record flow.
- **Static analysis integration**: Baselines, quality records, comparison tooling, and quality gates that block execution when standards degrade.
- **Remediation loops**: Structured investigation and remediation workflows triggered by quality degradation or architecture violations.
- **MCP surface**: Expose the full durable operating system — CRUD for all artifacts, quality operations, rule management, traceability queries, note fetch/score, baseline capture/comparison, validation recording, workflow states.
- **CLI surface**: Developer-native access to all operations including init/bootstrap, create/query, architecture/rules inspection, validation recording, baseline capture, quality deltas, traceability, note inspection, JSON export.
- **Templates**: Artifact templates (all document types) and workflow templates (bugfix, feature slice, refactor, migration, architecture change, brownfield evaluation, remediation, investigation, greenfield bootstrap) — context-aware, prefilling from parent artifacts and architecture scope.
- **Migration path**: Smooth upgrade from original Metis concepts to Cadre concepts.
- **Design-aware planning**: First-class references to design specs, approved patterns, and visual standards linked to implementation work.
- **Plugin-based execution**: Skills, agents, hooks, and execution commands delivered as a Claude Code plugin leveraging cadre MCP tools for state management.

### Post-MVP Features (Leverage Existing Tools or Defer)

- **Work leasing**: Isolated execution ownership so agents can claim and work on tasks without conflicts. Defer unless adoption pressure is strong.
- **Git worktree isolation**: Automated worktree lifecycle for leased work. Defer — existing superpowers plugin handles worktrees.
- **Multi-agent orchestrator**: Full orchestrated execution with work decomposition, scoped dispatch, conflict detection, execution log merge. Defer until repo-native state layer is mature.
- **Full custom rule execution engine**: For MVP, partially delegate runtime enforcement to existing plugins and hooks. Cadre owns the persisted governed rule model and traceability.
- **GUI productization**: Valuable but not MVP-critical. Prototype with existing rapid GUI tools. Eventually: hierarchy tree, architecture browser, traceability graph, quality dashboards, note browser, investigation tracker, rule browser, blocked work view.
- **Bespoke analyzers**: Use existing tools where they suffice; only build custom when uniquely valuable.
- **Monorepo-root orchestration**: Cross-package coordination with per-package architecture profiles. Defer until single-project model is proven.

## Success Criteria

 1. Cadre can represent a complete product development lifecycle from product definition through architecture selection, design, planning, execution, and quality enforcement
 2. The cognitive operation kernel and reusable loops are implemented as the universal reasoning substrate — all workflow templates compose from them
 3. A curated architecture catalog exists with practical patterns for common project types, starting with JavaScript/TypeScript
 4. Every initialized repo has a persisted Reference Architecture — either selected from the catalog (greenfield), matched/captured from a strong existing architecture (brownfield-good), or recommended as a replacement for a weak existing architecture with user consent (brownfield-bad) — with the user always having final say
 5. The selected architecture drives rule generation, analysis expectations, and planning guidance
 6. Engineering rules are enforced and cannot be bypassed without explicit approval workflows, with layered scoping from platform to task level
 7. The durable insight note system captures, fetches, scores, prunes, and archives reusable local knowledge as a self-maintaining repo memory layer
 8. Execution records provide a complete audit spine linking intent, context, tools, validations, decisions, and disposition for every meaningful work run
 9. Gates, escalation triggers, and autonomy modes are configurable and functional across all workflow types
10. The internal cognition vs durable persistence design rule is enforced — promotion rules govern when internal reasoning becomes durable state
11. Static analysis baselines are tracked, compared, and used to gate execution — including architecture boundary enforcement
12. All original Metis capabilities (CLI, MCP, search, indexing) are preserved and extended
13. A working migration path exists from Metis to Cadre document models
14. The system is auditable, explainable, bounded by rules, architecture-aware, evidence-backed, mode-aware, adaptable, and self-correcting through review, validation, and note pruning
15. The system is usable in a real repo for real software development work

## Principles

 1. **Documents are durable memory, not chat context.** All planning, design, rules, quality data, and reusable insight must be persisted as repo-local artifacts.
 2. **The repo and durable artifacts are the source of truth.** Not conversation history, not external tools.
 3. **Enforced structure is more important than prompt-only behavior.** The system should make it hard to do the wrong thing, not just suggest the right thing.
 4. **Product, design, architecture, and code quality should be connected.** Traceability from product intent through design to implementation to quality outcomes.
 5. **Architecture should be explicit, not improvised.** Every repo should have a selected or derived architecture pattern persisted as a durable reference. The AI should never improvise repo structure — it should follow the chosen architecture.
 6. **Workflows compose from reusable cognitive operations.** The operation kernel and loop model replace brittle one-off workflow scripts with a universal, auditable reasoning substrate.
 7. **Only promote what matters.** Internal reasoning stays internal. Only confirmed, reusable, governing, cross-agent relevant, or risk-relevant knowledge becomes durable state.
 8. **Static tools first, AI reasoning second.** Prefer deterministic tools for answering questions, enforcing constraints, and producing validations. AI reasoning selects, interprets, synthesizes, and records.
 9. **Build only what is uniquely valuable.** Use existing plugins and tools aggressively for execution, review, enforcement, scanning, and docs. Reserve custom investment for the durable repo-native layer that other tools do not provide.
10. **The same model should support both single-agent and orchestrated workflows.** No separate systems for different execution modes.
11. **Extend, don't rebuild.** Reuse and evolve Metis foundations wherever possible.
12. **Repo-native by default.** Everything lives in the repo. No external dependencies for core functionality.

## Constraints

1. Must build on top of existing Metis foundations — not a greenfield rewrite
2. Must remain repo-local and file-based — no external service dependencies for core functionality
3. Must preserve backward compatibility with existing Metis document formats during migration
4. The `metis/` folder is read-only reference; all new work goes into `cadre/`
5. Must work with current MCP server patterns and Claude Code integration
6. Rust codebase — maintain language consistency with original Metis

## Durable Repo Structure

```
.metis/
  visions/                    # Vision documents
  strategies/                 # Strategy documents (full preset)
    SMET-S-NNNN/
      initiatives/            # Initiatives under strategy
        SMET-I-NNNN/
          tasks/              # Tasks under initiative
  initiatives/                # Initiatives (streamlined preset, direct under vision)
  tasks/                      # Standalone backlog items
  adrs/                       # Architecture Decision Records
  config.yaml                 # Project configuration (preset, prefix, enabled types)
```

## Workspace Model

Cadre is structured as a Rust monorepo:

```
crates/
  cadre-core/     # Domain types, templates, operations kernel
  cadre-store/    # File persistence layer
  cadre-mcp/      # MCP server binary
  cadre-cli/      # CLI binary
plugins/
  cadre/          # Claude Code plugin (skills, agents, hooks, commands)
apps/                   # Future: control-web, control-api, machine-runner
```

The `.metis/` directory in any project holds durable state. The `plugins/cadre/` directory holds the Claude Code plugin that provides skills, guidance agents, lifecycle hooks, and execution commands on top of the MCP tool layer.
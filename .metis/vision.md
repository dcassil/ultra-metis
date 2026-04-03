---
id: cadre-repo-native-ai
level: vision
title: "Cadre: Repo-Native AI Engineering Orchestration for Monorepo"
short_code: "SMET-V-0001"
created_at: 2026-03-11T19:55:32.128760+00:00
updated_at: 2026-03-19T00:00:00.000000+00:00
archived: false

tags:
  - "#vision"
  - "#phase/published"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# CADRE: Constrained Autonomous Developer [Really Really Awesome] Engine

## Purpose

CADRE was built by mapping, in detail, every step a senior software engineer follows when working a bug or building a feature — the discovery, the context gathering, the reasoning, the mental models held in memory, the coding, the validation. That map was then layered with deterministic static tools (linters, type checkers, test runners), first-class architecture patterns and code principles, and a structured product planning workflow modeled on kanban and scrum.

The result is a repo-native engine where the human acts as PM and architect — owning product vision, design direction, and architecture decisions — and AI acts as a developer or team of developers executing within a governed, structured environment.

The core mechanism is **cascading curated context**. Product vision and design define what we are making, which is recorded in epics. Epics, informed by vision, design, and a working knowledge of architecture, shape stories. Stories, informed by architecture and the relevant slices of product and design, shape tasks. Each level concentrates and curates exactly the context the next level needs. By the time a task reaches a subagent, it carries a small, self-contained context blob — no redundant codebase discovery, no re-reading major docs, no context compression.

The design is governed by three principles:

1. **Deterministic tools over AI reasoning, everywhere possible.** If a linter can catch it, don't ask AI to notice it. If a dependency graph can answer "what files are affected," don't let AI search. Every token spent on discovery or re-learning is waste. The system front-loads that work into durable, queryable artifacts — dependency graphs, tagged notes, architecture maps — so AI operates on pre-computed knowledge, not open-ended exploration.

2. **Only durable, reusable, governing, or risk-relevant knowledge is persisted.** Moment-to-moment reasoning stays internal to the model. Only confirmed findings, reusable insight, governing decisions, and risk-relevant context are promoted to durable repo state. This keeps the system clean without losing what matters.

3. **The repo is the long-term memory and governance surface.** Architecture, product intent, design references, plans, rules, quality history, and reusable insight are persisted as repo-native files. Chat context and transient agent state are not durable — the repo is.

## Foundational Commitments

These commitments govern all design decisions across the system:

 1. **All durable project memory lives in the repo.** Architecture, planning, rules, quality history, governance artifacts, and reusable insight notes are persisted as repo-native files. The repo is the long-term memory — not chat context, not external tools.
 2. **Every repo gets a persisted reference architecture.** The system maintains a growing library of architecture patterns organized by language, purpose, and framework (e.g., TypeScript + React + Atomic Design). For **greenfield repos**, the system walks the user through selection — language, purpose, frameworks, libraries — and suggests one or more matching patterns from the library. For **existing repos**, the system asks AI to summarize the current architecture and evaluate how well it conforms to industry best practices. From there: if the architecture is strong and matches an existing library pattern at 95%+, that pattern becomes the reference. If it's strong but only a partial match (75%+), the closest library pattern is used as a base and AI defines a custom reference with the necessary modifications. If the architecture is strong and novel — no good library match — AI defines it as a new pattern, and the user is asked whether they'd be willing to submit it to the architecture library for future users (no code is shared, only structural concepts). If the architecture is weak, the system recommends a better-fitting pattern from the library; the user may accept (which flags refactoring initiatives for non-conforming areas) or decline, in which case the current architecture is captured as-is so the system still has a defined reference to work against. The user always has final say. The system guides architecture selection, it does not force it.
 3. **Reference architecture drives rules, structure, and analysis.** The reference architecture is a living control artifact. It defines expected structure, module boundaries, dependency direction, naming conventions, and testing shape. Engineering rules are seeded from it, and analysis enforces it.
 4. **Quality includes architectural integrity and historical baselines.** Quality is measured as an evolving historical signal — snapshots, comparisons, and trends make progress visible over time. Boundary adherence, dependency direction, and architectural conformance are part of the quality model.
 5. **Planning is durable and traceable from product intent to execution.** ProductDoc → Epic → Story → Task creates clear traceability. Workflow progression is backed by persisted artifacts, not temporary conclusions.
 6. **All workflows are composed from a fixed set of reusable cognitive operations.** The system uses a cognitive operation kernel — frame objective, acquire context, build model, locate focus, analyze boundaries, trace flow, assess impact, shape solution, decompose work, create artifact, validate, reassess — as the universal reasoning substrate for all work types.
 7. **Internal reasoning stays internal unless promoted.** Ephemeral reasoning (file guesses, temporary hypotheses, micro-sequencing) remains in the model. Only confirmed, reusable, governing, cross-agent relevant, or risk-relevant knowledge is promoted to durable state. This prevents transcript archives while preserving meaningful repo memory.
 8. **Static tools are preferred over unconstrained reasoning.** When a question can be answered by a tool, prefer the tool. When a constraint can be enforced by a tool, prefer the tool. When a validation can be produced by a tool, require the tool result. AI reasoning selects tools, interprets outputs, synthesizes decisions, and creates durable records.
 9. **The system supports configurable autonomy during execution.** The human and AI always collaborate on Product Vision, Design, and Architecture — that phase is inherently interactive. Once the planning foundation is set, execution runs in one of two modes: **Full auto** — AI executes end-to-end, with optional approval gates at epic, story, or task boundaries. **Interactive** — AI follows agent/model defaults for how often to pause and request input.
10. **Work is complete when required evidence exists, not when code changes.** Completion requires success criteria satisfied, validations passed, architecture conflicts resolved, rules complied with, artifacts updated, and approvals logged.
11. **The system is built around intentional, durable structure.** Its value comes from making the desired path clear, durable, and operationally grounded — structural guidance over improvisation.

## Product Overview

**Target audience:** Developers who want to direct AI as their engineering team within a structured, governed system — owning product vision, design, and architecture while AI handles execution with full context and deterministic guardrails.

**Key benefits:**

- **Cognitive operation kernel** — all workflows composed from 12 reusable engineering operations, not brittle one-off scripts
- **Reusable loops** — cognitive operations compose into workflow loops applicable to any work type (bug fixes, features, refactors, migrations, investigations)
- Curated architecture catalog with selection flow — repos get intentional, enforced architecture rather than improvised structure
- Richer planning hierarchy (ProductDoc → Epic → Story → Task) connecting product intent through design to implementation, with typed Stories (feature, bugfix, refactor, migration, investigation, etc.)
- Protected engineering rules that cannot be casually modified, with architecture-derived rule seeding and layered rule scopes (platform → org → repo → package → component → task)
- Static analysis baselines with quality gates, degradation investigations, and architecture-aware boundary enforcement
- **Durable insight note system** — lightweight, self-pruning repo memory for hotspots, recurring patterns, validation hints, and subsystem quirks
- **Execution records** as the audit spine — every work run traces intent, context, tools, validations, decisions, and disposition
- **Configurable autonomy** — full auto with optional gates, or interactive with agent/model defaults
- **Static-tool-first execution** — deterministic tools preferred over unconstrained reasoning; AI selects, interprets, and records
- **Static tools first, then existing plugins, then custom** — prefer deterministic static tools over AI reasoning. When a workflow needs more, use existing plugins if they're a good fit — don't force something that doesn't match. Build custom capability only when no existing plugin fits or when the workflow needs tighter integration than an external plugin can provide.
- Brownfield-aware initialization that evaluates existing repos and maps them to stable architecture patterns
- Durable repo-local artifacts as the source of truth, not chat context

## Future State

Cadre should be a complete repo-native AI engineering operating system that supports:

### Core Operating Model

- **Cognitive Operation Kernel**: All workflows composed from 12 reusable operations — frame objective, acquire context, build/refine model, locate focus, analyze structure/boundaries, trace flow/causality, assess impact/risk, shape/select solution, decompose/sequence work, create/modify artifact, validate against reality, reassess/adapt. These operations are the universal reasoning substrate for bug fixes, feature work, refactors, migrations, architecture changes, brownfield evaluation, greenfield setup, remediation, code review, and quality investigations.
- **Reusable Loops**: Operations compose into loops — objective framing, context sufficiency, model construction, focus narrowing, trace, risk/impact, solution shaping, decomposition, artifact production, validation, adaptation. A "workflow" is a predefined composition of loops with entry conditions, required artifacts, required validations, escalation rules, and completion rules. This is the central abstraction of the system.
- **Internal Cognition vs Durable Persistence**: First-class design rule with three state categories — (A) ephemeral internal reasoning kept inside the model, (B) durable lightweight insight stored as notes, (C) durable governed records stored as formal docs. Promotion rules define when internal information should be promoted: when it becomes confirmed, reusable, governing, cross-agent relevant, human-review relevant, risk-relevant, repeated enough to matter, or needed for auditability.

### Planning Artifacts

- **Product Doc**: A repo-level product definition that anchors all planning
- **Design Context**: References to approved UI patterns, design specs, and visual standards
- **Epics**: Major product capabilities or engineering initiatives that group related implementation work
- **Stories**: Implementation slices within epics, typed by purpose (feature, bugfix, refactor, migration, architecture-change, investigation, remediation, setup/bootstrap), sized for meaningful delivery
- **Tasks**: Execution-level work items with clear ownership

### Governance Artifacts

- **Rules Config**: Protected engineering rules with controlled change workflows, layered scopes (platform → org → repo → package → component → task), typed by purpose (behavioral, architectural, operational, information-handling, decision-making, validation/quality, approval/escalation, execution-safety), partially seeded from the selected architecture pattern
- **Approval Records**: Durable records of who approved what, when, and why
- **Validation Policies**: Configurable policies defining what validations are required for different work types
- **Ownership Maps**: Who is responsible for what scope
- **Constraint Records**: Explicit constraints that govern decision boundaries
- **Design Change Proposals**: Structured proposals for design-level changes
- **Architecture Investigations**: Triggered when quality degrades, repeated failures occur, architecture drift appears, or enforcement and actual code shape diverge

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

### Remote AI Operations

Monitor and interact with running AI sessions from any device. When AI needs approval, input, or a decision, respond from your phone instead of being tethered to the terminal. A mobile-first dashboard shows all active sessions across projects with live progress, pending prompts, and session history.

### Benchmarking and Evaluation

A repeatable framework for measuring how well AI performs when using CADRE end-to-end: document generation quality, work decomposition accuracy, and final build outcome. Tracks improvements and regressions across system revisions, and compares CADRE execution against original Metis to validate that the structured approach produces better outcomes.

## Major Features

### MVP Features (What Cadre Must Uniquely Own)

- **Cognitive operation kernel and reusable loops**: The universal reasoning substrate — 12 operations composing into loops that define all workflows. This is the central abstraction replacing brittle one-off workflow scripts.
- **Core planning hierarchy**: ProductDoc → Epic → Story → Task with typed Stories (feature, bugfix, refactor, migration, architecture-change, investigation, remediation, setup/bootstrap) and required planning fields (objective, scope, rationale, acceptance criteria, dependencies, architecture relevance, validation expectations, risk level)
- **Durable state model and storage**: Repo-native persistence for all artifact classes — planning, architecture, governance, quality, execution, and memory artifacts. All durable objects cross-linkable and queryable.
- **Architecture catalog and selection**: A curated catalog of architecture patterns organized by language/project type. During setup, the system suggests strong options, explains tradeoffs, and lets the user choose. The selected architecture becomes a durable reference that guides all downstream planning, rules, and analysis.
- **Brownfield architecture evaluation**: For existing repos, analyze current structure, inspect import/dependency shape, naming/layout conventions, and quality signals. Determine whether architecture is strong/coherent, mixed/salvageable, or weak/drifting. Match to catalog, derive local reference, or capture current-state and target-state separately.
- **Architecture-driven rules and analysis**: The selected architecture seeds engineering rules with layered scopes and defines analysis expectations. Rules typed by purpose with change proposal flow for protected rules.
- **Internal cognition vs durable persistence**: Formal promotion rules defining when internal reasoning becomes durable state. Three categories enforced throughout.
- **Durable insight note system**: Lightweight self-pruning repo memory with fetch tracking, use-driven feedback (helpful/meh/harmful), automatic prune candidate detection, human review flagging on conflicts, and archival. Integrated into task start and wrap-up.
- **Execution records and audit spine**: Every meaningful work run emits a durable record linking intent, context, tools, validations, decisions, and disposition. This is the traceability backbone.
- **Configurable autonomy**: Full auto with optional approval gates at epic/story/task boundaries, or interactive following agent/model defaults.
- **Baseline capture/storage/comparison**: Ingest external tool outputs (lint, type-check, tests, coverage, dependency analysis, security, complexity, dead code) rather than building custom scanners.
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
- **Plugin-based execution**: Skills, agents, hooks, and execution commands delivered as a Claude Code plugin leveraging CADRE MCP tools for state management. Existing plugins (superpowers, ralph loop, etc.) used wherever they're a good fit.
- **Remote AI Operations MVP**: Monitor and interact with running AI sessions from any device — approve, respond, and unblock sessions from your phone across multiple projects.
- **Benchmarking**: Repeatable framework for measuring CADRE execution quality (document generation, decomposition, build outcome) and comparing against original Metis.

### Post-MVP Features

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
 9. Autonomy modes (full auto with optional gates, interactive) are configurable and functional
10. The internal cognition vs durable persistence design rule is enforced — promotion rules govern when internal reasoning becomes durable state
11. Static analysis baselines are tracked, compared, and used to gate execution — including architecture boundary enforcement
12. Remote AI Operations MVP: developers can monitor and interact with running AI sessions from any device
13. Benchmarking framework produces repeatable measurements of CADRE execution quality and comparison against original Metis
14. The system is auditable, evidence-backed, architecture-aware, and mode-aware
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
 9. **Static tools first, existing plugins second, custom build third.** Use existing plugins and tools when they're a good fit. Build custom capability only when no existing tool fits or when the workflow needs tighter integration than an external plugin can provide. Reserve custom investment for the durable repo-native layer that other tools do not provide.
10. **Repo-native by default.** Everything lives in the repo. No external dependencies for core functionality.

## Constraints

1. Must remain repo-local and file-based — no external service dependencies for core engine functionality
2. Must work with current MCP server patterns and Claude Code integration
3. Rust for core engine crates; TypeScript/Next.js for remote operations dashboard
4. Static tools and existing plugins preferred over custom solutions

## Durable Repo Structure

```
.cadre/
  product-docs/               # Product vision documents
  epics/                      # Epics
    CADRE-E-NNNN/
      stories/                # Stories under epic
        CADRE-S-NNNN/
          tasks/              # Tasks under story
  tasks/                      # Standalone backlog items
  adrs/                       # Architecture Decision Records
  config.yaml                 # Project configuration
```

## Workspace Model

CADRE is structured as a polyglot monorepo:

```
crates/
  ultra-metis-core/     # Domain types, templates, operations kernel
  ultra-metis-store/    # File persistence layer
  ultra-metis-mcp/      # MCP server binary
  ultra-metis-cli/      # CLI binary
  ultra-metis-agents/   # Agent coordination (planned)
  ultra-metis-events/   # Event system (planned)
  ultra-metis-notes/    # Notes system (planned)
  ultra-metis-policy/   # Policy engine (planned)
plugins/
  ultra-metis/          # Claude Code plugin (skills, agents, hooks, commands)
apps/
  control-web/          # Remote operations dashboard (Next.js)
  control-api/          # Control service API
  machine-runner/       # Local execution daemon
packages/
  config/               # Shared configuration
  shared-contracts/     # Shared types/contracts
  ui/                   # Shared UI components
infra/                  # Deployment infrastructure
benchmarks/             # Benchmark framework
docs/                   # Documentation
```

The `.cadre/` directory in any project holds durable state. The `plugins/ultra-metis/` directory holds the Claude Code plugin that provides skills, guidance agents, lifecycle hooks, and execution commands on top of the MCP tool layer.

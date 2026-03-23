---
id: ultra-metis-core-engine-repo
level: strategy
title: "Ultra-Metis Core Engine: Repo-Native AI Engineering OS"
short_code: "SMET-S-0001"
created_at: 2026-03-17T19:51:13.185787+00:00
updated_at: 2026-03-17T19:51:13.185787+00:00
parent: SMET-V-0001
blocked_by: []
archived: false

tags:
  - "#strategy"
  - "#phase/shaping"


exit_criteria_met: false
risk_level: medium
stakeholders: []
strategy_id: ultra-metis-core-engine-repo
initiative_id: NULL
---

# Ultra-Metis Core Engine: Repo-Native AI Engineering OS Strategy

*This template includes sections for various types of strategic documents. Delete sections that don't apply to your specific use case.*

## Problem Statement

AI-assisted software engineering today relies on transient chat context, brittle one-off workflow scripts, and no durable memory of architecture decisions, quality history, or engineering rules. When an AI session ends, all reasoning is lost. There is no governed, repo-native layer that preserves product intent, architecture guidance, quality baselines, and reusable local insight across sessions and agents.

The original Metis system provides a solid document management foundation (Vision → Initiative → Task, MCP server, CLI, SQLite indexing) but lacks the deeper engineering OS capabilities needed to support governed, multi-agent, multi-mode AI software delivery. This strategy covers the design and implementation of all core engine capabilities on top of that foundation.

## Success Metrics

- All 15 MVP feature areas implemented and integrated into a working Rust codebase
- MCP server exposes the full durable operating system surface (CRUD, quality ops, rules, traceability, notes, baselines)
- CLI provides developer-native access to all operations
- Cognitive operation kernel and reusable loops serve as the universal reasoning substrate
- Architecture catalog contains practical patterns for JS/TS projects with greenfield selection flow
- Brownfield evaluation can analyze an existing repo and produce a reference architecture
- Quality gates block execution on degradation; baselines are tracked and compared
- Engineering rules are enforced with layered scopes and require explicit change proposals
- Durable insight notes fetch, score, prune, and archive across sessions
- Execution records provide a complete audit spine for every meaningful work run
- All original Metis capabilities preserved; migration path from Metis to Super-Metis exists
- All tests pass; benchmark suite shows parity or improvement over original Metis

## Solution Approach

Build on top of the original Metis foundation (file-based docs, SQLite indexing, MCP server, CLI) and extend it into a full repo-native AI engineering OS. All new capability is added as Rust crates in the `crates/` directory. The implementation follows the vision's 15 MVP feature areas:

1. **Cognitive operation kernel and reusable loops** — 12 operations composing into workflow loops
2. **Core planning hierarchy** — ProductDoc → Epic → Story → Task with typed Stories
3. **Durable state model** — repo-native persistence for all artifact classes
4. **Architecture catalog and selection** — curated patterns with greenfield selection flow
5. **Brownfield architecture evaluation** — analyze existing repos, match or derive architecture reference
6. **Architecture-driven rules and analysis** — seeded from catalog, layered scopes, protected change flow
7. **Internal cognition vs durable persistence** — promotion rules for when reasoning becomes durable state
8. **Durable insight note system** — lightweight self-pruning repo memory with feedback scoring
9. **Execution records and audit spine** — every work run emits a durable traceability record
10. **Gates, escalation, and autonomy model** — 7 gates, explicit triggers, 3 autonomy modes
11. **Baseline capture/storage/comparison** — ingest external tool outputs, track quality over time
12. **Governance artifact types** — RulesConfig, ApprovalRecord, ValidationPolicy, OwnershipMap
13. **Quality artifact types** — AnalysisBaseline, QualityRecord, ValidationRecord, RemediationRecord
14. **Workflow states and traceability backbone** — TransitionRecord, DecisionRecord, CrossReferenceIndex
15. **MCP and CLI surface** — full exposure of all engine capabilities

## Scope

**In Scope:**
- All Rust crate development in `crates/ultra-metis-core/`, `ultra-metis-store/`, `ultra-metis-mcp/`, `ultra-metis-cli/`
- All 15 MVP feature areas as described in SMET-V-0001
- Protected engineering rules with layered scopes and change proposal workflow
- Remediation and investigation loops for quality degradation
- Workflow state enforcement and transition traceability
- Repo-aware setup and bootstrap flows
- Architecture catalog, brownfield evaluation, and reference architecture persistence
- Cognitive operation kernel and reusable loop composition
- Durable insight note system with fetch, score, prune, archive
- Execution records linking intent, context, tools, validations, decisions
- Quality baselines, gates, and comparison tooling
- Template registry and context-aware rendering
- Migration path from original Metis document formats
- Namespace standardization (ultra-metis-*)
- Monorepo directory restructure (apps/, crates/, packages/, infra/, docs/)
- Local installation and end-to-end working setup
- MCP and CLI extensions for all new capabilities
- Error handling, benchmark testing, and quality verification

**Out of Scope:**
- Control Dashboard (web app) — see SMET-S-0002
- Control Service (API/backend) — see SMET-S-0002
- Machine Runner (local daemon) — see SMET-S-0002
- GUI productization beyond prototype — post-MVP
- Work leasing and git worktree isolation — post-MVP
- Multi-agent orchestrator — post-MVP
- Monorepo-root cross-package orchestration — post-MVP

## Risks & Unknowns

- **Scope creep**: The vision is large; MVP feature boundaries must stay disciplined
- **Metis migration**: Preserving backward compatibility while extending document models adds complexity
- **Rust crate boundaries**: Getting the right abstraction boundaries between core, store, mcp, cli as complexity grows
- **Performance**: SQLite indexing must scale as document counts and cross-reference queries grow
- **Template quality**: Generated planning artifacts must be good enough to be useful, not just syntactically correct

## Implementation Dependencies

The monorepo restructure (SMET-I-0038) must complete before any new component work that assumes the `apps/`, `crates/`, `packages/` layout. All other core engine initiatives are largely independent within their tracks:

- **Domain track**: SMET-I-0018, -0019, -0020, -0027, -0028, -0029, -0030, -0031, -0032
- **Quality track**: SMET-I-0021, -0022, -0006
- **Governance track**: SMET-I-0004, -0007
- **Integration track**: SMET-I-0009, -0010, -0011, -0034
- **Foundation track**: SMET-I-0008, -0014, -0015, -0033, -0035, -0036, -0037, -0038
- **Post-MVP track**: SMET-I-0017, -0023, -0024, -0025, -0026

## Change Log

### 2026-03-17 — Initial Strategy
- **Change**: Created strategy document to capture all existing core engine work under SMET-V-0001
- **Rationale**: Switching from streamlined to full preset requires grouping initiatives under strategies. All existing initiatives belong to this core engine strategy.
- **Impact**: All existing initiatives (SMET-I-0004 through SMET-I-0038) are associated with this strategy.
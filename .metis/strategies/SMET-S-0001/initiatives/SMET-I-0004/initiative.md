---
id: add-protected-engineering-rules
level: initiative
title: "Add Protected Engineering Rules and Controlled Rule-Change Workflow"
short_code: "SMET-I-0004"
created_at: 2026-03-11T19:59:27.611669+00:00
updated_at: 2026-03-17T01:52:54.838496+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"
  - "#feature-quality"
  - "#category-quality-governance"


exit_criteria_met: false
estimated_complexity: L
strategy_id: cadre-core-engine-repo
initiative_id: add-protected-engineering-rules
---

# Add Protected Engineering Rules and Controlled Rule-Change Workflow

## Context

AI agents and developers working in a monorepo need guardrails — coding standards, architectural constraints, dependency rules, testing requirements. Today these are typically expressed as linter configs, CLAUDE.md files, or informal conventions that can be casually overridden or ignored.

CADRE should introduce a Rules Config document type that stores protected engineering rules as durable, versioned artifacts. Rules should not be modifiable through casual edits — changes should require explicit proposals and approval, similar to how ADRs work but specifically for engineering governance.

## Governing Commitments

This initiative directly serves:
- **Protected governance** (Vision #4, #10). Rules and architecture-related controls are managed through explicit, durable change processes. Casual override is structurally prevented, not just discouraged.
- **Architecture-driven rules** (Vision #4). Engineering rules are seeded from the chosen Reference Architecture. This keeps enforcement aligned to the actual repo model rather than generic best-practice advice.
- **All durable project memory lives in the repo.** Rules are versioned, persistent artifacts — not transient linter configs or chat-context agreements. Rule history is preserved as part of the repo's governance memory.
- **Reference architecture drives rules, structure, and analysis** (Vision #4). The Reference Architecture is a control artifact. Rules seeded from it carry a traceable link back to their source pattern, so architecture changes can propagate to affected rules.
- **The system is built around intentional, durable structure.** Protection enforcement makes it structurally difficult to bypass governance — the system makes it hard to do the wrong thing.

## Goals & Non-Goals

**Goals:**
- Introduce a Rules Config document type for storing engineering rules
- Implement protection semantics that prevent casual modification of active rules
- Create a rule change proposal workflow: propose → review → approve → apply
- Support layered rule scopes: platform → organization → repo → package/subsystem → component → task/execution mode. Lower scopes may narrow but not silently violate higher protected scopes unless an approved override exists.
- Support rule categories typed by purpose: behavioral, architectural, operational, information-handling, decision-making, validation/quality, approval/escalation, execution-safety
- Support architecture-derived rule seeding: when a Reference Architecture is selected, automatically generate a starter set of rules from the architecture pattern's seed data (folder structure rules, dependency direction rules, naming conventions, module boundary rules, testing placement rules)
- Make rules queryable so agents and tools can check them programmatically
- For MVP, runtime rule enforcement may be partially delegated to existing enforcement plugins and hooks. The durable value of CADRE is the persisted, governed rule model and traceability, not necessarily the first implementation of every enforcement mechanism.

**Non-Goals:**
- Implementing lint rule enforcement at the code level (that's the linter's job)
- Auto-fixing code to match rules (that's tooling beyond CADRE scope)
- Building a full policy engine — this is structured documentation with protection, not runtime enforcement

## Detailed Design

### What to Reuse from `metis/`
- ADR document type and workflow as inspiration for the change proposal pattern
- Document storage and versioning infrastructure
- Frontmatter-based metadata for rule categorization
- Search and indexing for rule discovery

### What to Change from `metis/`
- Extend the concept of "protected" documents beyond just published visions
- Add file-level protection markers that prevent direct edits without going through the proposal workflow
- Update the edit_document flow to check protection status

### What is Net New
- Rules Config document type with categories, severity levels, and applicability scopes
- Protection enforcement — attempts to edit active rules must go through a Rule Change Proposal
- Rule Change Proposal document type with phases: draft → review → approved → applied → superseded
- Rule versioning — when a rule changes, the old version is preserved as history
- Programmatic rule query API (for agents to check rules before taking actions)
- Rule inheritance — repo-level rules that apply everywhere, plus directory-scoped overrides
- Architecture-derived rule seeding: generate initial Rules Config documents from a Reference Architecture's seed data, with proper categorization and scoping
- Architecture reference linkage: rules seeded from an architecture pattern should reference their source, so changes to the architecture can be traced to affected rules

## Alternatives Considered

1. **Use CLAUDE.md for rules**: Rejected because CLAUDE.md has no protection, versioning, or change workflow. Rules in CLAUDE.md can be silently overridden.
2. **Use ADRs for rules**: Rejected because ADRs capture decisions, not active enforcement rules. Different lifecycle and semantics.
3. **External policy engine**: Rejected because it violates the repo-native principle. Rules should live in the repo alongside the code they govern.

## Implementation Plan

Phase 1: Define Rules Config document schema with categories and protection levels
Phase 2: Implement Rules Config domain type (coordinate with SMET-I-0001)
Phase 3: Implement protection enforcement on edit operations
Phase 4: Implement Rule Change Proposal type and workflow
Phase 5: Implement rule versioning and history
Phase 6: Add programmatic rule query API
Phase 7: Add CLI/MCP commands for rule management

## Acceptance Criteria

- Rules Config documents can be created with category, severity, and scope
- Active rules cannot be directly edited without a Rule Change Proposal
- Rule Change Proposals follow the defined approval workflow
- Rule history is preserved when changes are applied
- Rules are queryable by category, scope, and severity
- Agents can programmatically check applicable rules before taking actions
- Rule inheritance works correctly (repo-level + directory-scoped)

## Risks / Dependencies

- Depends on SMET-I-0001 for domain types
- Protection enforcement must be robust — a loophole undermines the entire concept
- Risk of making rules too rigid — need an escape hatch for emergencies (force override with audit trail)
- Must coordinate with SMET-I-0009 (MCP tools) for programmatic rule access
- Must coordinate with SMET-I-0016 (architecture catalog) for rule seed data format and seeding workflow

## Codebase Areas to Inspect

- `metis/src/domain/` — ADR type as pattern for rule change proposals
- `metis/src/commands/edit.rs` or equivalent — where edit protection would be enforced
- `metis/src/storage/` — document versioning patterns
- `metis/src/mcp/` — tool patterns for programmatic access

## Suggested Tasks for Decomposition

1. Define Rules Config document schema and frontmatter spec
2. Define Rule Change Proposal schema and phase flow
3. Implement Rules Config domain type with protection semantics
4. Implement edit-time protection enforcement
5. Implement Rule Change Proposal domain type and workflow
6. Implement rule versioning and history tracking
7. Add programmatic rule query API
8. Implement rule inheritance (repo-level + directory-scoped)
9. Add force-override with audit trail for emergencies
10. Integration test rule protection and change workflow
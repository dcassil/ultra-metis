---
id: add-first-class-design-aware
level: initiative
title: "Add First-Class Design-Aware Planning and Reference Handling"
short_code: "SMET-I-0003"
created_at: 2026-03-11T19:59:23.687658+00:00
updated_at: 2026-03-11T19:59:23.687658+00:00
parent: SMET-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: add-first-class-design-aware
---

# Add First-Class Design-Aware Planning and Reference Handling

## Context

In real software development, design decisions — UI mockups, component patterns, approved visual standards — directly influence implementation. Current Metis has no concept of design artifacts or design references. Developers and AI agents working from Metis plans have no structured way to reference approved designs or ensure implementation matches design intent.

Super-Metis should make design a first-class concern: design references should be linkable to epics and stories, design context documents should be durable planning artifacts, and design change proposals should go through a structured workflow.

## Governing Commitments

This initiative directly serves:
- **All durable project memory lives in the repo.** Design context, approved patterns, and visual standards are persisted as repo-native artifacts — not scattered across external tools without a local anchor.
- **Planning is durable and traceable from product intent to execution.** Design references on Epics and Stories create traceable links between what was designed and what is being built.
- **Evidence-based workflow progression** (Vision #7). Design Change Proposals follow a structured approval workflow — design decisions are recorded artifacts, not casual agreements.
- **The system is built around intentional, durable structure.** Design-aware planning means implementers and agents work from explicit references, not improvised interpretations.

## Goals & Non-Goals

**Goals:**
- Introduce a Design Context document type that stores references to design specs, mockups, component patterns, and visual standards
- Allow design references to be linked from Epics and Stories so implementers know what they're building toward
- Introduce a Design Change Proposal workflow for modifying approved designs
- Support references to external design tools (Figma, Storybook, etc.) as well as repo-local design artifacts

**Non-Goals:**
- Building a design tool — Super-Metis references designs, it doesn't create them
- Rendering design mockups in the GUI (though links should be clickable)
- Enforcing pixel-perfect design compliance at the code level

## Detailed Design

### What to Reuse from `metis/`
- Document creation and storage infrastructure
- Markdown + frontmatter format for Design Context documents
- Tag and cross-reference patterns
- Template system for generating Design Context documents

### What to Change from `metis/`
- Add frontmatter fields for design references on Epic and Story documents
- Extend cross-reference validation to include design links
- Update search/indexing to include design artifacts

### What is Net New
- Design Context document type with: design system references, component inventory, approved patterns, visual standards, external tool links
- Design Change Proposal document type with phases: draft → review → approved → implemented
- Design reference fields on Epic and Story frontmatter (links to Design Context docs and external URLs)
- Validation that stories with design references have corresponding Design Context documents
- CLI/MCP commands for creating and managing design artifacts

## Alternatives Considered

1. **Store design info as tags on existing documents**: Rejected because design context is rich enough to warrant its own document type with proper structure.
2. **External design system integration**: Deferred — start with references and links, add deeper integration later if needed.
3. **Design as a section within Epic documents**: Rejected because design context often applies across multiple epics and should be independently referenceable.

## Implementation Plan

Phase 1: Define Design Context and Design Change Proposal document schemas
Phase 2: Implement domain types (coordinate with SMET-I-0001)
Phase 3: Add design reference fields to Epic and Story frontmatter
Phase 4: Implement cross-reference validation for design links
Phase 5: Create templates for design documents
Phase 6: Add CLI/MCP support for design artifact management

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- Design Context documents can be created, stored, and searched
- Epics and Stories can reference Design Context documents
- Design Change Proposals follow a structured approval workflow
- Design references are validated (referenced documents must exist)
- Templates produce useful starting points for design documentation
- External design tool URLs are stored and accessible

## Risks / Dependencies

- Depends on SMET-I-0001 (domain model) for type definitions
- Depends on SMET-I-0002 (hierarchy) for phase flow definitions
- Risk that teams don't adopt design documentation — keep it lightweight
- Must coordinate with SMET-I-0014 (templates) for design document templates

## Codebase Areas to Inspect

- `metis/src/domain/` — where to add new document types
- `metis/src/templates/` — template system for new document types
- `metis/src/storage/` — document storage patterns to extend
- Any existing cross-reference or linking code

## Suggested Tasks for Decomposition

1. Define Design Context document schema and frontmatter spec
2. Define Design Change Proposal schema and phase flow
3. Implement Design Context domain type
4. Implement Design Change Proposal domain type
5. Add design reference fields to Epic and Story types
6. Implement cross-reference validation for design links
7. Create Design Context and Design Change Proposal templates
8. Add search/indexing support for design artifacts
---
id: improve-templates-and-generated
level: initiative
title: "Improve Templates and Generated Planning Artifacts"
short_code: "SMET-I-0014"
created_at: 2026-03-11T20:00:11.420530+00:00
updated_at: 2026-03-17T02:14:07.801216+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: ultra-metis-core-engine-repo
initiative_id: improve-templates-and-generated
---

# Improve Templates and Generated Planning Artifacts

## Context

Metis uses templates to generate initial document content when new documents are created. The current templates are generic and include many conditional sections that users must manually clean up. Super-Metis introduces many new document types, each needing purpose-built templates that guide users (and agents) toward producing high-quality planning artifacts.

Good templates are critical for AI agents — they need clear structure and guidance to produce useful documents, not generic placeholders.

## Governing Commitments

This initiative directly serves:
- **The system is built around intentional, durable structure.** Templates enforce consistent document structure across every artifact type. They make it hard to produce poorly-formed documents — structure is provided, not improvised.
- **All durable project memory lives in the repo.** Templates ensure that every created artifact starts with the right sections and guidance, so the resulting repo-native documents are complete and useful from creation.
- **Every repo gets a persisted reference architecture.** Architecture Catalog Entry and Reference Architecture templates define the structure that makes architecture artifacts consistent, complete, and machine-queryable.
- **Structural guidance over improvisation** (Principle #3). Templates guide both humans and agents toward producing high-quality artifacts. Clear section guidance replaces blank-page improvisation.

## Goals & Non-Goals

**Goals:**
- Create purpose-built artifact templates for all new Super-Metis document types: ProductDoc, DesignContext, Epic, Story (with type variants), Task, RulesConfig, AnalysisBaseline, QualityRecord, ValidationRecord, RemediationRecord, DesignChangeProposal, ArchitectureInvestigation, RuleChangeProposal, ArchitectureCatalogEntry, ReferenceArchitecture, ApprovalRecord, ValidationPolicy, OwnershipMap, ConstraintRecord, ExecutionRecord, TransitionRecord, DecisionRecord, DurableInsightNote
- Create workflow templates for common work types: bugfix, feature slice, refactor, migration, architecture change, brownfield evaluation, remediation, investigation, greenfield bootstrap. Workflow templates define which cognitive operation loops to run, in what order, with what entry/exit conditions (coordinate with SMET-I-0029).
- Make templates context-aware: pre-populate fields based on parent documents, project context, and architecture scope
- Remove generic conditional sections — each template should be specific to its document type
- Include clear guidance comments that help agents and humans understand what each section needs
- Support template customization per-project

**Non-Goals:**
- AI-generated content within templates — templates provide structure, not pre-written content
- Template versioning (beyond what document schema versioning provides)
- Template marketplace or community templates

## Detailed Design

### What to Reuse from `metis/`
- The existing template engine and rendering system
- Frontmatter generation patterns
- Template file storage conventions

### What to Change from `metis/`
- Replace generic, conditional templates with specific, focused templates per type
- Add context-aware template rendering (access to parent document data, project config)
- Remove the `[CONDITIONAL]` / `[REQUIRED]` pattern in favor of type-specific sections

### What is Net New
- Templates for: Product Doc, Design Context, Epic, Story, Task, Rules Config, Analysis Baseline, Quality Record, Design Change Proposal, Architecture Investigation, Rule Change Proposal, Architecture Catalog Entry, Reference Architecture
- Context-aware rendering: templates can reference parent document fields
- Project-aware rendering: templates can reference project configuration (detected languages, tools, etc.)
- Template customization: projects can override default templates with custom versions
- Guidance comments: clear, actionable instructions within templates for each section

## Alternatives Considered

1. **Let agents generate documents from scratch without templates**: Rejected because structure enforcement is a core principle — templates ensure consistency.
2. **AI-generated template content**: Deferred — start with structured templates, add AI-assisted content generation later.
3. **YAML/JSON templates instead of markdown**: Rejected because markdown is more readable and writable for both humans and agents.

## Implementation Plan

Phase 1: Audit existing templates and identify what works
Phase 2: Design template structure for each new document type
Phase 3: Implement context-aware template rendering
Phase 4: Create all new document type templates
Phase 5: Implement project-level template customization
Phase 6: Test templates with both human and agent workflows

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- Every Super-Metis document type has a purpose-built template
- Templates include clear guidance for each section
- No generic conditional sections — each template is specific
- Context-aware rendering correctly pre-populates from parent documents
- Projects can override default templates with custom versions
- Templates produce documents that agents can reliably fill in
- Template rendering is fast (< 100ms)

## Risks / Dependencies

- Depends on SMET-I-0001 for knowing all document types and their fields
- Depends on SMET-I-0002 for knowing phase flows to document in templates
- Template quality directly affects agent output quality — templates need to be excellent
- Must coordinate with SMET-I-0008 for bootstrap-generated templates

## Codebase Areas to Inspect

- `metis/src/templates/` — existing template files and rendering engine
- `metis/src/commands/create.rs` or equivalent — how templates are used during creation
- Any Handlebars/Tera/template engine configuration

## Suggested Tasks for Decomposition

1. Audit existing Metis template system and rendering engine
2. Design template structure guidelines (what makes a good template)
3. Implement context-aware template rendering (parent doc access)
4. Create Product Doc template
5. Create Epic and Story templates
6. Create Task template (refined from existing)
7. Create Design Context and Design Change Proposal templates
8. Create Rules Config and Rule Change Proposal templates
9. Create Analysis Baseline and Quality Record templates
10. Create Architecture Investigation template
11. Create Architecture Catalog Entry template (with sections for use case, tradeoffs, folder structure, layers, dependency rules, module boundaries, naming conventions, testing layout, anti-patterns, rules seed data, analysis expectations)
12. Create Reference Architecture template (selected pattern reference, tailoring notes, repo-specific overrides)
13. Implement project-level template customization
14. Test all templates with agent workflows
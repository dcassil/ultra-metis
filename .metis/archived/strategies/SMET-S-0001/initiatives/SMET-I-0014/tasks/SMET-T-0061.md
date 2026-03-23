---
id: build-template-registry-with-type
level: task
title: "Build Template Registry with Type-Safe Access"
short_code: "SMET-T-0061"
created_at: 2026-03-17T02:01:18.857641+00:00
updated_at: 2026-03-17T02:06:01.387900+00:00
parent: SMET-I-0014
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: SMET-I-0014
---

# Build Template Registry with Type-Safe Access

## Objective

Create a TemplateRegistry module at `domain/templates/` that provides centralized, type-safe access to all document templates. Currently each document type uses `include_str!` directly. The registry provides a single entry point for looking up templates by DocumentType, supports template categories, and lays groundwork for project-level overrides.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] TemplateRegistry struct with `get(doc_type) -> &TemplateSet` method
- [ ] TemplateSet struct holding frontmatter, content, and acceptance_criteria templates
- [ ] TemplateCategory enum (Frontmatter, Content, AcceptanceCriteria)
- [ ] `render_content(doc_type, context) -> Result<String>` for rendering templates with Tera
- [ ] All 9 Document-trait types covered (ProductDoc, DesignContext, Epic, Story, Task, Adr, Specification, Vision, Initiative)
- [ ] Comprehensive unit tests for registry lookup and rendering
- [ ] Module wired into domain/mod.rs and re-exported from lib.rs

## Implementation Notes

### Technical Approach
1. Create `domain/templates/mod.rs` with TemplateRegistry struct
2. Define TemplateCategory enum and TemplateSet struct
3. Use `include_str!` to embed all template files at compile time
4. Implement lookup by DocumentType returning TemplateSet references
5. Add Tera-based render method that takes a context map

### Dependencies
- Existing document type templates in each module directory
- Tera crate (already in Cargo.toml)

## Status Updates

*To be added during implementation*
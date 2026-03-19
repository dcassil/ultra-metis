---
id: add-context-aware-template
level: task
title: "Add Context-Aware Template Rendering"
short_code: "SMET-T-0063"
created_at: 2026-03-17T02:01:21.127736+00:00
updated_at: 2026-03-17T02:12:20.551914+00:00
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

# Add Context-Aware Template Rendering

## Objective

Add context-aware template rendering so templates can access parent document data and project configuration during rendering. A Task template can reference its parent Epic's title, a Story can reference its Epic's scope.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] TemplateContext struct with parent_title, parent_short_code, parent_type, project_name fields
- [ ] `render_with_context(doc_type, template_context) -> Result<String>` method on TemplateRegistry
- [ ] Context variables available in templates with Tera `default()` fallbacks
- [ ] Tests verifying rendering with and without parent context
- [ ] cargo test passes

## Implementation Notes

### Technical Approach
1. Define TemplateContext struct in domain/templates/mod.rs
2. Add render_with_context that populates Tera context with parent info
3. Templates use `{{ parent_title | default(value="") }}` syntax for optional context
4. Tests verify both paths: with context provided and without

## Status Updates

*To be added during implementation*
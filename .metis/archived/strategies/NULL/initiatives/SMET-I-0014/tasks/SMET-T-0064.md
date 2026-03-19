---
id: add-project-level-template
level: task
title: "Add Project-Level Template Customization Support"
short_code: "SMET-T-0064"
created_at: 2026-03-17T02:01:22.127053+00:00
updated_at: 2026-03-17T02:13:56.768827+00:00
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

# Add Project-Level Template Customization Support

## Objective

Add support for project-level template customization. Projects can override default templates by placing custom templates in a `.metis/templates/` directory. The TemplateRegistry checks for project-level overrides before falling back to built-in templates.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `TemplateRegistry::with_custom_dir(path)` constructor
- [ ] Directory layout: `.metis/templates/{doc_type}/content.md`
- [ ] Fallback logic: custom dir first, built-in second
- [ ] CustomTemplateLoader that reads from filesystem
- [ ] Validation of custom templates for required Tera variables
- [ ] Tests with tempdir-based custom templates
- [ ] cargo test passes

## Implementation Notes

### Technical Approach
1. Add optional `custom_templates_dir` field to TemplateRegistry
2. Implement CustomTemplateLoader that scans the directory
3. get() method checks custom dir first, falls back to built-in
4. Validate custom templates can parse as Tera templates
5. Tests create temp directories with custom templates and verify override behavior

## Status Updates

*To be added during implementation*
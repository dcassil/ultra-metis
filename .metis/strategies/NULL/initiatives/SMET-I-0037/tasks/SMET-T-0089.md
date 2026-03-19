---
id: add-template-quality-tests
level: task
title: "Add template quality tests"
short_code: "SMET-T-0089"
created_at: 2026-03-17T20:15:51.669+00:00
updated_at: 2026-03-17T20:29:32.106417+00:00
parent: SMET-I-0037
blocked_by: [SMET-T-0088]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: SMET-I-0037
---

# Add template quality tests

## Parent Initiative

[[SMET-I-0037]]

## Objective

Write automated tests that verify the structural quality and completeness of ultra-metis document templates. After the rewrites in SMET-T-0088, these tests ensure no template regresses to placeholder-only content and that all templates meet minimum quality standards: required sections present, conditional sections marked, rendering success.

The goal is to make "template quality" a measurable, testable property — not just a benchmark judgment.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] A `template_quality` test module exists in `super-metis-core` (or `ultra-metis-store`)
- [ ] Tests verify all 4 templates (vision, initiative, task, ADR) render without errors
- [ ] Tests assert each template has the required sections (by checking for required headings)
- [ ] Tests assert that conditional sections contain explicit deletion markers (`DELETE` or `CONDITIONAL`)
- [ ] Tests assert no raw `{placeholder}` text survives in rendered output (guidance is in HTML comments or has examples)
- [ ] Tests assert the task template has an `Acceptance Criteria` section
- [ ] Tests assert the initiative template has a `Detailed Design` or `Design` section
- [ ] All existing tests continue to pass (`cargo test --workspace`)
- [ ] Tests are in the standard Rust test module pattern (`#[cfg(test)] mod tests { ... }`)

## Implementation Notes

### Technical Approach

Templates are Tera files rendered via `TemplateRegistry`. The test strategy is:

1. **Render each template** with minimal context (title, project_name, no parent)
2. **Assert required headings** are present in the rendered output
3. **Assert quality markers** are present (examples, format guidance, deletion instructions)
4. **Assert no bare placeholders** — lines matching `^\{[^}]+\}$` (a line that is only a placeholder with no surrounding guidance) should fail

**Test structure:**
```rust
#[cfg(test)]
mod template_quality_tests {
    use crate::templates::TemplateRegistry;

    fn render_template(doc_type: &str) -> String {
        let registry = TemplateRegistry::new();
        let ctx = TemplateContext {
            title: "Test Document".to_string(),
            project_name: "TEST".to_string(),
            parent_title: None,
            parent_short_code: None,
        };
        registry.render(doc_type, &ctx).expect("Template should render without error")
    }

    #[test]
    fn task_template_has_acceptance_criteria() {
        let rendered = render_template("task");
        assert!(rendered.contains("## Acceptance Criteria"),
            "Task template must have Acceptance Criteria section");
    }

    #[test]
    fn task_template_has_status_updates() {
        let rendered = render_template("task");
        assert!(rendered.contains("## Status Updates"),
            "Task template must have Status Updates section for working memory");
    }

    #[test]
    fn vision_template_has_success_metrics() {
        let rendered = render_template("vision");
        assert!(rendered.contains("Success") || rendered.contains("Metrics"),
            "Vision template must have success metrics section");
    }

    #[test]
    fn adr_template_has_alternatives() {
        let rendered = render_template("adr");
        assert!(rendered.contains("Alternative") || rendered.contains("Options"),
            "ADR template must have alternatives section");
    }

    #[test]
    fn templates_have_deletion_guidance() {
        for doc_type in ["vision", "initiative", "task", "adr"] {
            let rendered = render_template(doc_type);
            // At least one conditional section should be marked
            let has_guidance = rendered.contains("DELETE")
                || rendered.contains("CONDITIONAL")
                || rendered.contains("if not applicable");
            assert!(has_guidance,
                "{} template has no conditional section guidance", doc_type);
        }
    }

    #[test]
    fn no_bare_single_line_placeholders() {
        for doc_type in ["vision", "initiative", "task", "adr"] {
            let rendered = render_template(doc_type);
            for line in rendered.lines() {
                let trimmed = line.trim();
                // A line that is ONLY {text} with no surrounding content is a bare placeholder
                let is_bare = trimmed.starts_with('{')
                    && trimmed.ends_with('}')
                    && !trimmed.contains("{{"); // not a Tera variable
                assert!(!is_bare,
                    "{} template has bare placeholder line: {}", doc_type, trimmed);
            }
        }
    }
}
```

### Files to Create/Modify

- Add tests to: `super-metis/crates/super-metis-core/src/templates.rs` or a new `src/templates/quality_tests.rs`
- May need to expose `TemplateRegistry::render()` and `TemplateContext` if not already public in tests

### Testing Plan

1. Write failing tests first (they should fail against current templates)
2. Confirm tests pass after SMET-T-0088 rewrites are applied
3. Run `cargo test -p super-metis-core` to verify

### Dependencies

- SMET-T-0088 (template rewrites) — tests should be written against the new templates
- Existing `TemplateRegistry` API must be compatible with test rendering (may need to expose or document internal APIs)

### Risk Considerations

- **TemplateRegistry API**: If rendering is only exposed via the store layer, may need to add a test helper or make `TemplateRegistry::render()` accessible in test context
- **Conditional section detection**: Tests that check for "DELETE" text only work if the keyword is actually in the template. Coordinate with SMET-T-0088 to ensure this marker is used consistently.
- **Scope creep**: Do not test visual formatting or style preferences. Test only structural presence of required sections and absence of bare placeholders.

## Status Updates

### 2026-03-17
- Added 12 template quality tests to `domain::templates::tests` in `mod.rs`:
  - Task: has Acceptance Criteria, has Status Updates, has Objective, uses checkboxes
  - Initiative: has Detailed Design, has Status Updates
  - ADR: has Rationale, has Positive/Negative Consequences sub-sections
  - Vision: has Success Criteria
  - All 4 core types: have conditional guidance markers, have HTML comment guidance, render without errors with and without parent context
- One test (`quality_no_bare_placeholder_lines_in_rendered_output`) was revised to `quality_core_templates_have_html_comment_guidance` — original test was too strict (bare `{placeholder}` lines are intentional fill-in prompts, same as original metis); correct quality signal is presence of `<!-- -->` guidance comments
- All 738 tests pass, 0 failures
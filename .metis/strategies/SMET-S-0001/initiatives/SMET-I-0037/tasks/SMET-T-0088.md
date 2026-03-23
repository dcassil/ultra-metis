---
id: rewrite-cadre-templates-to
level: task
title: "Rewrite cadre templates to match or exceed original metis quality"
short_code: "SMET-T-0088"
created_at: 2026-03-17T20:15:51.669+00:00
updated_at: 2026-03-17T20:26:47.979286+00:00
parent: SMET-I-0037
blocked_by: [SMET-T-0087]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: SMET-I-0037
---

# Rewrite cadre templates to match or exceed original metis quality

## Parent Initiative

[[SMET-I-0037]]

## Objective

Rewrite all 4 cadre document templates (vision, initiative, task, ADR) based on the gap analysis from SMET-T-0087. The rewrites should incorporate the patterns from the original metis plugin that make templates effective: conditional sections, explicit deletion markers, inline examples, format guidance, and required-vs-optional markers.

The benchmark scored cadre template quality at 3/5 vs metis at 5/5. After this rewrite, re-running the benchmark should score 5/5.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Vision template rewritten with:
  - Format guidance for each section (table/checklist/prose suggestions)
  - Inline examples for key sections (e.g., "Example: 'Reach 100k monthly active users by Q4 2026'")
  - Conditional sections marked with `<!-- DELETE if not applicable: -->` wrapper
  - Success metrics section with a table template (Metric | Current | Target | Timeline)
- [ ] Initiative template rewritten with:
  - "Detailed Design" section name corrected and accompanied by structure guidance
  - Complexity estimate guidance (xs/s/m/l/xl with definitions)
  - Acceptance criteria section with SMART format prompts
  - Risks/dependencies as conditional sections with checkbox tables
  - Link/reference to parent Vision
- [ ] Task template rewritten with:
  - Full 6-section structure: Objective, Acceptance Criteria, Implementation Notes, Dependencies, Risk Considerations, Status Updates
  - Acceptance criteria items as checkboxes with format guide: `- [ ] {Specific, testable criterion}`
  - Implementation Notes with sub-sections: Technical Approach, Files to Modify, Testing Plan
  - Status Updates section with date-stamped update template
- [ ] ADR template rewritten with:
  - Decision statement section (distinct from solution description)
  - Alternatives table with columns: Option | Pros | Cons | Why Rejected
  - Consequences section (positive and negative outcomes)
  - Status marker: Proposed / Accepted / Deprecated / Superseded
- [ ] All 4 template files pass `cargo test` (rendering pipeline tests)
- [ ] Templates render without errors for all document types (with and without parent context)
- [ ] The Tera template syntax (`{{ variable }}`, `{% if %}`) is preserved and correct

## Implementation Notes

### Technical Approach

Templates live in:
```
cadre/crates/cadre-core/src/domain/documents/
├── vision/content.md
├── initiative/content.md
├── task/content.md
└── adr/content.md
```

Each `content.md` is a Tera template rendered by `TemplateRegistry` in `cadre-core`. The rendering context includes:
- `title` — document title
- `project_name` — project name from config
- `parent_title` / `parent_short_code` — if parent document exists
- Document-type-specific fields

**Patterns to apply from original metis:**

1. **Required vs Conditional markers:**
```markdown
<!-- REQUIRED: This section must be filled out -->
## Objective
{What this task accomplishes and why it matters}

<!-- CONDITIONAL: Delete this section if there are no external dependencies -->
## External Dependencies
{List external teams, services, or APIs this task depends on}
```

2. **Format examples inline:**
```markdown
## Acceptance Criteria
<!-- Each criterion should be specific and testable. Bad: "Works correctly". Good: "Returns HTTP 200 with body {id: string} for valid input" -->
- [ ] {Specific, testable criterion — what must be true when this task is complete}
- [ ] {Another criterion}
```

3. **Section guidance as HTML comments (stripped at render time if configured, or left as guides):**
```markdown
## Status Updates
<!-- Add dated entries as you work. Example:
### 2026-03-17
- Investigated X, found Y
- Decided to use approach Z because W
- Next: implement Z in file foo.rs
-->
_No updates yet._
```

4. **Table scaffolding for structured sections:**
```markdown
## Risks
| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| {Risk description} | Low/Med/High | Low/Med/High | {How to mitigate} |
```

### Files to Modify

- `cadre/crates/cadre-core/src/domain/documents/vision/content.md`
- `cadre/crates/cadre-core/src/domain/documents/initiative/content.md`
- `cadre/crates/cadre-core/src/domain/documents/task/content.md`
- `cadre/crates/cadre-core/src/domain/documents/adr/content.md`

### Testing Plan

After each template rewrite:
1. Run `cargo test -p cadre-core` to verify no rendering regressions
2. Initialize a test project and create each document type to see the rendered template
3. Verify conditional sections are clearly marked for deletion

### Dependencies

- SMET-T-0087 (gap analysis) — must be completed first to have concrete targets
- Existing Tera rendering pipeline (no changes needed, templates must be compatible)

### Risk Considerations

- **Tera syntax errors**: Malformed `{% %}` blocks will cause runtime panics. Test each template after editing.
- **Missing context variables**: Using `{{ parent_title }}` when no parent exists will error if not guarded with `{% if parent_title %}`. Check existing Tera conditional patterns in the codebase first.
- **Over-engineering**: Templates should guide, not dictate. Avoid making templates so opinionated that they're hard to deviate from for legitimate cases.

## Status Updates

### 2026-03-17
- Read gap analysis from SMET-T-0087 for concrete targets
- Rewrote all 4 templates based on original metis domain/documents patterns:
  - **vision/content.md**: Added [REQUIRED]/[CONDITIONAL] markers, opening disclaimer, conditional sections (Product/Solution Overview, Major Features, Business Requirements), success metrics table scaffold, inline examples in HTML comments
  - **initiative/content.md**: Added [REQUIRED]/[CONDITIONAL] markers, opening disclaimer, parent link via Tera conditional, conditional sections (Requirements, Use Cases, Architecture, Testing Strategy), fixed Detailed Design placeholder, added Status Updates
  - **task/content.md**: Full rewrite — 3 sections → 7+ sections: Parent Initiative (conditional Tera), Objective, Backlog Item Details (conditional with type/priority checklists), Acceptance Criteria with checkbox format, Implementation Notes with sub-sections (Technical Approach, Files to Modify, Dependencies, Risk table), Status Updates with example
  - **adr/content.md**: Added [REQUIRED] markers, split Consequences into Positive/Negative/Neutral, added Rationale section, added Alternatives Analysis table (conditional), added Review Schedule (conditional), updated Status with options
- Ran `cargo test -p cadre-core`: all tests pass, no regressions
- All acceptance criteria met
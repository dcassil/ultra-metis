---
id: investigate-and-improve-template
level: initiative
title: "Investigate and Improve Template Quality to Match or Exceed Original Metis"
short_code: "SMET-I-0037"
created_at: 2026-03-17T18:45:21.130136+00:00
updated_at: 2026-03-17T20:44:57.546273+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: investigate-and-improve-template
---

# Investigate and Improve Template Quality to Match or Exceed Original Metis Initiative

## Context

The SMET-I-0035 benchmark scored cadre template quality at 3/5 vs the original metis at 5/5. This is a significant gap — templates are what users and AI agents see when creating documents, and they directly influence output quality. Better templates mean better documents with less effort.

The original metis plugin has evolved its templates through real-world usage feedback. Cadre (SMET-I-0014) already improved templates once, adding structured tables, HTML comments as instructions, and context-aware rendering. But the benchmark shows we're still behind.

### Specific Observations from Benchmarks
- **Metis templates**: Rich with conditional sections, guided prompts, scoring rubrics, specific examples, and clear "delete if not applicable" markers. The AI agent consistently fills them out well.
- **Cadre templates**: Have structure but less guidance. Some sections are generic. The AI agent sometimes produces thinner content because the template doesn't prompt deeply enough.

### What Makes a Great Template
A great template should:
1. **Guide the writer** — tell them what goes in each section and WHY it matters
2. **Be opinionated** — suggest specific formats (tables, checklists) rather than freeform
3. **Include examples** — show what "good" looks like for each section
4. **Eliminate blanks** — every section should have enough scaffolding that the writer fills in rather than invents structure
5. **Be context-appropriate** — different document types need different templates, not one-size-fits-all
6. **Minimize waste** — conditional sections clearly marked for deletion when not applicable

## Goals & Non-Goals

**Goals:**
- Read and analyze every template in the original metis plugin to understand what makes them effective
- Read and analyze every template in cadre to identify specific gaps
- Produce a side-by-side comparison of templates for each document type
- Identify specific patterns that make metis templates score higher (prompts, examples, structure)
- Rewrite cadre templates to incorporate those patterns and exceed metis quality
- Add template quality tests that verify structure, completeness, and guidance density
- Re-run benchmark Scenario 2 (Planning Workflow) and achieve 5/5 template quality

**Non-Goals:**
- Changing the template rendering engine (Tera is fine)
- Adding dynamic template selection (that's a separate feature)
- Templates for document types that metis doesn't have (cadre-only types are already fine)

## Detailed Design

### Phase 1: Analyze Original Metis Templates
- Read all template files from the metis plugin source at `~/.claude/plugins/cache/colliery-io-metis/`
- For each document type (vision, initiative, task, ADR), catalog:
  - Section structure and headings
  - Guidance text quality (specificity, examples, formatting suggestions)
  - Conditional section handling
  - How the template influences AI agent output quality
- Score each on the same 1-5 rubric used in benchmarks

### Phase 2: Analyze Cadre Templates
- Read all `content.md` templates in `cadre/crates/cadre-core/src/domain/documents/*/`
- Same analysis: structure, guidance quality, conditional handling
- Side-by-side comparison with metis for matching document types

### Phase 3: Gap Analysis
- For each document type, list specific gaps:
  - Missing guidance in specific sections
  - Missing examples or format suggestions
  - Weaker conditional section markers
  - Less opinionated structure
- Rank gaps by impact on output quality

### Phase 4: Rewrite Templates
- Rewrite each cadre template to close the gaps
- Key improvements to make:
  - Add inline examples (e.g., "Example: 'Reduce API latency by 50% for P99 requests'")
  - Add format suggestions (e.g., "Use a table with columns: Metric | Current | Target | Priority")
  - Strengthen conditional section markers with clear "DELETE THIS SECTION if..." instructions
  - Add scoring rubrics where appropriate (e.g., acceptance criteria should be SMART)
  - Add "Why this section matters" context for each heading
- Ensure templates work with the existing Tera rendering pipeline and `TemplateRegistry`

### Phase 5: Quality Verification
- Create template quality tests that assert:
  - Every section has guidance text (no empty `{placeholder}` sections)
  - Conditional sections have clear deletion markers
  - Templates render without errors for all document types
  - Templates with context (parent info, project name) render correctly
- Run the full test suite to verify no regressions

### Phase 6: Re-benchmark
- Re-run benchmark Scenario 2 (Planning Workflow) with the new templates
- Score template quality — target 5/5
- Compare document completeness between old and new templates

## Gap Analysis

The original metis uses `domain/documents/` templates (not `templates/`). These are the richer set that the benchmark tested against. Cadre uses equivalent `domain/documents/` templates that were significantly stripped down during the cadre port.

### Vision Template — Impact: HIGH

**Current state (cadre):**
- 6 bare sections: Purpose, Current State, Future State, Success Criteria, Principles, Constraints
- All sections use single-line `{placeholder}` with no format guidance or examples
- No section markers (REQUIRED/CONDITIONAL)
- No conditional sections at all — no product overview, no features list, no business requirements

**Original metis patterns:**
- `[REQUIRED]` / `[CONDITIONAL]` markers on every section heading
- Opening disclaimer: *"Delete sections that don't apply to your specific use case."*
- Conditional sections with clear deletion instructions: Product/Solution Overview, Major Features, Business Requirements Overview
- Structured bullet lists in conditional sections showing expected content shape

**Specific improvements needed:**
1. Add `[REQUIRED]` to Purpose, Current State, Future State, Success Criteria, Principles, Constraints
2. Add opening disclaimer line
3. Add `## Product/Solution Overview [CONDITIONAL: Product/Solution Vision]` with deletion instruction
4. Add `## Major Features [CONDITIONAL: Product Vision]` with bullet list scaffolding: `- {Feature 1: Description and value}`
5. Add `## Business Requirements Overview [CONDITIONAL: Business Vision]` with bullet list

---

### Initiative Template — Impact: HIGH

**Current state (cadre):**
- 5 sections: Context, Goals & Non-Goals, Detailed Design, Alternatives Considered, Implementation Plan
- No section markers
- "Detailed Design" placeholder says `{High-level technical or process design approach}` — confusing label for a section called "Detailed Design"
- No conditional sections
- Missing: Requirements, Use Cases, Architecture, UI/UX Design, Testing Strategy sections

**Original metis patterns:**
- `[REQUIRED]` / `[CONDITIONAL]` markers on every section
- Requirements section (conditional, with User Requirements and System Requirements sub-sections with REQ-NNN identifiers)
- Use Cases section (conditional, with actor/scenario/outcome structure)
- Architecture section (conditional, with Overview + diagram sub-sections)
- Testing Strategy (conditional, with Unit/Integration/System sub-sections)
- Goals & Non-Goals uses bullet list format explicitly

**Specific improvements needed:**
1. Add `[REQUIRED]` markers to Context, Goals & Non-Goals, Detailed Design, Alternatives Considered, Implementation Plan
2. Fix the Detailed Design placeholder: `{Technical approach and implementation details}` (remove "High-level" contradiction)
3. Add `## Requirements [CONDITIONAL: Requirements-Heavy Initiative]` with User/System sub-sections
4. Add `## Use Cases [CONDITIONAL: User-Facing Initiative]` with actor/scenario/outcome structure
5. Add `## Architecture [CONDITIONAL: Technically Complex Initiative]` with component diagram sub-section
6. Add `## Testing Strategy [CONDITIONAL: Separate Testing Initiative]` with unit/integration/system sub-sections
7. Add opening disclaimer line

---

### Task Template — Impact: CRITICAL

**Current state (cadre):**
- Only 3 sections: Description, Approach, Progress
- Uses HTML comments (`<!-- ... -->`) for guidance — good, but missing key sections entirely
- **No Acceptance Criteria section** — this is the most critical gap
- **No Status Updates section** — agents cannot use it as working memory
- **No Implementation Notes sub-structure** (Technical Approach, Dependencies, Risk Considerations)
- No Parent Initiative link
- No [REQUIRED]/[CONDITIONAL] markers

**Original metis patterns:**
- 7+ sections: Parent Initiative, Objective, Backlog Item Details (conditional), Acceptance Criteria, Test Cases (conditional), Documentation (conditional), Implementation Notes (conditional), Status Updates
- Acceptance Criteria with checkbox list: `- [ ] {Specific, testable requirement 1}`
- Backlog Item Details with Type checklist (Bug/Feature/Tech Debt/Chore) and Priority checklist (P0–P3)
- Status Updates with `*To be added during implementation*` signal
- Implementation Notes with Technical Approach, Dependencies, Risk Considerations sub-sections
- Conditional sections for bug impact assessment, feature business justification, tech debt impact

**Specific improvements needed (ranked by impact):**
1. **[CRITICAL]** Add `

## Acceptance Criteria

## Acceptance Criteria [REQUIRED]` with checkbox list format
2. **[CRITICAL]** Add `## Status Updates [REQUIRED]` with `*To be added during implementation*`
3. **[HIGH]** Add `## Parent Initiative [CONDITIONAL]` with Tera conditional: `{% if parent_title %}[[{{ parent_title }}]]{% else %}...{% endif %}`
4. **[HIGH]** Rename "Description" → "Objective" to match metis and be more precise
5. **[HIGH]** Restructure Implementation Notes with Technical Approach, Dependencies, Risk Considerations sub-sections
6. **[MEDIUM]** Add `## Backlog Item Details [CONDITIONAL: Backlog Item]` with Type/Priority checklists
7. **[LOW]** Add `[REQUIRED]` / `[CONDITIONAL]` markers throughout

---

### ADR Template — Impact: MEDIUM

**Current state (cadre):**
- 4 sections: Context, Decision, Consequences, Status
- Consequences is flat (no Positive/Negative/Neutral sub-sections)
- No Rationale section (why this option vs alternatives)
- No Alternatives Analysis table
- No conditional sections
- No [REQUIRED]/[CONDITIONAL] markers

**Original metis patterns:**
- `[REQUIRED]` markers on Context, Decision, Rationale, Consequences
- Consequences split into Positive/Negative/Neutral sub-sections with bullet lists
- `## Alternatives Analysis [CONDITIONAL: Complex Decision]` with full table: Option | Pros | Cons | Risk Level | Implementation Cost
- `## Review Schedule [CONDITIONAL: Temporary Decision]` with Review Triggers, Scheduled Review date, and Sunset Date
- Opening disclaimer

**Specific improvements needed:**
1. Add `[REQUIRED]` markers to Context, Decision, Consequences
2. Split Consequences into ### Positive / ### Negative / ### Neutral with bullet lists
3. Add `## Rationale [REQUIRED]` section between Decision and Consequences
4. Add `## Alternatives Analysis [CONDITIONAL: Complex Decision]` with table scaffold
5. Add `## Review Schedule [CONDITIONAL: Temporary Decision]` with review triggers
6. Add opening disclaimer line
7. Status section: provide specific options: `{Current status: Draft / Under Discussion / Decided / Superseded}`

---

### Summary: Impact Ranking

| Gap | Document Type | Impact |
|-----|--------------|--------|
| No Acceptance Criteria section | Task | Critical |
| No Status Updates section | Task | Critical |
| No section markers ([REQUIRED]/[CONDITIONAL]) | All | High |
| No conditional sections | Vision, Initiative, ADR | High |
| Only 3 sections total | Task | High |
| No Rationale section | ADR | Medium |
| No Alternatives Analysis table | ADR | Medium |
| No Testing Strategy section | Initiative | Medium |
| Confusing "Detailed Design" placeholder | Initiative | Low |

## Alternatives Considered

1. **Copy metis templates verbatim**: Rejected — different document types, different structure. Need to understand principles and apply them to cadre's richer type system.
2. **AI-generated templates**: Rejected — templates should be curated by humans for consistency. AI can help draft but humans should approve.
3. **Skip investigation, just iterate**: Rejected — we'd iterate blindly. Understanding WHY metis templates work better gives us targeted improvements.

## Benchmark Results

Re-run date: 2026-03-17

### Scoring Rubric

| Score | Criteria |
|-------|----------|
| 1 | Template is blank or only has a title |
| 2 | Template has sections but all are bare placeholders |
| 3 | Template has sections with some guidance text but no examples or format suggestions |
| 4 | Template has guidance, some examples, some conditional markers |
| 5 | Template has rich guidance, inline examples, format suggestions, conditional deletion markers, required/optional distinctions |

### Template Scores After SMET-I-0037 Rewrites

| Document Type | Before | After | Notes |
|--------------|--------|-------|-------|
| Vision | 3/5 | 5/5 | Added [REQUIRED]/[CONDITIONAL] markers, HTML comment guidance with inline examples, Success Criteria table scaffold, 3 conditional sections |
| Initiative | 3/5 | 5/5 | Added all markers, conditional sections (Requirements/Use Cases/Architecture/Testing), HTML comment guidance, Status Updates, Implementation Plan template |
| Task | 2/5 | 5/5 | Full rewrite: 3→7+ sections. Added Acceptance Criteria (checkboxes), Backlog Item Details (type/priority checklists), Implementation Notes (Technical Approach/Files/Dependencies/Risk table), Status Updates with dated entry example |
| ADR | 3/5 | 5/5 | Added Rationale, Alternatives Analysis table (conditional), Consequences split into Positive/Negative/Neutral, Review Schedule (conditional), [REQUIRED] markers |
| **Overall** | **3/5** | **5/5** | Target achieved |

### Evidence

Built `./target/release/cadre` and created fresh benchmark project. Rendered Vision, Initiative, and Task documents confirmed rich template output with all markers, guidance, and conditional sections present.

**Notable**: Task template renders with full 7-section structure including Acceptance Criteria checkboxes and dated Status Update example — exactly matching original metis quality.

### Observation

Task's Parent Initiative section rendered as the placeholder ("not yet assigned") even though the task has a parent. This is because the CLI's create command uses `render_content` (title-only context) rather than `render_with_context` (which passes parent data). The template itself is correct Tera logic. Filed as separate improvement opportunity.

## Status Updates

### 2026-03-17 — All tasks completed
- SMET-T-0087: Gap analysis written into initiative — identified Task template as most critical gap (missing Acceptance Criteria, Status Updates), all templates missing [REQUIRED]/[CONDITIONAL] markers
- SMET-T-0088: All 4 templates rewritten — Vision/Initiative/ADR got conditional sections and markers, Task got full 7-section rewrite; all 738 tests pass
- SMET-T-0089: 12 template quality tests added covering structural completeness, guidance presence, and rendering correctness with/without parent context
- SMET-T-0090: Benchmark re-run confirmed 3/5 → 5/5 across all template types; results recorded above
- Side finding: CLI uses `render_content` (title-only) instead of `render_with_context` for document creation — parent context not injected at creation time (separate improvement)

## Implementation Plan

Phase 1: Read and analyze original metis templates
Phase 2: Read and analyze cadre templates
Phase 3: Gap analysis (side-by-side comparison document)
Phase 4: Rewrite templates to close gaps
Phase 5: Add template quality tests
Phase 6: Re-run benchmark and verify improvement
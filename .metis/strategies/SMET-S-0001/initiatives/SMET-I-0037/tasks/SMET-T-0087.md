---
id: document-template-audit-findings
level: task
title: "Document template audit findings and gap analysis"
short_code: "SMET-T-0087"
created_at: 2026-03-17T20:15:51.669+00:00
updated_at: 2026-03-17T20:24:22.275273+00:00
parent: SMET-I-0037
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: SMET-I-0037
---

# Document template audit findings and gap analysis

## Parent Initiative

[[SMET-I-0037]]

## Objective

The research for SMET-I-0037 (Phases 1–3) was completed via parallel agents during the decompose session. This task transcribes those findings into a structured gap analysis document embedded in the initiative, and writes the final side-by-side comparison that will drive the rewrites in SMET-T-0088.

Research findings are known; they need to be organized and recorded so the rewrite task has concrete targets for each section of each document type template.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] The gap analysis is written into the SMET-I-0037 initiative document under a `## Gap Analysis` section
- [ ] For each document type (vision, initiative, task, ADR), the analysis lists:
  - Current ultra-metis template sections and their weaknesses
  - Corresponding original metis patterns that work better
  - Specific improvements needed (with concrete examples of better guidance text)
- [ ] Each identified gap is ranked by impact (high/medium/low) on output quality
- [ ] The gap analysis is readable by the implementer of SMET-T-0088 without needing to re-read raw template files
- [ ] No code changes required — this is a documentation-only task

## Implementation Notes

### Findings Summary (from research agents)

**Original Metis Template Strengths:**
- Conditional sections marked with `**[CONDITIONAL: delete if not applicable]**` so writers know exactly what to remove
- `**[REQUIRED]**` markers on mandatory sections
- Placeholder text shows format, not just a description: `{Specific, testable requirement 1}` instead of `{describe requirements}`
- Inline examples scattered throughout: `"Example: 'Reduce API latency by 50% for P99 requests'"`
- "DELETE THIS SECTION if..." explicit deletion instructions at the start of optional sections
- Checkbox enforcement for risk/dependency categories so nothing is missed
- "Status Updates: To be added during implementation" as a living-document signal

**Ultra-Metis Template Weaknesses (by document type):**

*Vision (`super-metis/crates/super-metis-core/src/domain/documents/vision/content.md`)*:
- 6 sections, all with single-line placeholders like `{Why this vision exists and what problem it solves}`
- No format guidance, no examples, no conditional markers
- Missing: measurable success criteria format, stakeholder table, time horizon scaffolding

*Initiative (`super-metis/crates/super-metis-core/src/domain/documents/initiative/content.md`)*:
- "Detailed Design" section labeled as "High-level" (contradictory)
- No conditional sections for optional fields (risk table, dependencies, alternatives)
- Missing: complexity estimate guidance, phase timeline template, acceptance criteria format, link to parent vision

*Task (`super-metis/crates/super-metis-core/src/domain/documents/task/content.md`)*:
- Only 3 sections: Description, Approach, Progress
- No Acceptance Criteria section
- No Implementation Notes section
- No Dependencies, Risk, or Status Updates sections
- A task template with only 3 placeholder lines is the weakest relative to metis

*ADR (`super-metis/crates/super-metis-core/src/domain/documents/adr/content.md`)*:
- Missing: consequences section, alternatives table format, decision status marker
- No guidance on how to write a decision statement vs. a solution description

### Technical Approach

1. Edit SMET-I-0037 to add a `## Gap Analysis` section after `## Detailed Design`
2. For each document type, write a subsection with:
   - Current state (what the template has)
   - Target state (what it should have)
   - Specific changes (bulleted list of edits to make)
3. Include concrete examples of improved guidance text for the rewrite implementer

### Dependencies

- Research findings from SMET-I-0037 decompose session (already collected)
- No code dependencies

### Risk Considerations

- Low risk: documentation-only task
- The analysis must be specific enough to drive SMET-T-0088 without ambiguity

## Status Updates

### 2026-03-17
- Read both ultra-metis and original metis domain/documents templates (4 types each)
- Discovered original metis uses `domain/documents/` templates (not `templates/`), which are significantly richer
- Key findings: Task template is most critical gap (missing Acceptance Criteria + Status Updates), all templates missing [REQUIRED]/[CONDITIONAL] markers, Vision/Initiative/ADR missing conditional sections
- Wrote full gap analysis into SMET-I-0037 under `## Gap Analysis` section with per-type analysis, specific improvements, and impact ranking table
- All acceptance criteria met: gap analysis written, all 4 types covered, gaps ranked by impact
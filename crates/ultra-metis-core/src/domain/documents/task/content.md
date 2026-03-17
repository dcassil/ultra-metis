# {{ title }}

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

{% if parent_title is defined and parent_title != "" %}[[{{ parent_title }}]]{% if parent_short_code is defined and parent_short_code != "" %} ({{ parent_short_code }}){% endif %}{% else %}{This task is not yet assigned to an initiative. Delete this section for standalone backlog items.}{% endif %}

## Objective **[REQUIRED]**

<!-- Clear statement of what this task accomplishes and why it matters.
Example: "Implement rate limiting on the /auth endpoint to prevent credential stuffing attacks."
One or two sentences max — if it takes more, the task may need to be split. -->

{Clear statement of what this task accomplishes and why it matters}

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [ ] Bug — Production issue that needs fixing
- [ ] Feature — New functionality or enhancement
- [ ] Tech Debt — Code improvement or refactoring
- [ ] Chore — Maintenance or setup work

### Priority
- [ ] P0 — Critical (blocks users or revenue)
- [ ] P1 — High (important for user experience)
- [ ] P2 — Medium (nice to have)
- [ ] P3 — Low (when time permits)

### Impact Assessment **[CONDITIONAL: Bug]**
- **Affected Users**: {Number or percentage of users affected}
- **Reproduction Steps**:
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what actually happens}

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics or revenue}
- **Effort Estimate**: {Rough size — S/M/L/XL}

### Technical Debt Impact **[CONDITIONAL: Tech Debt]**
- **Current Problems**: {What is difficult, slow, or buggy now}
- **Benefits of Fixing**: {What improves after this work}
- **Risk of Not Fixing**: {What gets worse if we skip this}

## Acceptance Criteria **[REQUIRED]**

<!-- Each criterion must be specific and testable. An AC is done when someone can verify it without asking the author.
Bad: "Works correctly" — Good: "Returns HTTP 200 with {id: string} for valid input, HTTP 400 for missing fields"
Use checkboxes so progress is visible during implementation. -->

- [ ] {Specific, testable criterion — what must be true when this task is complete}
- [ ] {Another criterion — observable behavior or measurable outcome}
- [ ] {Another criterion}

## Implementation Notes **[CONDITIONAL: Technical Task]**

{Keep for technical tasks; delete for non-technical work}

### Technical Approach

<!-- How this will be implemented. Name the files, functions, modules, or APIs involved.
Include key design decisions and why alternatives were not chosen. -->

{How this task will be implemented — specific enough to start without additional clarification}

### Files to Modify

- `{path/to/file.rs}` — {what changes and why}
- `{path/to/other.rs}` — {what changes and why}

### Dependencies

<!-- Other tasks, systems, or external APIs this task depends on. -->

- {Dependency 1 — e.g., "SMET-T-XXXX must be merged first"}
- {External dependency — e.g., "Requires access to staging database"}

### Risk Considerations

<!-- Technical risks and mitigation strategies. What could go wrong?
Example: "Tera syntax errors will cause runtime panics — test each template change immediately." -->

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| {Risk description} | Low/Med/High | Low/Med/High | {How to mitigate} |

## Status Updates **[REQUIRED]**

<!-- Add dated entries as you work. Record findings, decisions, and blockers.
This section is your working memory — update it after each significant step.

Example:
### 2026-03-17
- Investigated X, found Y
- Decided to use approach Z because W
- Next: implement Z in src/foo.rs
-->

*To be added during implementation*

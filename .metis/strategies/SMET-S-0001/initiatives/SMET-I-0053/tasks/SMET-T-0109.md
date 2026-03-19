---
id: durable-memory-insight-notes
level: task
title: "Durable Memory & Insight Notes: Scoring & Conflict Detection"
short_code: "SMET-T-0109"
created_at: 2026-03-17T22:06:42.876363+00:00
updated_at: 2026-03-17T22:32:40.426075+00:00
parent: SMET-I-0053
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0053
---

# Durable Memory & Insight Notes: Scoring & Conflict Detection

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[SMET-I-0053]]

## Objective

Investigate durable memory and insight notes system in Ultra-Metis. Document note structure, categorization, scoring system, prune candidates detection, conflict detection, and integration with task workflows.

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [ ] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement  
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [ ] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Impact Assessment **[CONDITIONAL: Bug]**
- **Affected Users**: {Number/percentage of users affected}
- **Reproduction Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what happens}

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **[CONDITIONAL: Tech Debt]**
- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Durable insight notes structure and categorization documented
- [x] Scoring and feedback system documented
- [x] Conflict detection and prune logic documented
- [x] Task integration mechanisms documented
- [x] Memory system capabilities in Metis identified and compared

## Test Cases **[CONDITIONAL: Testing Task]**

{Delete unless this is a testing task}

### Test Case 1: {Test Case Name}
- **Test ID**: TC-001
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

### Test Case 2: {Test Case Name}
- **Test ID**: TC-002
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

## Documentation Sections **[CONDITIONAL: Documentation Task]**

{Delete unless this is a documentation task}

### User Guide Content
- **Feature Description**: {What this feature does and why it's useful}
- **Prerequisites**: {What users need before using this feature}
- **Step-by-Step Instructions**:
  1. {Step 1 with screenshots/examples}
  2. {Step 2 with screenshots/examples}
  3. {Step 3 with screenshots/examples}

### Troubleshooting Guide
- **Common Issue 1**: {Problem description and solution}
- **Common Issue 2**: {Problem description and solution}
- **Error Messages**: {List of error messages and what they mean}

### API Documentation **[CONDITIONAL: API Documentation]**
- **Endpoint**: {API endpoint description}
- **Parameters**: {Required and optional parameters}
- **Example Request**: {Code example}
- **Example Response**: {Expected response format}

## Implementation Notes **[CONDITIONAL: Technical Task]**

{Keep for technical tasks, delete for non-technical. Technical details, approach, or important considerations}

### Technical Approach
{How this will be implemented}

### Dependencies
{Other tasks or systems this depends on}

### Risk Considerations
{Technical risks and mitigation strategies}

## Status Updates **[REQUIRED]**

### Investigation Complete - March 17, 2026

**Comprehensive analysis of Ultra-Metis DurableInsightNote system completed.** Investigated module location:
`/Users/danielcassil/projects/ultra-metis/crates/ultra-metis-core/src/domain/documents/durable_insight_note/mod.rs`

**Findings Summary**:
- Ultra-Metis implements complete durable memory system; Original Metis has no equivalent
- 7 comprehensive comparison grids (M1-M7) created covering all aspects:
  - M1: System Overview & Capabilities (7 dimensions)
  - M2: Note Structure & Content Model (7 fields)
  - M3: Categorization & Classification (7 types + parsing)
  - M4: Scoring & Feedback System (9 metrics & ratios)
  - M5: Prune Candidate Detection (8 criteria)
  - M6: Conflict Detection & Human Review (9 review mechanisms)
  - M7: Task Workflow Integration & Lifecycle (9 integration points)

**Key Capability Gaps**:
- Priority: CRITICAL (entirely missing from original Metis)
- Complexity: XL (sophisticated multi-subsystem design)
- Ultra-Metis robustness: Significantly better (typed enums, FromStr parsing, state machine validation)

**Architectural Insights**:
- Bidirectional integration: fetch notes at task start, record feedback at wrap-up
- 4-state lifecycle: Active → PruneCandidate → NeedsHumanReview → Archived
- Scope-based matching: repository, package, subsystem, paths, symbols
- Multi-factor pruning: staleness, harmful ratio, meh accumulation, low value threshold
- 5 typed conflict reasons for risk-aware review workflows

Section M of SMET-I-0053 updated with full investigation findings.
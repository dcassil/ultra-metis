---
id: bootstrap-repository-detection
level: task
title: "Bootstrap & Repository Detection: Language, Tool & Monorepo"
short_code: "SMET-T-0106"
created_at: 2026-03-17T22:06:40.705562+00:00
updated_at: 2026-03-17T22:29:37.863753+00:00
parent: SMET-I-0053
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0053
---

# Bootstrap & Repository Detection: Language, Tool & Monorepo

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[SMET-I-0053]]

## Objective

Investigate bootstrap and repository detection capabilities in Ultra-Metis. Document language detection, build tool identification, package manager detection, monorepo detection, and dev tool discovery. Compare with bootstrap capabilities in Metis.

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

## Acceptance Criteria

- [x] Language, build tool, and package manager detection documented
- [x] Monorepo detection and package discovery documented
- [x] Dev tool discovery documented
- [x] Bootstrap orchestration flow documented
- [x] Bootstrap capabilities in Metis identified and compared

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

### Execution Complete

**Date**: 2026-03-17
**Status**: COMPLETED

### Investigation Findings

Completed comprehensive analysis of bootstrap and repository detection capabilities in Ultra-Metis vs original Metis. Created 8 detailed comparison grids (J1-J8) documenting:

1. **Language Detection (J1)**: Ultra-Metis supports 23+ languages with statistical ranking; Original Metis only detects Metis workspaces
2. **Build Tools (J2)**: Ultra-Metis detects 14 build systems; Original Metis has no explicit detection
3. **Package Managers (J3)**: Ultra-Metis identifies 11 package managers with ecosystem-specific detection; Original Metis is workspace-agnostic
4. **Monorepo Detection (J4)**: Ultra-Metis detects 9 monorepo patterns with package discovery/classification; Original Metis lacks this capability
5. **Dev Tools (J5)**: Ultra-Metis discovers 30+ development tools across 7 categories; Original Metis has implicit tool integration only
6. **Project Type Inference (J6)**: Ultra-Metis infers 7 project archetypes; Original Metis treats all projects uniformly
7. **Brownfield Detection (J7)**: Ultra-Metis classifies projects as brownfield/greenfield; Original Metis has no maturity classification
8. **Bootstrap Orchestration (J8)**: Ultra-Metis provides end-to-end automated analysis; Original Metis requires manual configuration

### Key Findings Summary

- **Ultra-Metis Strengths**: Comprehensive polyglot support, automated project discovery, intelligent recommendations, structured output, extensive test coverage
- **Original Metis Strengths**: Focused workspace detection, proven reliability for Metis projects
- **Gap Analysis**: Ultra-Metis covers 8 distinct detection/analysis dimensions that original Metis lacks entirely
- **Architecture Quality**: Ultra-Metis uses pure functions on path lists (no I/O in domain layer), enabling easier testing and composition

### Acceptance Criteria Met

- [x] Language, build tool, and package manager detection documented
- [x] Monorepo detection and package discovery documented  
- [x] Dev tool discovery documented
- [x] Bootstrap orchestration flow documented
- [x] Bootstrap capabilities in Metis identified and compared
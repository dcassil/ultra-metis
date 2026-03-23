---
id: mcp-server-tool-integration
level: task
title: "MCP Server & Tool Integration: Complete Tool Coverage"
short_code: "SMET-T-0101"
created_at: 2026-03-17T22:06:37.718597+00:00
updated_at: 2026-03-17T22:34:41.106955+00:00
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

# MCP Server & Tool Integration: Complete Tool Coverage

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[SMET-I-0053]]

## Objective

Document all MCP server tools in both systems. Map tool implementations, parameters, request/response handling, filtering/querying, output formats. Compare feature completeness and API differences between original and Cadre implementations.

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

- [x] All MCP tools documented for both systems (list, search, read, create, edit, transition, archive, reassign)
- [x] Tool signatures, parameters, filtering, and output formats compared
- [x] Tool coverage gaps identified
- [x] API differences documented (parameter names, return types, behavior)

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

### Completion Summary (2026-03-17)

**Investigation Complete**: All MCP tools in both Original Metis and Cadre have been thoroughly analyzed and compared.

**Tools Investigated (10 total):**
1. initialize_project - Both ✓
2. list_documents - Both ✓
3. search_documents - Both ✓ (gaps identified)
4. read_document - Both ✓
5. create_document - Both ✓ (gaps identified)
6. edit_document - Both ✓ (gaps identified)
7. transition_phase - Both ✓ (gaps identified)
8. archive_document - Both ✓
9. reassign_parent - Original ✓, Cadre ✗ MISSING
10. index_code - Original ✓, Cadre ✗ MISSING

**Comparison Grids Created: E1-E9**
- E1: Tool Inventory & Coverage (10x4 grid)
- E2: Initialize Project Tool (7x3 grid)
- E3: List Documents Tool (8x3 grid)
- E4: Search Documents Tool (9x3 grid)
- E5: Read Document Tool (8x3 grid)
- E6: Create Document Tool (11x3 grid)
- E7: Edit Document Tool (9x3 grid)
- E8: Transition Phase Tool (11x3 grid)
- E9: Archive & Reassign Tools (13x3 grid)

**Key Findings:**
- 8/10 tools have complete parity (80%)
- 2/10 tools missing in Cadre (20%)
- 4 tools have parameter gaps reducing functionality
- Original Metis uses production-grade database-driven approach (SQLite FTS5)
- Cadre uses simpler file-based stateless approach
- Gap areas identified for prioritized feature parity work

**Critical Gaps (for follow-up initiatives):**
- reassign_parent tool (blocks task reorganization)
- index_code tool (code indexing capability)
- search_documents filtering (document_type, limit, include_archived)
- edit_document replace_all parameter
- transition_phase force parameter

**All acceptance criteria met. Initiative section E.Tool Integration & APIs updated with detailed comparison grids.**
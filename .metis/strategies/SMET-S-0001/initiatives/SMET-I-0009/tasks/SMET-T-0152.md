---
id: integration-tests-for-all-new-mcp
level: task
title: "Integration Tests for All New MCP Tools"
short_code: "SMET-T-0152"
created_at: 2026-03-20T17:47:25.110742+00:00
updated_at: 2026-03-20T21:05:54.894344+00:00
parent: SMET-I-0009
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0009
---

# Integration Tests for All New MCP Tools

## Context

Each new MCP tool needs end-to-end integration tests that exercise the full stack: tool definition, parameter parsing, store operations, and response formatting. The existing tools have some coverage through the `tool_*` handler functions but there is no dedicated integration test suite.

This task creates a comprehensive test suite that verifies all new tools work correctly through the `call_tool()` dispatch function, which is the entry point used by the MCP protocol handler.

## Implementation Plan

### Test Infrastructure

1. **Create a test helper module** with:
   - A function to set up a temporary project directory with `DocumentStore::initialize()`
   - A function to call a tool by name with JSON arguments and assert success
   - A function to call a tool and assert a specific error
   - Pre-seeded test documents (vision, initiative, task, rules config, baseline, notes, cross-references)

### Test Cases by Tool Group

2. **Quality Tools Tests:**
   - `capture_quality_baseline` with valid eslint JSON output -> creates AnalysisBaseline
   - `capture_quality_baseline` with valid clippy output -> creates AnalysisBaseline
   - `capture_quality_baseline` with invalid tool name -> returns error
   - `compare_quality_baselines` with two baselines from same tool -> creates QualityRecord
   - `compare_quality_baselines` with mismatched tools -> returns error
   - `list_quality_records` with and without status filter
   - `check_architecture_conformance` with valid reference architecture

3. **Rule Tools Tests:**
   - `query_rules` with no filters -> returns all rules
   - `query_rules` with scope filter -> returns only matching scope
   - `query_rules` with protection level filter -> returns only protected/standard
   - `get_applicable_rules` at Package scope -> returns Platform + Organization + Repository + Package rules
   - `list_protected_rules` -> returns only protected rules
   - All queries with empty result sets -> informative message

4. **Insight Note Tools Tests:**
   - `create_insight_note` with full scope -> creates note with correct attributes
   - `fetch_insight_notes` by repo scope -> returns matching notes, increments fetch count
   - `fetch_insight_notes` by path scope -> returns matching notes
   - `score_insight_note` with helpful signal -> increments thumbs_up_count
   - `score_insight_note` with harmful signal triggering prune candidate -> status changes
   - `list_insight_notes` with status filter

5. **Traceability Tools Tests:**
   - `create_cross_reference` with valid source/target -> creates relationship
   - `create_cross_reference` with self-reference -> returns error
   - `create_cross_reference` with nonexistent document -> returns error
   - `query_relationships` outgoing -> correct results
   - `query_relationships` incoming -> correct results
   - `trace_ancestry` ancestors -> correct chain
   - `trace_ancestry` descendants -> correct tree
   - `trace_ancestry` siblings -> correct peers
   - `list_cross_references` with and without filters

6. **Architecture Catalog Tools Tests:**
   - `query_architecture_catalog` by language -> returns matching entries
   - `query_architecture_catalog` with no match -> informative message
   - `list_catalog_languages` -> returns all languages and project types
   - `read_reference_architecture` with valid short code -> returns full details
   - `evaluate_brownfield` against a test directory

### Test Execution

- All tests use `call_tool()` directly with JSON `Value` arguments
- Each test creates its own temporary project directory for isolation
- Tests verify both the success/error status and the content of responses

## Files to Modify

- `crates/ultra-metis-mcp/src/tools.rs` — Add `#[cfg(test)]` module with integration tests
- Alternatively, create `crates/ultra-metis-mcp/tests/integration_tests.rs` for a separate test binary

## Dependencies

- Depends on all other tasks in this initiative (SMET-T-0145 through SMET-T-0151)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] At least 30 integration tests covering all new tool handlers
- [ ] Tests cover success paths, error paths, and edge cases for every new tool
- [ ] Tests exercise the full stack through `call_tool()` dispatch
- [ ] All tests pass with `cargo test` in the ultra-metis-mcp crate
- [ ] Test helpers are reusable for future tool test additions
- [ ] Tests are isolated (each creates its own temp project, no shared state)

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[SMET-I-0009]]

## Objective **[REQUIRED]**

{Clear statement of what this task accomplishes}

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

## Acceptance Criteria **[REQUIRED]**

- [ ] {Specific, testable requirement 1}
- [ ] {Specific, testable requirement 2}
- [ ] {Specific, testable requirement 3}

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

*To be added during implementation*
---
id: implement-traceability-and-cross
level: task
title: "Implement Traceability and Cross-Reference MCP Tools"
short_code: "SMET-T-0149"
created_at: 2026-03-20T17:47:21.930220+00:00
updated_at: 2026-03-20T20:56:12.879904+00:00
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

# Implement Traceability and Cross-Reference MCP Tools

## Context

The traceability system (`crates/ultra-metis-core/src/domain/documents/cross_reference/`) provides typed relationships between documents (ParentChild, Governs, References, DerivedFrom, Supersedes, ConflictsWith, Validates, Blocks, ApprovedBy) and a `TraceabilityIndex` that supports graph queries: ancestors, descendants, siblings, outgoing/incoming relationships by type, and general involvement queries.

Agents need MCP tools to:
- Create cross-references between documents to build the traceability graph
- Query relationships involving a specific document
- Walk the document hierarchy (ancestors, descendants, siblings)
- Find all documents governed by, blocked by, or validated by a given document

## Implementation Plan

### New MCP Tools to Add

1. **`create_cross_reference`** — Create a typed relationship between two documents.
   - Parameters: `project_path`, `source_ref` (short code), `target_ref` (short code), `relationship_type` (parent_child/governs/references/derived_from/supersedes/conflicts_with/validates/blocks/approved_by), `description` (optional), `bidirectional` (optional bool, default false)
   - Validates both source and target documents exist in the store
   - Creates a `CrossReference` document and persists it
   - Returns the cross-reference short code and relationship summary

2. **`query_relationships`** — Query all relationships involving a specific document.
   - Parameters: `project_path`, `short_code`, `direction` (optional: outgoing/incoming/all, default all), `relationship_type` (optional filter)
   - Loads all CrossReference documents from the store
   - Builds a `TraceabilityIndex` and queries it
   - Returns a table of relationships with source, target, type, and description

3. **`trace_ancestry`** — Walk the document hierarchy to find ancestors and descendants.
   - Parameters: `project_path`, `short_code`, `direction` (ancestors/descendants/siblings)
   - Loads all CrossReference documents, builds TraceabilityIndex
   - Uses `ancestors()`, `descendants()`, or `siblings()` methods
   - Returns the lineage chain with document titles and types

4. **`list_cross_references`** — List all cross-references with optional filtering.
   - Parameters: `project_path`, `relationship_type` (optional), `involving` (optional short code)
   - Returns a table of all cross-references

### Changes to Existing Code

- Add tool definitions to `get_tool_definitions()` in `tools.rs`
- Add handler functions in `tools.rs`
- Register in `call_tool()` match block
- The TraceabilityIndex is built in-memory from stored CrossReference documents each time a query runs (no persistent index file needed)

## Files to Modify

- `crates/ultra-metis-mcp/src/tools.rs` — New tool definitions and handlers

## Dependencies

- Depends on SMET-T-0145 (store must support CrossReference type)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `create_cross_reference` tool creates typed relationships between documents with validation
- [ ] `query_relationships` tool returns outgoing, incoming, or all relationships for a document
- [ ] `query_relationships` tool supports filtering by relationship type
- [ ] `trace_ancestry` tool correctly walks ancestors (upward), descendants (downward), and siblings
- [ ] `list_cross_references` tool returns all cross-references with optional filtering
- [ ] Bidirectional relationships work correctly in both directions
- [ ] Self-references are rejected with a clear error message
- [ ] All tools return structured markdown output consistent with existing tool formatting

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
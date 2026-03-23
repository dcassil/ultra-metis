---
id: implement-quality-baseline-and
level: task
title: "Implement Quality Baseline and Record MCP Tools"
short_code: "SMET-T-0146"
created_at: 2026-03-20T17:47:18.341733+00:00
updated_at: 2026-03-20T20:48:24.298560+00:00
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

# Implement Quality Baseline and Record MCP Tools

## Context

Cadre has complete quality infrastructure in `crates/cadre-core/src/domain/quality/`: baseline capture (`BaselineCaptureService`), baseline comparison (`BaselineComparisonEngine`), conformance checking (`ArchitectureConformanceChecker`), and tool output parsers. However, none of this is accessible via MCP tools. Agents need to be able to:

- Capture tool output into durable AnalysisBaseline documents
- List and read existing baselines
- Compare two baselines to produce QualityRecord documents
- Query quality records by status (pass/warn/fail)

## Implementation Plan

### New MCP Tools to Add

1. **`capture_quality_baseline`** — Parse raw tool output and create an AnalysisBaseline document.
   - Parameters: `project_path`, `tool_name` (eslint/clippy/tsc/coverage), `raw_output` (string), `linked_rules_config` (optional short code)
   - Uses the appropriate parser from `quality::parsers` to produce `ParsedToolOutput`
   - Calls `BaselineCaptureService::capture()` to create an `AnalysisBaseline`
   - Persists to the store and returns the short code + summary

2. **`compare_quality_baselines`** — Compare two baselines and produce a QualityRecord.
   - Parameters: `project_path`, `before_short_code`, `after_short_code`
   - Reads both AnalysisBaseline documents from the store
   - Uses `BaselineComparisonEngine::compare()` and `to_quality_record()`
   - Persists the QualityRecord and returns the comparison summary (metric deltas, new/resolved findings, overall status)

3. **`list_quality_records`** — List quality records with optional status filtering.
   - Parameters: `project_path`, `status` (optional: pass/warn/fail), `limit` (optional)
   - Reads all QualityRecord documents from the store
   - Filters by `overall_status` if provided
   - Returns a table of short code, tool name, date, status

4. **`check_architecture_conformance`** — Run conformance check against a reference architecture.
   - Parameters: `project_path`, `reference_arch_short_code`, `file_patterns` (glob patterns for files to check)
   - Reads the ReferenceArchitecture, collects file paths, runs `ArchitectureConformanceChecker::check()`
   - Returns conformance score, violations, and warnings

### Changes to Existing Code

- Add tool definitions to `get_tool_definitions()` in `tools.rs`
- Add handler functions (`tool_capture_quality_baseline`, etc.) in `tools.rs`
- Register handlers in the `call_tool()` match block
- The store extension from SMET-T-0145 must be complete for AnalysisBaseline and QualityRecord types

## Files to Modify

- `crates/cadre-mcp/src/tools.rs` — New tool definitions and handler functions
- `crates/cadre-mcp/Cargo.toml` — May need dependency on quality module types

## Dependencies

- Depends on SMET-T-0145 (store must support AnalysisBaseline and QualityRecord types)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `capture_quality_baseline` tool creates an AnalysisBaseline from raw tool output for at least eslint and clippy formats
- [ ] `compare_quality_baselines` tool produces a QualityRecord with metric deltas and overall status
- [ ] `list_quality_records` tool returns filtered quality records
- [ ] `check_architecture_conformance` tool returns conformance score and violations
- [ ] All tools return structured, human-readable markdown output consistent with existing tool formatting
- [ ] All tools handle error cases gracefully (missing documents, mismatched tools, invalid input)

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

### 2026-03-20: Completed
- Added 4 quality MCP tools: capture_quality_baseline, compare_quality_baselines, list_quality_records, check_architecture_conformance
- Added cadre-core and glob as MCP crate dependencies
- capture_quality_baseline uses existing parsers (eslint, clippy, tsc, coverage) and BaselineCaptureService
- All tools follow existing patterns with structured markdown output
- All tests pass (738 core + 58 store + 0 MCP)
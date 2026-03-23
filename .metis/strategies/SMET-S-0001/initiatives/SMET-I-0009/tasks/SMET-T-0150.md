---
id: implement-architecture-catalog
level: task
title: "Implement Architecture Catalog Query MCP Tools"
short_code: "SMET-T-0150"
created_at: 2026-03-20T17:47:22.872597+00:00
updated_at: 2026-03-20T20:59:49.113782+00:00
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

# Implement Architecture Catalog Query MCP Tools

## Context

The architecture catalog (`crates/ultra-metis-core/src/domain/catalog/`) provides built-in reference architectures for different language/project-type combinations (currently JavaScript: server, react-app, component-lib, cli-tool, node-util). The `CatalogQueryEngine` supports querying by language, project type, and phase, plus listing available languages and project types. The `BrownfieldEvaluator` can analyze existing repos against catalog entries.

Agents need MCP tools to:
- Browse available architecture catalog entries by language and project type
- Query for a specific architecture recommendation
- Read reference architecture details for a project
- Evaluate how well a brownfield repo matches catalog entries

## Implementation Plan

### New MCP Tools to Add

1. **`query_architecture_catalog`** — Search the architecture catalog by language and project type.
   - Parameters: `project_path`, `language` (optional), `project_type` (optional)
   - Uses `CatalogQueryEngine::with_builtins()` (plus any custom entries loaded from store)
   - Returns matched catalog entries with language, project type, folder layout, layers, naming conventions, and dependency rules

2. **`list_catalog_languages`** — List all available languages and project types in the catalog.
   - Parameters: `project_path`
   - Uses `CatalogQueryEngine::languages()` and `project_types_for_language()`
   - Returns a hierarchical view: language -> [project types]

3. **`read_reference_architecture`** — Read the project's selected reference architecture.
   - Parameters: `project_path`, `short_code` (optional, reads the active one if omitted)
   - Loads the ReferenceArchitecture document from the store
   - Returns the full architecture details: linked catalog entry, layer overrides, additional boundaries, dependency rules, tolerated exceptions, status

4. **`evaluate_brownfield`** — Evaluate how well the current repo matches a catalog entry.
   - Parameters: `project_path`, `language`, `project_type`
   - Uses the `BrownfieldEvaluator` to analyze the repo structure against the matching catalog entry
   - Returns the evaluation outcome (good_match/partial_match/poor_match/no_catalog_match) with a score and detailed findings

### Changes to Existing Code

- Add tool definitions to `get_tool_definitions()` in `tools.rs`
- Add handler functions in `tools.rs`
- Register in `call_tool()` match block
- The CatalogQueryEngine is instantiated fresh for each query (no persistent state needed beyond the store)
- For `evaluate_brownfield`, need to invoke `BrownfieldEvaluator` which reads the filesystem

## Files to Modify

- `crates/ultra-metis-mcp/src/tools.rs` — New tool definitions and handlers

## Dependencies

- Depends on SMET-T-0145 (store must support ArchitectureCatalogEntry and ReferenceArchitecture types)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `query_architecture_catalog` tool returns matching catalog entries filtered by language and project type
- [ ] `list_catalog_languages` tool returns all available languages and their project types
- [ ] `read_reference_architecture` tool reads and displays a project's reference architecture
- [ ] `evaluate_brownfield` tool runs brownfield evaluation and returns match outcome with score
- [ ] All tools load custom catalog entries from the store in addition to builtins
- [ ] All tools return structured markdown output consistent with existing tool formatting
- [ ] Empty result sets return helpful messages (e.g., "No catalog entries found for language 'rust'")

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
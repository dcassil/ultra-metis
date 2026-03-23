---
id: implement-rule-query-mcp-tools
level: task
title: "Implement Rule Query MCP Tools with Scope Inheritance"
short_code: "SMET-T-0147"
created_at: 2026-03-20T17:47:19.658202+00:00
updated_at: 2026-03-20T20:51:06.196479+00:00
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

# Implement Rule Query MCP Tools with Scope Inheritance

## Context

The rule system in ultra-metis-core provides `RulesConfig` documents with scope hierarchy (Platform > Organization > Repository > Package > Component > Task), protection levels, and a `RuleQueryEngine` for filtering. Agents need MCP tools to query applicable rules by scope, category, and protection level so they can understand what engineering rules govern their current work.

The existing `RuleQueryEngine` (`crates/ultra-metis-core/src/domain/rules/query.rs`) already supports scope-inheritance-aware queries, filtered queries by category/protection/architecture-ref, and protected rule enumeration. This task wraps that engine in MCP tools.

## Implementation Plan

### New MCP Tools to Add

1. **`query_rules`** — Query engineering rules by scope, category, and protection level.
   - Parameters: `project_path`, `scope` (optional: platform/organization/repository/package/component/task), `category` (optional: architectural/behavioral/operational/testing/documentation), `protection_level` (optional: standard/protected), `source_architecture_ref` (optional short code), `include_archived` (optional bool)
   - Loads all RulesConfig documents from the store
   - Constructs a `RuleQuery` from the parameters and runs it through `RuleQueryEngine`
   - Returns a table of matching rules with short code, title, scope, protection level, and category tags

2. **`get_applicable_rules`** — Get all rules that apply at a given scope via inheritance.
   - Parameters: `project_path`, `target_scope` (required: one of the 6 scope levels), `category` (optional)
   - Uses `RuleQueryEngine::applicable_at_scope()` or `applicable_at_scope_with_category()`
   - Returns rules ordered by scope breadth (platform first, then narrowing)
   - This is the primary tool agents use to understand "what rules apply to my current work"

3. **`list_protected_rules`** — List all protected rules (governance audit view).
   - Parameters: `project_path`
   - Uses `RuleQueryEngine::protected_rules()`
   - Returns a table of protected rules with their scope and source architecture reference

### Changes to Existing Code

- Add tool definitions to `get_tool_definitions()` in `tools.rs`
- Add handler functions in `tools.rs`
- Register handlers in `call_tool()` match block
- Need to load all RulesConfig docs from the store, convert to references, and pass to `RuleQueryEngine::new()`

## Files to Modify

- `crates/ultra-metis-mcp/src/tools.rs` — New tool definitions and handlers

## Dependencies

- Depends on SMET-T-0145 (store must support RulesConfig type)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `query_rules` tool filters rules by scope, category, protection level, and architecture ref
- [ ] `get_applicable_rules` tool returns inherited rules for a given scope level
- [ ] `list_protected_rules` tool returns all protected rules
- [ ] Scope inheritance works correctly (e.g., querying Package scope returns Platform + Organization + Repository + Package rules)
- [ ] All tools return structured markdown tables consistent with existing tool output format
- [ ] Empty result sets return informative messages rather than errors

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
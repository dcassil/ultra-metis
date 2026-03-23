---
id: implement-durable-insight-note-mcp
level: task
title: "Implement Durable Insight Note MCP Tools with Scope-Based Fetch and Feedback"
short_code: "SMET-T-0148"
created_at: 2026-03-20T17:47:20.975713+00:00
updated_at: 2026-03-20T20:53:45.255881+00:00
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

# Implement Durable Insight Note MCP Tools with Scope-Based Fetch and Feedback

## Context

The Durable Insight Note system (`crates/ultra-metis-core/src/domain/documents/durable_insight_note/`) provides lightweight, scoped, self-pruning repo memory. Notes capture reusable local knowledge (hotspot warnings, recurring failures, misleading names, validation hints, subsystem quirks). They have a rich lifecycle: creation, scope-based fetching, feedback scoring (helpful/meh/harmful), prune candidate detection, human review flagging, and archival.

Agents need MCP tools to:
- Create new insight notes when they discover reusable knowledge during work
- Fetch relevant notes by scope (repo, package, subsystem, file paths, symbols) at the start of tasks
- Record feedback after using a note to drive the self-pruning lifecycle
- Query notes by status/category for governance views

## Implementation Plan

### New MCP Tools to Add

1. **`create_insight_note`** — Create a new durable insight note.
   - Parameters: `project_path`, `title`, `note` (the insight text), `category` (hotspot_warning/recurring_failure/misleading_name/validation_hint/local_exception/boundary_warning/subsystem_quirk), `scope_repo` (optional), `scope_package` (optional), `scope_subsystem` (optional), `scope_paths` (optional array of file paths), `scope_symbols` (optional array of symbol names)
   - Creates a `DurableInsightNote` with the given scope and category
   - Persists to the store and returns the short code + summary

2. **`fetch_insight_notes`** — Fetch notes relevant to a given scope.
   - Parameters: `project_path`, `scope_repo` (optional), `scope_package` (optional), `scope_subsystem` (optional), `scope_paths` (optional array), `scope_symbols` (optional array), `category` (optional filter), `limit` (optional, default 10)
   - Loads all active DurableInsightNote documents from the store
   - Builds an `InsightScope` query and filters notes using `scope.matches()`
   - Optionally filters by category
   - Calls `record_fetch()` on each matched note and saves it back
   - Returns matched notes with their content, category, and fetch/feedback stats
   - This is the primary tool agents call at task start to load contextual knowledge

3. **`score_insight_note`** — Record feedback on a note after using it.
   - Parameters: `project_path`, `short_code`, `signal` (helpful/meh/harmful)
   - Loads the note, calls `record_feedback()`, saves it back
   - Also runs `should_be_prune_candidate()` and auto-marks if needed
   - Returns updated stats and any status change

4. **`list_insight_notes`** — List notes with optional status and category filtering.
   - Parameters: `project_path`, `status` (optional: active/prune_candidate/needs_human_review/archived), `category` (optional), `include_archived` (optional bool)
   - Returns a table of notes with short code, title, category, status, fetch count, helpful ratio

### Changes to Existing Code

- Add tool definitions to `get_tool_definitions()` in `tools.rs`
- Add handler functions in `tools.rs`
- Register in `call_tool()` match block
- Need to handle the `InsightScope` construction from flat tool parameters

## Files to Modify

- `crates/ultra-metis-mcp/src/tools.rs` — New tool definitions and handlers

## Dependencies

- Depends on SMET-T-0145 (store must support DurableInsightNote type)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `create_insight_note` tool creates a note with the given scope, category, and content
- [ ] `fetch_insight_notes` tool returns notes matching the query scope and increments fetch counts
- [ ] `score_insight_note` tool records feedback and triggers prune candidate detection
- [ ] `list_insight_notes` tool returns filtered views of all notes with stats
- [ ] Scope matching works correctly (any overlap in repo/package/subsystem/paths/symbols counts as a match)
- [ ] Auto-prune candidate detection works after feedback scoring
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
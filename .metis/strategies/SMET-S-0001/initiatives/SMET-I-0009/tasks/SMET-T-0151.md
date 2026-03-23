---
id: rewrite-mcp-system-prompt-with
level: task
title: "Rewrite MCP System Prompt with Complete Tool and Workflow Documentation"
short_code: "SMET-T-0151"
created_at: 2026-03-20T17:47:24.190654+00:00
updated_at: 2026-03-20T21:02:55.452225+00:00
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

# Rewrite MCP System Prompt with Complete Tool and Workflow Documentation

## Context

The MCP server currently has no system prompt — there is no `prompts/` directory and no prompt-related code in `protocol.rs`. Agents using the cadre MCP tools have no built-in guidance on:
- What tools are available and when to use them
- How tool parameters map to domain concepts
- The expected workflow for common operations (setup, quality capture, rule queries, etc.)
- Error handling patterns and retry strategies

A comprehensive system prompt will dramatically improve agent usability by providing inline documentation that helps agents choose the right tools and use them correctly.

## Implementation Plan

### Create System Prompt Content

1. **Tool Reference Section** — Document every MCP tool with:
   - Purpose (1-2 sentences)
   - Required and optional parameters with types and valid values
   - Example usage pattern
   - Return format description
   - Common error cases and how to handle them

2. **Domain Concept Guide** — Brief explanations of:
   - Document types and their lifecycle phases
   - Quality baselines and records (capture -> compare -> review)
   - Engineering rules and scope inheritance
   - Durable insight notes and the feedback/prune lifecycle
   - Cross-references and traceability graph
   - Architecture catalog and brownfield evaluation

3. **Workflow Recipes** — Step-by-step tool sequences for common operations:
   - "Set up quality tracking for a new project" (capture baseline -> set rules -> configure gates)
   - "Check quality before a phase transition" (capture new baseline -> compare -> check conformance)
   - "Start a new task with context" (fetch insight notes -> read applicable rules -> read parent initiative)
   - "Record a discovery during work" (create insight note -> create cross-reference)
   - "Evaluate architecture fit" (list catalog -> evaluate brownfield -> select reference architecture)

4. **Naming Conventions** — Document short code prefixes, document type names, phase sequences, and tag formats

### Technical Implementation

- Add a `prompts/` directory under `crates/cadre-mcp/src/` (or a `system_prompt.rs` module)
- Implement MCP `prompts/list` and `prompts/get` protocol handlers in `protocol.rs`
- The system prompt should be a static string compiled into the binary
- Keep the prompt under 4000 tokens to avoid bloating context windows

## Files to Modify

- `crates/cadre-mcp/src/protocol.rs` — Add prompt protocol handlers
- `crates/cadre-mcp/src/` — New module for system prompt content (system_prompt.rs or prompts/)

## Dependencies

- Should be done after all other tool tasks (SMET-T-0146 through SMET-T-0150) so the prompt documents all tools
- Can be started in parallel if tool signatures are finalized

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] System prompt documents all existing and new MCP tools
- [ ] Each tool has parameter descriptions, valid values, and example usage
- [ ] Domain concept guide covers all major subsystems (quality, rules, notes, traceability, architecture)
- [ ] At least 5 workflow recipes are included for common agent operations
- [ ] MCP `prompts/list` returns the available system prompt
- [ ] MCP `prompts/get` returns the full system prompt content
- [ ] System prompt is under 4000 tokens
- [ ] Prompt content is accurate and consistent with actual tool behavior

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
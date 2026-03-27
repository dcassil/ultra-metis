---
id: update-mcp-system-prompt-with
level: task
title: "Update MCP System Prompt with analyze_project Documentation and Enriched Init Flow"
short_code: "SMET-T-0192"
created_at: 2026-03-27T15:53:00.235743+00:00
updated_at: 2026-03-27T16:01:56.000201+00:00
parent: SMET-I-0093
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0093
---

# Update MCP System Prompt with analyze_project Documentation and Enriched Init Flow

## Parent Initiative

[[SMET-I-0093]]

## Objective

Update the MCP system prompt (`crates/cadre-mcp/src/system_prompt.rs`) to document the new `analyze_project` tool and the enriched `initialize_project` response. Add a workflow recipe for the recommended post-initialization flow.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `analyze_project` tool documented in the system prompt's tool reference section
- [ ] `initialize_project` documentation updated to reflect enriched brownfield/greenfield response
- [ ] New workflow recipe: "Setting Up a New Project" showing the recommended tool chain
- [ ] System prompt stays within ~4000 token budget (check existing length before adding)
- [ ] `cargo build` succeeds

## Implementation Notes

### Technical Approach

Modify `crates/cadre-mcp/src/system_prompt.rs`:

1. Add `analyze_project` to the appropriate tools section (likely near `initialize_project` under a "Project Setup Tools" heading)
2. Update the `initialize_project` entry to note it now returns bootstrap analysis for brownfield projects
3. Add or update the workflow recipe for project setup:

```
### Setting Up a New Project
1. `initialize_project` — creates .cadre/ and returns bootstrap analysis
2. If brownfield: review the bootstrap analysis, then `evaluate_brownfield` for architecture matching
3. Create a `product_doc` document to capture the product vision
4. If architecture catalog match: create a `reference_architecture` document
5. `index_code` to build the symbol index
6. Optionally `capture_quality_baseline` for initial quality snapshot
```

### Files to Modify
- **Modify**: `crates/cadre-mcp/src/system_prompt.rs`

### Dependencies
- Should be done after T-0190 and T-0191 so the tool names and response formats are finalized

## Status Updates

*To be added during implementation*
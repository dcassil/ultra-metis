---
id: add-status-command-with-dashboard
level: task
title: "Add status Command with Dashboard Aggregation"
short_code: "SMET-T-0123"
created_at: 2026-03-18T04:31:31.724150+00:00
updated_at: 2026-03-18T04:38:44.709084+00:00
parent: SMET-I-0056
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0056
---

# Add status Command with Dashboard Aggregation

## Parent Initiative

[[SMET-I-0056]] - CLI Architecture: Add Missing Commands and Parameter Parity

## Objective

Add a `status` CLI command that shows a dashboard of active work — initiatives by phase, tasks grouped by parent, and summary phase counts. Gives a quick overview of project health.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `ultra-metis status` shows dashboard of active work
- [ ] Lists active initiatives with phase and child task counts
- [ ] Lists active tasks grouped by parent initiative
- [ ] Shows summary: total docs, counts by type, counts by phase
- [ ] Formatted table output aligned for terminal readability
- [ ] Handles empty project gracefully (no documents)
- [ ] Unit tests for aggregation logic

## Implementation Notes

### Technical Approach
1. Use existing `list_documents(false)` to get all non-archived docs
2. Group by type (vision/initiative/task) and phase
3. For initiatives: count child tasks in each phase
4. Display sections:
   - **Initiatives** table: Code | Title | Phase | Tasks (todo/active/done)
   - **Active Tasks** table: Code | Title | Parent | Phase
   - **Summary**: N visions, N initiatives, N tasks | N todo, N active, N completed
5. Add `Status` variant to CLI Commands enum

### Key Files
- `crates/ultra-metis-cli/src/main.rs` — add Status command

## Status Updates

*To be added during implementation*
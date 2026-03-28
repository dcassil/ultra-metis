---
id: session-planning-context-panel-and
level: task
title: "Session Planning Context Panel and Integration Tests"
short_code: "SMET-T-0245"
created_at: 2026-03-28T00:33:50.540358+00:00
updated_at: 2026-03-28T01:12:32.345859+00:00
parent: SMET-I-0071
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0071
---

# Session Planning Context Panel and Integration Tests

## Parent Initiative

[[SMET-I-0071]] Planning Data Views in Control Dashboard

## Objective

Add a "Planning Context" collapsible panel to the existing Session Detail page showing the linked work item (task/story) and its full hierarchy chain. Also add integration tests validating the complete planning data flow from control-api through the dashboard. This task integrates planning data with the existing session management UI.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

### Session Planning Context Panel
- [ ] SessionDetailPage shows a collapsible "Planning Context" section
- [ ] Panel displays the linked work item (if any) with title, short code, phase badge
- [ ] Shows ancestry chain: linked task -> parent initiative -> parent strategy -> vision
- [ ] Each item in the chain is clickable, navigating to `/planning/documents/:shortCode`
- [ ] Panel collapses by default, remembers state in localStorage
- [ ] Graceful handling when session has no linked work item ("No work item linked")
- [ ] Sessions API extended to include optional `work_item_id` field

### Integration Tests
- [ ] Control-api integration test: create temp .metis project, verify all `/api/planning/*` endpoints return correct data
- [ ] Dashboard build passes with all new planning pages and components
- [ ] No TypeScript errors in the full dashboard codebase
- [ ] All existing control-api tests still pass (no regressions)

## Implementation Notes

### Technical Approach
- Read the existing `SessionDetailPage.tsx` and add a new section after the existing session info
- Create a `PlanningContextPanel` component that takes a work item short code and fetches its hierarchy
- The panel reuses `ShortCodeLink`, `DocumentTypeBadge`, `PhaseBadge` from the shared planning components
- For the API: sessions may need a `work_item_id` column in the DB schema (or this can be fetched from the cadre store if sessions link to tasks by convention)
- Integration tests: follow existing test patterns in `apps/control-api/tests/`

### Dependencies
- All previous tasks (T-0240 through T-0244) — this is the integration/finalization task
- SMET-I-0040 (Remote Session Lifecycle) — completed, provides session detail page

### Files to Create/Modify
- `src/pages/SessionDetailPage.tsx` — add Planning Context section
- `src/components/planning/PlanningContextPanel.tsx` — new component
- `apps/control-api/tests/planning_integration.rs` — new test file
- `apps/control-api/src/db.rs` — optional: add work_item_id to sessions table

## Status Updates

### 2026-03-28: Implementation Complete
- Created `PlanningContextPanel.tsx` — collapsible panel showing work item hierarchy chain
- Panel walks parent chain from work_item_id up to root (Vision/Strategy)
- Each ancestor shows DocumentTypeBadge, ShortCodeLink, title, PhaseBadge
- Current work item highlighted with primary-50 background
- Collapse state persisted to localStorage
- Integrated into SessionDetailPage.tsx — shows when session has work_item_id
- All 10 planning integration tests pass, 29 existing tests pass (39 total, zero regressions)
- Dashboard builds cleanly with tsc + vite (994 modules, 0 errors)
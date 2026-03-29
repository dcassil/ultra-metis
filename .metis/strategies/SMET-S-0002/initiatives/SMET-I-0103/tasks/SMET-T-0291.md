---
id: list-pages-card-integration-per
level: task
title: "List Pages Card Integration: Per-Page Card Field Priority Configuration"
short_code: "SMET-T-0291"
created_at: 2026-03-29T02:21:09.977815+00:00
updated_at: 2026-03-29T02:29:58.287386+00:00
parent: SMET-I-0103
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0103
---

# List Pages Card Integration: Per-Page Card Field Priority Configuration

## Parent Initiative

[[SMET-I-0103]] — Mobile-Responsive Dashboard

## Objective

Wire up each list page (Machines, Sessions, History, Documents, Notifications) to use the Table component's card mode by providing per-page `mobileCardConfig` defining which columns map to card header, body, and hidden fields.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] MachinesPage: cards show name + status dot as header, trust tier + session mode + last seen as body, OS/architecture hidden
- [ ] SessionsPage: cards show title + state badge as header, machine name + started at + duration as body, session ID hidden
- [ ] HistoryPage: cards show title + outcome badge as header, machine + completed at + duration as body, session ID hidden
- [ ] DocumentsPage: cards show title + type badge as header, phase + parent + updated at as body, short code as subtitle
- [ ] NotificationsPage: cards show message + type badge as header, created at + related session as body, notification ID hidden
- [ ] All cards tappable with existing row click navigation
- [ ] Desktop table views unchanged on all pages

## Implementation Notes

### Technical Approach

For each list page, add a `mobileCardConfig` prop to the `<Table>` invocation. The config maps column keys to card sections. Each page may need minor adjustments to how column data is rendered (e.g., extracting badge components for the card header).

**Per-page changes:**
- `src/pages/MachinesPage.tsx` — add mobileCardConfig, may need to extract status dot rendering
- `src/pages/SessionsPage.tsx` — add mobileCardConfig
- `src/pages/HistoryPage.tsx` — add mobileCardConfig
- `src/pages/planning/DocumentsPage.tsx` — add mobileCardConfig
- `src/pages/NotificationsPage.tsx` — add mobileCardConfig

### Dependencies
- SMET-T-0288 (Table card mode) must be completed first.

## Status Updates

*To be added during implementation*
---
id: dashboard-ui-review-usability-data
level: initiative
title: "Dashboard UI Review: Usability, Data Integrity, and Information Architecture"
short_code: "SMET-I-0101"
created_at: 2026-03-28T23:22:09.761751+00:00
updated_at: 2026-03-28T23:22:09.761751+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: SMET-S-0002
initiative_id: dashboard-ui-review-usability-data
---

# Dashboard UI Review: Usability, Data Integrity, and Information Architecture Initiative

## Context

The control dashboard has been built incrementally across multiple initiatives (SMET-I-0095, I-0039 through I-0044, I-0071, I-0098, I-0099). While functional at a feature level, an interactive hands-on review has revealed significant usability gaps, data integrity issues, and unclear information architecture. The dashboard needs a holistic pass to ensure everything functions as intended, orphaned/confusing elements are removed, and the UI is intuitive for day-to-day use.

## Goals & Non-Goals

**Goals:**
- Fix all broken interactions and non-functional UI elements
- Establish clear machine lifecycle with status visibility (connected, active, disconnected, idle)
- Resolve orphaned machine data from runner restarts generating new IDs
- Make sessions children of machines with proper navigation between them
- Rethink information architecture for projects, architecture docs, rules, and quality scores
- Simplify the UI so every element has a clear, intuitive purpose and path
- Remove or repurpose the confusing "policies" tab

**Non-Goals:**
- Adding net-new features not related to fixing current UX issues
- Backend protocol changes (e.g., WebSocket migration — that's SMET-A-0002)
- Mobile-specific redesign (covered by SMET-T-0262)

## Known Issues

### 1. Machine Clickability
Machines are listed in the Machines tab but individual machines are not clickable. There is no machine detail view to see status, sessions, logs, or configuration for a specific machine.

### 2. Policies Tab Confusion
The Policies tab appears to be similar to the Machines tab and its purpose is unclear. Terminal log info seems to live here but it's hard to find and the relationship to policies is not intuitive.

### 3. No Machine Removal
There is no way to remove machines from the dashboard. Combined with issue #5, this leads to an ever-growing list of stale entries.

### 4. Machine Status Unclear
There is no clear indication of whether a machine is currently connected, active, disconnected, idle, etc. Users cannot tell at a glance which machines are alive.

### 5. Orphaned Machines on Restart
Every time the machine runner restarts, it generates a new machine ID. Old machine listings become orphaned with no way to clean them up or associate them with the restarted runner.

### 6. AI Agent Dialog Lost on Reload
The active dialog from the AI agent disappears if you reload the page and does not show in history. Live session state is not persisted or recoverable.

### 7. Terminal Logs Hard to Find
Terminal/log information is buried in the Policies section rather than being easily accessible from the machine or session context where users would naturally look for it.

### 8. Sessions Not Linked to Machines
Sessions should be children of machines. While a global sessions tab with filtering is useful, clicking on a machine should also show the list of sessions for that machine in its detail view.

### 9. Project Information Architecture
Need to think through how to structure projects, architecture docs, rules, quality scores, etc. Projects should probably be parent to architecture, rules, and quality for that project. Product docs, epics, etc. could live under a project but there may be merit to them living in a global board (similar to how scrum does it). Could have them nested but still display epics, stories, and tasks in a global view with filters.

### 10. General UI Simplification
The UI needs simplification so everything has a clear, easy path that is intuitive. Remove unnecessary complexity and ensure navigation patterns are consistent.

## Detailed Design

This initiative will be executed as an interactive review — walking through the dashboard hands-on, identifying each issue in context, and fixing them iteratively. Design decisions will be made during the design phase with human review.

### Approach
1. **Audit**: Walk through every tab and interaction in the dashboard, catalog what works and what doesn't
2. **Design**: Propose navigation restructure, machine detail view, session-machine relationship, and project IA
3. **Implement**: Fix issues in priority order — broken interactions first, then IA improvements, then polish

### Key Design Questions (for Design Phase)
- What should the machine detail view look like? (status, sessions, logs, config)
- How do we handle machine identity persistence across restarts? (stable ID? name-based matching?)
- What replaces the Policies tab? Where do policies, logs, and enforcement info live?
- How should the project hierarchy render? Nested with global views + filters, or flat with grouping?
- Should we persist live session state for reload recovery, or just ensure history captures it?

## UI/UX Design

### Key User Flows to Fix
1. **Machine Management**: List → Click machine → See detail (status, sessions, logs) → Remove machine
2. **Session Monitoring**: From machine detail OR from global sessions list → Live output → History
3. **Log Access**: Logs accessible from machine detail view and session detail view, not buried in policies
4. **Project Navigation**: Clear hierarchy (Project → Architecture / Rules / Quality) with global board views

### Navigation Restructure (Proposal — TBD in Design Phase)
- **Machines** tab: List with status indicators → clickable to detail view
- **Sessions** tab: Global list with machine filter → clickable to session detail/replay
- **Projects** tab: Hierarchy view with nested architecture, rules, quality
- **Policies** tab: Repurpose or remove — enforcement config could live in settings or under projects
- **Logs** tab or integrated into machine/session detail views

## Implementation Plan

1. **Discovery**: Interactive walkthrough to validate and expand the issue list above
2. **Design**: Propose concrete wireframes/layouts for machine detail view, navigation restructure, and project IA. Human review required.
3. **Decompose**: Break into tasks covering each fix area
4. **Execute**: Implement fixes iteratively, testing each interaction as we go
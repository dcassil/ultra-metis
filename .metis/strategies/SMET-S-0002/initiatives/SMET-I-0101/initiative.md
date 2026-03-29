---
id: dashboard-ui-review-usability-data
level: initiative
title: "Dashboard UI Review: Usability, Data Integrity, and Information Architecture"
short_code: "SMET-I-0101"
created_at: 2026-03-28T23:22:09.761751+00:00
updated_at: 2026-03-29T01:15:55.903698+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


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

## Issue Analysis and Design Decisions

### Issue 1: Machine Clickability

**Root Cause**: `MachinesPage.tsx` renders the Table component without passing `onRowClick`. The route `/machines/:id` and `MachineDetailPage` both exist and work correctly — they just can't be reached from the list.

**Current State**: Table component supports `onRowClick` prop (used correctly in PoliciesPage and SessionsPage). This is a one-line fix.

**Decision**: Add `onRowClick` to the machines Table to navigate to `/machines/{id}`. Add visual affordance (chevron or hover state) so rows look clickable.

**Files to change**: `src/pages/MachinesPage.tsx` — add `onRowClick={(row) => navigate('/machines/' + row.id)}` and import `useNavigate`.

---

### Issue 2: Policies Tab — Remove and Redistribute

**Root Cause**: `PoliciesPage.tsx` is a near-duplicate of `MachinesPage`. It lists machines with their policy columns (Session Mode, Max Autonomy), and clicking a row navigates to `/machines/:id` — the same destination as the machines list. It adds no unique functionality.

**Current Sidebar Structure** (11 items across 3 groups):
```
Main:        Machines, Sessions
Planning:    Documents, Hierarchy, Quality, Rules
Operations:  Monitoring, History, Notifications, Policies, Violations
```

**Decision**: Remove the Policies page entirely. Policy information is already accessible from the Machine Detail page (which has a full PolicyEditor). The "Session Mode" column is already shown on the Machines list page. Move "Violations" under the machine detail as a tab (alongside Details and Logs) rather than as a standalone top-level page.

**What moves where**:
- Policy editing: Already on Machine Detail page (stays)
- Session Mode display: Already on Machines list (stays)
- Violations: Becomes a tab on Machine Detail page instead of standalone page; also accessible from Session Detail (already there)

**Files to change**:
- Delete `src/pages/PoliciesPage.tsx`
- Remove from `App.tsx` routes, `Sidebar.tsx` navigation
- Move violations table into `MachineDetailPage.tsx` as new "Violations" tab
- Keep global violations endpoint for potential future use but remove top-level nav

---

### Issue 3: Machine Removal

**Root Cause**: The `revokeMachine()` API and "Revoke" button exist on `MachineDetailPage`, but since machines weren't clickable from the list (Issue 1), users couldn't reach them. Additionally, "revoke" is a soft-delete (status change to 'revoked') — revoked machines still appear in the list.

**Decision**: Two changes:
1. **Fix reachability**: Once Issue 1 is fixed, the existing Revoke button on Machine Detail becomes accessible.
2. **Add DELETE endpoint and "Remove" action**: Add a true deletion endpoint `DELETE /api/machines/{machine_id}` that removes the machine record from the database (only if not actively running sessions). Add a "Remove" button on the machine list (via row action or bulk select) and on the machine detail page. "Revoke" disconnects a machine; "Remove" deletes stale records.
3. **Bulk cleanup**: Add a "Remove all offline machines" action on the Machines page header for cleaning up orphaned entries from Issue 5.

**Files to change**:
- `apps/control-api/src/routes.rs` — add `DELETE /api/machines/{id}` endpoint
- `apps/control-api/src/db.rs` — add `delete_machine()` function
- `apps/control-dashboard/src/api/machines.ts` — add `deleteMachine()` 
- `src/pages/MachinesPage.tsx` — add "Remove offline" bulk action button
- `src/pages/MachineDetailPage.tsx` — add "Remove" button alongside Revoke

---

### Issue 4: Machine Status Unclear

**Root Cause**: The `connectivity_status` field exists on machines (online/stale/offline/unknown) and is displayed via `StatusBadge`, but the visual treatment is weak — small text badges that don't convey urgency. The list doesn't sort by status or highlight online machines. There's no relative time showing "last seen X ago".

**Decision**:
1. **Enhance status display on machine list**: Add a colored dot indicator (green=online, yellow=stale, gray=offline) before the machine name. Add "Last Seen" column showing relative time from `last_heartbeat`.
2. **Sort defaults**: Default sort machines by status (online first, then stale, then offline) rather than arbitrary order.
3. **Machine detail**: Already shows `Last Heartbeat` with `RelativeTime` — this is good. Add a prominent status banner at the top of the detail page (e.g., green bar for online, red warning for offline).
4. **Auto-stale detection**: The API should mark machines as "stale" if `last_heartbeat` is older than 2x heartbeat interval (configurable). Currently the runner sends heartbeats every 30s — so stale after ~90s of silence.

**Files to change**:
- `src/pages/MachinesPage.tsx` — add status dot, last seen column, default sort
- `src/pages/MachineDetailPage.tsx` — add status banner
- Potentially `apps/control-api/src/routes.rs` — derive stale status on query if not already

---

### Issue 5: Orphaned Machines on Restart

**Root Cause**: `runner.rs:187-204` calls `client.register()` on every startup, and `routes.rs:50` generates a new `uuid::Uuid::new_v4()` for each registration. The machine_id is stored in `self.machine_id` (in-memory only) — never persisted to disk. So every runner restart = new registration = new UUID = orphaned old record.

**Decision**: **Persist machine_id locally and re-register with it.**
1. After first registration, save the assigned machine_id to a local file (e.g., `~/.cadre/machine_id` or in the Tauri app data directory).
2. On subsequent startups, read the persisted ID and send it in the registration request.
3. Server-side: If the registration request includes an existing machine_id AND the name matches, update the existing record instead of creating a new one. Reset `connectivity_status` to online and update `last_heartbeat`.
4. If the persisted ID doesn't match any server record (e.g., DB was wiped), fall through to new registration.
5. **Cleanup for existing orphans**: The "Remove all offline machines" action from Issue 3 handles cleaning up existing orphans.

**Files to change**:
- `apps/machine-runner/src/runner.rs` — persist machine_id to file after registration, load on startup
- `apps/machine-runner/src/client.rs` — add optional `machine_id` field to `RegisterRequest`
- `apps/control-api/src/routes.rs` — modify `register_machine` to handle re-registration with existing ID
- Tauri app: Use app data directory for persistence

---

### Issue 6: AI Agent Dialog Lost on Reload

**Root Cause**: Session output events ARE persisted to the `session_output_events` database table (with sequence numbers for ordering). However, the `LiveOutput` component only consumes events from the `useSessionEventStream` SSE hook, which starts fresh on each page load. The `getSessionEvents()` API client function exists in `src/api/events.ts` but is never called by the session detail page.

**Decision**: **Hydrate from history, then append live events.**
1. When `SessionDetailPage` mounts, fetch historical events from `GET /api/sessions/{id}/events` (paginated, ordered by sequence_num).
2. Display historical events immediately in LiveOutput.
3. Connect SSE stream and append only events with `sequence_num` greater than the last historical event.
4. This gives seamless continuity — reload shows full history plus live tail.
5. For terminal sessions (completed/failed/stopped), skip SSE entirely and just show history.

**Files to change**:
- `src/hooks/useSessionEventStream.ts` — accept optional `initialEvents` and `startSequence` params
- `src/pages/SessionDetailPage.tsx` — fetch historical events on mount, pass to stream hook
- `src/api/events.ts` — may need pagination params if not already supported

---

### Issue 7: Terminal Logs Hard to Find

**Root Cause**: Machine logs are accessible in the Machine Detail page under the "Logs" tab (via `MachineLogViewer`). But since machines weren't clickable (Issue 1) and the only other path to logs was through the Policies page (which just redirected to machine detail), logs were effectively hidden.

**Decision**: Once Issue 1 (clickable machines) and Issue 2 (remove Policies page) are resolved, logs become naturally accessible via: Machines list → click machine → Logs tab. Additionally:
1. **Add "Logs" as a direct link** on the machine row (small icon/button) so users can jump straight to logs without going through the detail page.
2. **Session-level logs**: Add a "Logs" tab to `SessionDetailPage` that shows machine logs filtered to the session's time range. Currently session detail has LiveOutput (agent events) but not machine-level debug/system logs.
3. **No standalone Logs page needed** — logs are always in the context of a machine or session.

**Files to change**:
- `src/pages/MachinesPage.tsx` — add quick-link log icon on each row
- `src/pages/SessionDetailPage.tsx` — add "Logs" tab using MachineLogViewer filtered by session timeframe

---

### Issue 8: Sessions as Children of Machines

**Root Cause**: `MachineDetail` type has an `active_sessions: unknown[]` field but it's never rendered. Sessions reference machines by `machine_id` (shown as a raw string in the session list), but there's no navigation from machine → sessions. The relationship is one-way.

**Decision**: **Add Sessions tab to Machine Detail, make machine_id clickable in sessions list.**
1. Add a "Sessions" tab to `MachineDetailPage` (alongside Details, Logs, and Violations) that lists all sessions for that machine (active + historical).
2. Type the `active_sessions` field properly and fetch sessions filtered by machine_id.
3. In the global Sessions list, make the "Machine" column a clickable link that navigates to `/machines/{machine_id}`.
4. In the Machine Detail sessions tab, allow filtering by state (active/completed/failed).
5. **Tab order on Machine Detail**: Details | Sessions | Logs | Violations — sessions should be prominent.

**Files to change**:
- `src/pages/MachineDetailPage.tsx` — add "Sessions" tab, fetch sessions filtered by machine_id
- `src/pages/SessionsPage.tsx` — make machine_id column a clickable Link to `/machines/{id}`
- `src/api/machines.ts` — properly type `active_sessions` or fetch from sessions API with machine filter
- `src/api/sessions.ts` — ensure `listSessions({machine_id})` filter works

---

### Issue 9: Project Information Architecture

**Root Cause**: The Planning section currently has four flat pages (Documents, Hierarchy, Quality, Rules) with no concept of "Project" as an organizing entity. All documents are listed globally. The hierarchy view shows the full vision → strategy → initiative → task tree but doesn't group by project. Architecture docs, rules, and quality records exist as separate concerns with no project-level container.

**Current Navigation**:
```
Planning:
  Documents  — flat list with type/phase filters
  Hierarchy  — expandable tree (vision → strategy → initiative → task)
  Quality    — quality gate results per document
  Rules      — architecture rules by scope
```

**Decision**: **Introduce a Project concept as an organizing container, but keep global views.**

This is the most complex item and needs a phased approach:

**Phase 1 (this initiative)**: Reorganize the existing Planning UI without backend changes.
- Rename "Planning" section to "Work" in the sidebar
- Keep the global Document list and Hierarchy views
- Merge Quality and Rules into a single "Governance" page with tabs (Quality Gates | Rules)
- This reduces 4 items to 3: Documents, Hierarchy, Governance

**Phase 2 (future initiative)**: Introduce Project as a first-class entity.
- A Project groups: Architecture docs, Rules, Quality baselines, and optionally Work items
- The Hierarchy view becomes project-scoped by default with a global toggle
- Work items (initiatives/tasks) can be viewed globally (scrum board style) or nested under a project
- This matches the mental model: "Project X has architecture Y, rules Z, and these work items"

**Rationale for phased approach**: Adding a Project entity requires backend domain model changes (new document type or container concept), which is beyond a UI review initiative. Phase 1 simplifies what we have; Phase 2 is a separate initiative.

**Files to change (Phase 1)**:
- `src/components/Sidebar.tsx` — rename "Planning" to "Work", merge Quality+Rules into "Governance"
- Create `src/pages/planning/GovernancePage.tsx` — tabs for Quality Gates and Rules
- Remove standalone `QualityPage.tsx` and `RulesPage.tsx`
- Update `App.tsx` routes

---

### Issue 10: General UI Simplification

**Root Cause**: The sidebar has 11 navigation items across 3 groups (Main, Planning, Operations). Several items overlap or are rarely used (Monitoring is a placeholder, Policies duplicates Machines, Violations could be under Machines).

**Current Sidebar** (11 items):
```
[Start Session]
Main:        Machines, Sessions
Planning:    Documents, Hierarchy, Quality, Rules
Operations:  Monitoring, History, Notifications, Policies, Violations
```

**Decision**: Consolidate to 7 items across 2 groups:

**Proposed Sidebar** (7 items):
```
[Start Session]
Main:        Machines, Sessions, History
Work:        Documents, Hierarchy, Governance
Settings:    Notifications
```

**What changes**:
- **Remove Policies** (Issue 2): Content lives on Machine Detail
- **Remove Violations** (Issue 2): Moves to Machine Detail tab
- **Remove Monitoring** (placeholder): Re-add when actually implemented
- **Merge Quality + Rules → Governance** (Issue 9)
- **Rename "Planning" → "Work"**: More intuitive for the content it contains
- **Move History to Main group**: Session history is a primary workflow, not an "operations" concern
- **Notifications stays** but moves to a simpler "Settings" or becomes just a bell icon in the header (common pattern)
- **Default landing page**: Change from `/machines` to `/sessions` — sessions are the primary workflow

**Sidebar with counts/badges**:
- Machines: pending approval count (already exists)
- Sessions: active session count
- Notifications: unread count (move to header bell icon instead of sidebar item)

**Files to change**:
- `src/components/Sidebar.tsx` — restructure navigation groups
- `src/App.tsx` — update default redirect, remove deleted routes
- `src/components/Header.tsx` — add notification bell icon with unread badge
- Delete pages: `PoliciesPage.tsx`, `ViolationsPage.tsx`, `MonitoringPage.tsx`
- Create: `GovernancePage.tsx`

## Detailed Design

All 10 issues have been analyzed with root causes identified in the codebase. Design decisions are captured above per-issue.

### Design Questions — Resolved

| Question | Decision |
|----------|----------|
| Machine detail view structure? | 4 tabs: Details, Sessions, Logs, Violations |
| Machine identity across restarts? | Persist machine_id to local file, re-register with same ID |
| What replaces Policies tab? | Nothing — policy editing already on Machine Detail, violations become a Machine Detail tab |
| Project hierarchy structure? | Phase 1: Merge Quality+Rules into Governance, rename Planning→Work. Phase 2 (future): Introduce Project entity |
| Session reload recovery? | Hydrate historical events from DB on mount, then append live SSE events |

### Scope Summary

**Frontend changes** (control-dashboard):
- 3 pages deleted: PoliciesPage, ViolationsPage, MonitoringPage
- 1 page created: GovernancePage (tabs: Quality Gates | Rules)
- Major edits: MachinesPage, MachineDetailPage, SessionsPage, SessionDetailPage, Sidebar, Header, App.tsx
- Hooks: useSessionEventStream (add historical hydration)

**Backend changes** (control-api):
- New endpoint: `DELETE /api/machines/{id}`
- Modified endpoint: `POST /api/machines/register` (support re-registration with existing ID)
- DB: add `delete_machine()` function

**Machine Runner changes**:
- Persist machine_id to disk after first registration
- Load persisted ID on startup, send in registration request
- RegisterRequest: add optional `machine_id` field

### Dependency Graph

```
Issue 1 (clickable machines) ← Issue 3 (removal), Issue 7 (logs access)
Issue 2 (remove policies)   ← Issue 10 (simplification)
Issue 5 (orphan fix)        ← Issue 3 (bulk cleanup of existing orphans)
Issue 9 (project IA)        ← Issue 10 (sidebar restructure)
Issue 6 (dialog persistence) — independent
Issue 4 (status display)     — independent
Issue 8 (sessions on machines) — independent
```

### Suggested Task Execution Order

1. **Issue 1**: Machine clickability (1-line fix, unblocks others)
2. **Issue 4**: Machine status display enhancement
3. **Issue 2 + 10**: Remove Policies/Violations/Monitoring pages, restructure sidebar (do together)
4. **Issue 8**: Sessions tab on Machine Detail, clickable machine links in Sessions
5. **Issue 3**: Machine removal (DELETE endpoint + UI)
6. **Issue 5**: Persist machine_id to prevent orphans (backend + runner)
7. **Issue 6**: Historical event hydration for session reload
8. **Issue 7**: Log access improvements (quick-link, session logs tab)
9. **Issue 9**: Merge Quality+Rules into Governance page

## UI/UX Design

### Resolved Navigation Structure

**Before (11 items, 3 groups)**:
```
[Start Session]
Machines        (pending badge)
Sessions
── Planning ──
Documents
Hierarchy
Quality
Rules
── Operations ──
Monitoring      (placeholder)
History
Notifications
Policies        (duplicate)
Violations
```

**After (7 items, 2 groups + header icon)**:
```
[Start Session]
Machines        (pending badge)
Sessions        (active count badge)
History
── Work ──
Documents
Hierarchy
Governance
[Bell icon in header] (notification count)
```

### Machine Detail Page — Final Tab Layout

```
[Machine Name]  [StatusBadge]  [SessionModeBadge]    [Revoke] [Remove]

Details | Sessions | Logs | Violations

Details tab:   Machine metadata, repos, policy editor, repo-level overrides
Sessions tab:  All sessions for this machine (active + historical), state filter
Logs tab:      MachineLogViewer (live + history modes, level filter)
Violations tab: Policy violation records for this machine (replaces standalone page)
```

### Session Detail Page — Enhanced Tabs

```
[Session Title]  [StateBadge]    [Stop] [Pause] [Resume] [Force Stop]

Overview | Live Output | Timeline | Logs

Overview:    Session metadata, planning context, instructions, violations
Live Output: Historical events hydrated from DB + live SSE append (fixes reload)
Timeline:    Significant events (state changes, approvals, guidance, violations)
Logs:        Machine-level debug logs filtered to session time range (new)
```

## Implementation Plan

This initiative is ready for decomposition into tasks. Suggested grouping:

**Task Group A — Quick Wins (frontend-only)**:
- Issue 1: Make machine rows clickable
- Issue 4: Status dots + last seen column on machine list

**Task Group B — Sidebar & Navigation Restructure**:
- Issues 2 + 10: Remove pages, restructure sidebar, create Governance page, move notifications to header

**Task Group C — Machine Detail Enrichment**:
- Issue 8: Add Sessions tab to Machine Detail
- Issue 7: Add Logs quick-link, session-level logs tab
- Issue 2 (partial): Move Violations into Machine Detail tab

**Task Group D — Backend Fixes**:
- Issue 3: DELETE endpoint + removal UI
- Issue 5: Persist machine_id in runner + re-registration support

**Task Group E — Session Improvements**:
- Issue 6: Historical event hydration + SSE continuation

**Task Group F — Information Architecture**:
- Issue 9 Phase 1: Create GovernancePage (Quality + Rules tabs)

**Future Initiative** (out of scope):
- Issue 9 Phase 2: Project entity as first-class domain type

## Decomposition Summary

8 tasks created, covering all 10 issues:

| Task | Title | Issues Covered | Scope |
|------|-------|---------------|-------|
| SMET-T-0277 | Machine List Row Clickability and Status Display Enhancement | 1, 4 | Frontend |
| SMET-T-0278 | Sidebar and Navigation Restructure | 2, 10 | Frontend |
| SMET-T-0279 | Machine Detail Enrichment: Sessions Tab, Violations Tab, Cross-Navigation | 8, 2 (partial) | Frontend |
| SMET-T-0280 | Machine Deletion API and Bulk Cleanup UI | 3 | Full-stack |
| SMET-T-0281 | Persistent Machine Identity: Local ID Storage and Re-Registration | 5 | Backend + Runner |
| SMET-T-0282 | Session Event History Hydration | 6 | Frontend |
| SMET-T-0283 | Log Access Improvements: Quick-Link and Session Logs Tab | 7 | Frontend |
| SMET-T-0284 | Governance Page: Merge Quality and Rules | 9 (Phase 1) | Frontend |

**Execution order**: T-0277 → T-0278 → T-0279 → T-0280 → T-0281 → T-0282 → T-0283 → T-0284
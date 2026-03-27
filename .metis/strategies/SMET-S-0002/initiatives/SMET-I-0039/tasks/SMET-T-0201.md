---
id: control-dashboard-machine-list-and
level: task
title: "Control Dashboard Machine List and Detail Views"
short_code: "SMET-T-0201"
created_at: 2026-03-27T16:18:43.376196+00:00
updated_at: 2026-03-27T20:31:34.873757+00:00
parent: SMET-I-0039
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0039
---

# Control Dashboard Machine List and Detail Views

## Parent Initiative

[[SMET-I-0039]] — Machine Connectivity and Trust

## Objective

Build the machine list page and machine detail page in the Control Dashboard (`apps/control-dashboard/`). These are the primary UI surfaces for users to see their registered machines, their connectivity status, available repos, and trust levels. The views consume data from the `GET /machines` and `GET /machines/{id}` API endpoints.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Machine list page at `/machines` route in the dashboard
- [ ] Machine list displays a table/card layout with columns: machine name, platform, status badge (online/stale/offline with color coding: green/yellow/red), trust tier badge (trusted/restricted/pending), active session count, last heartbeat (relative time, e.g., "12s ago")
- [ ] Status badges auto-refresh on a polling interval (every 10 seconds) so the user sees near-real-time connectivity changes
- [ ] Machine list is sorted by status (online first, then stale, then offline) by default, with clickable column headers for re-sorting
- [ ] Clicking a machine name navigates to the machine detail page at `/machines/{id}`
- [ ] Machine detail page shows: full machine metadata (name, platform, OS, arch), trust tier with visual indicator, connectivity status with last heartbeat timestamp, list of available repos (name + path + Cadre-managed badge), and a revoke button
- [ ] Machine detail page shows an expandable "Metadata" section for any additional JSON metadata the machine reported
- [ ] Empty state: when no machines are registered, show a helpful message explaining how to install and configure the Machine Runner
- [ ] Responsive layout: machine list is usable on both desktop and tablet screen sizes
- [ ] Loading and error states: show a spinner while fetching, show an error message with retry button if the API call fails

## Implementation Notes

### Technical Approach
- Build as pages/components in the dashboard's frontend framework (React/Next.js or whatever the monorepo establishes)
- Machine list fetches from `GET /machines` on mount and polls every 10 seconds using `setInterval` or a data-fetching library with refetch interval
- Machine detail fetches from `GET /machines/{id}` on mount
- Status badge component: takes a connectivity status string and renders a colored dot + label (green "Online", yellow "Stale", red "Offline")
- Trust tier badge component: "Trusted" (blue), "Restricted" (orange), "Pending" (gray)
- Last heartbeat is displayed as relative time using a library like `date-fns` `formatDistanceToNow`
- The revoke button on the detail page calls `POST /machines/{id}/revoke` and redirects back to the machine list on success

### UI Components to Create
1. `MachineListPage` — fetches and renders the machine table
2. `MachineDetailPage` — fetches and renders a single machine's full info
3. `StatusBadge` — reusable colored badge for online/stale/offline
4. `TrustTierBadge` — reusable badge for trusted/restricted/pending
5. `RepoList` — renders the list of repos on a machine detail page
6. `EmptyState` — shown when no machines exist

### Dependencies
- SMET-T-0198 (Control Service Machine Registry API) must be complete — the dashboard consumes the list and detail endpoints
- Dashboard app scaffolding from SMET-I-0095 (Monorepo Restructure) must exist

### Risk Considerations
- Polling every 10 seconds creates load on the API — acceptable for MVP with a single user, but should be replaced with WebSocket/SSE push in the future
- Ensure the revoke button has a confirmation dialog to prevent accidental revocations

## Status Updates

*To be added during implementation*
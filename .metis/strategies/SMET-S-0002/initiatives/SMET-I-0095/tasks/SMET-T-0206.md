---
id: application-shell-client-side
level: task
title: "Application Shell & Client-Side Routing"
short_code: "SMET-T-0206"
created_at: 2026-03-27T16:28:39.295062+00:00
updated_at: 2026-03-27T19:09:24.303309+00:00
parent: SMET-I-0095
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0095
---

# Application Shell & Client-Side Routing

## Parent Initiative

[[SMET-I-0095]] — Control Dashboard Foundation

## Objective

Build the top-level application shell (sidebar navigation, header bar, main content area) and configure React Router with placeholder pages for each feature area. After this task, the dashboard has a fully navigable layout with responsive behavior — collapsible sidebar on mobile, active route highlighting, and placeholder content for each route that subsequent initiatives will replace with real views.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] React Router v6 is installed and configured with `BrowserRouter`
- [ ] Application shell layout renders with three regions: fixed sidebar, top header bar, and scrollable main content area
- [ ] Sidebar navigation includes links for all five feature areas: Machines (`/machines`), Sessions (`/sessions`), Monitoring (`/monitoring`), History (`/history`), Policies (`/policies`)
- [ ] Root route (`/`) redirects to `/machines` (default landing page)
- [ ] Each route renders a placeholder page component with the feature name and "Coming soon" message (e.g., "Machine Management — Coming soon")
- [ ] Active route is visually highlighted in the sidebar navigation
- [ ] Sidebar is collapsible on mobile viewports (hamburger menu toggle in header)
- [ ] On mobile, sidebar overlays content when open and closes on route navigation
- [ ] On desktop (>= 1024px), sidebar is always visible as a fixed panel
- [ ] Header bar displays the application name ("Cadre Control") and has a placeholder slot for the connection status indicator (to be wired in SMET-T-0210)
- [ ] Layout uses Tailwind CSS utility classes and is fully responsive
- [ ] 404 catch-all route shows a "Page not found" message with a link back to home

## Implementation Notes

### Technical Approach

1. Install React Router: `npm install react-router-dom`
2. Create the layout component hierarchy:
   - `src/layouts/DashboardLayout.tsx` — the shell with sidebar, header, and `<Outlet />` for page content
   - `src/components/Sidebar.tsx` — navigation links, collapsible on mobile
   - `src/components/Header.tsx` — app title, mobile menu toggle, status indicator slot
3. Configure routes in `src/App.tsx`:
   ```
   /                -> Redirect to /machines
   /machines        -> MachinesPlaceholder
   /sessions        -> SessionsPlaceholder
   /monitoring      -> MonitoringPlaceholder
   /history         -> HistoryPlaceholder
   /policies        -> PoliciesPlaceholder
   *                -> NotFound
   ```
4. Create placeholder page components in `src/pages/`:
   - Each renders a centered card with an icon, feature name, and "Coming soon" description
   - Indicates which initiative will populate this view (e.g., "Will be built in SMET-I-0039")
5. Implement responsive sidebar:
   - Use React state for mobile open/close toggle
   - Tailwind classes: `hidden lg:flex` for desktop sidebar, overlay with backdrop for mobile
   - Close sidebar on route change (via `useLocation` effect)
6. Style the active nav link using React Router's `NavLink` component with `className` callback
7. Use Heroicons for navigation item icons (ServerIcon for Machines, PlayIcon for Sessions, ChartBarIcon for Monitoring, ClockIcon for History, ShieldCheckIcon for Policies)

### Dependencies

- SMET-T-0205 (Project Scaffold) must be complete — needs the Vite/React/Tailwind foundation

### File Structure Created

```
src/
  layouts/
    DashboardLayout.tsx
  components/
    Sidebar.tsx
    Header.tsx
  pages/
    MachinesPlaceholder.tsx
    SessionsPlaceholder.tsx
    MonitoringPlaceholder.tsx
    HistoryPlaceholder.tsx
    PoliciesPlaceholder.tsx
    NotFound.tsx
```

## Status Updates

*To be added during implementation*
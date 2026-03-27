---
id: control-dashboard-foundation
level: initiative
title: "Control Dashboard Foundation"
short_code: "SMET-I-0095"
created_at: 2026-03-27T16:02:18.460426+00:00
updated_at: 2026-03-27T19:13:50.293410+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0002
initiative_id: control-dashboard-foundation
---

# Control Dashboard Foundation Initiative

## Context

Every initiative in the Remote AI Operations Layer (SMET-S-0002) assumes it can add views to a "Control Dashboard" — but no initiative owns the dashboard application itself. SMET-I-0039 adds machine list/detail views, SMET-I-0040 adds session start/list views, SMET-I-0041 adds live monitoring views, SMET-I-0043 adds history/replay views, and SMET-I-0044 adds policy management views.

Without a foundation initiative, the first initiative to need a dashboard view (SMET-I-0039) would have to absorb the entire app scaffold — framework selection, project setup, routing, layout, navigation, auth middleware, deployment pipeline — making an already-L initiative even larger and mixing infrastructure concerns with feature work.

This initiative delivers the empty but functional dashboard shell that all subsequent initiatives build on.

**Pre-requisites**: SMET-I-0038 (Monorepo Restructure) — the dashboard lives in `apps/control-dashboard/`.

**Components touched**: Control Dashboard (new application), Control Service (CORS configuration, health endpoint for dashboard connectivity check).

## Goals & Non-Goals

**Goals:**
- Web application scaffold in `apps/control-dashboard/` with framework, bundler, and dev server
- Application shell: top-level layout with navigation sidebar/header, main content area, responsive design
- Client-side routing with placeholder pages for each known feature area (machines, sessions, history, policies)
- Auth middleware scaffold: request interceptor that attaches user context (MVP: hardcoded user, same pattern as Control Service)
- API client layer: typed HTTP client for Control Service with base URL configuration, error handling, and auth header injection
- Health/connectivity check: dashboard shows connection status to Control Service
- Development workflow: `npm run dev` with hot reload, `npm run build` for production
- Basic design system: color palette, typography, spacing, component primitives (buttons, badges, cards, tables, form elements)
- Responsive layout that works on desktop and mobile (mobile is a primary use case for remote monitoring)
- Deployment pipeline: build artifact, static hosting configuration

**Non-Goals:**
- Any feature-specific views (machine list, session start, monitoring, etc. — those belong to their respective initiatives)
- Real authentication (JWT, OAuth) — MVP uses hardcoded user context matching the Control Service
- Server-side rendering — static SPA is sufficient
- Complex state management — each feature initiative will add its own state as needed

## Detailed Design

### Application Scaffold
- Framework: React + TypeScript (widely understood, strong ecosystem for dashboards)
- Bundler: Vite (fast dev server, good TypeScript support)
- Styling: Tailwind CSS (utility-first, fast iteration, responsive by default)
- Component library: Headless UI or Radix for accessible primitives (modals, dropdowns, toggles)
- Project location: `apps/control-dashboard/`

### Application Shell
- Layout: fixed sidebar navigation (collapsible on mobile) + top header bar + scrollable main content area
- Navigation items (placeholder routes, no content yet):
  - Machines (`/machines`) — will be populated by SMET-I-0039
  - Sessions (`/sessions`) — will be populated by SMET-I-0040
  - Monitoring (`/monitoring`) — will be populated by SMET-I-0041
  - History (`/history`) — will be populated by SMET-I-0043
  - Policies (`/policies`) — will be populated by SMET-I-0044
- Each route renders a placeholder page ("Coming soon — Machine management") so navigation works immediately
- Active route highlighted in nav

### Auth Middleware (MVP)
- Axios/fetch interceptor that adds `Authorization: Bearer static-token` to all API requests
- Matches the Control Service's MVP middleware that maps static token → `user_id=1`
- When real auth lands, this interceptor swaps to JWT attachment — no other code changes needed
- User context available via React context: `useCurrentUser()` returns `{ id: 1, name: "Default User" }`

### API Client Layer
- Typed API client in `src/api/` with methods grouped by domain (machines, sessions, etc.)
- Base URL from environment variable (`VITE_API_BASE_URL`)
- Centralized error handling: network errors, 401s, 4xx/5xx with structured error display
- Request/response interceptors for auth and error handling

### Design System Foundation
- Color palette: primary, secondary, success, warning, danger, neutral scales
- Typography: heading and body scales, monospace for code/output
- Spacing: consistent scale (4px base)
- Component primitives: Button, Badge (with status variants: online/offline/pending/error), Card, Table, FormInput, Select, Toggle, Modal
- Status badge variants designed to work across all initiatives (machine status, session state, policy violations)
- Mobile-first responsive breakpoints

### Health Check
- Dashboard polls `GET /health` on the Control Service at startup and periodically
- Connection status indicator in the header: green (connected), yellow (degraded), red (disconnected)
- If disconnected, a banner explains the issue and prevents actions that would fail

## Alternatives Considered

- **Next.js with SSR**: More feature-rich but adds server complexity. The dashboard is a pure client-side app consuming a REST API — SSR adds deployment complexity without clear benefit. Rejected.
- **Vue or Svelte**: Viable but React has the largest ecosystem for dashboard component libraries and the broadest team familiarity. Rejected for pragmatic reasons.
- **No design system — just build views ad-hoc**: Leads to inconsistent UI across initiatives. A small upfront investment in primitives pays off across 5+ feature initiatives. Rejected.
- **Embed dashboard in Control Service (server-rendered)**: Couples frontend and backend deployment. Separate SPA allows independent iteration and deployment. Rejected.
- **Skip this initiative — let I-0039 bootstrap the dashboard**: Bloats I-0039 with framework decisions and infrastructure work unrelated to machine connectivity. Rejected.

## Implementation Plan

1. Initialize project: Vite + React + TypeScript in `apps/control-dashboard/`
2. Configure Tailwind CSS, set up design tokens (colors, typography, spacing)
3. Build application shell: sidebar nav, header, main content area, responsive layout
4. Set up client-side routing with placeholder pages for each feature area
5. Implement auth middleware (static token interceptor) and user context provider
6. Build API client layer with base URL config, error handling, typed methods
7. Build design system primitives: Button, Badge, Card, Table, FormInput, Select, Toggle, Modal
8. Implement health check indicator (poll Control Service /health)
9. Configure build pipeline and static hosting setup
10. Verify: dev server runs, all routes navigate, API client connects to Control Service, responsive layout works on mobile viewport
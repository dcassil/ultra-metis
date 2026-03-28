---
id: dashboard-planning-foundation
level: task
title: "Dashboard Planning Foundation: Navigation, API Client, and Shared Components"
short_code: "SMET-T-0241"
created_at: 2026-03-28T00:33:47.400627+00:00
updated_at: 2026-03-28T00:57:44.986301+00:00
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

# Dashboard Planning Foundation: Navigation, API Client, and Shared Components

## Parent Initiative

[[SMET-I-0071]] Planning Data Views in Control Dashboard

## Objective

Set up the dashboard frontend foundation for planning data views: add planning API client module, planning routes in the React Router config, a "Planning" section in the sidebar navigation, and shared components (DocumentTypeBadge, PhaseBadge, ShortCodeLink) used across all planning pages.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `src/api/planning.ts` created with typed functions: `listDocuments()`, `getDocument()`, `searchDocuments()`, `getHierarchy()`, `getRules()`, `getQualityRecords()`
- [ ] TypeScript interfaces for all planning API response types (PlanningDocument, HierarchyNode, Rule, QualityRecord)
- [ ] Planning routes added to App.tsx: `/planning/documents`, `/planning/documents/:shortCode`, `/planning/hierarchy`, `/planning/quality`, `/planning/rules`
- [ ] Sidebar updated with "Planning" section containing links: Documents, Hierarchy, Quality, Rules
- [ ] Shared components created: `DocumentTypeBadge` (color-coded by type), `PhaseBadge` (color-coded by phase), `ShortCodeLink` (clickable link to document detail)
- [ ] Placeholder pages created for each route (real content in subsequent tasks)
- [ ] `npm run build` passes with no TypeScript errors

## Implementation Notes

### Technical Approach
- Follow existing patterns from `src/api/machines.ts` and `src/api/sessions.ts` for the API client
- Follow existing badge patterns from `SessionStateBadge.tsx` and `TrustTierBadge.tsx` for new badges
- Add planning nav section to `Sidebar.tsx` between Sessions and Monitoring
- Use consistent color scheme: vision=purple, strategy=blue, initiative=teal, task=gray, adr=amber

### Dependencies
- SMET-T-0240 (Control API endpoints) should be complete or in progress — API client can be written against the expected API shape

### Files to Create
- `src/api/planning.ts` — API client
- `src/components/planning/DocumentTypeBadge.tsx`
- `src/components/planning/PhaseBadge.tsx`
- `src/components/planning/ShortCodeLink.tsx`
- `src/pages/planning/DocumentsPage.tsx` (placeholder)
- `src/pages/planning/DocumentDetailPage.tsx` (placeholder)
- `src/pages/planning/HierarchyPage.tsx` (placeholder)
- `src/pages/planning/QualityPage.tsx` (placeholder)
- `src/pages/planning/RulesPage.tsx` (placeholder)

### Files to Modify
- `src/App.tsx` — add planning routes
- `src/components/Sidebar.tsx` — add Planning nav section

## Status Updates

### 2026-03-28: Implementation Complete
- Created `src/api/planning.ts` with 6 typed API functions and all response interfaces
- Created `src/components/planning/DocumentTypeBadge.tsx` — color-coded by type (vision=purple, strategy=blue, initiative=teal, task=gray, adr=amber)
- Created `src/components/planning/PhaseBadge.tsx` — color-coded by phase (active=blue, completed=green, blocked=red, etc.)
- Created `src/components/planning/ShortCodeLink.tsx` — clickable monospace link to document detail
- Created 5 placeholder pages in `src/pages/planning/`
- Updated `App.tsx` with 5 planning routes
- Updated `Sidebar.tsx` with sectioned nav: main (Machines/Sessions), Planning (Documents/Hierarchy/Quality/Rules), Operations (Monitoring/History/Policies/Violations)
- Updated `src/api/index.ts` with planning type exports (also fixed pre-existing Session type export)
- `tsc -b` and full `vite build` pass cleanly
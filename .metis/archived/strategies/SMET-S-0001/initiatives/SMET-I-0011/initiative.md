---
id: upgrade-gui-for-stronger-model-and
level: initiative
title: "Upgrade GUI for Stronger Model and Traceability Views"
short_code: "SMET-I-0011"
created_at: 2026-03-11T19:59:59.632977+00:00
updated_at: 2026-03-11T19:59:59.632977+00:00
parent: SMET-S-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: cadre-core-engine-repo
initiative_id: upgrade-gui-for-stronger-model-and
---

# Upgrade GUI for Stronger Model and Traceability Views

> **STATUS: POST-MVP / DEFERRED** — The product spec states the GUI is "valuable but not MVP-critical unless needed for adoption." For MVP, prototype with existing rapid GUI tools where possible. Full GUI productization should come after the core repo-native model is proven.

## Context

Metis has GUI foundations for visualizing documents and workflows. Cadre introduces a richer document hierarchy, quality data, traceability chains, work leases, and governance artifacts — all of which benefit from visual representation. The GUI needs to evolve from a simple document viewer to an engineering dashboard that shows the full state of a product development effort.

## Governing Commitments

This initiative directly serves:
- **Planning is durable and traceable from product intent to execution.** The GUI makes traceability visual — hierarchy trees, ancestry views, and cross-reference graphs let users see the full path from product doc to task.
- **Quality includes architectural integrity and is tracked over time** (Vision #6). Quality dashboards surface baseline comparisons, trend lines across quality records, and gate pass/fail status — making the evolving quality signal visible rather than buried in individual files.
- **Controlled parallel execution** (Vision #8). The lease board visualizes who owns what, what's available, and what's in progress — making parallel work coordination visible at a glance.
- **All persisted project state lives in the repo** (Vision #1). The GUI reads from repo-native artifacts — it is a view into persisted state, not a separate data store. Every hierarchy tree, quality chart, and lease board reflects the same documents that CLI and MCP tools operate on.

## Goals & Non-Goals

**Goals:**
- Extend the GUI to display all new Cadre document types
- Add hierarchy visualization showing Product Doc → Epic → Story → Task trees
- Add traceability views that show document ancestry, descendants, and cross-references
- Add quality dashboard views showing baselines, trends, and gate status
- Add work lease visualization showing who owns what
- Add rule browser for viewing active engineering rules
- Support filtering and navigation across the expanded document set

**Non-Goals:**
- Building a full project management UI (Jira-like) — Cadre GUI is a dashboard, not a PM tool
- Real-time collaboration features — this is a local visualization tool
- Mobile support — desktop/browser only

## Detailed Design

### What to Reuse from `metis/`
- The existing GUI framework and component library
- Document rendering patterns (markdown to HTML)
- Navigation and routing patterns
- The existing data fetching layer

### What to Change from `metis/`
- Extend document list views to support new type filters and grouping
- Update document detail views to render new frontmatter fields
- Add hierarchy-aware navigation (breadcrumbs, tree views)
- Update phase visualization for new phase flows

### What is Net New
- Hierarchy tree view: interactive tree showing Product Doc → Epic → Story → Task
- Traceability view: graph visualization of document relationships and cross-references
- Quality dashboard: baseline comparison charts, trend lines, gate status indicators
- Lease board: Kanban-style view of work items with lease status
- Rule browser: searchable, filterable list of active engineering rules
- Investigation tracker: view of open investigations and their remediation status
- Design reference panel: view design context linked to the current document

## Alternatives Considered

1. **CLI-only, no GUI**: Rejected because visual representation of hierarchies, quality trends, and traceability is significantly better in a GUI.
2. **Web-based GUI with backend server**: This is likely the approach — extend the existing pattern.
3. **VS Code extension instead of standalone GUI**: Deferred — could be a future integration, but standalone GUI comes first.

## Implementation Plan

Phase 1: Extend document list and detail views for new types
Phase 2: Build hierarchy tree view
Phase 3: Build traceability graph view
Phase 4: Build quality dashboard
Phase 5: Build lease board view
Phase 6: Build rule browser
Phase 7: Build investigation tracker
Phase 8: Add design reference panel

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- All new document types render correctly in list and detail views
- Hierarchy tree view shows correct parent-child relationships
- Traceability view correctly shows ancestry, descendants, and cross-references
- Quality dashboard shows baseline comparisons and trend data
- Lease board accurately reflects current lease status
- Rule browser is searchable and filterable
- Navigation between related documents is intuitive (click to navigate)
- GUI loads and renders within 2 seconds for projects with up to 1000 documents

## Risks / Dependencies

- Depends on all domain model work (SMET-I-0001 through I-0007, I-0012)
- GUI work is lower priority than core model and CLI/MCP — can lag behind
- Risk of scope creep — keep views focused and simple
- Must understand the existing GUI framework before extending

## Codebase Areas to Inspect

- `metis/gui/` or `metis/src/gui/` — existing GUI code
- `metis/src/web/` or equivalent — web server for GUI
- Any frontend framework code (likely Tauri, Leptos, or web-based)
- `metis/src/api/` — data layer the GUI consumes

## Suggested Tasks for Decomposition

1. Audit existing GUI framework and component patterns
2. Extend document list view for new types with filtering
3. Extend document detail view for new frontmatter fields
4. Build hierarchy tree view component
5. Build traceability graph visualization
6. Build quality dashboard with charts
7. Build lease board (Kanban-style)
8. Build rule browser with search/filter
9. Build investigation tracker view
10. Add design reference panel to document detail view
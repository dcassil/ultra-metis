---
id: planning-data-views-in-control
level: initiative
title: "Planning Data Views in Control Dashboard"
short_code: "SMET-I-0071"
created_at: 2026-03-20T17:08:31.840484+00:00
updated_at: 2026-03-28T01:12:37.288580+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0002
initiative_id: planning-data-views-in-control
---

# Planning Data Views in Control Dashboard

## Context

The Remote AI Operations Control Dashboard (SMET-I-0039 through I-0046) provides session management, machine monitoring, and approval workflows. However, it has no visibility into cadre planning data — the documents, hierarchies, quality signals, and traceability chains that define what work is being done and why.

Users managing AI sessions remotely need to see the planning context alongside session activity: which task is this session working on, what's the parent story/epic, what quality gates apply, what rules are active. Without planning data views, the dashboard shows *how sessions run* but not *what they're building toward*.

This initiative adds planning data pages to the Control Dashboard, making cadre documents, hierarchy, quality, and traceability visible in the same web app where users manage sessions.

Derived from the archived SMET-I-0011 (GUI for Stronger Model), reframed as pages within the Control Dashboard rather than a standalone application.

## Goals & Non-Goals

**Goals:**
- Hierarchy tree view: interactive tree showing ProductDoc → Epic → Story → Task with expand/collapse, click-to-navigate
- Document list and detail views for all cadre document types, with type filtering and search
- Traceability view: visualize ancestry, descendants, and cross-reference links between documents
- Quality dashboard: baseline comparison display, quality trend indicators, gate pass/fail status per document
- Rule browser: searchable, filterable list of active engineering rules by scope
- Work item context panel: when viewing a session, show the linked task/story/epic with its planning context
- All views read from the cadre MCP API — the dashboard is a view layer, not a separate data store

**Non-Goals:**
- Editing documents from the dashboard (CLI and MCP tools handle mutations)
- Full project management features (Jira-like boards, sprint planning)
- Native mobile app (responsive web is sufficient, matching I-0042's mobile-first approach)
- Real-time collaboration on documents
- Architecture catalog browsing (can be added later)

## Detailed Design

### Data Source
All planning views query the cadre MCP server (or a REST wrapper around it). The dashboard makes read-only calls:
- `list_documents` — document list with filtering
- `read_document` — document detail with full content
- `search_documents` — full-text search
- Hierarchy is derived from parent references in document metadata

### Hierarchy Tree View
- Root nodes: ProductDocs (or Visions in Metis preset)
- Expandable: ProductDoc → Epics → Stories → Tasks
- Each node shows: title, short code, phase badge, assignee if any
- Click navigates to document detail
- Collapsed by default, remembers expand state in local storage

### Document List & Detail
- Filterable table: type, phase, parent, search
- Detail view renders markdown content with YAML frontmatter displayed as structured metadata
- Phase badge with color coding
- Links to parent and children

### Traceability View
- Given a document, show its full ancestry (up to root) and all descendants
- Show cross-reference links as a simple directed graph or linked list
- Useful for understanding: "what product goal does this task serve?"

### Quality Dashboard
- Per-document quality indicators (if quality records exist)
- Aggregate quality trends across a subtree (e.g., all tasks under an epic)
- Gate status: which documents have pending or failed quality gates
- Simple bar/sparkline charts, not full analytics

### Rule Browser
- List active rules with scope, description, protection level
- Filter by scope (repo, package, subsystem)
- Show which rules apply to a given document based on its location

### Session ↔ Work Item Integration
- Sessions (from I-0040) can link to a work item (task/story)
- Session detail page shows a collapsible "Planning Context" panel
- Panel shows: linked task → parent story → parent epic → product doc
- Quick navigation from session to the full planning hierarchy

## Alternatives Considered

- **Standalone planning app separate from Control Dashboard**: Rejected — adds a second web app to maintain and navigate between. Single dashboard with planning pages is simpler.
- **Embed planning views in VS Code extension**: Could be a future integration, but the web dashboard is the primary remote interface.
- **CLI-only for planning data**: CLI already exists (I-0010). The dashboard adds visual hierarchy and quality trends that text output can't match.

## Implementation Plan

### Task Decomposition (6 tasks)

| Order | Task | Title | Dependencies |
|-------|------|-------|-------------|
| 1 | SMET-T-0240 | Control API Planning Data Endpoints | None (foundation) |
| 2 | SMET-T-0241 | Dashboard Planning Foundation: Navigation, API Client, Shared Components | T-0240 |
| 3 | SMET-T-0242 | Document List and Detail Pages | T-0240, T-0241 |
| 4 | SMET-T-0243 | Hierarchy Tree View Page | T-0240, T-0241 |
| 5 | SMET-T-0244 | Quality Dashboard, Rule Browser, and Traceability Views | T-0240, T-0241 |
| 6 | SMET-T-0245 | Session Planning Context Panel and Integration Tests | T-0240 through T-0244 |

**Execution order:** T-0240 first (backend), T-0241 next (frontend foundation), then T-0242/T-0243/T-0244 can be parallel, T-0245 last (integration).

## Cadre ADR Alignment (SMET-A-0001)

**Audit date**: 2026-03-23 | **Recommendation**: Update scope (rename + execution model)

ADR point 1 (rename): All references to `cadre` in this initiative become `cadre`. MCP API calls change prefix to `mcp__cadre__`.

ADR point 3 (SDD execution): The dashboard should display SDD-style execution data — per-task subagent dispatches, review pass/fail results, model selection choices, token usage per task — not just ralph loop iterations. The "Session ↔ Work Item" integration should show the orchestrator→implementer→reviewer agent chain, not just a single session.

The existing design for hierarchy tree, document views, quality dashboard, and rule browser is unaffected.
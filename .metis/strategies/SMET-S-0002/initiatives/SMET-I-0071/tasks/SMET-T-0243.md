---
id: hierarchy-tree-view-page
level: task
title: "Hierarchy Tree View Page"
short_code: "SMET-T-0243"
created_at: 2026-03-28T00:33:48.968536+00:00
updated_at: 2026-03-28T00:33:48.968536+00:00
parent: SMET-I-0071
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0071
---

# Hierarchy Tree View Page

## Parent Initiative

[[SMET-I-0071]] Planning Data Views in Control Dashboard

## Objective

Build an interactive hierarchy tree view page that visualizes the full document hierarchy (Vision -> Strategy -> Initiative -> Task) as an expandable/collapsible tree. Users can navigate the planning structure visually and click any node to view its detail page.

## Acceptance Criteria

- [ ] HierarchyPage renders a tree with root nodes at the top level (visions/strategies)
- [ ] Each node shows: expand/collapse chevron, type icon, short code, title, phase badge
- [ ] Clicking expand loads and shows child documents
- [ ] Clicking a node title navigates to `/planning/documents/:shortCode`
- [ ] Expand/collapse state persisted in localStorage across page reloads
- [ ] "Expand All" and "Collapse All" buttons at the top
- [ ] Empty state when no documents exist
- [ ] Loading skeleton while hierarchy data loads
- [ ] Tree indentation visually clear with connecting lines or indentation guides
- [ ] Responsive: on mobile, tree scrolls horizontally if needed

## Implementation Notes

### Technical Approach
- Fetch hierarchy data from `GET /api/planning/hierarchy` which returns a tree structure
- Build a recursive `TreeNode` component that renders itself and its children
- Use `useState` for local expand/collapse state, sync to `localStorage` on change
- Type icons: use Heroicons (DocumentIcon for vision, FolderIcon for strategy/initiative, ClipboardIcon for task)
- Connect lines: use CSS `border-left` + `padding-left` pattern for tree guides
- Performance: lazy-load children on expand if tree is large, or load full hierarchy upfront if small

### Dependencies
- SMET-T-0241 (Dashboard Planning Foundation) — needs API client, routes, shared components
- SMET-T-0240 (Control API endpoints) — needs hierarchy endpoint

### Files to Create/Modify
- `src/pages/planning/HierarchyPage.tsx` — replace placeholder with full implementation
- `src/components/planning/TreeNode.tsx` — recursive tree node component

## Status Updates

*To be added during implementation*
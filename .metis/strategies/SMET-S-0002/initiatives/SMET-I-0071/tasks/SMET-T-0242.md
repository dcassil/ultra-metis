---
id: document-list-and-detail-pages
level: task
title: "Document List and Detail Pages"
short_code: "SMET-T-0242"
created_at: 2026-03-28T00:33:48.049177+00:00
updated_at: 2026-03-28T00:33:48.049177+00:00
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

# Document List and Detail Pages

## Parent Initiative

[[SMET-I-0071]] Planning Data Views in Control Dashboard

## Objective

Build the Document List page (filterable table of all cadre documents) and Document Detail page (full document view with markdown rendering, metadata panel, and parent/child navigation). These are the primary content pages for planning data.

## Acceptance Criteria

- [ ] DocumentsPage shows a table with columns: Short Code, Title, Type, Phase, Parent
- [ ] Type filter dropdown (vision, strategy, initiative, task, adr)
- [ ] Phase filter dropdown (all phases for selected type)
- [ ] Search input with debounced query to search endpoint
- [ ] Clicking a row navigates to `/planning/documents/:shortCode`
- [ ] DocumentDetailPage renders document markdown content
- [ ] Metadata panel shows: short code, type, phase, created/updated dates, parent link, blocked_by, tags
- [ ] Children section lists child documents with links
- [ ] Loading and error states handled gracefully
- [ ] Empty states with helpful messages
- [ ] Responsive layout matching existing dashboard patterns

## Implementation Notes

### Technical Approach
- DocumentsPage: use the existing Table component from `src/components/ui/Table.tsx` with column definitions
- Filters: use Select component for type/phase, FormInput for search
- DocumentDetailPage: use a markdown rendering library (react-markdown or similar — add to package.json)
- Metadata panel: Card component with key-value pairs
- Children section: simple list using ShortCodeLink component
- Use `useEffect` + `useState` pattern matching existing pages (e.g., MachinesPage)

### Dependencies
- SMET-T-0241 (Dashboard Planning Foundation) — needs API client and shared components
- SMET-T-0240 (Control API endpoints) — needs backend data

### Files to Create/Modify
- `src/pages/planning/DocumentsPage.tsx` — replace placeholder with full implementation
- `src/pages/planning/DocumentDetailPage.tsx` — replace placeholder with full implementation
- `package.json` — add react-markdown dependency

## Status Updates

*To be added during implementation*
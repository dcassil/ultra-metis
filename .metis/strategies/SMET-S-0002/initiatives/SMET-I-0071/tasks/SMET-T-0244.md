---
id: quality-dashboard-rule-browser-and
level: task
title: "Quality Dashboard, Rule Browser, and Traceability Views"
short_code: "SMET-T-0244"
created_at: 2026-03-28T00:33:49.858694+00:00
updated_at: 2026-03-28T01:11:10.780146+00:00
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

# Quality Dashboard, Rule Browser, and Traceability Views

## Parent Initiative

[[SMET-I-0071]] Planning Data Views in Control Dashboard

## Objective

Build three governance-focused views: (1) Quality Dashboard showing per-document quality indicators, gate pass/fail status, and aggregate trends; (2) Rule Browser with searchable/filterable list of active engineering rules by scope; (3) Traceability View showing ancestry chain and descendants for any document. These views provide governance visibility into the planning data.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

### Quality Dashboard
- [ ] QualityPage shows a summary of quality gate status across documents
- [ ] Per-document quality indicators: pass/fail badges for each gate
- [ ] Aggregate quality counts (X passing, Y failing, Z pending)
- [ ] Click a document to navigate to its detail page
- [ ] Graceful empty state when no quality records exist

### Rule Browser
- [ ] RulesPage shows a searchable list of active engineering rules
- [ ] Each rule displays: name, scope, description, protection level
- [ ] Filter by scope (repo, package, subsystem)
- [ ] Search input filters rules by name/description
- [ ] Protection level shown as colored badge (protected=red, standard=blue, advisory=gray)

### Traceability View
- [ ] Traceability section accessible from document detail page (or as a tab/panel)
- [ ] Shows full ancestry chain from document up to root (e.g., Task -> Initiative -> Strategy -> Vision)
- [ ] Shows all descendants below the document
- [ ] Each node is clickable to navigate to that document
- [ ] Visual breadcrumb-style display for ancestry, list for descendants

## Implementation Notes

### Technical Approach
- Quality: fetch from `GET /api/planning/quality/:shortCode` and aggregate across documents
- Rules: fetch from `GET /api/planning/rules` with scope query parameter
- Traceability: derive from hierarchy data — walk parent chain up, walk children down
- Use Card components for quality summary cards
- Use Table component for rule list
- Traceability can be a component embedded in DocumentDetailPage or standalone

### Dependencies
- SMET-T-0241 (Dashboard Planning Foundation) — needs API client and routes
- SMET-T-0240 (Control API endpoints) — needs rules and quality endpoints

### Files to Create/Modify
- `src/pages/planning/QualityPage.tsx` — replace placeholder
- `src/pages/planning/RulesPage.tsx` — replace placeholder
- `src/components/planning/TraceabilityPanel.tsx` — new component
- `src/components/planning/QualityGateBadge.tsx` — pass/fail indicator

## Status Updates

*To be added during implementation*
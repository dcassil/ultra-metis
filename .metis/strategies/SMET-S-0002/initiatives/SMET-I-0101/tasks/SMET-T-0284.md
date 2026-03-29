---
id: governance-page-merge-quality-and
level: task
title: "Governance Page: Merge Quality and Rules into Tabbed View"
short_code: "SMET-T-0284"
created_at: 2026-03-29T00:43:17.531855+00:00
updated_at: 2026-03-29T01:15:49.714592+00:00
parent: SMET-I-0101
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0101
---

# Governance Page: Merge Quality and Rules into Tabbed View

Covers Initiative Issue 9 (Phase 1). Simplifies the Planning/Work section navigation.

## Objective

Create a new GovernancePage that combines the existing Quality Gates and Rules views into a single tabbed page, replacing the standalone QualityPage and RulesPage. Update routing and navigation to reflect the merge.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] New `GovernancePage.tsx` exists at `src/pages/planning/GovernancePage.tsx`
- [ ] GovernancePage has two tabs: "Quality Gates" and "Rules"
- [ ] Quality Gates tab contains the exact same content as the current QualityPage
- [ ] Rules tab contains the exact same content as the current RulesPage
- [ ] `/planning/governance` route renders GovernancePage
- [ ] Old routes `/planning/quality` and `/planning/rules` redirect to `/planning/governance`
- [ ] Sidebar "Work" section has 3 items: Documents, Hierarchy, Governance
- [ ] QualityPage.tsx and RulesPage.tsx are deleted
- [ ] Tab state is preserved in URL (e.g., `/planning/governance?tab=rules`)
- [ ] Dashboard compiles and renders without errors

## Implementation Notes

### Technical Approach

**GovernancePage.tsx** — New page:
1. Create with two tabs: "Quality Gates" (default) and "Rules"
2. Extract the rendering logic from QualityPage and RulesPage into this page
3. Both tabs retain all their existing functionality (search, filters, table rendering)
4. Read initial tab from URL search params: `?tab=quality` or `?tab=rules`
5. Update URL when tab changes (without full navigation)

**App.tsx** — Route changes:
1. Add route: `<Route path="planning/governance" element={<GovernancePage />} />`
2. Add redirects: `<Route path="planning/quality" element={<Navigate to="/planning/governance?tab=quality" replace />} />`
3. Add redirect: `<Route path="planning/rules" element={<Navigate to="/planning/governance?tab=rules" replace />} />`
4. Remove old route entries after redirects are in place

**Sidebar.tsx** (already updated in SMET-T-0278):
- The "Work" section already references "Governance" — this task creates the actual page for it

**Delete files**:
- `src/pages/planning/QualityPage.tsx`
- `src/pages/planning/RulesPage.tsx`

### Files to Change
- Create: `apps/control-dashboard/src/pages/planning/GovernancePage.tsx`
- `apps/control-dashboard/src/App.tsx`
- Delete: `apps/control-dashboard/src/pages/planning/QualityPage.tsx`
- Delete: `apps/control-dashboard/src/pages/planning/RulesPage.tsx`

### Dependencies
- SMET-T-0278 (sidebar restructure) should be done first — it sets up the "Governance" nav entry

## Status Updates

*To be added during implementation*
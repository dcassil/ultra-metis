---
id: mobile-ui-cleanup-fix-remaining
level: initiative
title: "Mobile UI Cleanup: Fix Remaining Buttons, Votes, and Layout Issues"
short_code: "SMET-I-0104"
created_at: 2026-03-29T02:53:38.550247+00:00
updated_at: 2026-03-29T02:58:25.864931+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: S
strategy_id: SMET-S-0002
initiative_id: mobile-ui-cleanup-fix-remaining
---

# Mobile UI Cleanup: Fix Remaining Buttons, Votes, and Layout Issues Initiative

## Context

SMET-I-0103 completed a comprehensive mobile-responsive pass across the dashboard, adding card layouts, touch targets, and small-screen optimization. However, several components were missed or partially addressed. The user identified specific examples — revoke/remove machine buttons and voting UI — that still don't respect screen width on iPhone 16 (393px).

## Issues Found

### 1. PendingMachineCard — Approve/Reject buttons (HIGH)
**File:** `PendingMachineCard.tsx:85`
- Container uses `flex shrink-0 items-center gap-2` — no mobile stacking
- Parent card layout (`flex items-start justify-between gap-4`) puts buttons to the right of content — overflows on narrow screens
- Modal footer (line 113) uses `flex justify-end gap-2` without mobile stacking pattern

### 2. GuidanceInput — Three-element horizontal layout (MEDIUM)
**File:** `GuidanceInput.tsx:71`
- `flex items-center gap-3` puts type selector (~120px), text input (flex-1), and send button (~50px) in one row
- On 393px with padding, content width ~330px — elements squeeze/overflow
- Type selector buttons (px-2.5 py-1.5 text-xs) have inadequate touch targets
- Interrupt confirm modal footer (line 129) missing mobile stacking

### 3. ApprovalCard — Wrap vs stack (LOW)
**File:** `ApprovalCard.tsx:83`
- Uses `flex flex-wrap gap-2` — buttons have `w-full sm:w-auto` which works, but `flex-col sm:flex-row` would be more predictable

### 4. SessionsPage — Filter controls layout (LOW)
**File:** `SessionsPage.tsx:230`
- `flex flex-wrap items-end gap-4` for 4 filter dropdowns
- On narrow screens, dropdowns wrap awkwardly at inconsistent points
- Would benefit from grid layout on mobile

## Goals & Non-Goals

**Goals:**
- Fix all remaining button groups that don't stack on mobile
- Ensure all modal footers use the `flex-col-reverse sm:flex-row` pattern
- Make GuidanceInput usable on 393px screens
- Ensure consistent touch targets on all interactive elements

**Non-Goals:**
- New features or layout redesigns
- Desktop layout changes
- Backend changes

## Detailed Design

Apply the same patterns established in SMET-I-0103:
- Button containers: `flex flex-col sm:flex-row gap-2`
- Buttons within: `w-full sm:w-auto`
- Modal footers: `flex flex-col-reverse sm:flex-row sm:justify-end gap-2 sm:gap-3`
- Touch targets: `min-h-[44px]` on small interactive elements
- GuidanceInput: stack type selector above input row on mobile

## Implementation Plan

Single task — small scope, all fixes are mechanical application of existing patterns.
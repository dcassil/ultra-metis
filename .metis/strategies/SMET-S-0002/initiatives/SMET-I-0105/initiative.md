---
id: mobile-responsive-fix-pass-button
level: initiative
title: "Mobile Responsive Fix Pass: Button Overflow, Modal Footers, and Remaining Layout Issues"
short_code: "SMET-I-0105"
created_at: 2026-03-29T03:02:28.655441+00:00
updated_at: 2026-03-29T03:06:07.886767+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: XS
strategy_id: SMET-S-0002
initiative_id: mobile-responsive-fix-pass-button
---

# Mobile Responsive Fix Pass: Button Overflow, Modal Footers, and Remaining Layout Issues Initiative

## Context

Follow-up to SMET-I-0103 (Mobile-Responsive Dashboard). That initiative completed the major mobile layout work — card modes, LiveOutput wrapping, touch targets, tab bars, detail page stacking. However, several smaller responsive issues remain that were missed or introduced during that pass:

1. **Modal footer buttons overflow on mobile** — Multiple modals (Revoke Machine, Remove Machine, Reject Machine, Confirm Interrupt) have footer buttons without `w-full sm:w-auto` classes, causing them to extend beyond the viewport on narrow screens.
2. **GuidanceInput injection type selector** — The button group uses `shrink-0` and doesn't adapt width on mobile, potentially overflowing on 393px screens.
3. **Notification action buttons** — Use `items-stretch` without explicit width constraints.
4. **Any other remaining overflow or touch-target issues** discovered during the fix pass.

## Goals & Non-Goals

**Goals:**
- All modal footer buttons properly stack and fill width on mobile (`w-full sm:w-auto`)
- GuidanceInput injection type selector wraps or scrolls gracefully on narrow screens
- No horizontal overflow on any screen at 393px viewport width
- All interactive elements maintain 44px minimum touch targets
- Desktop layout (`lg`+) completely unchanged

**Non-Goals:**
- New mobile features or layout changes beyond fixing overflow/touch issues
- Changes to component APIs
- Backend changes

## Detailed Design

### 1. Modal Footer Button Fix

Add `w-full sm:w-auto` to all modal footer buttons that are missing it:

- **MachineDetailPage.tsx** (lines ~712-717): Revoke Machine modal — Cancel and Revoke buttons
- **MachineDetailPage.tsx** (lines ~732-737): Remove Machine modal — Cancel and Remove buttons
- **PendingMachineCard.tsx** (lines ~116-121): Reject Machine modal — Cancel and Reject buttons
- **GuidanceInput.tsx** (lines ~133-138): Confirm Interrupt modal — Cancel and Send Interrupt buttons

Pattern: Every `<Button>` inside a modal `footer` prop should have `className="w-full sm:w-auto"` (or include those classes if it already has a className).

### 2. GuidanceInput Injection Type Selector

The injection type button group (`shrink-0` container with 4 buttons) can overflow on 393px screens. Fix by:
- Remove `shrink-0` from the button group container, or
- Add `flex-wrap` so buttons wrap to a second row on very narrow screens, or
- Add `overflow-x-auto` for horizontal scrolling

### 3. Notification Action Buttons

Verify the notification card action buttons (Mark Read, Dismiss) have proper width constraints on mobile. Add `w-full sm:w-auto` if missing.

### 4. Full Audit

Do a final sweep of all pages at 393px to catch any remaining overflow — check with browser dev tools or build inspection of all flex/button containers.

## Alternatives Considered

**Do nothing** — These are cosmetic issues on mobile. Rejected because buttons extending beyond the viewport is a usability blocker — users can't tap truncated buttons.

## Implementation Plan

Single task — this is a targeted CSS fix pass across ~5 files. Estimated XS complexity (< 1 day).
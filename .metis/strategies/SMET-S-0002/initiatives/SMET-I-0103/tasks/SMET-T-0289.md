---
id: liveoutput-console-mobile-word
level: task
title: "LiveOutput Console Mobile: Word Wrapping, Font Scaling, and Sticky Guidance Input"
short_code: "SMET-T-0289"
created_at: 2026-03-29T02:21:08.354867+00:00
updated_at: 2026-03-29T02:26:43.472030+00:00
parent: SMET-I-0103
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0103
---

# LiveOutput Console Mobile: Word Wrapping, Font Scaling, and Sticky Guidance Input

## Parent Initiative

[[SMET-I-0103]] — Mobile-Responsive Dashboard

## Objective

Make the LiveOutput console (terminal-style session output viewer) usable on iPhone 16 (393px) without horizontal scrolling. Add word wrapping, reduce font size on mobile, make the guidance input sticky at the bottom, and reposition approval cards above the console on mobile.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Long lines wrap within the console — no horizontal scroll at 393px width
- [ ] `overflow-wrap: break-word` and `word-break: break-all` applied to console output
- [ ] Font size: `text-xs` on mobile, `text-sm` at `sm`+ breakpoint
- [ ] Scroll-lock toggle and connection indicator in a sticky bar at top of console
- [ ] Guidance input sticky at bottom of viewport (visible while scrolling output)
- [ ] Approval cards render above console output on mobile (priority position)
- [ ] Full scrollable output preserved — no artificial line limit
- [ ] Console takes remaining viewport height minus header/action bars
- [ ] Desktop layout unchanged

## Implementation Notes

### Technical Approach

Modify `src/components/LiveOutput.tsx`:

1. **Word wrapping**: Add Tailwind classes `break-all` and `overflow-wrap: break-word` to the output container and individual line elements.

2. **Font scaling**: Change monospace text from current size to `text-xs sm:text-sm` on the output lines.

3. **Sticky controls bar**: Wrap scroll-lock toggle and connection indicator in a `sticky top-0 z-10 bg-secondary-950` container so they stay visible while scrolling.

4. **Sticky guidance input**: On the `SessionDetailPage`, wrap `GuidanceInput` in a `sticky bottom-0` container on mobile so it stays visible as the user scrolls through output.

5. **Approval card repositioning**: On mobile, render `ApprovalCard` components in a `sm:hidden` block above the LiveOutput, and keep the current position for desktop via `hidden sm:block`.

### Files to Change
- `src/components/LiveOutput.tsx` — word wrap, font scaling, sticky controls
- `src/pages/SessionDetailPage.tsx` — sticky guidance input, approval card reordering
- `src/components/GuidanceInput.tsx` — ensure full-width on mobile

### Dependencies
None — independent of Table card mode work.

## Status Updates

*To be added during implementation*
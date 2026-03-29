---
id: mobile-responsive-dashboard-card
level: initiative
title: "Mobile-Responsive Dashboard: Card Layouts, Touch Targets, and Small-Screen Optimization"
short_code: "SMET-I-0103"
created_at: 2026-03-29T01:13:17.495962+00:00
updated_at: 2026-03-29T01:13:17.495962+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0002
initiative_id: mobile-responsive-dashboard-card
---

# Mobile-Responsive Dashboard: Card Layouts, Touch Targets, and Small-Screen Optimization Initiative

## Context

The control dashboard was built desktop-first across multiple initiatives (SMET-I-0095, I-0039 through I-0044, I-0071, I-0101). While the sidebar/header already has a mobile overlay pattern at the `lg` (1024px) breakpoint, the actual page content — tables, the LiveOutput console, forms, and detail pages — does not scale well to phone-sized screens. The primary mobile user is on an iPhone 16 (393px CSS width, portrait mode) and needs to monitor agent status, respond to approval requests, provide guidance, and dispatch new sessions from their phone.

**Current state of mobile support:**
- Sidebar: overlay modal on mobile (works)
- Header: responsive (works)
- Tables: `overflow-x-auto` horizontal scroll (poor UX on phone)
- LiveOutput console: no word wrapping, horizontal scroll required
- Forms: mostly `w-full` (ok), but button groups don't stack
- Only two breakpoints used: default and `lg` (1024px), with occasional `sm` (640px)
- No `md` (768px) breakpoint for tablet middle-ground
- No card-based mobile layouts for data lists
- Touch targets not consistently 44px minimum

**Tech stack:** Tailwind CSS v4.2.2, Headless UI v2.2.9, React 19, Vite.

## Goals & Non-Goals

**Goals:**
- Every screen usable on iPhone 16 (393px portrait) without horizontal scrolling
- Tables render as card-based layouts below `sm` breakpoint
- LiveOutput console word-wraps for narrow screens, full scrollable output preserved
- All interactive elements meet 44px minimum touch target
- Add `md` (768px) breakpoint for tablet transitional layouts
- Button groups stack vertically on mobile
- Modals fit within mobile viewport with proper margins
- Tab bars scroll horizontally rather than wrapping on narrow screens
- Desktop layout completely unchanged

**Non-Goals:**
- Native mobile app or PWA features
- Separate mobile-specific routes or views
- Backend API changes
- New features — this is strictly responsive layout work
- Offline support or service workers

## Primary Mobile Use Cases

### UC-1: Monitor Agent Status
- **Actor**: User on iPhone
- **Flow**: Open dashboard → see sessions list as cards → see status badge, machine name, duration at a glance → tap card to view session detail
- **Key requirement**: Status information must be scannable without tapping into each session

### UC-2: Respond to Approval Request
- **Actor**: User on iPhone (likely from notification)
- **Flow**: Tap notification → land on session detail → see approval card prominently → tap approve/reject → provide optional comment
- **Key requirement**: Approval card and action buttons must be immediately visible without scrolling past console output

### UC-3: Provide Guidance to Running Session
- **Actor**: User on iPhone
- **Flow**: Open session detail → view recent output in LiveOutput → type guidance in input → submit
- **Key requirement**: Guidance input must be accessible (not buried), keyboard should not obscure it

### UC-4: Start New Session
- **Actor**: User on iPhone
- **Flow**: Tap "Start Session" button → select machine from dropdown → enter prompt/task → submit
- **Key requirement**: Full-width form elements, machine selector usable on touch, submit button prominent

## Detailed Design

### Breakpoint Strategy

Three tiers, mobile-first:

| Breakpoint | Width | Target | Layout |
|---|---|---|---|
| Default | < 640px | iPhone 16 (393px) | Single column, cards, stacked actions |
| `sm` | 640px+ | Large phones landscape, small tablets | Transitional — some side-by-side |
| `md` | 768px+ | Tablets | Grid layouts begin |
| `lg` | 1024px+ | Desktop | Current layout, no changes |

The existing `lg` sidebar/header responsive behavior stays exactly as-is.

### Table → Card Transformation

Add a `cardMode` rendering path to the existing `<Table>` component in `src/components/ui/Table.tsx`. Below `sm` breakpoint, each row renders as a card:

- **Card header**: Primary field (name/title) with status badge inline
- **Card body**: Secondary fields as `label: value` pairs, stacked vertically
- **Card footer**: Action buttons if applicable
- **Touch target**: Entire card is tappable (preserves existing `onRowClick` handler)
- **Spacing**: 12px gap between cards, 16px horizontal padding
- **Implementation**: Use Tailwind's `sm:hidden` / `hidden sm:table` to toggle between card and table views. The card layout is rendered alongside the table in the same component — no separate component needed.

**Applies to these pages:**
- `MachinesPage.tsx` — machines list
- `SessionsPage.tsx` — sessions list
- `HistoryPage.tsx` — session history list
- `planning/DocumentsPage.tsx` — planning documents list
- `NotificationsPage.tsx` — notifications list

**Card field priority per table** (what shows prominently vs. secondary):

| Page | Card Header | Card Body Fields | Hidden on Mobile |
|---|---|---|---|
| Machines | Name + status dot | Trust tier, session mode, last seen | OS, architecture |
| Sessions | Title + state badge | Machine name, started at, duration | Session ID |
| History | Title + outcome badge | Machine, completed at, duration | Session ID |
| Documents | Title + type badge | Phase, parent, updated at | Short code (show in header subtitle) |
| Notifications | Message + type badge | Created at, related session | Notification ID |

### LiveOutput Console

Changes to `src/components/LiveOutput.tsx` and related styles:

- Add `overflow-wrap: break-word` and `word-break: break-all` for long unbroken strings (file paths, URLs, hashes)
- Reduce monospace font from current size to `text-xs` (12px) on mobile via `text-xs sm:text-sm`
- **Scroll-lock toggle and connection indicator**: Reposition as a sticky bar at the top of the console container (currently inline). On mobile, these controls need to be visible without competing with output text.
- **No line limit**: Full scrollable output preserved. The console takes remaining viewport height minus header and action bars.
- **Guidance input**: Renders full-width below the console output. On mobile, use `position: sticky; bottom: 0` so it stays visible as the user scrolls output.
- **Approval cards**: Render above the console output on mobile (priority content) rather than alongside it.

### Forms and Action Bars

- **Button groups**: Add `flex-col sm:flex-row` to all button containers. Individual buttons get `w-full sm:w-auto`.
- **Session start form** (`NewSessionPage.tsx`): Single-column layout on mobile. Machine selector (Headless UI Listbox) already renders as a dropdown — ensure the options panel has sufficient max-height and doesn't overflow the viewport.
- **Modal sizing**: Change from fixed `max-w-md` to `max-w-[calc(100vw-2rem)] sm:max-w-md` so modals have 1rem margin on each side on mobile.
- **Form inputs**: Already `w-full` — no changes needed. Verify label + input stacking is consistent.

### Page-Specific Adjustments

**Session Detail Page** (`SessionDetailPage.tsx`):
- Tab bar: `overflow-x-auto whitespace-nowrap` so tabs scroll horizontally if they overflow on narrow screens. No wrapping.
- LiveOutput and guidance input get priority layout space — approval cards above, guidance input sticky at bottom.
- Metadata section: stack vertically on mobile.

**Machine Detail Page** (`MachineDetailPage.tsx`):
- Same horizontal-scroll tab bar treatment.
- Status banner: stack badge elements vertically on mobile (`flex-col sm:flex-row`).
- Action buttons (Revoke, Remove): full-width stacked on mobile.

**Hierarchy Tree Page** (`planning/HierarchyPage.tsx`):
- Reduce indent per level from current spacing to `pl-3` on mobile (vs `pl-6` desktop).
- Tree nodes get larger touch targets: minimum `py-3` padding.
- Allow horizontal scroll if tree depth causes overflow.

**Document Detail Page** (`planning/DocumentDetailPage.tsx`):
- Metadata grid: `grid-cols-1 sm:grid-cols-2 md:grid-cols-3` (currently `grid-cols-2 sm:grid-cols-3`).
- Markdown content: ensure code blocks have `overflow-x-auto` and don't break layout.

**Notifications Page** (`NotificationsPage.tsx`):
- Already close to card layout — ensure dismiss/action buttons have 44px touch targets.
- Action buttons stack vertically on mobile.

### Global Touch Target Rules

All interactive elements must meet 44x44px minimum touch area:
- Buttons: already sized adequately with `px-4 py-2` (md size)
- Small buttons (`sm` size): increase padding on mobile or add invisible touch area via `min-h-[44px] min-w-[44px]`
- Table row click areas / card click areas: full width, minimum `py-3` height
- Sidebar nav items: already adequate with icon + text padding
- Tab bar items: ensure `py-3 px-4` minimum

### What Stays Unchanged

- Sidebar overlay behavior (already works on mobile)
- Header layout (already responsive)  
- Color system, typography scale, spacing tokens
- Desktop layout (`lg`+) — zero changes
- Backend APIs — this is purely frontend CSS/layout work
- Component API signatures (Table, Card, Badge, etc.) — only internal rendering changes

## Alternatives Considered

### B: Shared Mobile Layout System
Build a `<ResponsiveContainer>` wrapper and `<CardList>` component that all pages use, with a mobile-first design system layer. **Rejected** because the upfront abstraction cost isn't justified for ~13 routes. The existing Table component can handle card mode internally without a new system.

### C: Mobile-Specific Routes  
Create separate `/m/sessions`, `/m/machines` routes with purpose-built mobile views. **Rejected** because it duplicates logic, creates two codebases to maintain, and breaks shared URLs (notifications link to specific sessions — those URLs need to work on both mobile and desktop).

## Implementation Plan

Component-level responsive pass, screen by screen. Suggested task breakdown:

1. **Table Card Mode** — Add card rendering path to `<Table>` component, toggle below `sm` breakpoint. This is the foundation that all list pages benefit from.

2. **LiveOutput Mobile** — Word wrapping, font size reduction, sticky guidance input, approval card repositioning.

3. **Global Mobile Utilities** — Button stacking, modal sizing, touch target enforcement, tab bar horizontal scroll. These are cross-cutting changes to shared UI components.

4. **List Pages Card Integration** — Apply card mode to each list page (Machines, Sessions, History, Documents, Notifications) with per-page card field priority configuration.

5. **Detail Pages Mobile** — Session detail, machine detail, document detail, hierarchy tree — all the page-specific adjustments for stacking, tab bars, and metadata grids.

6. **Verification Pass** — Test all screens at 393px width, verify no horizontal scroll, verify touch targets, verify approval/guidance flows work end-to-end on mobile.
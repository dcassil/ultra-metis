---
id: design-system-primitives
level: task
title: "Design System Primitives"
short_code: "SMET-T-0209"
created_at: 2026-03-27T16:28:41.912597+00:00
updated_at: 2026-03-27T19:09:25.286187+00:00
parent: SMET-I-0095
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0095
---

# Design System Primitives

## Parent Initiative

[[SMET-I-0095]] — Control Dashboard Foundation

## Objective

Build the reusable UI component primitives that all feature initiatives will use to construct their views. This includes Button, Badge (with status variants), Card, Table, FormInput, Select, Toggle, and Modal. Each component uses Tailwind CSS for styling, follows accessible patterns (using Headless UI / Radix where appropriate), and supports the design tokens defined in the Tailwind config. After this task, feature initiatives can compose views from these primitives without building basic UI elements from scratch.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `Button` component with variants: `primary`, `secondary`, `danger`, `ghost`; sizes: `sm`, `md`, `lg`; supports `disabled` and `loading` states
- [ ] `Badge` component with status variants: `online` (green), `offline` (gray), `pending` (amber), `error` (red), `info` (blue); designed to represent machine status, session state, policy violations
- [ ] `Card` component with optional `title`, `subtitle`, `footer` slots; clean border/shadow styling
- [ ] `Table` component with typed column definitions, sortable column headers (visual only — sort logic is consumer responsibility), responsive horizontal scroll on mobile
- [ ] `FormInput` component with label, placeholder, error message display, required indicator; supports `text`, `number`, `password`, `email` types
- [ ] `Select` component (using Headless UI `Listbox`) with label, options list, placeholder, error state
- [ ] `Toggle` component (using Headless UI `Switch`) with label and description text
- [ ] `Modal` component (using Headless UI `Dialog`) with title, body slot, footer actions, close on backdrop click, close on Escape key
- [ ] All components are fully typed with TypeScript props interfaces
- [ ] All interactive components are keyboard accessible (tab navigation, Enter/Space activation)
- [ ] All components are responsive and render correctly on mobile viewports (320px+)
- [ ] A `ComponentShowcase` page at `/dev/components` (only in dev mode) renders every component in all variants for visual verification
- [ ] Components are exported from `src/components/ui/index.ts` barrel

## Implementation Notes

### Technical Approach

1. Create `src/components/ui/` directory for all design system primitives
2. Implement each component as a standalone file with co-located types:
   - `Button.tsx` — uses `<button>` element, Tailwind classes for variants, spinner icon for loading state
   - `Badge.tsx` — uses `<span>` element, variant-to-color mapping via Tailwind classes
   - `Card.tsx` — uses `<div>` with conditional rendering for title/subtitle/footer sections
   - `Table.tsx` — generic component `Table<T>` with typed column definitions (`{ key: keyof T; header: string; render?: (value: T[key]) => ReactNode }`)
   - `FormInput.tsx` — uses `<input>` with `<label>`, error message via `aria-describedby`, red border on error
   - `Select.tsx` — wraps Headless UI `Listbox` with label and error styling
   - `Toggle.tsx` — wraps Headless UI `Switch` with label and description
   - `Modal.tsx` — wraps Headless UI `Dialog` with transition animations (fade backdrop, scale content)
3. Create `src/components/ui/index.ts` barrel that exports all components
4. Create `src/pages/ComponentShowcase.tsx`:
   - Renders every component in every variant/state
   - Add route `/dev/components` (only rendered when `import.meta.env.DEV` is true)
5. Use consistent Tailwind patterns:
   - `rounded-lg` for containers, `rounded-md` for inputs/buttons
   - `shadow-sm` for cards, `shadow-lg` for modals
   - Focus rings: `focus:ring-2 focus:ring-primary-500 focus:ring-offset-2`
   - Transition: `transition-colors duration-150` for interactive elements

### Design Token Usage

All components reference the design tokens from `tailwind.config.ts` (set up in SMET-T-0205):
- Colors: `primary-*`, `success-*`, `warning-*`, `danger-*`, `neutral-*`
- Typography: `font-sans` for UI text, `font-mono` for code/output
- Spacing: standard Tailwind scale (4px base)

### Dependencies

- SMET-T-0205 (Project Scaffold) — Tailwind CSS, Headless UI, and Heroicons must be installed
- SMET-T-0206 (Application Shell) — the showcase page route is added to the router

### File Structure Created

```
src/
  components/
    ui/
      Button.tsx
      Badge.tsx
      Card.tsx
      Table.tsx
      FormInput.tsx
      Select.tsx
      Toggle.tsx
      Modal.tsx
      index.ts
  pages/
    ComponentShowcase.tsx
```

## Status Updates

*To be added during implementation*
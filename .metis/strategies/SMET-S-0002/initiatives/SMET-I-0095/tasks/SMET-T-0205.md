---
id: project-scaffold-dev-workflow
level: task
title: "Project Scaffold & Dev Workflow"
short_code: "SMET-T-0205"
created_at: 2026-03-27T16:28:37.982392+00:00
updated_at: 2026-03-27T18:52:04.887437+00:00
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

# Project Scaffold & Dev Workflow

## Parent Initiative

[[SMET-I-0095]] — Control Dashboard Foundation

## Objective

Initialize the Control Dashboard web application in `apps/control-dashboard/` with Vite, React, TypeScript, and Tailwind CSS. Set up the project structure, development tooling, and configuration so that all subsequent tasks have a working foundation to build on. After this task, `npm run dev` starts a hot-reloading development server and `npm run build` produces a production bundle.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `apps/control-dashboard/` directory exists with a valid `package.json`
- [ ] Vite is configured as the bundler with React and TypeScript plugins
- [ ] TypeScript strict mode is enabled with appropriate `tsconfig.json`
- [ ] Tailwind CSS is installed and configured with a `tailwind.config.ts` that includes design token stubs (color palette, font family, spacing scale)
- [ ] A minimal `src/main.tsx` entry point renders a root React component
- [ ] `npm run dev` starts a local development server with hot module replacement (HMR)
- [ ] `npm run build` produces an optimized production build in `dist/`
- [ ] `npm run preview` serves the production build locally for verification
- [ ] ESLint and Prettier are configured for the project with React and TypeScript rules
- [ ] `.env.example` file exists documenting `VITE_API_BASE_URL` environment variable
- [ ] Project builds without errors or warnings

## Implementation Notes

### Technical Approach

1. Run `npm create vite@latest control-dashboard -- --template react-ts` inside `apps/`
2. Install and configure Tailwind CSS v3+:
   - `npm install -D tailwindcss postcss autoprefixer`
   - Create `tailwind.config.ts` with content paths pointing to `src/**/*.{ts,tsx}`
   - Add Tailwind directives to `src/index.css`
3. Configure design token stubs in `tailwind.config.ts` `theme.extend`:
   - Colors: primary (blue scale), secondary (gray scale), success (green), warning (amber), danger (red), neutral (slate)
   - Font families: sans (system stack), mono (for code/output display)
   - Spacing: 4px base scale (Tailwind default is already 4px-based, confirm)
4. Set up TypeScript with `strict: true` in `tsconfig.json`
5. Install and configure ESLint with `@typescript-eslint/parser`, `eslint-plugin-react`, `eslint-plugin-react-hooks`
6. Install Prettier with Tailwind CSS plugin for class sorting
7. Install Headless UI (`@headlessui/react`) and Heroicons (`@heroicons/react`) as accessible component primitives
8. Create `.env.example` with `VITE_API_BASE_URL=http://localhost:3000` (Control Service default)
9. Add npm scripts: `dev`, `build`, `preview`, `lint`, `format`
10. Verify the scaffold compiles and runs cleanly

### Dependencies

- SMET-I-0038 (Monorepo Restructure) must be complete so that `apps/` directory exists
- Node.js 18+ and npm available in the development environment

### File Structure Created

```
apps/control-dashboard/
  package.json
  tsconfig.json
  tsconfig.node.json
  vite.config.ts
  tailwind.config.ts
  postcss.config.js
  .env.example
  index.html
  src/
    main.tsx
    App.tsx
    index.css
    vite-env.d.ts
```

## Status Updates

- **Completed**: Scaffold created with Vite 8 + React 19 + TypeScript 5.9 + Tailwind CSS 4.2
- Installed: Headless UI, Heroicons, React Router, Axios
- Design tokens configured in CSS @theme: primary/secondary/success/warning/danger color scales, font families
- .nvmrc set to Node 20, .env.example with VITE_API_BASE_URL and VITE_AUTH_TOKEN
- Build output: 190KB JS (60KB gzipped), 6.7KB CSS (2KB gzipped), zero warnings
# JavaScript React App (SPA / Next.js)

## Overview

Feature-based React application architecture. Each feature is a self-contained vertical slice with its own components, hooks, services, and tests. Shared UI components live in a common directory. Suitable for SPAs and Next.js apps.

## Structure

Features are organized by domain (e.g., auth, dashboard, settings). Each feature contains components, hooks, services, and types. Shared components, hooks, and utilities live in common directories.

## Dependency Rules

- Features may import from shared/ but not from other features
- Components depend on hooks and services within the same feature
- Hooks orchestrate service calls and state management
- Services handle API communication

## Anti-Patterns

- Cross-feature imports (feature A importing from feature B)
- God components with too many responsibilities
- Business logic in components instead of hooks/services

## Quality Expectations

- ESLint with React-specific rules
- TypeScript strict mode
- Component tests co-located with source
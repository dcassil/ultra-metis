# JavaScript Component Library

## Overview

Shared UI library with one component per folder. Storybook-friendly structure with explicit public API via index exports. Designed for publishing to npm or internal registries. Suitable for design system implementations.

## Structure

Each component lives in its own folder with its implementation, styles, tests, and stories. A root index.ts re-exports the public API. Internal utilities are in a utils/ directory not exposed publicly.

## Dependency Rules

- Components may depend on shared utilities only
- Components should not depend on each other (compose at consumer level)
- All public exports go through root index.ts

## Anti-Patterns

- Components importing other components directly
- Leaking internal utilities through the public API
- Tightly coupling to a specific framework version
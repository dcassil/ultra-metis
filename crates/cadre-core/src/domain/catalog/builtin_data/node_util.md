# JavaScript Node Utility Library

## Overview

Utility or library package for Node.js. Flat or domain-grouped source structure. Comprehensive unit tests. Clean public API via root index.ts. Suitable for shared libraries, SDK packages, and utility collections.

## Structure

Source files are organized flat or by domain group. Each module is self-contained. A root index.ts defines the public API surface. Internal helpers are not exported. Tests mirror the source structure.

## Dependency Rules

- Public modules may depend on internal helpers
- Internal helpers should not depend on public modules
- No circular dependencies between modules
- All public exports via root index.ts

## Anti-Patterns

- Exporting everything (kitchen-sink public API)
- Circular dependencies between modules
- Side effects on import
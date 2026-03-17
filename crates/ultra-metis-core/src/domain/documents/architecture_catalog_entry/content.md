# {{ title }}

## Overview

<!-- Describe the architecture pattern: what it is, when to use it, and what problems it solves. -->
- **Pattern Name**: (canonical name, e.g., "Hexagonal Architecture", "Modular Monolith")
- **Best For**: (project types, team sizes, scale requirements where this pattern excels)
- **Avoid When**: (situations where this pattern is a poor fit)

## Structure

<!-- Define the expected folder layout, layers, and module boundaries. -->

### Folder Layout
```
(expected directory structure)
```

### Layers
<!-- List architectural layers from outermost to innermost. -->
1. (layer name -- purpose and what belongs here)
2. (next layer)

### Module Boundaries
<!-- Define how code is organized into modules and what each module owns. -->
- (module name -- boundary definition and contents)

## Dependency Rules

<!-- Direction constraints: which layers/modules can depend on which. Use MUST/MUST NOT. -->
- (rule, e.g., "Domain layer MUST NOT depend on infrastructure layer")

## Naming Conventions

<!-- File, module, and export naming patterns expected in repos using this architecture. -->
- (convention, e.g., "Service files use snake_case with _service suffix")

## Anti-Patterns

<!-- Known anti-patterns to watch for. Each should be detectable by analysis tools. -->
- (anti-pattern name -- description and why it is harmful)

## Quality Expectations

<!-- Analysis tool configuration and baseline thresholds for repos using this pattern. -->
- **Required Tools**: (list analysis tools, e.g., clippy, eslint, tsc)
- **Baseline Thresholds**: (minimum acceptable metrics)

## Rules Seed Data

<!-- RulesConfig entries that should be auto-generated when this pattern is selected. -->
- (rule name -- what it enforces and its protection level)
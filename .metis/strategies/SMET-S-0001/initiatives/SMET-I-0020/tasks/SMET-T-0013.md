---
id: implement-architecturecatalogentry
level: task
title: "Implement ArchitectureCatalogEntry domain type"
short_code: "SMET-T-0013"
created_at: 2026-03-16T21:12:17.298119+00:00
updated_at: 2026-03-16T21:18:51.754609+00:00
parent: SMET-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: SMET-I-0020
---

# Implement ArchitectureCatalogEntry domain type

## Objective

Implement the ArchitectureCatalogEntry domain type following the governance type pattern (lighter-weight, no Document trait). This type represents reusable architecture patterns in the catalog (e.g., "Rust CLI with workspace", "Next.js monorepo with Turborepo").

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] ArchitectureCatalogEntry struct with all fields: identity (name, description, tags, language, project_type), structure (folder_layout, layer_definitions, module_boundaries), rules (dependency_rules, naming_conventions, anti_patterns), quality (analysis_tool_config, baseline_thresholds), seeding (rules_seed_hints, analysis_expectations)
- [ ] Supporting enums: ProjectLanguage, ProjectType (or flexible string-based approach)
- [ ] Layer, DependencyRule, NamingConvention, AntiPattern structs for rich sub-structures
- [ ] Frontmatter YAML template (frontmatter.yaml)
- [ ] Content markdown template (content.md)
- [ ] from_file/to_file serialization matching existing governance type patterns
- [ ] from_content/to_content for in-memory round-trip
- [ ] Unit tests for creation, serialization round-trip, and field validation

## Implementation Notes

### Pattern to Follow
Follow the governance type pattern from RulesConfig, AnalysisBaseline, etc:
- Own module at `src/domain/documents/architecture_catalog_entry/mod.rs`
- Frontmatter template at `src/domain/documents/architecture_catalog_entry/frontmatter.yaml`
- Content template at `src/domain/documents/architecture_catalog_entry/content.md`
- Uses DocumentCore for shared fields
- Tera-based frontmatter rendering
- gray_matter for parsing

### Key Fields
- `core: DocumentCore` (shared metadata, tags, etc.)
- `language: String` (e.g., "rust", "typescript", "javascript")
- `project_type: String` (e.g., "cli", "web-app", "library", "monorepo")
- `folder_layout: Vec<String>` (expected directory structure)
- `layers: Vec<LayerDefinition>` (architectural layers with allowed dependencies)
- `module_boundaries: Vec<ModuleBoundary>` (module isolation rules)
- `dependency_rules: Vec<String>` (dependency direction constraints)
- `naming_conventions: Vec<String>` (naming patterns)
- `anti_patterns: Vec<String>` (known anti-patterns to avoid)
- `rules_seed_hints: Vec<String>` (RulesConfig entries to generate)
- `analysis_expectations: Vec<String>` (AnalysisBaseline entries to generate)

## Progress

*Updated during implementation*
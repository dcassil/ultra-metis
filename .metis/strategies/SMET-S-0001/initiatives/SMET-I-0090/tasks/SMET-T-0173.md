---
id: layering-axis-config-parsers-and
level: task
title: "Layering Axis Config Parsers and Boundary Extraction"
short_code: "SMET-T-0173"
created_at: 2026-03-26T18:01:57.338759+00:00
updated_at: 2026-03-26T18:15:01.782056+00:00
parent: SMET-I-0090
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0090
---

# Layering Axis Config Parsers and Boundary Extraction

## Parent Initiative

[[SMET-I-0090]]

## Objective

Implement parsers that read layering/boundary-axis config file contents and extract declared module boundaries, layer relationships, and import restrictions. Produce a layering strictness score (0-100) and extract structured layer declarations that will be used downstream for `StructureAnalysis` inference.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `LayeringStrictnessResult` struct with `score: f64` (0-100), `declared_layers: Vec<String>` (layer names extracted from configs), `declared_boundaries: Vec<BoundaryRule>` (allowed/forbidden relationships), `signals: Vec<String>`, `language: Language`
- [ ] `BoundaryRule` struct capturing source layer/module, target layer/module, and whether it's allowed or forbidden
- [ ] JS/TS parser: extracts layer definitions from dependency-cruiser configs (forbidden/allowed rules with `from`/`to` paths), Nx module boundary tags from `nx.json`/`project.json`, `eslint-plugin-boundaries` element definitions, `no-restricted-imports` patterns
- [ ] Rust parser: detects Cargo workspace members from workspace `Cargo.toml` `[workspace] members` array — each member is a boundary; extracts `deny.toml` bans section for dependency restrictions between crates
- [ ] Python parser: extracts layer contracts from import-linter config (`[tool.import_linter]` in `pyproject.toml` or standalone `.importlinter`), including contract type (layers/independence/forbidden) and module names
- [ ] Go parser: detects `depguard` rules in `.golangci.yml` (allow/deny lists per package), detects `internal/` package boundaries from file paths
- [ ] Scoring logic: mere presence of boundary config scores 40+, configs with explicit layer declarations score 60+, configs with both allowed AND forbidden rules score 80+
- [ ] `evaluate_layering(detected_configs: &DetectedConfigs, file_paths: &[String], reader: &dyn FileContentReader) -> LayeringStrictnessResult`
- [ ] Unit tests: dependency-cruiser with layer rules scores high, Cargo workspace with 4+ crates scores high, import-linter with layer contracts scores high, empty/minimal configs score low

## Implementation Notes

### Technical Approach
- Dependency-cruiser configs are JS modules — can't fully parse, but can extract `forbidden`/`allowed` arrays and `from.path`/`to.path` patterns using regex/string matching on the file content
- Nx module boundaries: parse `nx.json` for `@nx/enforce-module-boundaries` rule in `targetDefaults` or project-level configs
- For Rust, the workspace `Cargo.toml` `members` field directly gives us boundaries — each crate name becomes a `declared_layer`
- Import-linter configs in `pyproject.toml` use TOML format with `[[tool.import_linter.contracts]]` array
- `file_paths` parameter is needed for Go `internal/` detection and Rust workspace member inference from path structure

### Dependencies
- SMET-T-0171 (FileContentReader trait and DetectedConfigs types)

## Status Updates

- 2026-03-26: Implemented layering axis parsers for JS/TS (dependency-cruiser forbidden/allowed rules, Nx module boundaries, eslint-plugin-boundaries, no-restricted-imports), Rust (Cargo workspace member detection, deny.toml bans), Python (import-linter layers/forbidden/independence contracts with layer extraction), Go (internal/ packages, depguard rules). BoundaryRule type with Allowed/Forbidden kinds. 12 layering-specific tests pass.
---
id: filecontentreader-trait-and-rules
level: task
title: "FileContentReader Trait and Rules Engine Config Detection Registry"
short_code: "SMET-T-0171"
created_at: 2026-03-26T18:01:55.227605+00:00
updated_at: 2026-03-26T18:08:01.520128+00:00
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

# FileContentReader Trait and Rules Engine Config Detection Registry

## Parent Initiative

[[SMET-I-0090]]

## Objective

Create the foundational types for the rules config fast-path: the `FileContentReader` trait for abstracting file content access, the `RulesConfigRegistry` containing known config file patterns per language for both quality and layering axes, and the config detection function that scans file paths against the registry.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `FileContentReader` trait defined with `fn read_content(&self, path: &str) -> Option<String>`
- [ ] `FsContentReader` impl that reads from real filesystem
- [ ] `MockContentReader` impl for testing (accepts a HashMap of path→content)
- [ ] `RulesConfigRegistry` data structure mapping languages to config file patterns for both axes
- [ ] Registry covers minimum: JS/TS, Rust, Python, Go (extensible for Java/Kotlin, C#)
- [ ] Axis 1 (quality) config patterns: `.eslintrc.*`, `eslint.config.*`, `tsconfig.json`, `biome.json`, `clippy.toml`, `deny.toml`, `ruff.toml`, `pyproject.toml`, `mypy.ini`, `pyrightconfig.json`, `.golangci.yml`
- [ ] Axis 2 (layering) config patterns: `.dependency-cruiser.*`, `nx.json`, `.importlinter`, `depguard` presence in `.golangci.yml`, Cargo workspace member detection
- [ ] `detect_configs(file_paths: &[String], registry: &RulesConfigRegistry) -> DetectedConfigs` function
- [ ] `DetectedConfigs` struct contains per-language lists of detected quality and layering config paths
- [ ] Unit tests: detection finds correct configs from mixed file lists, handles no configs found, handles partial (quality only, layering only)
- [ ] All new code in `crates/cadre-core/src/domain/catalog/brownfield_evaluator/rules_config_analyzer.rs`

## Implementation Notes

### Technical Approach
- Create `rules_config_analyzer.rs` as a new module in the brownfield_evaluator directory
- Register it in `brownfield_evaluator/mod.rs`
- Config patterns should use filename matching (not glob) since we receive flat file path strings
- For Rust layering, detect Cargo workspace by presence of multiple `Cargo.toml` files at different depths
- For Go layering, detect `internal/` directory convention from paths + `depguard` keyword in `.golangci.yml`
- The registry should be a static/const data structure, not dynamically loaded

### Dependencies
- None — this is the foundation task

## Status Updates

- 2026-03-26: Created `rules_config_analyzer.rs` with FileContentReader trait, FsContentReader, MockContentReader, RulesConfigRegistry (27 patterns across 6 languages), detect_configs() function, DetectedConfigs result type. All 16 unit tests pass.
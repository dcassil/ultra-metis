---
id: quality-axis-config-parsers-and
level: task
title: "Quality Axis Config Parsers and Strictness Scoring"
short_code: "SMET-T-0172"
created_at: 2026-03-26T18:01:56.460364+00:00
updated_at: 2026-03-26T18:11:41.587703+00:00
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

# Quality Axis Config Parsers and Strictness Scoring

## Parent Initiative

[[SMET-I-0090]]

## Objective

Implement parsers that read quality-axis config file contents (via `FileContentReader`) and evaluate strictness. Each parser extracts signals like preset extensions, strict mode flags, and rule counts, then produces a quality strictness score (0-100).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `QualityStrictnessResult` struct with `score: f64` (0-100), `signals: Vec<String>` (human-readable explanations of what was detected), `language: Language`
- [ ] JS/TS parser: detects `strict: true` in tsconfig, `extends` with known strict presets (`eslint:recommended`, `@typescript-eslint/strict`, `plugin:@typescript-eslint/recommended-type-checked`), biome strict linting mode
- [ ] Rust parser: detects `clippy::pedantic`/`clippy::nursery` in workspace `Cargo.toml` `[lints]` section or `clippy.toml`, `deny.toml` with advisories/licenses/bans sections populated
- [ ] Python parser: detects `strict = true` in mypy config, `select = ["ALL"]` or high rule coverage in ruff, `strict` mode in pyrightconfig
- [ ] Go parser: counts enabled linters in `.golangci.yml`, scores based on ratio of key linters enabled (govet, staticcheck, revive, errcheck, gosec, gocritic)
- [ ] Each parser uses `FileContentReader` to read config contents — no direct filesystem access
- [ ] Scoring logic: preset/strict-mode detection gives high base score (70+), rule-count analysis adds granularity, missing configs score 0
- [ ] `evaluate_quality(detected_configs: &DetectedConfigs, reader: &dyn FileContentReader) -> QualityStrictnessResult`
- [ ] Unit tests per language with realistic config file contents (strict configs score high, minimal configs score low, no configs score 0)

## Implementation Notes

### Technical Approach
- Config content is plain text — parse JSON for eslint/tsconfig/biome/pyright, TOML for Cargo.toml/clippy.toml/deny.toml/ruff.toml, INI for mypy.ini/.flake8, YAML for .golangci.yml
- Use `serde_json`, `toml`, and basic string matching — avoid pulling in heavy parsing dependencies
- For YAML (golangci-lint), use `serde_yaml` or simple line-based parsing for the linters list
- Config formats that embed in other files (e.g., `[tool.ruff]` in `pyproject.toml`) need section extraction before parsing

### Dependencies
- SMET-T-0171 (FileContentReader trait and DetectedConfigs types)

## Status Updates

- 2026-03-26: Implemented quality axis parsers for JS/TS (tsconfig strict, eslint presets, biome rules), Rust (clippy pedantic/nursery, deny.toml sections, workspace lints), Python (mypy strict, ruff ALL, pyright strict, pyproject.toml sections), Go (golangci-lint key linter counting). 15 quality-specific tests pass. Combined bonuses for multi-tool configs.
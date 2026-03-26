---
id: rules-engine-config-fast-path-for
level: initiative
title: "Rules Engine Config Fast-Path for Brownfield Architecture Evaluation"
short_code: "SMET-I-0090"
created_at: 2026-03-26T17:40:44.385546+00:00
updated_at: 2026-03-26T18:25:43.290666+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0001
initiative_id: rules-engine-config-fast-path-for
---

# Rules Engine Config Fast-Path for Brownfield Architecture Evaluation Initiative

## Context

The `BrownfieldEvaluator` (in `crates/cadre-core/src/domain/catalog/brownfield_evaluator/`) currently evaluates architecture by analyzing file paths — detecting directory layers, naming conventions, test patterns, and module boundaries from the file tree. This works but requires scanning all source files and relies on structural inference.

Many well-maintained codebases already have **rules engine configurations** that explicitly declare and enforce their architecture. An ESLint config with `eslint-plugin-boundaries`, a `dependency-cruiser` config, or a Rust workspace with `cargo-deny` tells us more about architectural intent than directory names ever could. If a codebase enforces both code quality policies AND module boundary/layering rules through config-driven tooling, we can infer structure quality directly from those configs — skipping the file-level structure analysis entirely.

This initiative adds a **config-based fast path** as an early-exit optimization within `BrownfieldEvaluator::evaluate()`. It scans file paths for known rules engine config files, reads their contents to evaluate strictness, and if both quality and layering axes pass, infers a `StructureAnalysis` from the config declarations rather than from file tree scanning.

### Relationship to Existing Code

- **Modifies**: `BrownfieldEvaluator::evaluate()` — adds early-exit before `StructureAnalyzer::analyze()`
- **Produces**: Same `StructureAnalysis` type — downstream `PatternMatcher` is unchanged
- **Depends on**: Existing `StructureAnalysis`, `EvaluationResult`, `EvaluationOutcome` types
- **New input**: Will need file content access (current API only takes `&[String]` file paths)

## Goals & Non-Goals

**Goals:**
- Catalog major rules engines per language that enforce code quality and module boundaries
- Detect rules engine config files from file paths
- Read and evaluate config contents for strictness on two axes: quality enforcement and layering enforcement
- Infer `StructureAnalysis` from config declarations (layers, boundaries, conventions) when both axes pass
- Short-circuit `BrownfieldEvaluator` to skip file-level analysis when config-inferred analysis is sufficient
- Support at minimum: JS/TS, Rust, Python, Go (with extensibility for Java/Kotlin, C#)

**Non-Goals:**
- Actually executing the rules engines (we read configs, not run linters)
- Replacing the existing file-level evaluator (this is a fast path, not a replacement)
- Validating that the rules are actually passing in the codebase (we trust that enforced configs = enforced architecture)
- Supporting every possible linter/formatter (focus on ones that provide architectural signal)

## Rules Engine Catalog

### Axis 1: Code Quality Enforcement

| Language | Config Files | "Strict" Signals |
|----------|-------------|------------------|
| **JS/TS** | `.eslintrc.*`, `eslint.config.*`, `tsconfig.json`, `biome.json` | `strict: true` in tsconfig, extends `recommended`/`strict` presets, `biome` with strict linting |
| **Rust** | `clippy.toml`, `rustfmt.toml`, `deny.toml`, workspace `Cargo.toml` `[lints]` | `clippy::pedantic`, `clippy::nursery`, `deny` with advisories/licenses/bans |
| **Python** | `ruff.toml`, `pyproject.toml [tool.ruff]`, `.flake8`, `mypy.ini`, `pyrightconfig.json` | `strict = true` in mypy, `select = ["ALL"]` in ruff, `strict` mode in pyright |
| **Go** | `.golangci.yml` | Multiple linters enabled (govet, staticcheck, revive, errcheck, gosec) |
| **Java/Kotlin** | `checkstyle.xml`, `pmd.xml`, `spotbugs.xml`, `detekt.yml` | Low threshold configs, many rules enabled |
| **C#** | `.editorconfig`, `Directory.Build.props` | `TreatWarningsAsErrors`, Roslyn analyzer packages |

### Axis 2: Module Boundary / Layering Enforcement

| Language | Config Files | What They Enforce |
|----------|-------------|-------------------|
| **JS/TS** | `.dependency-cruiser.js`/`.cjs`/`.mjs`, Nx `nx.json` + `project.json` with `@nx/enforce-module-boundaries`, ESLint `eslint-plugin-boundaries` rules, `no-restricted-imports` rules | Import restrictions between layers, circular dependency prevention, module boundary tags |
| **Rust** | Cargo workspace with multiple crates (language-enforced boundaries), `cargo-deny` | Crate visibility is compile-enforced; `deny.toml` bans controls dependency graph |
| **Python** | `.importlinter` / `[tool.import_linter]` in `pyproject.toml` or `setup.cfg` | Layer contracts (e.g., "domain must not import from infrastructure"), forbidden cross-package imports |
| **Go** | `depguard` in `.golangci.yml`, `internal/` package convention | Import allow/deny lists per package; `internal/` is language-enforced boundary |
| **Java/Kotlin** | ArchUnit configs/test conventions, Gradle/Maven multi-module with dependency constraints | Architecture test rules (layered, onion, hexagonal), module dependency declarations |

### Strictness Evaluation

**Quality axis passes when**: Config extends a known strict preset OR enables a high ratio of available rules OR enables strict mode flag.

**Layering axis passes when**: At least one boundary-enforcement config exists AND it declares explicit layer/module relationships (not just "no circular deps" but actual allowed/forbidden import paths between named layers).

**Both must pass to take the fast path.** Quality-only configs still indicate a well-maintained codebase but give no architectural confidence.

## Detailed Design

### New Component: `RulesConfigAnalyzer`

A new module alongside `StructureAnalyzer` in the brownfield evaluator directory:

```
brownfield_evaluator/
├── mod.rs
├── evaluator.rs          # Modified: adds fast-path check
├── structure_analyzer.rs # Unchanged
├── pattern_matcher.rs    # Unchanged
└── rules_config_analyzer.rs  # NEW
```

**`RulesConfigAnalyzer`** responsibilities:
1. **Config Detection** — scan file paths for known config filenames, return detected configs per language
2. **Config Parsing** — read config file contents, extract relevant strictness signals (presets extended, rules enabled, layers declared)
3. **Strictness Evaluation** — score each axis (quality: 0-100, layering: 0-100) based on parsed signals
4. **Structure Inference** — when both axes pass threshold, produce a `StructureAnalysis` derived from config declarations:
   - `detected_layers` from boundary config layer declarations
   - `module_boundaries` from declared modules/packages
   - `structure_quality_score` computed from config strictness scores
   - Other fields inferred where possible, defaulted conservatively where not

### Modified Evaluate Flow

```
evaluate(file_paths, catalog_entries, short_code) {
    // NEW: Fast path attempt
    let config_result = RulesConfigAnalyzer::analyze(file_paths, content_reader);
    if config_result.quality_score >= threshold 
       AND config_result.layering_score >= threshold {
        // Infer StructureAnalysis from config declarations
        let analysis = config_result.infer_structure_analysis();
        // Skip to pattern matching with config-inferred analysis
        let match_result = PatternMatcher::match_against(&analysis, catalog_entries);
        return fast_path_result(analysis, match_result);
    }
    
    // EXISTING: Full file-level analysis (unchanged)
    let analysis = StructureAnalyzer::analyze(file_paths);
    // ... rest of existing flow
}
```

### File Content Access

The current `evaluate()` signature only takes `&[String]` file paths. To read config contents, we need a content access abstraction:

```rust
pub trait FileContentReader {
    fn read_content(&self, path: &str) -> Option<String>;
}
```

This keeps the evaluator testable (mock reader in tests) and decoupled from filesystem IO.

## Alternatives Considered

1. **Standalone capability (option B)** — Building this as an independent service usable by code-review, architecture hooks, etc. Rejected because the primary consumer is the brownfield evaluator and we want to avoid premature abstraction. Can be extracted later if needed.

2. **Research-only initiative (option C)** — Catalog rules engines first, then decide integration. Rejected because the integration point is clear and the catalog is a means to an end, not the end itself.

3. **Config presence only (no content reading)** — Just check if config files exist without reading contents. Rejected because presence alone says nothing about strictness — a minimal eslint config with 2 rules provides no architectural confidence.

## Implementation Plan

### Phase 1: Rules Engine Catalog and Detection
- Define config file patterns per language (Axis 1 and Axis 2)
- Implement `FileContentReader` trait
- Implement config file detection from file paths
- Unit tests for detection across all supported languages

### Phase 2: Config Parsing and Strictness Evaluation
- Implement parsers for each config format (JSON, YAML, TOML, JS module analysis)
- Implement quality strictness scoring per language
- Implement layering strictness scoring per language
- Unit tests for strictness evaluation with real-world config examples

### Phase 3: Structure Inference and Fast-Path Integration
- Implement `StructureAnalysis` inference from config declarations
- Integrate fast path into `BrownfieldEvaluator::evaluate()`
- Add `FileContentReader` parameter to evaluate signature
- Integration tests verifying fast-path triggers correctly and produces valid results
- Integration tests verifying fallback to full analysis when configs are insufficient

## Task Breakdown

| Task | Title | Dependencies |
|------|-------|-------------|
| SMET-T-0171 | FileContentReader Trait and Rules Engine Config Detection Registry | None |
| SMET-T-0172 | Quality Axis Config Parsers and Strictness Scoring | T-0171 |
| SMET-T-0173 | Layering Axis Config Parsers and Boundary Extraction | T-0171 |
| SMET-T-0174 | StructureAnalysis Inference from Config Declarations | T-0171, T-0172, T-0173 |
| SMET-T-0175 | Fast-Path Integration into BrownfieldEvaluator | T-0174 |

**Execution order**: T-0171 first, then T-0172 and T-0173 can run in parallel, then T-0174, then T-0175.

## Status Updates

- 2026-03-26: All 5 tasks completed. Implementation adds `RulesConfigAnalyzer` as a fast-path in `BrownfieldEvaluator`. New `evaluate_with_reader()` method enables the fast path; existing `evaluate()` is unchanged. 51 new unit tests + all 957 workspace tests pass. Languages supported: JS/TS, Rust, Python, Go (extensible for Java/Kotlin, C#).
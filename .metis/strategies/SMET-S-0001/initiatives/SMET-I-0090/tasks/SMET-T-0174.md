---
id: structureanalysis-inference-from
level: task
title: "StructureAnalysis Inference from Config Declarations"
short_code: "SMET-T-0174"
created_at: 2026-03-26T18:01:58.387199+00:00
updated_at: 2026-03-26T18:17:08.017591+00:00
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

# StructureAnalysis Inference from Config Declarations

## Parent Initiative

[[SMET-I-0090]]

## Objective

Implement the logic that combines quality and layering strictness results to produce a `StructureAnalysis` — the same type used by the existing `PatternMatcher`. The inferred analysis derives its fields from config declarations rather than file tree scanning, allowing the fast path to feed directly into the existing pattern matching pipeline.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `RulesConfigAnalysisResult` struct combining `QualityStrictnessResult` + `LayeringStrictnessResult` + `inferred_analysis: Option<StructureAnalysis>`
- [ ] `infer_structure_analysis()` method that produces a `StructureAnalysis` from config results when both axes pass threshold
- [ ] `detected_layers` populated from `LayeringStrictnessResult.declared_layers` (e.g., dependency-cruiser layer names, Cargo workspace member names, import-linter contract modules)
- [ ] `module_boundaries` populated from `LayeringStrictnessResult.declared_boundaries` source/target modules
- [ ] `structure_quality_score` computed as weighted combination: 50% quality score + 50% layering score (since both are required)
- [ ] `top_level_dirs` inferred from file paths (reuse logic from existing `StructureAnalyzer` or extract shared helper)
- [ ] `file_naming_convention` inferred from file paths (reuse existing logic)
- [ ] `test_pattern` inferred from file paths (reuse existing logic)
- [ ] `has_src_root`, `total_files`, `depth_distribution` computed from file paths (lightweight, no content reads needed)
- [ ] Configurable thresholds for both axes via `RulesConfigAnalyzerConfig` (default: quality >= 70, layering >= 60)
- [ ] `RulesConfigAnalyzer::analyze()` top-level method that orchestrates: detect → evaluate quality → evaluate layering → infer analysis
- [ ] Unit tests: both axes pass → analysis produced with correct layers/boundaries; one axis fails → `inferred_analysis` is `None`; thresholds are configurable

## Implementation Notes

### Technical Approach
- Several `StructureAnalysis` fields (top_level_dirs, naming convention, test pattern, depth distribution) don't come from config files — they come from file paths. Consider extracting these computations from `StructureAnalyzer` into shared utility functions that both `StructureAnalyzer::analyze()` and `infer_structure_analysis()` can use, OR simply call the relevant parts of the existing analysis for these fields.
- The key difference is that `detected_layers`, `module_boundaries`, and `structure_quality_score` come from config analysis instead of file tree inference — these are the fields that make the fast path more accurate than the slow path.
- Keep the threshold defaults conservative: it's better to fall through to full analysis than to skip it when configs aren't strict enough.

### Dependencies
- SMET-T-0171 (foundation types)
- SMET-T-0172 (quality scoring)
- SMET-T-0173 (layering scoring)

## Status Updates

- 2026-03-26: Implemented RulesConfigAnalyzer orchestrator with configurable thresholds, RulesConfigAnalysisResult combining both axes, infer_structure_analysis() that produces StructureAnalysis from config declarations + file path metadata. 8 orchestrator tests pass (both axes pass, quality fails, layering fails, no configs, rust workspace, file path metadata, custom thresholds, default config). Total: 51 tests.
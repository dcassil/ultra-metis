---
id: implement-qualitygateconfig-and
level: task
title: "Implement QualityGateConfig and threshold domain types"
short_code: "SMET-T-0021"
created_at: 2026-03-17T00:18:40.851935+00:00
updated_at: 2026-03-17T00:23:58.603572+00:00
parent: SMET-I-0022
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: SMET-I-0022
---

# Implement QualityGateConfig and threshold domain types

## Parent Initiative

[[SMET-I-0022]] — Quality Gates and Phase Transition Integration

## Objective

Define and implement the Rust domain types for quality gate configuration: `QualityGateConfig`, `GateThreshold`, `ThresholdType`, `MetricGateRule`, and supporting enums. These types define per-project thresholds for different quality metrics, configurable per document type and phase transition.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `QualityGateConfig` struct with project-level gate settings, default thresholds, and per-transition overrides
- [ ] `GateThreshold` struct supporting absolute, relative (percentage regression), and trend threshold types
- [ ] `ThresholdType` enum: `Absolute(f64)`, `RelativeRegression(f64)`, `Trend(TrendDirection)`
- [ ] `MetricGateRule` struct tying a metric name to a threshold and severity (blocking vs advisory)
- [ ] `GateSeverity` enum: `Blocking`, `Advisory`
- [ ] `TransitionGateConfig` struct for per-transition gate overrides (e.g., stricter gates for active→completed)
- [ ] All types derive Serialize, Deserialize, Debug, Clone, PartialEq
- [ ] Follows existing governance type patterns (private `DocumentCore`, standalone validate/to_content/from_content)
- [ ] Markdown+YAML frontmatter serialization round-trips without data loss

## Implementation Notes

### Technical Approach
- Create new module `quality_gate_config/` under `domain/documents/`
- Follow the same pattern as `validation_policy/mod.rs`: private `DocumentCore`, standalone methods
- `QualityGateConfig` frontmatter fields: `level: quality_gate_config`, `schema_version: 1`, `default_thresholds` (list), `transition_overrides` (map)
- Threshold types stored as tagged YAML: `{ type: absolute, value: 10.0 }`, `{ type: relative, value: 5.0 }`, `{ type: trend, direction: improving }`
- Content body sections: `## Default Gates`, `## Transition Overrides`, `## Notes`

### Dependencies
- SMET-I-0021 (completed) — QualityRecord types that gates evaluate against
- Existing `DocumentCore`, `DocumentMetadata`, `FrontmatterParser` infrastructure

## Status Updates

### 2026-03-17
- Created `quality_gate_config/` module with frontmatter.yaml, content.md, acceptance_criteria.md, mod.rs
- Implemented types: `GateSeverity`, `ThresholdType`, `TrendRequirement`, `MetricGateRule`, `TransitionGateConfig`, `QualityGateConfig`
- All types follow governance type pattern (private DocumentCore, standalone methods)
- Convenience constructors: `blocking_absolute`, `blocking_relative`, `advisory_absolute`
- `thresholds_for_transition()` resolves per-transition overrides with fallback to defaults
- Full YAML frontmatter serialization with Tera templates
- Registered in `domain/documents/mod.rs` and exported from `lib.rs`
- 8 unit tests all passing: creation, validation, severity parsing, threshold parsing, constructor helpers, transition matching, override resolution, round-trip serialization
- `cargo build` and `cargo test` clean
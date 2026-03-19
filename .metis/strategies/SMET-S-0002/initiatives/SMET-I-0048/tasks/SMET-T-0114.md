---
id: implement-results-persistence-and
level: task
title: "Implement results persistence and comparison report generator"
short_code: "SMET-T-0114"
created_at: 2026-03-17T22:06:53.737376+00:00
updated_at: 2026-03-17T22:25:40.060556+00:00
parent: SMET-I-0048
blocked_by: [SMET-T-0112]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0048
---

# Implement results persistence and comparison report generator

## Parent Initiative

[[SMET-I-0048]]

## Objective

Persist benchmark run results as JSON and generate a markdown comparison report from two runs (autonomous vs validated). The report format should mirror `benchmarks/REPORT.md` — tables comparing metrics side-by-side with a summary and recommendations.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `BenchmarkRun` serializes to JSON and saves to `benchmarks/practical/results/run_<timestamp>.json`
- [ ] `benchmarks/practical/results/latest_run.json` symlink always points to most recent run
- [ ] `generate_comparison_report()` takes two `BenchmarkRun`s and writes `benchmarks/practical/results/comparison_<timestamp>.md`
- [ ] Report includes: per-initiative tables, aggregate metrics, token overhead %, quality delta, ROI, error detection rate
- [ ] Report format mirrors existing `benchmarks/REPORT.md` (tables, executive summary, findings)
- [ ] `results_history.csv` appended with one row per run (for trend tracking)
- [ ] Unit tests verify JSON round-trip and report contains required sections

## Implementation Notes

### Technical Approach

Create `benchmarks/practical/src/reports.rs`:

```rust
pub fn save_run(run: &BenchmarkRun, results_dir: &Path) -> anyhow::Result<PathBuf>
pub fn generate_comparison_report(
    autonomous: &BenchmarkRun,
    validated: &BenchmarkRun,
    output_path: &Path,
) -> anyhow::Result<()>
pub fn append_history(run: &BenchmarkRun, history_path: &Path) -> anyhow::Result<()>
```

**Report structure** (mirrors REPORT.md):
```markdown
# Practical Benchmark Comparison Report

**Date**: ...
**Scenario**: File Processing Toolkit

## Executive Summary
...

## Per-Initiative Results
| Initiative | Autonomous Tokens | Validated Tokens | Quality Delta |
...

## Aggregate Metrics
| Metric | Autonomous | Validated | Delta |
...

## Gate Effectiveness
...

## Recommendations
...
```

### Dependencies
- SMET-T-0112 (need real run data to test serialization with)
- Can be implemented in parallel with SMET-T-0113

### Risk Considerations
- Keep report format stable — changes break historical comparisons
- CSV history format must be append-only, never rewrite old rows

## Status Updates

### Session complete
- Created `benchmarks/practical/src/reports.rs` with `save_run()`, `generate_comparison_report()`, `append_history()`
- `save_run()`: serializes BenchmarkRun to `run_<timestamp>.json`, copies to `latest_run.json`
- `generate_comparison_report()`: generates markdown comparison report mirroring `benchmarks/REPORT.md` format (Executive Summary, Per-Initiative Results table, Aggregate Metrics table, Gate Effectiveness, Recommendations)
- `append_history()`: appends CSV row to `results_history.csv` with header on first write
- Report uses `BenchmarkAnalysis::compare()` from analysis.rs for ROI/quality delta calculations
- 5 unit tests: save creates file, latest_run updated, JSON round-trip, report has required sections, CSV history appended correctly
- All 34 tests pass, clean build (no warnings)
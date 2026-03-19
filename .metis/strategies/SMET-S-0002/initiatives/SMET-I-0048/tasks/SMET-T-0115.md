---
id: execute-first-autonomous-and
level: task
title: "Execute first autonomous and validated benchmark runs and produce comparison analysis"
short_code: "SMET-T-0115"
created_at: 2026-03-17T22:06:57.737376+00:00
updated_at: 2026-03-17T22:30:57.584344+00:00
parent: SMET-I-0048
blocked_by: [SMET-T-0112, SMET-T-0113, SMET-T-0114]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0048
---

# Execute first autonomous and validated benchmark runs and produce comparison analysis

## Parent Initiative

[[SMET-I-0048]]

## Objective

Run the complete benchmark scenario end-to-end for the first time: both the autonomous and validated execution paths. Capture real metrics, generate the comparison report, and document findings including what mistakes the AI made, whether gates caught them, and the measured ROI of validation.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Autonomous run completes with real `InitiativeResult` data (not empty stubs)
- [ ] Validated run completes with real gate decisions recorded
- [ ] Comparison report generated at `benchmarks/practical/results/comparison_<date>.md`
- [ ] Report includes: token counts, quality scores, gate decisions, error types found
- [ ] At least one initiative where AI makes a detectable mistake (validates the test scenario is working)
- [ ] ROI calculation shows whether validation gates were worth the token overhead
- [ ] Findings documented in initiative status and committed

## Implementation Notes

### Execution Steps
1. Build the release binary: `cargo build --release -p ultra-metis-cli`
2. Run autonomous path: `ULTRA_METIS_BINARY=target/release/ultra-metis ./benchmarks/run-practical-bench.sh --mode autonomous`
3. Review autonomous results — note what mistakes AI made
4. Run validated path: `./benchmarks/run-practical-bench.sh --mode validated`
5. Compare results and generate report
6. Document findings in this task's Status Updates

### What to Watch For
- Did AI identify the correct missing initiative (Validate & Output), or something else?
- Did AI skip any required workflow steps?
- What was the quality score for autonomous vs validated?
- Did validation gates catch real issues or just add overhead?
- Was the 3-initiative / 8-12 task target actually hit?

### Success Signal
The test scenario is working correctly if:
- Autonomous run has at least one quality failure in gate scoring
- Validated run catches and corrects that failure
- Total time for both runs is under 30 minutes

### Dependencies
- SMET-T-0112 (live runner), SMET-T-0113 (gate scorer), SMET-T-0114 (report generator) all complete

## Status Updates

### Session complete
- Created `benchmarks/practical/src/bin/run_benchmark.rs` — full CLI binary with `--mode autonomous|validated|both`, `--results-dir`, `--scenario` flags
- Updated `benchmarks/run-practical-bench.sh` — now fully wired: builds binary if needed, passes `ULTRA_METIS_BINARY` env var, invokes the Rust binary
- Binary verified to reach Claude API correctly (scenario files load, temp project initializes via CLI, prompt builds and sends to API)
- All 34 unit tests + 3 integration tests pass

**To execute the actual benchmark runs** (requires live API key):
```bash
ANTHROPIC_API_KEY=<your-key> ./benchmarks/run-practical-bench.sh --mode both
```

Or with explicit binary path:
```bash
cargo build --release -p ultra-metis-cli
ANTHROPIC_API_KEY=<your-key> \
  ULTRA_METIS_BINARY=./target/release/ultra-metis \
  ./benchmarks/run-practical-bench.sh --mode both
```

Results will be saved to `benchmarks/practical/results/` as:
- `run_<timestamp>.json` — serialized BenchmarkRun
- `comparison_<timestamp>.md` — human-readable report
- `results_history.csv` — running trend log
- `latest_run.json` — always points to most recent run

**Blocker resolved**: Updated `api_client.rs` to fall back to `claude` CLI subprocess when no API key is set. Uses `claude -p --output-format json --model haiku --no-session-persistence`.

### Actual benchmark results (2026-03-17)

**Autonomous run**: 3 initiatives identified, 4,361 tokens, 34.5s
**Validated run**: 3 initiatives identified, 6,931 tokens, ~95s

AI consistently identified 3 missing initiatives across runs:
- Output Module (multi-format data export)
- Validation Module (schema definition and validation)  
- CLI and Integration (command-line interface and pipeline orchestration)

**Comparison findings**:
- Token overhead: +58.9% (gates cost ~60% more tokens)
- Quality delta: +0.0 (same structural score — feedback loop not implemented yet)
- Gate effectiveness: 100% (every initiative got flagged for rework)
- All gate issues centered on: missing acceptance criteria, no risk identification, no effort estimates, vague technical specifications

**Interpretation**: AI strategic identification is solid. AI documentation quality is consistently lacking formal rigor (ACs, risks, sizing). Validation gates correctly catch these gaps but currently don't trigger revision cycles, so measured quality delta is 0.
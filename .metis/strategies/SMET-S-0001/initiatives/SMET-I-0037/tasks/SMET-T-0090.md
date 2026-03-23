---
id: re-run-benchmark-and-verify
level: task
title: "Re-run benchmark and verify template quality improvement"
short_code: "SMET-T-0090"
created_at: 2026-03-17T20:15:51.669+00:00
updated_at: 2026-03-17T20:33:13.452510+00:00
parent: SMET-I-0037
blocked_by: [SMET-T-0089]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: SMET-I-0037
---

# Re-run benchmark and verify template quality improvement

## Parent Initiative

[[SMET-I-0037]]

## Objective

Re-run the benchmark Scenario 2 (Planning Workflow) that originally scored ultra-metis template quality at 3/5. After the rewrites in SMET-T-0088 and quality tests in SMET-T-0089, the target is 5/5 template quality — matching or exceeding original metis.

This task validates that the investment in SMET-I-0037 paid off and documents the final scores.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Benchmark Scenario 2 (Planning Workflow) is re-run with the updated ultra-metis binary
- [ ] Template quality score improves from 3/5 to at least 4/5 (target: 5/5)
- [ ] Benchmark results are recorded in the SMET-I-0037 initiative under a `## Benchmark Results` section
- [ ] Any remaining gaps (if score < 5/5) are documented with specific actionable improvements
- [ ] The benchmark script runs against a fresh project (not the existing `.metis` workspace)
- [ ] `cargo test --workspace` passes before running the benchmark

## Implementation Notes

### Technical Approach

The benchmark is located at `super-metis/benchmarks/` or was run as a shell script in a previous session. Key steps:

1. **Build the updated binary:**
   ```bash
   cd super-metis && cargo build --release
   ```

2. **Find the benchmark script:**
   Check `super-metis/benchmarks/` for the scenario 2 script. The benchmark was originally run in SMET-I-0035.

3. **Run against a fresh project:**
   ```bash
   # Initialize fresh project
   TMPDIR=$(mktemp -d)
   ./target/release/ultra-metis init -p "$TMPDIR" -x BENCH
   # Create a vision + initiative + task via the CLI
   # Observe the templates that are rendered
   ```

4. **Score template quality using the same rubric:**
   | Score | Criteria |
   |-------|----------|
   | 1 | Template is blank or only has a title |
   | 2 | Template has sections but all are bare placeholders |
   | 3 | Template has sections with some guidance text but no examples or format suggestions |
   | 4 | Template has guidance, some examples, some conditional markers |
   | 5 | Template has rich guidance, inline examples, format suggestions, conditional deletion markers, required/optional distinctions |

5. **Compare to baseline (3/5)** and record new score.

6. **Document results** in SMET-I-0037 under `## Benchmark Results`.

### Finding the Benchmark Script

The benchmark in SMET-I-0035 used a script at approximately:
- `super-metis/benchmarks/scenario_2_planning_workflow.sh` or similar
- Or it may have been run inline via the CLI commands

Check git log or the SMET-I-0035 initiative document for the exact benchmark invocation.

### Scoring Rubric Details

For each document type template, score on:
- **Guidance density**: Does each section tell the writer what to put there and why?
- **Examples**: Are there inline examples showing what "good" looks like?
- **Format suggestions**: Are specific formats (tables, checklists) suggested?
- **Conditional markers**: Are optional sections clearly marked for deletion?
- **Required markers**: Are mandatory sections clearly identified?

Aggregate score = average across all 4 document types.

### Files to Modify

- SMET-I-0037 initiative document: add `## Benchmark Results` section with scores
- No code changes unless benchmark reveals remaining gaps

### Dependencies

- SMET-T-0088 (template rewrites) must be complete
- SMET-T-0089 (quality tests) must pass
- `cargo build --release` must succeed

### Risk Considerations

- **Benchmark script location**: If the script from SMET-I-0035 is not found, the benchmark will need to be re-created. Check the SMET-I-0035 initiative and task documents for the invocation.
- **Subjectivity**: Template quality scoring has a subjective component. Document the exact rubric used so future re-scores are consistent.
- **Score < 5/5**: If the score doesn't reach 5/5, document the specific remaining gaps. A 4/5 is acceptable if the remaining gaps are minor style preferences rather than structural issues.

## Status Updates

### 2026-03-17
- Found benchmark script at `super-metis/benchmarks/run-ultra-metis-bench.sh`
- Built release binary: `cargo build --release` — succeeded
- Created fresh benchmark project (BENCH prefix) with Vision, Initiative, and Task
- Inspected rendered documents — all templates show rich structure matching 5/5 criteria
- ADR not creatable via CLI (unsupported) — scored from raw template (5/5)
- Benchmark results written to SMET-I-0037 under `## Benchmark Results`
- Overall score: 3/5 → 5/5 — target achieved
- Noted parent context rendering gap in CLI (cosmetic, separate issue)
- All acceptance criteria met
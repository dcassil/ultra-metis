---
id: practical-benchmark-ai-execution
level: initiative
title: "Practical Benchmark: AI Execution Quality and Strategic Completeness"
short_code: "SMET-I-0048"
created_at: 2026-03-17T21:32:24.190289+00:00
updated_at: 2026-03-18T03:48:34.924046+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"
  - "#feature-benchmarks"
  - "#category-parity-migration"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0002
initiative_id: practical-benchmark-ai-execution
---

# Practical Benchmark: AI Execution Quality and Strategic Completeness Initiative

## Context

Current benchmarks (SMET-I-0035, SMET-I-0036, SMET-I-0037) test whether CADRE generates valid documents and system correctness. This initiative extends benchmarking to measure the **real-world value proposition**: how well can an AI use CADRE docs to successfully build complex systems, recognize scope gaps, and execute complete workflows?

We need two parallel benchmark setups:
1. **System benchmark** — Does CADRE itself work? (existing tests)
2. **Practical benchmark** — Does CADRE help AI succeed in the wild? (this initiative)

## Goals & Non-Goals

**Goals:**
- Design a realistic scenario (File Processing Toolkit) that exposes workflow, integration, spec, and documentation mistakes
- Implement test harness that runs both autonomous (baseline) and validated (gated) execution paths
- Measure and compare: token efficiency, time, code quality, instruction adherence, error detection rates
- Quantify the ROI of validation gates vs cost of mistakes propagating
- Create a foundation for ongoing AI effectiveness benchmarking

**Non-Goals:**
- Building the actual File Processing Toolkit (it's test infrastructure, not product code)
- UI-focused benchmarks (noted as future work with comment in code)
- Measuring performance/latency of generated code (focus is on AI execution quality)

## Test Scenario: File Processing Toolkit

### Domain Overview
Build a system that ingests files in multiple formats (CSV, JSON, YAML), transforms data through a chain of operations (filter, aggregate, join), and validates + exports results in multiple formats. This creates realistic surface area for AI mistakes:
- **Workflow mistakes** — skipping decomposition, coding before design docs
- **Integration mistakes** — components built in isolation, incompatible I/O contracts
- **Specification mistakes** — missing edge cases, building the wrong thing
- **Documentation mistakes** — poor docs, wrong format, incomplete records

### Initiative Breakdown (3 initiatives, 2-4 tasks each = 8-12 total tasks)
1. **Parse Module** — Support CSV, JSON, YAML input with format detection and unified data model
2. **Transform Module** — Implement filter, aggregate, join, and custom transform operations
3. **Validate & Output** — Schema validation, error handling, multi-format export

### Scenario Flow

**Phase 1: Strategic Completeness Test**
- Provide: Vision document + 2/3 initiative summaries (Parse + Transform)
- AI task: **Assess whether existing initiatives are sufficient. Create any additional initiatives you believe are needed based on the vision and existing work.**
- Measure: Does AI identify gaps? Are proposed initiatives well-justified? Do they cover real gaps or add unnecessary scope?

**Phase 2: Autonomous Execution (Baseline Run)**
- AI executes all initiatives (2 or 3+) end-to-end with no human gates
- Records: tokens, time, code generated, tests written, docs created
- Measures: code quality, test coverage, doc accuracy, instruction adherence

**Phase 3: Validated Execution (Gated Run)**
- Same scenario, but validation gates after each initiative:
  - AI self-validates against acceptance criteria
  - Human/checker reviews quality gate decisions
  - Records corrections, rework costs, gate outcomes
- Measures: same as Phase 2 + gate effectiveness

**Phase 4: Comparative Analysis**
- Compare autonomous (Phase 2) vs validated (Phase 3):
  - Token efficiency (validation overhead?)
  - Quality delta (did gates improve outcomes?)
  - Error types & detection rates (what mistakes were made and caught?)
  - Strategic assessment quality (did AI identify the right initiative set?)

### Success Criteria
✓ 3 initiatives generate 8-12 concrete tasks
✓ Each task is 1-3 days of work (can run in CI in <30 min per run)
✓ Tools have clear I/O contracts with defined inputs/outputs (testable)
✓ Scenario exposes: workflow mistakes, integration bugs, spec gaps, doc quality issues
✓ Both autonomous and validated runs complete in <30 min total
✓ Metrics collected for each run: tokens, time, code quality, test coverage, doc accuracy
✓ Comparative analysis generated showing ROI of validation gates

### Future Enhancement
Add UI-focused benchmark variant (e.g., dashboard for task management or data explorer)

## Detailed Design

### Test Scenario Structure
The File Processing Toolkit is a Rube Goldberg machine with intentional complexity:
- **Parse Module** reads 3 formats with format detection and unified data model
- **Transform Module** chains operations with interdependencies
- **Validate & Output** ensures data integrity with format-specific rules

This design ensures AI faces realistic challenges:
- Format incompatibilities if not carefully specified
- Type mismatches at module boundaries if not properly designed
- Missing error paths if specifications are incomplete
- Poor documentation that makes integration difficult

### Execution Model

**Autonomous Run** (Baseline):
1. AI receives vision + 2 initiative summaries
2. AI assesses completeness and creates additional initiatives if needed
3. AI executes all initiatives through design → tasks → code → tests
4. No human intervention, gates, or validation pauses
5. Records all tokens, time, generated artifacts

**Validated Run** (Gated):
1. Same setup and initial work
2. After each initiative completes:
   - AI self-validates against acceptance criteria
   - Checker (human/automated) reviews quality gate
   - Decision: approve or request rework
3. Records gate decisions, corrections, rework costs

### Metrics Collection

Per run, track:
- **Efficiency**: Total tokens, time elapsed, cost per initiative
- **Code Quality**: Lines of code, test coverage %, cyclomatic complexity
- **Documentation**: Words per doc, adherence to format, completeness vs spec
- **Correctness**: Tests passed/failed, code review findings, spec compliance
- **Instruction Adherence**: Did AI follow all requirements? Skip any steps? Miss edge cases?
- **Strategic Assessment**: Did AI propose right initiative set? Justify additions well?

Comparison metrics:
- Tokens autonomous vs validated (overhead of gates)
- Quality improvement from validation (delta across metrics)
- Error detection rate (what % of errors did gates catch?)
- Rework cost (total tokens spent on corrections in gated run)
- ROI: Is quality improvement worth validation overhead?

## Alternatives Considered

1. **Single AI run without validation gates** — Rejected because it doesn't measure the value of validation; can't assess error detection effectiveness
2. **Only validation run (no baseline)** — Rejected because you can't measure validation overhead without a control
3. **Simpler scenario (CRUD app instead of Rube Goldberg)** — Rejected because it wouldn't expose enough mistakes for meaningful measurement
4. **UI benchmark instead of File Processing** — Deferred as future work; noted in code for later

## Status Updates

### Tool Comparison Implementation Complete (2026-03-17)

Added cadre vs original metis template quality comparison benchmark:
- `benchmarks/practical/src/doc_quality.rs` — Markdown document quality scorer (placeholder detection, section fill analysis, completeness %)
- `benchmarks/practical/src/tool_comparison.rs` — Runs same 3-module scenario through each tool's template, asks Claude to fill each, scores results
- `benchmarks/practical/src/bin/run_tool_comparison.rs` — Binary entry point
- All 39 tests pass (including 4 new tool_comparison tests + 4 doc_quality tests)

Run with: `cargo run -p practical-benchmark --bin run_tool_comparison -- --results-dir benchmarks/practical/results`

### Phase 1 Complete (2026-03-17)

Initial implementation committed (b1411f6). All phases of the plan executed:
- Scenario docs created: vision.md, parse-initiative.md, transform-initiative.md, spec.md
- Rust harness crate: types, autonomous runner, gated runner, metrics collector, analysis engine
- 8 tests pass (5 unit + 3 integration)
- run-practical-bench.sh entry point script
- Sample results JSON with expected metrics shape

**What's stubbed (next step):** Runners have placeholder AI integration — they return synthetic results. Real execution requires wiring to an actual AI session to capture live tokens/artifacts. That's the next phase of this initiative.

### Decomposed (2026-03-17)

4 tasks created covering all remaining work:
- **SMET-T-0112**: Wire runners to live cadre CLI + Claude API
- **SMET-T-0113**: Implement validation gate scorer (blocked on T-0112)
- **SMET-T-0114**: Results persistence + comparison report generator
- **SMET-T-0115**: Execute first full benchmark runs (blocked on T-0112, T-0113, T-0114)

## Implementation Plan

### Phase 1: Scenario Design & Specification (Discovery Phase)
- Create detailed File Processing Toolkit specification with all edge cases and requirements
- Document 3 initiatives with full acceptance criteria, detailed specs, and task definitions
- Create vision document and 2/3 initiative summaries as test inputs
- Define all metrics collection points and comparison calculations

### Phase 2: Benchmark Harness Implementation (Design & Ready Phases)
- Build test runner that feeds scenario to AI and captures all tokens, time, artifacts
- Implement validation gate logic (self-validation + checker review mechanism)
- Create metric collection and analysis engine
- Set up results comparison and ROI calculation

### Phase 3: Initial Run & Analysis (Active Phase)
- Execute both autonomous and validated runs with benchmark scenario
- Analyze results, generate comparison metrics, ROI analysis
- Document findings and recommendations

### Phase 4: Integration & CI/CD (Completed Phase)
- Integrate benchmark into test suite for recurring runs
- Set up regression detection for quality degradation
- Add benchmark results to project metrics dashboard
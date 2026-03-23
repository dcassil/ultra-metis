---
id: benchmark-test-suite-ultra-metis
level: initiative
title: "Benchmark Test Suite: Ultra-Metis vs Original Metis Comparison"
short_code: "SMET-I-0035"
created_at: 2026-03-17T03:00:24.948712+00:00
updated_at: 2026-03-17T18:40:07.280402+00:00
parent: SMET-S-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: ultra-metis-core-engine-repo
initiative_id: benchmark-test-suite-ultra-metis
---

# Benchmark Test Suite: Ultra-Metis vs Original Metis Comparison Initiative

## Context

Ultra-metis is intended to be a better metis. Before shipping it, we need hard evidence that it actually IS better. This initiative creates a structured benchmark suite that runs identical real-world scenarios through both the original metis plugin and ultra-metis, measuring:

- **Speed**: Wall clock time, MCP tool call latency, time-to-first-result
- **Token usage**: Total tokens consumed per workflow (input + output), tokens per document operation
- **Output quality**: Document completeness, template quality, structural correctness
- **Code quality**: When used to drive code generation tasks — compile success, test pass rate, code review scores
- **Workflow fidelity**: Phase transitions, hierarchy enforcement, validation correctness

This depends on SMET-I-0034 (local installation) being complete so ultra-metis is actually runnable.

## Goals & Non-Goals

**Goals:**
- Design a set of 5-8 reproducible benchmark scenarios covering common workflows
- Build a benchmark harness that runs each scenario against both metis and ultra-metis
- Capture quantitative metrics: speed (ms), tokens (count), document quality (scored rubric)
- Capture qualitative metrics: code quality scoring, document completeness grading
- Produce a comparison report for each benchmark run
- Make benchmarks repeatable so they can be re-run after changes

**Non-Goals:**
- Benchmarking against non-metis tools (just metis vs ultra-metis)
- Optimizing ultra-metis performance (that's a separate initiative if needed)
- Load testing or concurrent user testing
- Automated CI integration (manual runs first)

## Detailed Design

### Benchmark Scenarios

**Scenario 1: Project Bootstrap**
- Initialize a new project from scratch
- Measure: time to initialize, token usage, config quality
- Run with both: `metis initialize_project` vs `ultra-metis init`

**Scenario 2: Planning Workflow (Vision → Initiative → Tasks)**
- Create a vision, create an initiative under it, decompose into 3 tasks
- Measure: total tokens, time per operation, document content quality (rubric-scored)
- Evaluate: template quality, placeholder elimination, structural correctness

**Scenario 3: Phase Transition Flow**
- Take an initiative through discovery → design → ready → decompose → active → completed
- Measure: time per transition, validation correctness, error message quality on invalid transitions

**Scenario 4: Code Generation Task Execution**
- Give both systems the same coding task (e.g., "implement a URL shortener in Python")
- Use Ralph loop (metis) vs ultra-metis task execution
- Measure: tokens consumed, time to completion, code quality (does it compile, tests pass, lint clean)

**Scenario 5: Search and Query**
- Pre-populate with 20+ documents, then search/list/filter
- Measure: response time, result accuracy, token usage per query

**Scenario 6: Error Handling**
- Attempt invalid operations (skip phases, create orphan tasks, invalid edits)
- Measure: error message quality, recovery guidance, tokens wasted on errors

**Scenario 7: Full Feature Build (End-to-End)**
- Complete a realistic feature from vision to committed code
- Measure everything: total tokens, total time, code quality score, document trail quality
- This is the headline benchmark

### Benchmark Harness

- Shell scripts or Python scripts that drive Claude Code sessions
- Each scenario produces a structured JSON result: `{ scenario, system, metrics: { time_ms, tokens_input, tokens_output, quality_scores } }`
- Comparison script generates a side-by-side report (markdown table)
- Quality scoring uses a rubric (1-5 scale) for subjective measures, evaluated by a review agent

### Quality Rubric

| Dimension | 1 (Poor) | 3 (Adequate) | 5 (Excellent) |
|-----------|----------|--------------|---------------|
| Document completeness | Template placeholders remain | Most sections filled | All sections filled with relevant content |
| Structural correctness | Missing required fields | Fields present, some issues | All fields correct, hierarchy valid |
| Code quality | Doesn't compile | Compiles, basic tests pass | Clean code, comprehensive tests, lint-clean |
| Error messages | Cryptic or missing | Identifies the problem | Identifies problem + suggests fix |
| Workflow efficiency | Many wasted tokens/retries | Mostly direct path | Clean execution, minimal waste |

### Metrics Collection

- **Token counting**: Use Claude API usage headers or `/usage` command output
- **Timing**: `time` wrapper on each MCP tool call or CLI command
- **Quality scoring**: Automated where possible (compile check, test pass), rubric-scored by review agent for subjective dimensions

## Alternatives Considered

1. **Manual ad-hoc comparison**: Rejected — not reproducible, subject to cherry-picking.
2. **Automated CI benchmarks only**: Rejected — need qualitative assessment too, not just metrics.
3. **Single benchmark scenario**: Rejected — need breadth to catch strengths/weaknesses in different areas.

## Discovery Findings

- Ultra-metis CLI binary: `/Users/danielcassil/projects/ultra-metis/super-metis/target/release/ultra-metis` (commands: init, list, read, create, edit, transition, search, archive)
- Original metis: available via MCP tools (`mcp__plugin_metis_metis__*`)
- Benchmark approach: shell scripts with `time` and `date` for timing, byte counting for output size, manual rubric scoring
- Adjusted scenarios: Dropping Scenario 4 (Code Generation) and Scenario 7 (Full Feature Build) since they require driving full Claude sessions which isn't practical in a benchmark script. Keeping scenarios 1-3, 5-6 plus a combined end-to-end scenario.

## Implementation Plan

Phase 1: Design benchmark scenarios and scoring rubric
Phase 2: Build benchmark harness (scripts to drive scenarios and collect metrics)
Phase 3: Run baseline benchmarks against original metis
Phase 4: Run same benchmarks against ultra-metis
Phase 5: Generate comparison report
Phase 6: Identify gaps and file improvement tasks
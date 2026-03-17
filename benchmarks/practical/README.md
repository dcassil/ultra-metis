# Practical Benchmark: File Processing Toolkit

This benchmark measures how well AI executes Super-Metis workflows end-to-end, comparing autonomous execution vs validation-gated execution.

## Scenario

**File Processing Toolkit** — A system that ingests data in multiple formats (CSV, JSON, YAML), transforms it through chainable operations, validates schemas, and exports results.

**3 Initiatives, 8-12 Tasks:**
1. Parse Module — Multi-format ingestion and unified data model
2. Transform Module — Chainable filter, aggregate, and join operations
3. Validate & Output — Schema validation and multi-format export

## Running the Benchmark

### Full Run (both autonomous and validated)
```bash
./run-practical-bench.sh
```

This will:
1. Execute autonomous run (baseline)
2. Execute validated run (with gates)
3. Generate comparison report
4. Save results to `results/run_YYYYMMDD_HHMMSS.json`

### Individual Runs

Autonomous only:
```bash
cargo run --package practical-benchmark --release -- --mode autonomous
```

Validated only:
```bash
cargo run --package practical-benchmark --release -- --mode validated
```

## Metrics Captured

**Per Initiative:**
- Total tokens used
- Execution time
- Lines of code generated
- Test coverage %
- Documentation accuracy %
- Instruction adherence %

**Gate Metrics (Validated Run Only):**
- Gate decision (approved/rework/rejected)
- Issues found
- Rework tokens
- Gate effectiveness %

**Comparison:**
- Token overhead (%)
- Quality improvement (points)
- ROI (quality per % token cost)
- Error detection rate (%)

## Results

Results are saved as JSON with full metrics:
```json
{
  "run_id": "uuid",
  "timestamp": "2026-03-17T...",
  "execution_mode": "autonomous" or "validated",
  "initiatives": [...],
  "total_metrics": {...}
}
```

View results:
```bash
cat results/latest_run.json | jq .total_metrics
```

## Future Enhancements

- [ ] UI benchmark variant (dashboard for task management)
- [ ] Integration with continuous benchmarking
- [ ] Historical trend analysis
- [ ] Automated gate implementation (checker agent)

<!-- TODO: Add UI-focused benchmark variant in future iteration -->

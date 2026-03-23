#!/usr/bin/env bash
# Practical Benchmark Runner
# Runs both autonomous and validated execution paths against the File Processing
# Toolkit scenario, then generates a side-by-side comparison report.
#
# Usage:
#   ./benchmarks/run-practical-bench.sh [--mode autonomous|validated|both]
#
# Environment:
#   ANTHROPIC_API_KEY     (required)
#   CADRE_BINARY    path to cadre binary (default: target/release/cadre)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RESULTS_DIR="${REPO_ROOT}/benchmarks/practical/results"
SCENARIO_DIR="${REPO_ROOT}/benchmarks/practical/scenario"
MODE="${1:-both}"

if [ -z "${ANTHROPIC_API_KEY:-}" ]; then
    echo "Error: ANTHROPIC_API_KEY environment variable not set"
    exit 1
fi

# Resolve binary path
ULTRA="${CADRE_BINARY:-${REPO_ROOT}/target/release/cadre}"
if [ ! -x "${ULTRA}" ]; then
    echo "Building cadre binary..."
    cargo build --release -p cadre-cli 2>&1
fi

echo "=== Practical Benchmark Suite ==="
echo "Binary   : ${ULTRA}"
echo "Scenario : ${SCENARIO_DIR}"
echo "Results  : ${RESULTS_DIR}"
echo "Mode     : ${MODE}"
echo ""

# Run benchmark via the Rust binary
cd "${REPO_ROOT}"
CADRE_BINARY="${ULTRA}" \
    cargo run -q -p practical-benchmark --bin run_benchmark -- \
        --results-dir "${RESULTS_DIR}" \
        --scenario "${SCENARIO_DIR}" \
        --mode "${MODE}"

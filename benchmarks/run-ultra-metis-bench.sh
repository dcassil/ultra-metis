#!/usr/bin/env bash
# Ultra-Metis Benchmark Runner
# Runs all benchmark scenarios against the ultra-metis CLI and outputs timing data.
# Usage: ./run-ultra-metis-bench.sh [path-to-ultra-metis-binary]

set -euo pipefail

ULTRA="${1:-$(dirname "$0")/../target/release/ultra-metis}"
PROJ="/tmp/bench-ultra-metis-$(date +%s)"

if [ ! -x "$ULTRA" ]; then
    echo "Error: ultra-metis binary not found at $ULTRA"
    echo "Build with: cargo build --release (from repo root)"
    exit 1
fi

ms() { python3 -c "import time; print(int(time.time()*1000))"; }

echo "=== Ultra-Metis Benchmark Suite ==="
echo "Binary: $ULTRA"
echo "Project: $PROJ"
echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# --- Scenario 1: Init ---
echo "--- Scenario 1: Project Bootstrap ---"
mkdir -p "$PROJ"
S=$(ms)
$ULTRA init --path "$PROJ" --prefix BENCH 2>&1
E=$(ms)
echo "Time: $((E - S))ms"
echo ""

# --- Scenario 2: Planning Workflow ---
echo "--- Scenario 2: Planning Workflow ---"

S=$(ms); OUT=$($ULTRA create --type vision --path "$PROJ" "Benchmark Vision" 2>&1); E=$(ms)
echo "Create Vision: $((E - S))ms | $OUT"
VCODE=$(echo "$OUT" | grep -oE 'BENCH-V-[0-9]+' | head -1)

S=$(ms); OUT=$($ULTRA create --type initiative --path "$PROJ" --parent "$VCODE" "Test Initiative" 2>&1); E=$(ms)
echo "Create Initiative: $((E - S))ms | $OUT"
ICODE=$(echo "$OUT" | grep -oE 'BENCH-I-[0-9]+' | head -1)

for i in 1 2 3; do
    S=$(ms); OUT=$($ULTRA create --type task --path "$PROJ" --parent "$ICODE" "Task $i" 2>&1); E=$(ms)
    echo "Create Task $i: $((E - S))ms | $OUT"
done
echo ""

# --- Scenario 3: Phase Transitions ---
echo "--- Scenario 3: Phase Transitions ---"
for PHASE in design ready decompose active completed; do
    S=$(ms); OUT=$($ULTRA transition --path "$PROJ" "$ICODE" 2>&1); E=$(ms)
    echo "Transition to $PHASE: $((E - S))ms | $OUT"
done
echo ""

# --- Scenario 4: Search/Query ---
echo "--- Scenario 4: Search and Query ---"
# Add more docs
$ULTRA create --type initiative --path "$PROJ" --parent "$VCODE" "Database Design" >/dev/null 2>&1
$ULTRA create --type initiative --path "$PROJ" --parent "$VCODE" "API Gateway" >/dev/null 2>&1
for t in "Setup DB" "Create API" "Add auth" "Write tests" "Deploy"; do
    $ULTRA create --type task --path "$PROJ" --parent "$ICODE" "$t" >/dev/null 2>&1
done

S=$(ms); OUT=$($ULTRA search --path "$PROJ" "database" 2>&1); E=$(ms)
echo "Search 'database': $((E - S))ms | $(echo "$OUT" | wc -l | tr -d ' ') lines"

S=$(ms); OUT=$($ULTRA search --path "$PROJ" "API" 2>&1); E=$(ms)
echo "Search 'API': $((E - S))ms | $(echo "$OUT" | wc -l | tr -d ' ') lines"

S=$(ms); OUT=$($ULTRA list --path "$PROJ" 2>&1); E=$(ms)
echo "List all: $((E - S))ms | $(echo "$OUT" | wc -l | tr -d ' ') lines"
echo ""

# --- Scenario 5: Error Handling ---
echo "--- Scenario 5: Error Handling ---"

S=$(ms); OUT=$($ULTRA read --path "$PROJ" BENCH-X-9999 2>&1) || true; E=$(ms)
echo "Read non-existent: $((E - S))ms | $OUT"

S=$(ms); OUT=$($ULTRA create --type task --path "$PROJ" --parent BENCH-I-9999 "Orphan" 2>&1) || true; E=$(ms)
echo "Create bad parent: $((E - S))ms | $OUT"

S=$(ms); OUT=$($ULTRA transition --path "$PROJ" "$ICODE" 2>&1) || true; E=$(ms)
echo "Transition past completed: $((E - S))ms | $OUT"
echo ""

# Cleanup
rm -rf "$PROJ"
echo "=== Benchmark Complete ==="

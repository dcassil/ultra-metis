#!/usr/bin/env bash
# Cadre Benchmark Runner
# Runs all benchmark scenarios against the cadre CLI and outputs timing data.
# Usage: ./run-cadre-bench.sh [path-to-cadre-binary]

set -euo pipefail

CADRE="${1:-$(dirname "$0")/../target/release/cadre}"
PROJ="/tmp/bench-cadre-$(date +%s)"

if [ ! -x "$CADRE" ]; then
    echo "Error: cadre binary not found at $CADRE"
    echo "Build with: cargo build --release (from repo root)"
    exit 1
fi

ms() { python3 -c "import time; print(int(time.time()*1000))"; }

echo "=== Cadre Benchmark Suite ==="
echo "Binary: $CADRE"
echo "Project: $PROJ"
echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# --- Scenario 1: Init ---
echo "--- Scenario 1: Project Bootstrap ---"
mkdir -p "$PROJ"
S=$(ms)
$CADRE init --path "$PROJ" --prefix BENCH 2>&1
E=$(ms)
echo "Time: $((E - S))ms"
echo ""

# --- Scenario 2: Planning Workflow ---
echo "--- Scenario 2: Planning Workflow ---"

S=$(ms); OUT=$($CADRE create --type vision --path "$PROJ" "Benchmark Vision" 2>&1); E=$(ms)
echo "Create Vision: $((E - S))ms | $OUT"
VCODE=$(echo "$OUT" | grep -oE 'BENCH-V-[0-9]+' | head -1)

S=$(ms); OUT=$($CADRE create --type initiative --path "$PROJ" --parent "$VCODE" "Test Initiative" 2>&1); E=$(ms)
echo "Create Initiative: $((E - S))ms | $OUT"
ICODE=$(echo "$OUT" | grep -oE 'BENCH-I-[0-9]+' | head -1)

for i in 1 2 3; do
    S=$(ms); OUT=$($CADRE create --type task --path "$PROJ" --parent "$ICODE" "Task $i" 2>&1); E=$(ms)
    echo "Create Task $i: $((E - S))ms | $OUT"
done
echo ""

# --- Scenario 3: Phase Transitions ---
echo "--- Scenario 3: Phase Transitions ---"
for PHASE in design ready decompose active completed; do
    S=$(ms); OUT=$($CADRE transition --path "$PROJ" "$ICODE" 2>&1); E=$(ms)
    echo "Transition to $PHASE: $((E - S))ms | $OUT"
done
echo ""

# --- Scenario 4: Search/Query ---
echo "--- Scenario 4: Search and Query ---"
# Add more docs
$CADRE create --type initiative --path "$PROJ" --parent "$VCODE" "Database Design" >/dev/null 2>&1
$CADRE create --type initiative --path "$PROJ" --parent "$VCODE" "API Gateway" >/dev/null 2>&1
for t in "Setup DB" "Create API" "Add auth" "Write tests" "Deploy"; do
    $CADRE create --type task --path "$PROJ" --parent "$ICODE" "$t" >/dev/null 2>&1
done

S=$(ms); OUT=$($CADRE search --path "$PROJ" "database" 2>&1); E=$(ms)
echo "Search 'database': $((E - S))ms | $(echo "$OUT" | wc -l | tr -d ' ') lines"

S=$(ms); OUT=$($CADRE search --path "$PROJ" "API" 2>&1); E=$(ms)
echo "Search 'API': $((E - S))ms | $(echo "$OUT" | wc -l | tr -d ' ') lines"

S=$(ms); OUT=$($CADRE list --path "$PROJ" 2>&1); E=$(ms)
echo "List all: $((E - S))ms | $(echo "$OUT" | wc -l | tr -d ' ') lines"
echo ""

# --- Scenario 5: Error Handling ---
echo "--- Scenario 5: Error Handling ---"

S=$(ms); OUT=$($CADRE read --path "$PROJ" BENCH-X-9999 2>&1) || true; E=$(ms)
echo "Read non-existent: $((E - S))ms | $OUT"

S=$(ms); OUT=$($CADRE create --type task --path "$PROJ" --parent BENCH-I-9999 "Orphan" 2>&1) || true; E=$(ms)
echo "Create bad parent: $((E - S))ms | $OUT"

S=$(ms); OUT=$($CADRE transition --path "$PROJ" "$ICODE" 2>&1) || true; E=$(ms)
echo "Transition past completed: $((E - S))ms | $OUT"
echo ""

# Cleanup
rm -rf "$PROJ"
echo "=== Benchmark Complete ==="

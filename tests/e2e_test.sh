#!/bin/bash
# End-to-end test for Cadre
# Tests the full workflow: init -> create documents -> transition phases -> verify
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLI="cargo run --manifest-path $SCRIPT_DIR/Cargo.toml -p cadre-cli --"
TMPDIR=$(mktemp -d)

cleanup() {
    rm -rf "$TMPDIR"
}
trap cleanup EXIT

echo "=== Cadre End-to-End Test ==="
echo "Working directory: $TMPDIR"
echo

# 1. Initialize project
echo "1. Initializing project..."
$CLI init --path "$TMPDIR" --prefix E2E 2>/dev/null
test -d "$TMPDIR/.cadre" || { echo "FAIL: .cadre dir not created"; exit 1; }
test -f "$TMPDIR/.cadre/config.toml" || { echo "FAIL: config.toml not created"; exit 1; }
test -d "$TMPDIR/.cadre/docs" || { echo "FAIL: docs dir not created"; exit 1; }
echo "   PASS: Project initialized"

# 2. Create vision
echo "2. Creating vision..."
$CLI create -t vision "Product Vision" --path "$TMPDIR" 2>/dev/null
test -f "$TMPDIR/.cadre/docs/E2E-V-0001.md" || { echo "FAIL: vision file not created"; exit 1; }
echo "   PASS: Vision E2E-V-0001 created"

# 3. Create initiative under vision
echo "3. Creating initiative..."
$CLI create -t initiative "Feature Initiative" -P E2E-V-0001 --path "$TMPDIR" 2>/dev/null
test -f "$TMPDIR/.cadre/docs/E2E-I-0002.md" || { echo "FAIL: initiative file not created"; exit 1; }
echo "   PASS: Initiative E2E-I-0002 created"

# 4. Create task under initiative
echo "4. Creating task..."
$CLI create -t task "Implement Feature" -P E2E-I-0002 --path "$TMPDIR" 2>/dev/null
test -f "$TMPDIR/.cadre/docs/E2E-T-0003.md" || { echo "FAIL: task file not created"; exit 1; }
echo "   PASS: Task E2E-T-0003 created"

# 5. List documents
echo "5. Listing documents..."
OUTPUT=$($CLI list --path "$TMPDIR" 2>/dev/null)
echo "$OUTPUT" | grep -q "E2E-V-0001" || { echo "FAIL: vision not in list"; exit 1; }
echo "$OUTPUT" | grep -q "E2E-I-0002" || { echo "FAIL: initiative not in list"; exit 1; }
echo "$OUTPUT" | grep -q "E2E-T-0003" || { echo "FAIL: task not in list"; exit 1; }
echo "   PASS: All 3 documents listed"

# 6. Read a document
echo "6. Reading document..."
OUTPUT=$($CLI read E2E-V-0001 --path "$TMPDIR" 2>/dev/null)
echo "$OUTPUT" | grep -q "Product Vision" || { echo "FAIL: vision content not readable"; exit 1; }
echo "   PASS: Document readable"

# 7. Transition vision: draft -> review -> published
echo "7. Transitioning vision..."
$CLI transition E2E-V-0001 --path "$TMPDIR" 2>/dev/null  # draft -> review
$CLI transition E2E-V-0001 --path "$TMPDIR" 2>/dev/null  # review -> published
OUTPUT=$($CLI read E2E-V-0001 --path "$TMPDIR" 2>/dev/null)
echo "$OUTPUT" | grep -q "phase/published" || { echo "FAIL: vision not published"; exit 1; }
echo "   PASS: Vision transitioned to published"

# 8. Transition initiative: discovery -> design -> ready -> decompose -> active
echo "8. Transitioning initiative..."
$CLI transition E2E-I-0002 --path "$TMPDIR" 2>/dev/null  # discovery -> design
$CLI transition E2E-I-0002 --path "$TMPDIR" 2>/dev/null  # design -> ready
$CLI transition E2E-I-0002 --path "$TMPDIR" 2>/dev/null  # ready -> decompose
$CLI transition E2E-I-0002 --path "$TMPDIR" 2>/dev/null  # decompose -> active
OUTPUT=$($CLI read E2E-I-0002 --path "$TMPDIR" 2>/dev/null)
echo "$OUTPUT" | grep -q "phase/active" || { echo "FAIL: initiative not active"; exit 1; }
echo "   PASS: Initiative transitioned to active"

# 9. Transition task: todo -> active -> completed
echo "9. Transitioning task..."
$CLI transition E2E-T-0003 --path "$TMPDIR" 2>/dev/null  # todo -> active
$CLI transition E2E-T-0003 --path "$TMPDIR" 2>/dev/null  # active -> completed
OUTPUT=$($CLI read E2E-T-0003 --path "$TMPDIR" 2>/dev/null)
echo "$OUTPUT" | grep -q "phase/completed" || { echo "FAIL: task not completed"; exit 1; }
echo "   PASS: Task transitioned to completed"

# 10. Search
echo "10. Searching documents..."
OUTPUT=$($CLI search "Feature" --path "$TMPDIR" 2>/dev/null)
echo "$OUTPUT" | grep -q "E2E-I-0002" || { echo "FAIL: search didn't find initiative"; exit 1; }
echo "$OUTPUT" | grep -q "E2E-T-0003" || { echo "FAIL: search didn't find task"; exit 1; }
echo "   PASS: Search works"

# 11. Edit
echo "11. Editing document..."
$CLI edit E2E-V-0001 --search "# Product Vision" --replace "# Updated Product Vision" --path "$TMPDIR" 2>/dev/null
OUTPUT=$($CLI read E2E-V-0001 --path "$TMPDIR" 2>/dev/null)
echo "$OUTPUT" | grep -q "Updated Product Vision" || { echo "FAIL: edit didn't apply"; exit 1; }
echo "   PASS: Edit works"

# 12. Archive
echo "12. Archiving document..."
$CLI archive E2E-T-0003 --path "$TMPDIR" 2>/dev/null
OUTPUT=$($CLI list --path "$TMPDIR" 2>/dev/null)
echo "$OUTPUT" | grep -q "E2E-T-0003" && { echo "FAIL: archived doc still in default list"; exit 1; }
OUTPUT=$($CLI list --include-archived --path "$TMPDIR" 2>/dev/null)
echo "$OUTPUT" | grep -q "E2E-T-0003" || { echo "FAIL: archived doc not in archived list"; exit 1; }
echo "   PASS: Archive works"

echo
echo "=== ALL 12 TESTS PASSED ==="

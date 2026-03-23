#!/bin/bash
# Check that required plugins and tools are installed

set -euo pipefail

ERRORS=()

# Check cadre CLI
if ! command -v cadre &>/dev/null; then
  ERRORS+=("cadre CLI not found. Run 'make install' from the cadre repo root.")
fi

# Check ralph-loop plugin (look for its stop hook in plugin cache)
RALPH_FOUND=false
for dir in ~/.claude/plugins/cache/*/ralph-loop/*/hooks/stop-hook.sh; do
  if [ -f "$dir" ] 2>/dev/null; then
    RALPH_FOUND=true
    break
  fi
done
if [ "$RALPH_FOUND" = false ]; then
  # Also check marketplaces dir
  for dir in ~/.claude/plugins/marketplaces/*/plugins/ralph-loop/hooks/stop-hook.sh; do
    if [ -f "$dir" ] 2>/dev/null; then
      RALPH_FOUND=true
      break
    fi
  done
fi
if [ "$RALPH_FOUND" = false ]; then
  ERRORS+=("ralph-loop plugin not installed. Run: claude plugin add ralph-loop@claude-plugins-official")
fi

# Check superpowers plugin (installed or vendored)
SUPERPOWERS_FOUND=false
for dir in ~/.claude/plugins/cache/*/superpowers/*/; do
  if [ -d "$dir" ] 2>/dev/null; then
    SUPERPOWERS_FOUND=true
    break
  fi
done
if [ "$SUPERPOWERS_FOUND" = false ]; then
  for dir in ~/.claude/plugins/marketplaces/*/plugins/superpowers/; do
    if [ -d "$dir" ] 2>/dev/null; then
      SUPERPOWERS_FOUND=true
      break
    fi
  done
fi
# Fall back to vendored copy
if [ "$SUPERPOWERS_FOUND" = false ]; then
  VENDOR_DIR="$(cd "$(dirname "$0")/.." && pwd)/../../vendor/superpowers"
  if [ -d "$VENDOR_DIR" ]; then
    SUPERPOWERS_FOUND=true
  fi
fi
if [ "$SUPERPOWERS_FOUND" = false ]; then
  ERRORS+=("superpowers plugin not installed and no vendored copy found. Run: claude plugin add superpowers@claude-plugins-official")
fi

# Report errors
if [ ${#ERRORS[@]} -gt 0 ]; then
  echo "ERROR: Missing required dependencies:" >&2
  for err in "${ERRORS[@]}"; do
    echo "  - $err" >&2
  done
  exit 1
fi

exit 0

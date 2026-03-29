#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Vite requires Node 20+; use nvm if available
if [ -s "$HOME/.nvm/nvm.sh" ]; then
  . "$HOME/.nvm/nvm.sh"
  nvm use 20 --silent 2>/dev/null || nvm use node --silent
fi

cd "$SCRIPT_DIR"

echo "Starting dashboard on all interfaces..."
exec npx vite --host

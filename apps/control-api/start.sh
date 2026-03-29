#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

export CADRE_PROJECT_PATH="${CADRE_PROJECT_PATH:-$REPO_ROOT}"

echo "Building control-api..."
cargo build --release -p cadre-control-api --manifest-path "$REPO_ROOT/Cargo.toml"

echo "Starting control-api on 0.0.0.0:3000..."
exec "$REPO_ROOT/target/release/cadre-control-api"

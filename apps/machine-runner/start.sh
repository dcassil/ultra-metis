#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "Building machine-runner..."
cargo build --release -p cadre-machine-runner --manifest-path "$REPO_ROOT/Cargo.toml"

echo "Starting machine-runner..."
exec "$REPO_ROOT/target/release/cadre-machine-runner"

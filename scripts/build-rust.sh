#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"

echo "Building synapse-core + synapse-cli (target: $CARGO_TARGET_DIR)..."
cargo build -p synapse-cli --release

echo "Building synapse-node (N-API)..."
if command -v npm >/dev/null 2>&1; then
  npm install --workspace=@synapse/native --include-workspace-root=false
  npm run build:native --workspace=@synapse/native
fi

echo "Done. Native CLI: target/release/synapse"

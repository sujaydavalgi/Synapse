#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"

echo "Building spanda-core + spanda-cli (target: $CARGO_TARGET_DIR)..."
cargo build -p spanda-cli --release

echo "Building spanda-node (N-API)..."
if command -v npm >/dev/null 2>&1; then
  npm install --workspace=@spanda/native --include-workspace-root=false
  npm run build:native --workspace=@spanda/native
fi

echo "Done. Native CLI: target/release/spanda"

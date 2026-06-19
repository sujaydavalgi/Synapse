#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "Install wasm-pack: cargo install wasm-pack"
  exit 1
fi

wasm-pack build crates/spanda-wasm --target web --out-dir "$ROOT/packages/web/wasm" --release

echo "WASM built to packages/web/wasm/"

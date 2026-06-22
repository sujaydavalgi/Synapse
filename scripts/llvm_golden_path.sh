#!/usr/bin/env bash
# Golden path for LLVM native codegen (requires clang and spanda-cli with llvm feature).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

SPANDA="${SPANDA_BIN:-$ROOT/target/release/spanda}"
SOURCE="${1:-examples/hello_world.sd}"
OUT="${TMPDIR:-/tmp}/spanda-llvm-golden"

if ! command -v clang >/dev/null 2>&1; then
  echo "clang not found; skip LLVM golden path" >&2
  exit 0
fi

cargo build -p spanda-cli --release --features llvm
"$SPANDA" check "$SOURCE"
"$SPANDA" llvm-ir "$SOURCE" --out "$OUT.ll"
"$SPANDA" compile-native "$SOURCE" --out "$OUT"
test -x "$OUT"
echo "✓ LLVM golden path: $SOURCE -> $OUT"

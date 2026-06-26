#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${CARGO_TARGET_DIR:-$ROOT/target}/debug/spanda"
ROVER="$ROOT/examples/showcase/secure_boot/rover.sd"

cd "$ROOT"
# shellcheck source=lib/registry_env.sh
source "${ROOT}/scripts/lib/registry_env.sh"
ensure_spanda_registry_url "$ROOT"
cargo build -p spanda -q

echo "== secure boot tamper-check =="
TAMPER="$("$BIN" tamper-check "$ROVER" 2>&1 || true)"
echo "$TAMPER"
echo "$TAMPER" | grep -q "secure_boot"

echo "== secure boot integrity =="
INTEGRITY="$("$BIN" integrity "$ROVER" 2>&1 || true)"
echo "$INTEGRITY"
echo "$INTEGRITY" | grep -q "Secure boot:"

echo "== secure boot package trust =="
TRUST="$("$BIN" trust spanda-trust-jetson 2>&1 || true)"
echo "$TRUST" | grep -q "trust score 9"

echo "secure boot smoke ok"

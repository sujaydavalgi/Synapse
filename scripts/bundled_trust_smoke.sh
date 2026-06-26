#!/usr/bin/env bash
# Verify spanda demo trust works from bundled examples (no SPANDA_ROOT / repo checkout).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLI="${ROOT}/crates/spanda-cli/Cargo.toml"

echo "== bundled demo trust (no SPANDA_ROOT) =="
cd /tmp
unset SPANDA_ROOT
unset SPANDA_REGISTRY_URL
OUT="$(cargo run --manifest-path "${CLI}" -q -- demo trust 2>&1 || true)"
echo "$OUT" | grep -q "Trust & tamper"
echo "$OUT" | grep -q "Secure boot contracts"
echo "$OUT" | grep -q "trust.jetson"
echo "$OUT" | grep -q "Demo complete"

echo "bundled trust smoke ok"

#!/usr/bin/env bash
# Smoke Phase A platform maturity commands (graph, explain, trust, deploy gate).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
FILE="${ROOT}/examples/showcase/readiness/rover.sd"

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

echo "== graph =="
run_spanda graph "$FILE" --format text >/dev/null

echo "== explain =="
run_spanda explain "$FILE" >/dev/null

echo "== trust =="
run_spanda trust spanda-mqtt >/dev/null

echo "== deploy gate =="
(run_spanda deploy gate "$FILE" 2>&1 || true) | grep -q "Gate check"

echo "== demo maturity =="
export SPANDA_ROOT="${ROOT}"
run_spanda demo maturity

echo "Maturity smoke OK"

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
EXPLAIN="$(run_spanda explain "$FILE" 2>&1 || true)"
echo "$EXPLAIN" | head -5
echo "$EXPLAIN" | grep -q "composite_trust"

echo "== explain secure_boot on defense showcase =="
DEFENSE="${ROOT}/examples/showcase/compliance/defense_rover.sd"
export SPANDA_REGISTRY_URL="file://${ROOT}/registry"
EXPLAIN_SB="$(run_spanda explain "$DEFENSE" 2>&1 || true)"
echo "$EXPLAIN_SB" | grep -q "secure_boot"
unset SPANDA_REGISTRY_URL

echo "== trust package =="
run_spanda trust spanda-mqtt >/dev/null

echo "== trust program =="
TRUST_OUT="$(run_spanda trust "$FILE" 2>&1 || true)"
echo "$TRUST_OUT" | grep -q "Composite trust:"

echo "== deploy gate =="
GATE_OUT="$(run_spanda deploy gate "$FILE" 2>&1 || true)"
echo "$GATE_OUT" | grep -q "Gate check"
echo "$GATE_OUT" | grep -q "composite_trust"

echo "== demo maturity =="
export SPANDA_ROOT="${ROOT}"
DEMO_OUT="$(run_spanda demo maturity 2>&1 || true)"
echo "$DEMO_OUT" | grep -q "Platform maturity"

echo "== demo trust =="
export SPANDA_ROOT="${ROOT}"
TRUST_DEMO="$(run_spanda demo trust 2>&1 || true)"
echo "$TRUST_DEMO" | grep -q "Trust & tamper"

echo "Maturity smoke OK"

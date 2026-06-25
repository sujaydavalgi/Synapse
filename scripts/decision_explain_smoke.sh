#!/usr/bin/env bash
# Smoke decision trace explainability.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
TRACE="${ROOT}/examples/showcase/autonomous_rover/src/rover.trace"

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

echo "== explain decision trace =="
run_spanda explain decision "$TRACE" >/dev/null

echo "== explain decision trace json =="
run_spanda explain decision "$TRACE" --json >/dev/null

echo "Decision explain smoke OK"

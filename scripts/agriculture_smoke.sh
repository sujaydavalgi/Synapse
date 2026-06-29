#!/usr/bin/env bash
# Agriculture Official Solution Blueprint smoke — scaffold validation.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
AG="$ROOT/examples/solutions/agriculture"

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

echo "== agriculture blueprint check =="
run_spanda check "$AG/field_patrol.sd"

echo "== agriculture blueprint verify =="
run_spanda verify "$AG/field_patrol.sd" --target FieldRobotV1

echo "Agriculture blueprint smoke OK"

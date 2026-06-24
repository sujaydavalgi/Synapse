#!/usr/bin/env bash
# Smoke mission continuity commands in CI.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
WAREHOUSE="${ROOT}/examples/showcase/continuity/warehouse.sd"
DELIVERY="${ROOT}/examples/showcase/fleet_succession/delivery.sd"

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

echo "== spanda-assurance continuity tests =="
cargo test -p spanda-assurance continuity --quiet

echo "== continuity runtime + CLI tests =="
cargo test -p spanda-interpreter continuity_runtime --quiet
cargo test -p spanda continuity_cli --quiet
cargo test -p spanda-fleet swarm_continuity_plans --quiet

echo "== continuity CLI =="
run_spanda continuity "$WAREHOUSE" --failed ScannerAlpha --progress 72 --trigger robot_failed >/dev/null
run_spanda takeover "$WAREHOUSE" --failed ScannerAlpha --successor ScannerBeta --progress 72 >/dev/null
run_spanda delegate "$WAREHOUSE" --failed ScannerAlpha --to ScannerBeta --progress 60 >/dev/null
run_spanda succession "$DELIVERY" --failed CourierA --scope fleet >/dev/null

echo "== demo continuity =="
export SPANDA_ROOT="${ROOT}"
run_spanda demo continuity

echo "== TypeScript continuity mirror =="
npm test -- tests/mission-continuity.test.ts tests/continuity-diagnostics.test.ts --silent

echo "Mission continuity smoke OK"

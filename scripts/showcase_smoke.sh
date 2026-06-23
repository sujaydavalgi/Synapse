#!/usr/bin/env bash
# Smoke tests for showcase demos (CI + local).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SPANDA="${SPANDA_BIN:-$ROOT/target/release/spanda}"

if [[ ! -x "${SPANDA}" ]]; then
  cargo build -p spanda-cli --release
  SPANDA="${ROOT}/target/release/spanda"
fi

export SPANDA_BIN="${SPANDA}"
export SPANDA_ROOT="${ROOT}"

echo "== showcase smoke: safety =="
"${SPANDA}" demo safety

echo "== showcase smoke: verify =="
"${SPANDA}" demo verify

echo "== showcase smoke: health =="
"${SPANDA}" demo health

echo "== showcase smoke: fleet =="
"${SPANDA}" check examples/showcase/fleet_management/fleet.sd
"${SPANDA}" verify examples/showcase/fleet_management/fleet.sd --health

echo "== showcase smoke: capability =="
"${SPANDA}" check examples/showcase/capability_verification/rover.sd
"${SPANDA}" verify examples/showcase/capability_verification/rover.sd --capabilities

echo "== showcase smoke: replay =="
"${SPANDA}" check examples/showcase/replay/mission.sd
"${SPANDA}" sim examples/showcase/replay/mission.sd

echo "== showcase smoke: killer path =="
chmod +x scripts/killer_demo_golden_path.sh
./scripts/killer_demo_golden_path.sh

echo "Showcase smoke tests passed."

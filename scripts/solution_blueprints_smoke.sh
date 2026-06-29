#!/usr/bin/env bash
# Official Solution Blueprint scaffolds — agriculture, environmental, maritime.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

check_verify() {
  local label="$1"
  local file="$2"
  local target="$3"
  echo "== ${label} check =="
  run_spanda check "$file"
  echo "== ${label} verify =="
  run_spanda verify "$file" --target "$target"
}

check_verify "agriculture" \
  "$ROOT/examples/solutions/agriculture/field_patrol.sd" \
  FieldRobotV1

check_verify "environmental-monitoring" \
  "$ROOT/examples/solutions/environmental-monitoring/sensor_mesh.sd" \
  SensorNodeV1

check_verify "maritime" \
  "$ROOT/examples/solutions/maritime/harbor_patrol.sd" \
  CoastalVesselV1

echo "Solution blueprint smoke OK"

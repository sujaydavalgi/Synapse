#!/usr/bin/env bash
# Spatial Computing Solution Blueprint smoke — human registry, readiness, examples.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SC="$ROOT/examples/solutions/spatial-computing"
MAIN="$SC/warehouse-ar/pick_mission.sd"

cd "$ROOT"
export SPANDA_ROOT="${SPANDA_ROOT:-$ROOT}"

# shellcheck source=lib/registry_env.sh
source "${ROOT}/scripts/lib/registry_env.sh"
ensure_spanda_registry_url "$ROOT"
cargo build -p spanda -q

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

echo "== Spatial Computing Solution Blueprint smoke =="

check() {
  echo "--- $* ---"
  run_spanda "$@"
}

check config validate --config "$SC/spanda.toml"
check check "$MAIN"
check verify "$MAIN" --capabilities --config "$SC/spanda.toml"
check readiness "$MAIN" --profile human_collaboration --config "$SC/spanda.toml"

for example in \
  "$SC/remote-maintenance/repair.sd" \
  "$SC/vr-training/training_mission.sd" \
  "$SC/search-and-rescue-ar/sar_mission.sd" \
  "$SC/wearable-health/health_patrol.sd" \
  "$SC/operator-approval/collaborative_mission.sd"
do
  check check "$example"
done

echo ""
echo "Spatial computing smoke complete."

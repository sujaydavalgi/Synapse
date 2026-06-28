#!/usr/bin/env bash
# ADAS automotive sensor live-backend smoke — hub stubs and SPANDA_*_CMD bridges.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "== ADAS automotive sensor smoke =="

echo "--- provider unit tests ---"
cargo test -p spanda-providers --test automotive_hub -- --test-threads=1 -q
cargo test -p spanda-providers --lib iot_live::tests::live_radar -- --test-threads=1 -q

echo "--- live radar cmd bridge ---"
export SPANDA_LIVE_RADAR=1
export SPANDA_RADAR_CMD='echo 18.0'
cargo test -p spanda-providers live_radar_cmd_overrides_hub_stub -q
unset SPANDA_LIVE_RADAR SPANDA_RADAR_CMD

echo "ADAS automotive sensor smoke complete."

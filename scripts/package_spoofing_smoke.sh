#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
BIN="${CARGO_TARGET_DIR:-target}/debug/spanda"
if [[ ! -x "$BIN" ]]; then
  cargo build -p spanda --quiet
fi

echo "== package backends typecheck =="
"$BIN" check packages/registry/spanda-gps/src/positioning_gps.sd
"$BIN" check packages/registry/spanda-fusion/src/assurance_fusion.sd

echo "== gps_spoofing program with package imports =="
"$BIN" spoof-check examples/showcase/gps_spoofing/rover.sd | grep -q "spanda-gps package"
"$BIN" spoof-check examples/showcase/gps_spoofing/rover.sd | grep -q "spanda-fusion package"

echo "package spoofing smoke ok"

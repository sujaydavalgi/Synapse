#!/usr/bin/env bash
# Golden path for persistent telemetry store (sim + query CLI).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

STORE_DIR="$(mktemp -d)"
trap 'rm -rf "${STORE_DIR}"' EXIT
export SPANDA_TELEMETRY_STORE_PATH="${STORE_DIR}/telemetry.jsonl"
export SPANDA_TELEMETRY_HEARTBEAT_PATH="${STORE_DIR}/heartbeats.json"

SPANDA=(cargo run --quiet -p spanda --)

echo "== sim with --persist-telemetry =="
"${SPANDA[@]}" sim examples/end_to_end/validated_telemetry.sd --persist-telemetry >/dev/null

echo "== telemetry stats =="
STATS="$("${SPANDA[@]}" telemetry stats)"
echo "${STATS}"
echo "${STATS}" | grep -q "Sensor events:"
echo "${STATS}" | grep -q "Device events:"

echo "== telemetry list sensor events =="
LIST="$("${SPANDA[@]}" telemetry list --kind sensor --limit 3)"
echo "${LIST}"
echo "${LIST}" | grep -q '\[sensor\]'

echo "== telemetry session + runtime metrics =="
SESSIONS="$("${SPANDA[@]}" telemetry list --kind session --limit 2)"
echo "${SESSIONS}"
echo "${SESSIONS}" | grep -q '\[session\]'
METRICS="$("${SPANDA[@]}" telemetry list --kind runtime_metrics --limit 1)"
echo "${METRICS}"
echo "${METRICS}" | grep -q '\[runtime_metrics\]'

echo "== telemetry prometheus =="
PROM="$("${SPANDA[@]}" telemetry prometheus)"
echo "${PROM}" | head -5
echo "${PROM}" | grep -q 'spanda_telemetry_events_total'

echo "== telemetry latest device publish =="
LATEST="$("${SPANDA[@]}" telemetry latest --device TelemetryRover --metric /telemetry)"
echo "${LATEST}"
echo "${LATEST}" | grep -q 'TelemetryRover'

echo "Telemetry store golden path complete."

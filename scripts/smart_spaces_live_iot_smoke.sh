#!/usr/bin/env bash
# Smoke Smart Spaces live building I/O bridges (external cmd + Python mock fallback).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
BRIDGE="${ROOT}/scripts/spanda_python_bridge.py"

echo "== BACnet external cmd =="
unset SPANDA_LIVE_BACNET SPANDA_BACNET_CMD
export SPANDA_LIVE_BACNET=1
export SPANDA_BACNET_CMD='echo mock-bacnet:{device}:{object_id}'
cargo test -p spanda-providers live_bacnet_external_cmd_parses_stdout -- --nocapture

echo "== KNX Python bridge mock =="
export SPANDA_LIVE_KNX=1
RESULT="$(printf '%s\n' '{"fn":"knx_read_group","args":["1/2/3"]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q '"ok": true'

echo "== Thread Python bridge mock =="
export SPANDA_LIVE_THREAD=1
RESULT="$(printf '%s\n' '{"fn":"thread_read_endpoint","args":["thread-sensor-1"]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q '"ok": true'

echo "== Z-Wave Python bridge mock =="
export SPANDA_LIVE_ZWAVE=1
RESULT="$(printf '%s\n' '{"fn":"zwave_read_value","args":["zwave-lock-1","DoorLock"]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q '"ok": true'

echo "== Home Assistant Python bridge mock =="
export SPANDA_LIVE_HOME_ASSISTANT=1
RESULT="$(printf '%s\n' '{"fn":"home_assistant_get_state","args":["binary_sensor.leak_basement"]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q '"ok": true'

echo "Smart Spaces live IoT smoke complete."

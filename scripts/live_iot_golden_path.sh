#!/usr/bin/env bash
# Golden path for live IoT bridge handlers (mock fallback without hardware libraries).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
BRIDGE="${ROOT}/scripts/spanda_python_bridge.py"

echo "== modbus bridge mock path =="
unset SPANDA_LIVE_MODBUS
RESULT="$(printf '%s\n' '{"fn":"modbus_read_register","args":["127.0.0.1","502",40001]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q '"ok": true'

echo "== opcua bridge mock path =="
RESULT="$(printf '%s\n' '{"fn":"opcua_read_node","args":["opc.tcp://127.0.0.1:4840","ns=2;s=Temperature"]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q '"ok": true'

echo "== zigbee bridge mock path =="
export SPANDA_LIVE_ZIGBEE=1
RESULT="$(printf '%s\n' '{"fn":"zigbee_read_attribute","args":["sensor-1","temperature"]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q '"ok": true'

echo "== lora bridge mock path =="
export SPANDA_LIVE_LORA=1
RESULT="$(printf '%s\n' '{"fn":"lora_read_payload","args":["dev-42"]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q '"ok": true'

echo "== matter bridge mock path =="
export SPANDA_LIVE_MATTER=1
RESULT="$(printf '%s\n' '{"fn":"matter_read_cluster","args":["node-1","OnOff"]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q '"ok": true'

echo "== canbus bridge mock path =="
export SPANDA_LIVE_CANBUS=1
RESULT="$(printf '%s\n' '{"fn":"canbus_read_frame","args":[291]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q '"ok": true'

echo "== bacnet bridge mock path =="
export SPANDA_LIVE_BACNET=1
RESULT="$(printf '%s\n' '{"fn":"bacnet_read_point","args":["ahu-12","present-value"]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q '"ok": true'

echo "== knx bridge mock path =="
export SPANDA_LIVE_KNX=1
RESULT="$(printf '%s\n' '{"fn":"knx_read_group","args":["1/2/3"]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q '"ok": true'

echo "== thread bridge mock path =="
export SPANDA_LIVE_THREAD=1
RESULT="$(printf '%s\n' '{"fn":"thread_read_endpoint","args":["thread-sensor-1"]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q '"ok": true'

echo "== zwave bridge mock path =="
export SPANDA_LIVE_ZWAVE=1
RESULT="$(printf '%s\n' '{"fn":"zwave_read_value","args":["zwave-lock-1","DoorLock"]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q '"ok": true'

echo "== home assistant bridge mock path =="
export SPANDA_LIVE_HOME_ASSISTANT=1
RESULT="$(printf '%s\n' '{"fn":"home_assistant_get_state","args":["binary_sensor.leak_basement"]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q '"ok": true'

echo "== onnx bridge mock path =="
RESULT="$(printf '%s\n' '{"fn":"onnx_complete","args":["plan safe stop"]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q '"ok": true'

echo "Live IoT golden path complete."

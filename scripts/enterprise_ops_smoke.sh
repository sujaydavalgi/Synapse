#!/usr/bin/env bash
# Phase E1–E4 smoke — Control Center API through govern-and-trace endpoints.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
WAREHOUSE_FIXTURE="${ROOT}/crates/spanda-config/tests/fixtures/warehouse"
SMOKE_CONFIG_DIR="$(mktemp -d "${TMPDIR:-/tmp}/spanda-ops-smoke.XXXXXX")"
cp -R "${WAREHOUSE_FIXTURE}/." "${SMOKE_CONFIG_DIR}/"
CONFIG="${SMOKE_CONFIG_DIR}/spanda.toml"
PROGRAM="${ROOT}/examples/showcase/compliance/defense_rover.sd"

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

PORT="${SPANDA_CONTROL_CENTER_TEST_PORT:-}"
if [[ -z "$PORT" ]]; then
  PORT=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
fi
GRPC_PORT="${SPANDA_GRPC_TEST_PORT:-}"
if [[ -z "$GRPC_PORT" ]]; then
  GRPC_PORT=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
fi
BIND="127.0.0.1:${PORT}"
GRPC_BIND="127.0.0.1:${GRPC_PORT}"
export SPANDA_API_KEY="enterprise-ops-smoke-key"
export SPANDA_CONFIG_SNAPSHOT_KEY="${SPANDA_CONFIG_SNAPSHOT_KEY:-smoke-snapshot-key}"
if [[ -z "${SPANDA_REGISTRY_URL:-}" && -f "${ROOT}/registry/index.json" ]]; then
  export SPANDA_REGISTRY_URL="file://${ROOT}/registry"
fi
export SPANDA_DISCOVERY_WIFI_MATCHES="${SPANDA_DISCOVERY_WIFI_MATCHES:-smoke-wifi@192.168.1.50}"
export SPANDA_DISCOVERY_CELLULAR_MATCHES="${SPANDA_DISCOVERY_CELLULAR_MATCHES:-lte-modem@10.0.0.1}"
export SPANDA_DISCOVERY_SERIAL_MATCHES="${SPANDA_DISCOVERY_SERIAL_MATCHES:-gps@/dev/ttyUSB0}"
export SPANDA_DISCOVERY_MDNS_MATCHES="${SPANDA_DISCOVERY_MDNS_MATCHES:-smoke-mdns@127.0.0.1}"

export SPANDA_WS_STREAM_SECONDS=3
echo "== start control-center on ${BIND} + gRPC ${GRPC_BIND} (warehouse config + program) =="
run_spanda control-center serve --bind "$BIND" --grpc-bind "$GRPC_BIND" --config "$CONFIG" --program "$PROGRAM" &
SERVER_PID=$!
sleep 2

cleanup() {
  kill "$SERVER_PID" 2>/dev/null || true
  rm -rf "$SMOKE_CONFIG_DIR"
}
trap cleanup EXIT

chmod +x "${ROOT}/scripts/mock_otlp_traces_collector.py" "${ROOT}/scripts/ws_telemetry_probe.py" 2>/dev/null || true

fetch() {
  local path="$1"
  local attempt=0
  while [[ $attempt -lt 30 ]]; do
    if curl -sf --max-time 15 "http://${BIND}${path}"; then
      return 0
    fi
    attempt=$((attempt + 1))
    sleep 0.2
  done
  echo "failed to fetch ${path}" >&2
  return 1
}

echo "== GET /v1/health =="
fetch /v1/health | grep -q spanda-control-center

echo "== GET /v1/dashboard =="
fetch /v1/dashboard | grep -q device_pool

echo "== GET /v1/devices =="
fetch /v1/devices | grep -q '"devices"'

echo "== GET /v1/fleet/agents =="
fetch /v1/fleet/agents | grep -q '"agents"'

echo "== GET /v1/rbac/matrix =="
fetch /v1/rbac/matrix | grep -q Administrator

echo "== POST /v1/alerts/test (authenticated) =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  "http://${BIND}/v1/alerts/test" | grep -q '"ok":true'

echo "== GET /v1/alerts =="
fetch /v1/alerts | grep -q Control

echo "== GET / (Control Center UI) =="
curl -sf "http://${BIND}/" | grep -q "Spanda Control Center"

echo "== GET /v1/devices/lidar-front =="
fetch /v1/devices/lidar-front | grep -q lidar-front

echo "== GET /v1/robots =="
fetch /v1/robots | grep -q rover-001

echo "== GET /v1/device-tree =="
fetch /v1/device-tree | grep -q mapping

echo "== POST /v1/readiness/run =="
curl -sf -X POST "http://${BIND}/v1/readiness/run" | grep -q mission_ready

echo "== GET /v1/failover/chains =="
fetch /v1/failover/chains | grep -q chains

echo "== E2 failover drill (redundant chain smoke) =="
chmod +x "${ROOT}/scripts/failover_drill_smoke.sh"
"${ROOT}/scripts/failover_drill_smoke.sh"

echo "== GET /v1/device-reports =="
fetch /v1/device-reports | grep -q inventory

echo "== POST /v1/devices/discover (multi-transport) =="
SPANDA_DISCOVERY_MDNS_MATCHES="smoke-robot@127.0.0.1" \
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"transports":["mdns"],"timeout_ms":500}' \
  "http://${BIND}/v1/devices/discover" | grep -q '"registered"'

echo "== POST /v1/devices/gps-001/trust =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  "http://${BIND}/v1/devices/gps-001/trust" | grep -q 'device trusted by operator'

echo "== POST /v1/devices/gps-001/assign =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"robot_id":"rover-001"}' \
  "http://${BIND}/v1/devices/gps-001/assign" | grep -q '"ok":true'

echo "== POST /v1/devices/camera-front-001/trust (quarantine approval) =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  "http://${BIND}/v1/devices/camera-front-001/trust" | grep -q '"lifecycle_state":"verified"'

echo "== POST /v1/devices/drive-controller/quarantine =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  "http://${BIND}/v1/devices/drive-controller/quarantine" | grep -q quarantined

echo "== E2 GET /v1/discovery?transport=mdns =="
fetch "/v1/discovery?transport=mdns&timeout_ms=100" | grep -qE '"transport":"mdns(:spanda-discovery-mdns)?"'

echo "== E2 GET /v1/health/summary =="
fetch /v1/health/summary | grep -q overall_status

echo "== E2 GET /v1/assurance/summary =="
fetch /v1/assurance/summary | grep -q '"loaded":true'

echo "== E2 POST /v1/provision (expect readiness alert) =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"device_id":"lidar-front","robot_id":"rover-001"}' \
  "http://${BIND}/v1/provision" | grep -q '"ok":false'

echo "== E2 GET /v1/alerts (provisioning failure) =="
attempt=0
while [[ $attempt -lt 30 ]]; do
  if fetch /v1/alerts | grep -q readiness_failed; then
    break
  fi
  attempt=$((attempt + 1))
  sleep 0.2
done
fetch /v1/alerts | grep -q readiness_failed

echo "== E2 POST /v1/config/snapshots =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"label":"smoke-baseline"}' \
  "http://${BIND}/v1/config/snapshots" | grep -q '"ok":true'

echo "== E2 POST /v1/config/snapshots (encrypted at rest) =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"label":"smoke-encrypted","encrypt":true}' \
  "http://${BIND}/v1/config/snapshots" | grep -q '"encrypted":true'

echo "== E2 GET /v1/config/snapshots =="
SNAPSHOT_JSON=$(fetch /v1/config/snapshots)
echo "$SNAPSHOT_JSON" | grep -q smoke-baseline
BASELINE_ID=$(echo "$SNAPSHOT_JSON" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d["snapshots"][0]["id"])')

echo "== E2 config approval queue =="
APPROVAL_JSON=$(curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d "{\"snapshot_id\":\"${BASELINE_ID}\"}" \
  "http://${BIND}/v1/config/approvals")
echo "$APPROVAL_JSON" | grep -q '"approval"'
APPROVAL_ID=$(echo "$APPROVAL_JSON" | python3 -c 'import json,sys; print(json.load(sys.stdin)["approval"]["id"])')
fetch /v1/config/approvals | grep -q approvals
echo "== E2 config approval publish-on-approve =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{}' \
  "http://${BIND}/v1/config/approvals/${APPROVAL_ID}/approve" | grep -q '"publish"'

echo "== E2 CLI control-center remote API =="
export SPANDA_CONTROL_CENTER_URL="http://${BIND}"
echo "  dashboard"
run_spanda control-center dashboard | grep -q device_pool
echo "  drift report"
run_spanda control-center drift report --baseline-id "${BASELINE_ID}" | grep -q dimensions_checked
echo "  drift scan"
run_spanda control-center drift scan --baseline-id "${BASELINE_ID}" | grep -q '"scan"'
echo "  drift scans"
run_spanda control-center drift scans | grep -q '"scans"'
echo "  approvals list"
run_spanda control-center approvals list | grep -q approvals
echo "  evidence list"
run_spanda control-center evidence list | grep -q evidence
echo "  incidents list"
run_spanda control-center incidents list | grep -q incidents
echo "  sre summary"
run_spanda control-center sre summary | grep -q availability_percent
echo "  devices list"
run_spanda control-center devices list | grep -q '"devices"'
echo "  readiness run"
run_spanda control-center readiness run | grep -q mission_ready
echo "  ota plan"
run_spanda control-center ota plan --strategy canary --version smoke-cli-1.0 --dry-run | grep -Eq '"strategy"[[:space:]]*:[[:space:]]*"canary"'
echo "  trust package"
run_spanda control-center trust package --name spanda-mqtt | grep -q trust
echo "  alerts list"
run_spanda control-center alerts list | grep -q alerts

echo "== E3 GET /v1/openapi.json =="
fetch /v1/openapi.json | grep -q Spanda
fetch /v1/openapi.json | grep -q '"/v1/digital-thread/query"'
fetch /v1/openapi.json | grep -q '"/v1/compliance/export"'
fetch /v1/openapi.json | grep -q '"/v1/compliance/profiles"'
fetch /v1/openapi.json | grep -q '"/v1/reports/schedules"'

echo "== E3 OpenAPI REST parity test =="
cargo test -p spanda-api --test openapi_parity_tests --quiet

echo "== E3 GET /v1/drift?baseline_id =="
fetch "/v1/drift?baseline_id=${BASELINE_ID}" | grep -q dimensions_checked
export SPANDA_GRPC_BASELINE_ID="${BASELINE_ID}"

echo "== E3 native gRPC (tonic) probe =="
export SPANDA_GRPC_BIND="${GRPC_BIND}"
cargo test -p spanda-api --test grpc_live_probe grpc_live_control_center_endpoints --quiet

echo "== E3 POST /v1/ota/plan (canary dry-run) =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"strategy":"canary","version":"1.2.3","canary_percent":20,"dry_run":true,"assignments":[{"robot_name":"rover-001","hardware":"jetson"}]}' \
  "http://${BIND}/v1/ota/plan" | grep -q '"strategy":"canary"'

echo "== E3 POST /v1/ota/execute (dry-run fleet rollout) =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"strategy":"all","version":"1.2.3","dry_run":true,"assignments":[{"robot_name":"rover-001","hardware":"jetson"}]}' \
  "http://${BIND}/v1/ota/execute" | grep -q '"dry_run":true'

echo "== GET /v1/tenant (multi-tenant scope) =="
fetch /v1/tenant | grep -q tenant_id

echo "== GET /v1/observability/backend (OTEL collector config) =="
fetch /v1/observability/backend | grep -q spanda-otel-collector

echo "== E3 GET /v1/version (API policy) =="
fetch /v1/version | grep -q supported_versions

echo "== E3 GET /v1/audit/mutations =="
curl -sf -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  "http://${BIND}/v1/audit/mutations" | grep -q record_count
echo "== E3 GET /v1/audit/mutations/export =="
curl -sf -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  "http://${BIND}/v1/audit/mutations/export?format=cef" | grep -q 'CEF:0|Spanda'

echo "== E3 live OTA execute (deploy agent) =="
chmod +x "${ROOT}/scripts/ota_fleet_execute_smoke.sh"
"${ROOT}/scripts/ota_fleet_execute_smoke.sh"

echo "== E3 OTA fleet soak (multi-agent readiness-gated rollouts) =="
chmod +x "${ROOT}/scripts/ota_fleet_soak.sh"
"${ROOT}/scripts/ota_fleet_soak.sh"

echo "== E2 remote CLI OpenAPI parity test =="
cargo test -p spanda --test control_center_openapi_parity --quiet

echo "== E2 GET /v1/discovery?transport=ble (registry package) =="
fetch "/v1/discovery?transport=ble" | grep -q spanda-discovery-ble

echo "== E2 GET /v1/discovery?transport=usb (registry package) =="
fetch "/v1/discovery?transport=usb" | grep -q spanda-discovery-usb

echo "== E2 GET /v1/discovery?transport=wifi (registry package) =="
fetch "/v1/discovery?transport=wifi&timeout_ms=100" | grep -q spanda-discovery-wifi

echo "== E2 GET /v1/discovery?transport=cellular (registry package) =="
fetch "/v1/discovery?transport=cellular&timeout_ms=100" | grep -q spanda-discovery-cellular

echo "== E2 GET /v1/discovery?transport=serial (registry package) =="
fetch "/v1/discovery?transport=serial&timeout_ms=100" | grep -q spanda-discovery-serial

echo "== E3 GET /v1/trust/package?name=spanda-mqtt =="
fetch "/v1/trust/package?name=spanda-mqtt" | grep -q trust

echo "== E3 GET /v1/sre/summary =="
fetch /v1/sre/summary | grep -q availability_percent

echo "== E3 SRE incident workflow =="
INCIDENT_RESPONSE=$(curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"title":"smoke incident","description":"enterprise ops smoke","severity":"warning"}' \
  "http://${BIND}/v1/sre/incidents")
echo "$INCIDENT_RESPONSE" | grep -q '"ok":true'
INCIDENT_ID=$(echo "$INCIDENT_RESPONSE" | python3 -c 'import json,sys; print(json.load(sys.stdin)["incident"]["id"])')
echo "== E3 POST /v1/integrations/pagerduty/webhook =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d "{\"event\":\"incident.acknowledged\",\"incident_id\":\"${INCIDENT_ID}\",\"assignee\":\"pagerduty\"}" \
  "http://${BIND}/v1/integrations/pagerduty/webhook" | grep -q '"ok":true'
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"assignee":"oncall"}' \
  "http://${BIND}/v1/sre/incidents/${INCIDENT_ID}/ack" | grep -q '"ok":true'
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  "http://${BIND}/v1/sre/incidents/${INCIDENT_ID}/resolve" | grep -q '"ok":true'
fetch /v1/sre/summary | grep -q mttr_hint_ms
fetch /v1/sre/summary | grep -q mtbf_hint_ms
fetch /v1/sre/summary | grep -q health_trends
fetch /v1/sre/summary | grep -q burn_rate

echo "== E3 GET /v1/drift?baseline_id (seven dimensions) =="
fetch "/v1/drift?baseline_id=${BASELINE_ID}" | grep -q policy

echo "== E3 GET /v1/observability/traces (correlation IDs) =="
curl -sf -H "X-Correlation-ID: smoke-trace-1" "http://${BIND}/v1/health" >/dev/null
fetch /v1/observability/traces | grep -q smoke-trace-1

echo "== E3 POST /v1/rpc (gRPC gateway) =="
curl -sf -X POST \
  -H "Content-Type: application/json" \
  -d '{"method":"spanda.v1.SpandaService/GetHealth"}' \
  "http://${BIND}/v1/rpc" | grep -q spanda-control-center

echo "== E3 Python SDK health =="
PYTHONPATH="${ROOT}/packages/sdk-python/src:${PYTHONPATH:-}" \
  SPANDA_CONTROL_CENTER_URL="http://${BIND}" SPANDA_API_KEY="${SPANDA_API_KEY}" \
  python3 -c "from spanda_sdk import ControlCenterClient; c=ControlCenterClient(); assert c.health()['service']=='spanda-control-center'"

echo "== E3 Python SDK SRE incidents =="
PYTHONPATH="${ROOT}/packages/sdk-python/src:${PYTHONPATH:-}" \
  SPANDA_CONTROL_CENTER_URL="http://${BIND}" SPANDA_API_KEY="${SPANDA_API_KEY}" \
  python3 -c "
from spanda_sdk import ControlCenterClient
c = ControlCenterClient()
summary = c.sre_summary()
assert 'slo' in summary
created = c.create_incident('sdk-smoke', description='enterprise ops smoke')
iid = created['incident']['id']
c.ack_incident(iid, assignee='sdk')
c.resolve_incident(iid)
assert c.list_incidents()['incidents']
c.list_config_approvals()
c.list_compliance_evidence()
scorecard = c.executive_scorecard()['scorecard']
assert 'overall_score' in scorecard
thread = c.digital_thread_query()['digital_thread']
assert 'matched_node_count' in thread
assert 'snapshots' in c.list_config_snapshots()
"

echo "== E4 GET /v1/compliance/export?profile=defense =="
curl -sf -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  "http://${BIND}/v1/compliance/export?profile=defense" | grep -q audit_export_id

echo "== E4 GET /v1/compliance/export?profile=iso26262 =="
curl -sf -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  "http://${BIND}/v1/compliance/export?profile=iso26262" | grep -q audit_export_id

echo "== E4 GET /v1/compliance/profiles (signed catalog) =="
fetch /v1/compliance/profiles | grep -q defense
fetch /v1/compliance/profiles | grep -q iso26262
fetch /v1/compliance/profiles | grep -q iso13849

echo "== E4 POST /v1/reports/schedules =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"profile":"defense","format":"markdown","destination_url":"http://127.0.0.1:9/drop","interval_hours":24}' \
  "http://${BIND}/v1/reports/schedules" | grep -q '"id"'

echo "== E4 GET /v1/reports/schedules =="
fetch /v1/reports/schedules | grep -q schedules

echo "== E2 GET /v1/discovery?transport=mdns (TLS policy summary) =="
fetch "/v1/discovery?transport=mdns&timeout_ms=50" | grep -q require_tls

echo "== security audit prep =="
chmod +x "${ROOT}/scripts/security_audit_prep.sh"
SECURITY_AUDIT_LOG="$(mktemp "${TMPDIR:-/tmp}/spanda-security-audit.XXXXXX")"
"${ROOT}/scripts/security_audit_prep.sh" >"${SECURITY_AUDIT_LOG}" 2>&1
grep -q 'security-audit-prep ok' "${SECURITY_AUDIT_LOG}"
rm -f "${SECURITY_AUDIT_LOG}"

echo "== E4 GET /v1/digital-thread/query =="
fetch "/v1/digital-thread/query" | grep -q matched_node_count
fetch "/v1/digital-thread/query?lifecycle_phase=design" | grep -q lifecycle_rows

echo "== E4 GET /v1/executive/scorecard =="
fetch /v1/executive/scorecard | grep -q overall_score

echo "== E4 CLI govern shortcuts =="
run_spanda control-center scorecard | grep -q overall_score
run_spanda control-center digital-thread query | grep -q matched_node_count
run_spanda control-center audit list | grep -q record_count

echo "== E4 GET /v1/reports/export?format=markdown =="
curl -sf -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  "http://${BIND}/v1/reports/export?profile=defense&format=markdown" | grep -q executive

echo "== E4 GET /v1/reports/export?format=pdf =="
curl -sf -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  "http://${BIND}/v1/reports/export?profile=defense&format=pdf" | grep -q body_base64

echo "== OTLP GET /v1/observability/otlp/traces =="
fetch /v1/observability/otlp/traces | grep -q resourceSpans

echo "== OTLP GET /v1/observability/otlp/metrics =="
fetch /v1/observability/otlp/metrics | grep -q resourceMetrics

echo "== OTLP POST /v1/observability/otlp/export (mock Jaeger collector) =="
MOCK_OTLP_PORT=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
python3 "${ROOT}/scripts/mock_otlp_traces_collector.py" "${MOCK_OTLP_PORT}" &
MOCK_PID=$!
sleep 0.5
OTLP_RESPONSE=$(curl -s --max-time 10 -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  "http://${BIND}/v1/observability/otlp/export?endpoint=http://127.0.0.1:${MOCK_OTLP_PORT}/v1/traces")
echo "$OTLP_RESPONSE" | grep -q '"ok":true'
kill "$MOCK_PID" 2>/dev/null || true

echo "== WebSocket /v1/stream/telemetry =="
WS_PROBE_LOG="$(mktemp "${TMPDIR:-/tmp}/spanda-ws-probe.XXXXXX")"
ws_ok=false
for _attempt in 1 2 3 4 5; do
  if SPANDA_WS_STREAM_SECONDS=5 \
    python3 "${ROOT}/scripts/ws_telemetry_probe.py" "ws://${BIND}/v1/stream/telemetry" >"${WS_PROBE_LOG}" 2>/dev/null \
    && grep -q '"type":"hello"' "${WS_PROBE_LOG}"; then
    ws_ok=true
    break
  fi
  sleep 1
done
rm -f "${WS_PROBE_LOG}"
if [[ "${ws_ok}" != true ]]; then
  echo "websocket telemetry hello probe failed" >&2
  exit 1
fi

echo "== SDK POST /v1/programs/readiness =="
post_json() {
  local path="$1"
  local body="$2"
  curl -sf --max-time 30 -X POST \
    -H "Content-Type: application/json" \
    -d "$body" \
    "http://${BIND}${path}"
}
post_json /v1/programs/readiness "{\"file\":\"${PROGRAM}\"}" | grep -q '"report"'

echo "== SDK GET /v1/entities =="
fetch /v1/entities | grep -q '"entities"'

echo "== SDK Python SpandaClient (sdk/python) =="
PYTHONPATH="${ROOT}/sdk/python:${PYTHONPATH:-}" \
  SPANDA_CONTROL_CENTER_URL="http://${BIND}" \
  python3 -c "
from spanda import SpandaClient
c = SpandaClient()
assert c.list_entities()['count'] >= 0
report = c.readiness('${PROGRAM}')
assert 'report' in report
"

echo "Enterprise operations smoke OK"

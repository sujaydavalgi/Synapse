# Control Center

Web-based operational visibility for fleets, devices, readiness, and alerts. Phase E1 ships a REST API v1 and embedded UI served by the native CLI.

**Related:** [enterprise-operations-roadmap.md](./enterprise-operations-roadmap.md) · [device-pool.md](./device-pool.md) · [device-provisioning.md](./device-provisioning.md) · [telemetry-store.md](./telemetry-store.md) · [configuration.md](./configuration.md)

---

## Quick start

```bash
# Start API + UI (default http://127.0.0.1:8080)
export SPANDA_API_KEY="your-operator-key"
spanda control-center serve

# With project configuration (device pool from spanda.toml)
spanda control-center serve --config spanda.toml --bind 0.0.0.0:8080

# Native gRPC (tonic) on a separate port
spanda control-center serve --grpc-bind 127.0.0.1:50051
```

Open `http://127.0.0.1:8080/` for the Control Center UI, or use the **Control Center** view in `@spanda/web` (set API URL to the serve address).

---

## Native gRPC (tonic)

`spanda control-center serve --grpc-bind 127.0.0.1:50051` starts a tonic gRPC server alongside REST.

| RPC | Description |
|-----|-------------|
| `Health` | Liveness probe |
| `GetDashboard` | Dashboard JSON (device pool, fleet agents, alerts) |
| `ListDevices` | Device pool entries (`GET /v1/devices` parity) |
| `ListFleetAgents` | Registered fleet agents (`GET /v1/fleet/agents` parity) |
| `EvaluateReadiness` | Readiness rollup (`POST /v1/readiness/run` parity) |
| `GetSreSummary` | SRE availability rollup (`GET /v1/sre/summary` parity) |
| `GetTrustPackage` | Package trust score (`GET /v1/trust/package` parity) |
| `GetOpenApi` | OpenAPI 3.1 spec JSON (`GET /v1/openapi.json` parity) |
| `GetOtlpMetrics` | OTLP metrics preview (`GET /v1/observability/otlp/metrics`) |
| `GetHealthSummary` | Health rollup (`GET /v1/health/summary`) |
| `GetAssuranceSummary` | Assurance policy summary |
| `GetDiagnosisSummary` | Diagnosis policy summary |
| `GetExecutiveScorecard` | Executive scorecard |
| `QueryDigitalThread` | Digital thread query (`query` = URL query string) |
| `GetOtaStatus` | OTA rollout status |
| `DiscoverDevices` | Single-transport discovery (`query` = URL query string) |
| `RunDiscovery` | Multi-transport pool ingest (`POST /v1/devices/discover` parity) |
| `ProvisionDevice` | Provisioning workflow (`POST /v1/provision` parity) |
| `PlanOta` | OTA rollout plan (`POST /v1/ota/plan` parity) |
| `OperatorQuarantine` | Operator quarantine workflow |
| `OperatorMissionApprove` | Mission approval workflow |
| `ExportCompliance` | Compliance export (`query` = `profile=defense`, …) |
| `DetectDrift` | Operational drift report (`baseline_id` in request) |

Proto: `crates/spanda-api/proto/spanda/v1/control_center.proto`

```bash
# Example with grpcurl (reflection not enabled — use proto file)
grpcurl -plaintext -d '{}' 127.0.0.1:50051 spanda.v1.ControlCenter/Health
```

---

## REST API v1

| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/v1/health` | GET | — | Liveness |
| `/v1/dashboard` | GET | — | Device pool summary, fleet agent count, alerts |
| `/v1/devices` | GET | — | Device pool entries |
| `/v1/devices/{id}` | GET | — | Single device record |
| `/v1/devices/{id}` | PATCH | Bearer | Update `lifecycle_state` |
| `/v1/devices/discover` | POST | Bearer | Multi-transport discovery (registers matches) |
| `/v1/devices/{id}/provision` | POST | Bearer | Per-device provision workflow |
| `/v1/devices/{id}/assign` | POST | Bearer | Assign device to robot |
| `/v1/devices/{id}/quarantine` | POST | Bearer | Quarantine device |
| `/v1/devices/{id}/trust` | POST | Bearer (Approve) | Operator trust / approve device |
| `/v1/robots` | GET | — | Robot inventory from device tree |
| `/v1/fleets` | GET | — | Fleet inventory |
| `/v1/device-tree` | GET | — | Hierarchy + logical/physical mapping JSON |
| `/v1/readiness/run` | POST | — | Device readiness impact check |
| `/v1/device-reports` | GET | — | Inventory, trust, calibration reports |
| `/v1/fleet/agents` | GET | — | Registered fleet agents (`.spanda/fleet-agents.json`) |
| `/v1/alerts` | GET | — | Alert history |
| `/v1/alerts/test` | POST | Bearer | Dispatch test alert |
| `/v1/secrets` | GET | Bearer | Secret metadata (no values) |
| `/v1/rbac/matrix` | GET | — | Role permission matrix |
| `/v1/provision` | POST | Bearer | Run discover → ready workflow |
| `/v1/discovery` | GET | — | Package-backed discovery (`?transport=mdns` or `subnet`) |
| `/v1/config/snapshots` | GET/POST | POST: Bearer | List or save configuration snapshots |
| `/v1/health/summary` | GET | — | Device pool health rollup |
| `/v1/assurance/summary` | GET | — | Assurance policy from resolved config |
| `/v1/diagnosis/summary` | GET | — | Diagnosis policy from resolved config |
| `/v1/openapi.json` | GET | — | OpenAPI 3.1 specification |
| `/v1/drift` | GET | — | Operational drift vs baseline snapshot (`?baseline_id=`) |
| `/v1/ota/plan` | POST | Bearer | Plan canary / staged / blue_green rollout |
| `/v1/ota/status` | GET | — | OTA deploy state (`.spanda/deploy-state.json`) |
| `/v1/trust/package` | GET | — | Package trust evaluation (`?name=&version=`) |
| `/v1/sre/summary` | GET | — | Availability and alert rollup |
| `/v1/observability/traces` | GET | — | Recent API trace records |
| `/v1/observability/otlp/traces` | GET | — | OTLP/JSON trace preview for Jaeger |
| `/v1/observability/otlp/export` | POST | Bearer | Push API traces to OTLP collector |
| `/v1/stream/telemetry` | WebSocket | — | Live telemetry, traces, and alerts |
| `/v1/operator/quarantine` | POST | Bearer | Quarantine a device |
| `/v1/operator/mission/approve` | POST | Bearer | Approve or reject a mission |
| `/v1/rpc` | POST | — | gRPC-compatible JSON gateway |
| **gRPC (tonic)** | — | — | Native `ControlCenter` service on `--grpc-bind` (47 RPCs; full REST parity except JSON-RPC gateway) |
| `/v1/compliance/export` | GET/POST | Bearer | Accreditation bundle (`?profile=defense`) |
| `/v1/digital-thread/query` | GET | — | Trace chain (`?capability=`, `?device_id=`) |
| `/v1/executive/scorecard` | GET | — | Mission scorecard rollup |
| `/v1/analytics/readiness` | GET | — | Readiness trends and forecast |
| `/v1/reports/export` | GET | Bearer | Combined report (`format=markdown`, `json`, or `pdf`) |

Authenticate mutations with `Authorization: Bearer <SPANDA_API_KEY>`.

**API versioning:** `GET /v1/version` documents supported versions. Clients may send `X-Spanda-Api-Version: v1`; unsupported values return `400`. Breaking changes ship under a new `/v2/` path prefix.

**Rate limiting:** Set `SPANDA_API_RATE_LIMIT_PER_MINUTE` (per API key, or `anonymous` when unauthenticated). Excess requests return HTTP `429` with `Retry-After` (REST) or gRPC `RESOURCE_EXHAUSTED`.

Pass optional `X-Correlation-ID` on any request; the server echoes it on the response and records traces for `/v1/observability/traces`.

Govern-and-trace endpoints require a loaded program:

```bash
spanda control-center serve --config spanda.toml --program rover.sd
```

---

## Control Center UI sections

The `@spanda/web` Control Center panel includes:

| Section | Purpose |
|---------|---------|
| Dashboard | Pool summary, alerts, fleet agents |
| Devices | Device pool with lifecycle and assignment |
| Fleet | Fleets, robots, agents |
| Discovery | Multi-transport device discovery |
| Provisioning | Device inspect and provision workflows |
| Mapping | Logical ↔ physical mapping export |
| Health | Pool health rollup |
| Readiness | Readiness impact check |
| Traceability | Trust and identity trace view |

---

## Python SDK

```bash
pip install -e packages/sdk-python
export SPANDA_API_KEY=your-key
python -c "from spanda_sdk import ControlCenterClient; print(ControlCenterClient().health())"
```

Integration tests: `SPANDA_SDK_INTEGRATION=1 SPANDA_CONTROL_CENTER_URL=http://127.0.0.1:8080 pytest packages/sdk-python/tests`

WebSocket streaming (`pip install -e 'packages/sdk-python[stream]'`):

```python
from spanda_sdk import TelemetryStream
TelemetryStream("http://127.0.0.1:8080").wait_for_hello()
```

### OTLP traces (Jaeger)

Control Center API spans export as OTLP/JSON traces:

```bash
export SPANDA_OTLP_TRACES_ENDPOINT=http://localhost:4318/v1/traces
export SPANDA_OTLP_TRACE_AUTO_PUSH=1   # optional: push each API span
curl -H "Authorization: Bearer $SPANDA_API_KEY" \
  -X POST "http://127.0.0.1:8080/v1/observability/otlp/export"
```

Preview payload: `GET /v1/observability/otlp/traces`

Live telemetry WebSocket: `ws://127.0.0.1:8080/v1/stream/telemetry` (hello + telemetry/trace/alert events)

---

## Device Pool lifecycle

Devices in `[[devices]]` or the device tree carry optional lifecycle fields:

| State | Meaning |
|-------|---------|
| `discovered` | Seen but not verified |
| `quarantined` | Blocked pending review |
| `verified` | Identity and trust checks passed |
| `assigned` | Bound to a robot |
| `active` | Operational (`healthy` is an alias) |
| `degraded` / `offline` / `failed` | Runtime posture |
| `retired` | Removed from active pool |

Set in TOML:

```toml
[[devices]]
id = "lidar-front"
type = "lidar"
lifecycle_state = "active"
assigned_robot = "rover-1"
logical_name = "front_lidar"
```

See [device-pool.md](./device-pool.md) and [device-provisioning.md](./device-provisioning.md).

---

## Alerting

Configure delivery channels via environment variables:

| Variable | Effect |
|----------|--------|
| `SPANDA_ALERT_WEBHOOK_URL` | POST JSON alert payload |
| `SPANDA_ALERT_EMAIL_TO` | Email recipient (logs if `SPANDA_SMTP_HOST` unset) |
| `SPANDA_ALERT_EMAIL_DRY_RUN=1` | Log email without sending |

Default: log to stderr.

---

## Smoke test

```bash
./scripts/enterprise_ops_smoke.sh
./scripts/control_center_desktop_smoke.sh
```

---

## Desktop (Tauri)

Package: `@spanda/control-center-desktop` (`packages/control-center-desktop`).

1. Start the API: `spanda control-center serve --bind 127.0.0.1:8080`
2. Dev shell: `npm run control-center:desktop:dev` (Vite on port **5174**)
3. Optional API URL: `VITE_CONTROL_CENTER_URL=http://host:port`

The desktop shell reuses `ControlCenterPanel` from `@spanda/web`; it does not embed `spanda-api`. See [packages/control-center-desktop/README.md](../packages/control-center-desktop/README.md).

---

## Status

**Experimental** (Phase E1–E4). Includes device pool provisioning, multi-transport discovery, WebSocket telemetry streaming, OTLP trace export to Jaeger, compliance export, digital thread query, executive scorecard, report composer (including PDF), and Tauri desktop scaffold.

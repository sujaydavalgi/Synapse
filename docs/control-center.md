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

### Remote CLI (REST parity)

Query a running Control Center without curl — uses `SPANDA_CONTROL_CENTER_URL` (default `http://127.0.0.1:8080`) and `SPANDA_API_KEY` for mutations:

```bash
export SPANDA_CONTROL_CENTER_URL=http://127.0.0.1:8080
export SPANDA_API_KEY=your-operator-key

spanda control-center dashboard
spanda control-center drift --baseline-id <snapshot-id>
spanda control-center incidents list
spanda control-center approvals list
spanda control-center approvals submit --snapshot-id <id>
spanda control-center approvals approve <approval-id>
spanda control-center evidence list
spanda control-center sre summary
spanda control-center devices list
spanda control-center readiness run
spanda control-center ota plan --strategy canary --version 1.0 --dry-run
spanda control-center compliance export --profile defense
spanda control-center trust package --name spanda-mqtt

# Generic escape hatch for any /v1 path
spanda control-center api get /v1/sre/summary
spanda control-center api post /v1/ota/plan --body '{"strategy":"canary","version":"1.0","dry_run":true}'
```

---

## Native gRPC (tonic)

`spanda control-center serve --grpc-bind 127.0.0.1:50051` starts a tonic gRPC server alongside REST.

| RPC | Description |
|-----|-------------|
| `Health` | Liveness probe |
| `GetTenant` | Instance tenant scope (`GET /v1/tenant` parity) |
| `GetDashboard` | Dashboard JSON (device pool, fleet agents, alerts) |
| `ListDevices` | Device pool entries (`GET /v1/devices` parity) |
| `ListFleetAgents` | Registered fleet agents (`GET /v1/fleet/agents` parity) |
| `EvaluateReadiness` | Readiness rollup (`POST /v1/readiness/run` parity) |
| `GetSreSummary` | SRE availability rollup (`GET /v1/sre/summary` parity) |
| `ListSreIncidents` | Incident list (`GET /v1/sre/incidents` parity) |
| `CreateSreIncident` | Open incident (`POST /v1/sre/incidents` parity) |
| `GetTrustPackage` | Package trust score (`GET /v1/trust/package` parity) |
| `GetOpenApi` | OpenAPI 3.1 spec JSON (`GET /v1/openapi.json` parity) |
| `GetOtlpMetrics` | OTLP metrics preview (`GET /v1/observability/otlp/metrics`) |
| `GetObservabilityBackend` | Collector URL summary (`GET /v1/observability/backend`) |
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
| `ListComplianceEvidence` | Append-only evidence log (`GET /v1/compliance/evidence`) |
| `ListConfigApprovals` | Config publish approval queue (`GET /v1/config/approvals`) |
| `SubmitConfigApproval` | Submit snapshot for approval (`POST /v1/config/approvals`) |
| `ApproveConfigApproval` | Approve and publish snapshot (`POST /v1/config/approvals/{id}/approve`) |
| `RejectConfigApproval` | Reject pending approval (`POST /v1/config/approvals/{id}/reject`) |
| `DetectDrift` | Operational drift report (`baseline_id` in request) |

Proto: `crates/spanda-api/proto/spanda/v1/control_center.proto` — **proto semver `1.0.0`** (package `spanda.v1`). gRPC reflection is disabled; pin the proto file or read `GET /v1/version` → `grpc.proto_semver` and `grpc.rpc_count`.

```bash
# Example with grpcurl (reflection not enabled — use proto file)
grpcurl -plaintext -import-path crates/spanda-api/proto -proto spanda/v1/control_center.proto \
  -d '{}' 127.0.0.1:50051 spanda.v1.ControlCenter/Health
```

---

## REST API v1

| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/v1/health` | GET | — | Liveness |
| `/v1/tenant` | GET | — | Active tenant (`SPANDA_TENANT_ID`) |
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
| `/v1/discovery` | GET | — | Package-backed discovery (`?transport=mdns` or `subnet`); response includes `tls` policy summary |
| `/v1/config/snapshots` | GET/POST | POST: Bearer | List or save configuration snapshots (`encrypt: true` + `SPANDA_CONFIG_SNAPSHOT_KEY` for AES-256-GCM at rest) |
| `/v1/config/approvals` | GET | — | List config publish approval requests |
| `/v1/config/approvals` | POST | Bearer (Deploy) | Submit approval request for a snapshot (`required_approvals` or `SPANDA_CONFIG_APPROVALS_REQUIRED`) |
| `/v1/config/approvals/{id}/approve` | POST | Bearer (Approve) | Record approver vote; publish when quorum met (`quorum.received` / `quorum.required`) |
| `/v1/config/approvals/{id}/reject` | POST | Bearer (Approve) | Reject a pending config publish |
| `/v1/health/summary` | GET | — | Device pool health rollup |
| `/v1/assurance/summary` | GET | — | Assurance policy from resolved config |
| `/v1/diagnosis/summary` | GET | — | Diagnosis policy from resolved config |
| `/v1/openapi.json` | GET | — | OpenAPI 3.1 specification |
| `/v1/drift` | GET | — | Operational drift vs baseline snapshot (`?baseline_id=`) |
| `/v1/drift/scans` | GET | — | Recorded drift scan history |
| `/v1/drift/scan` | POST | Bearer | Trigger drift scan (optional `baseline_id`) |
| `/v1/ota/plan` | POST | Bearer | Plan canary / staged / blue_green rollout; `require_certify` enforced when `SPANDA_OTA_REQUIRE_CERTIFY` or `SPANDA_PRODUCTION_POLICY=production` |
| `/v1/ota/execute` | POST | Bearer | Execute rollout; `rollback_on_readiness_fail` gates post-deploy readiness; certification proof required in production policy |
| `/v1/ota/status` | GET | — | OTA deploy state (`.spanda/deploy-state.json`) |
| `/v1/trust/package` | GET | — | Package trust evaluation (`?name=&version=`) |
| `/v1/sre/summary` | GET | — | Availability, incidents, MTTR/MTBF hints, `health_trends`, `readiness_trends`, `slo`, and `burn_rate` |
| `/v1/sre/incidents` | GET | — | Incident list |
| `/v1/sre/incidents` | POST | Bearer | Open incident |
| `/v1/sre/incidents/{id}/ack` | POST | Bearer | Acknowledge incident |
| `/v1/sre/incidents/{id}/resolve` | POST | Bearer | Resolve incident |
| `/v1/integrations/pagerduty/webhook` | POST | Bearer | Inbound PagerDuty ack/resolve → incident workflow sync |
| `/v1/observability/traces` | GET | — | Recent API trace records |
| `/v1/observability/otlp/traces` | GET | — | OTLP/JSON trace preview for Jaeger |
| `/v1/observability/otlp/metrics` | GET | — | OTLP/JSON metrics preview |
| `/v1/observability/backend` | GET | — | Configured OTLP collector endpoints |
| `/v1/observability/otlp/export` | POST | Bearer | Push API traces to OTLP collector |
| `/v1/stream/telemetry` | WebSocket | — | Live telemetry, traces, and alerts |
| `/v1/operator/quarantine` | POST | Bearer | Quarantine a device |
| `/v1/operator/mission/approve` | POST | Bearer | Approve or reject a mission |
| `/v1/rpc` | POST | — | gRPC-compatible JSON gateway |
| **gRPC (tonic)** | — | — | Native `ControlCenter` service on `--grpc-bind` (60 RPCs; full REST parity except JSON-RPC gateway) |
| `/v1/compliance/export` | GET/POST | Bearer | Accreditation bundle (`?profile=defense`); appends immutable evidence log |
| `/v1/compliance/profiles` | GET | — | Signed profile catalog (defense, medical, ISO 26262) with Ed25519 verification |
| `/v1/compliance/evidence` | GET | Bearer | List append-only compliance evidence records |
| `/v1/digital-thread/query` | GET | — | Trace chain (`?capability=`, `?device_id=`, `?lifecycle_phase=`) |
| `/v1/executive/scorecard` | GET | — | Mission scorecard rollup |
| `/v1/analytics/readiness` | GET | — | Readiness trends and forecast |
| `/v1/reports/export` | GET | Bearer | Combined report (`format=markdown`, `json`, or `pdf`) |
| `/v1/reports/schedules` | GET | Bearer | List scheduled report delivery jobs |
| `/v1/reports/schedules` | POST | Bearer | Create scheduled webhook delivery (`profile`, `format`, `destination_url`, `interval_hours`) |
| `/v1/audit/mutations` | GET | Bearer | Hash-chained mutation audit trail |
| `/v1/audit/mutations/export` | GET | Bearer | SIEM export (`?format=cef` or `jsonl`) |

Authenticate mutations with `Authorization: Bearer <SPANDA_API_KEY>`.

**Multi-tenant isolation:** Set `SPANDA_TENANT_ID` on the Control Center instance (default `default`). API keys may include a `tenant_id` field in `SPANDA_API_KEYS_FILE` JSON; authenticated requests with a mismatched tenant return `403`.

**HA persistence:** Alert history and API trace log hydrate from `.spanda/control-center-alerts.json` and `.spanda/control-center-traces.json` on startup (override directory with `SPANDA_CONTROL_CENTER_STATE_DIR`).

**API versioning:** `GET /v1/version` documents supported versions. Clients may send `X-Spanda-Api-Version: v1`; unsupported values return `400`. Breaking changes ship under a new `/v2/` path prefix.

**Rate limiting:** Set `SPANDA_API_RATE_LIMIT_PER_MINUTE` (per API key, or `anonymous` when unauthenticated). Excess requests return HTTP `429` with `Retry-After` (REST) or gRPC `RESOURCE_EXHAUSTED`.

**Mutation audit:** Successful `POST`/`PATCH`/`PUT`/`DELETE` requests append hash-chained audit records (`GET /v1/audit/mutations`, Bearer required). Export for SIEM: `GET /v1/audit/mutations/export?format=cef|jsonl`. Persisted to `.spanda/control-center-mutations.jsonl` (override with `SPANDA_MUTATION_AUDIT_PATH`).

**Production policy:** Set `SPANDA_PRODUCTION_POLICY=production` to enable fleet defaults: OTA certification required (`SPANDA_OTA_REQUIRE_CERTIFY`), discovery TLS required (`SPANDA_DISCOVERY_REQUIRE_TLS`). See [stable-hardening-enterprise-ops.md](./stable-hardening-enterprise-ops.md).

**Scheduled reports:** Background delivery when `SPANDA_REPORT_SCHEDULE_INTERVAL_SECS` is set (e.g. `3600`). Schedules persist under `SPANDA_CONTROL_CENTER_STATE_DIR`.

**Discovery TLS:** `SPANDA_DISCOVERY_REQUIRE_TLS` blocks insecure `http://` / `mqtt://` endpoints; `SPANDA_DISCOVERY_TLS_CA_BUNDLE` points at vendor CA PEM. Registry package: `spanda-discovery-tls`.

**SRE burn monitor:** Fast-burn alerts when `SPANDA_SRE_BURN_SCAN_INTERVAL_SECS` > 0 (`SPANDA_SRE_BURN_RATE_FAST`, `SPANDA_SRE_BURN_WINDOW_HOURS`).

**Drift scans:** Background scheduler when `SPANDA_DRIFT_SCAN_INTERVAL_SECS` > 0.

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
| Digital Thread | Interactive capability→device graph with filters |
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
| `SPANDA_ALERT_DEDUP_WINDOW_*_SECS` | Per-severity dedup windows before dispatch |
| `SPANDA_PAGERDUTY_WEBHOOK_SECRET` | Optional HMAC for inbound PagerDuty webhook |

Registry packages: `spanda-alert-pagerduty` (bi-directional sync), `spanda-alert-escalation`, `spanda-alert-slack`, `spanda-alert-teams`.

Default: log to stderr.

---

## Smoke test

```bash
./scripts/enterprise_ops_smoke.sh
./scripts/control_center_desktop_smoke.sh
./scripts/security_audit_prep.sh      # third-party audit intake
./scripts/field_soak_gate.sh          # after 30-day pilot start date
./scripts/verify_sdk_publish_ready.sh # PyPI + npm pack readiness
```

---

## Stable promotion

Enterprise operations E1–E4 are **Experimental** with full stable-hardening checklist items **shipped in code**. Remaining gates before **Stable** tier: third-party security audit sign-off, 30-day field soak, and first production SDK/desktop releases. See [stable-hardening-enterprise-ops.md](./stable-hardening-enterprise-ops.md) · [field-soak-gate.md](./field-soak-gate.md) · [security-audit-third-party.md](./security-audit-third-party.md) · [desktop-release-runbook.md](./desktop-release-runbook.md).

---

## Desktop (Tauri)

Package: `@spanda/control-center-desktop` (`packages/control-center-desktop`).

1. Start the API: `spanda control-center serve --bind 127.0.0.1:8080`
2. Dev shell: `npm run control-center:desktop:dev` (Vite on port **5174**)
3. Optional API URL: `VITE_CONTROL_CENTER_URL=http://host:port`

The desktop shell reuses `ControlCenterPanel` from `@spanda/web`; it does not embed `spanda-api`. Production release: tag `desktop-v*`, CI workflow `.github/workflows/desktop-release.yml`, optional `./scripts/sign_tauri_macos.sh`. See [packages/control-center-desktop/README.md](../packages/control-center-desktop/README.md) · [desktop-release-runbook.md](./desktop-release-runbook.md).

---

## Status

**Experimental** (Phase E1–E4). Includes device pool provisioning, multi-transport discovery with production TLS policy, WebSocket telemetry streaming, OTLP trace/metrics export, SLO burn-rate monitor, PagerDuty bi-directional sync, compliance export with **signed profile catalog**, scheduled report delivery, digital thread query with **full lifecycle graph UI**, executive scorecard, report composer (including PDF), Grafana dashboard templates (`spanda-grafana-dashboards`), and Tauri desktop CI/signing scaffold.

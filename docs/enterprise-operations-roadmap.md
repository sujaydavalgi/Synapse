# Enterprise Operations Roadmap

Strategic expansion plan for Spanda as a **complete Autonomous Systems Platform** — production-ready for enterprise, industrial, robotics, medical, warehouse, agricultural, research, and defense deployments.

**Principle:** Every item strengthens at least one lifecycle phase: **Build · Verify · Simulate · Deploy · Operate · Observe · Recover · Govern · Audit · Continuously Improve** — without losing Spanda's core identity as a safety-first programming language and runtime.

**Lean-core rule:** Contracts and orchestration live in focused Rust crates; vendor integrations, transport adapters, and heavy UI ship in optional packages.

**Related:** [roadmap.md](./roadmap.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md) · [differentiation-roadmap.md](./differentiation-roadmap.md) · [feature-status.md](./feature-status.md) · [platform-overview.md](./platform-overview.md)

**Last updated:** 2026-06-26

---

## 0. Platform context

Spanda has evolved into a **complete Autonomous Systems Platform**. The enterprise operations expansion does **not** remove, redesign, or duplicate any existing capability — it adds operational governance, visibility, and integration surfaces on top of the pillars already shipped.

### Existing platform pillars (foundation)

| Category | Pillars already in the platform |
|----------|--------------------------------|
| **Build** | Language, Compiler, Package Ecosystem, Provider Registry |
| **Verify** | Hardware Verification, Capability Verification, Safety Validation, Package Trust (scoring engine) |
| **Simulate** | Simulation, Digital Twin, Replay |
| **Deploy** | Cascading Configuration, Device Tree, OTA deploy CLI, Deployment Gates |
| **Operate** | Readiness, Health, Fleet, Mission Assurance, Mission Continuity, Delegation, Takeover |
| **Observe** | Telemetry store, Persistent replay, Runtime fault detection |
| **Recover** | Diagnosis, Recovery (self-healing), Mission Continuity |
| **Govern** | Security, Encryption, RBAC hooks, Policy engine, Compliance profiles |
| **Audit** | Audit records, Decision audit trail, Explainability, Tamper Detection |
| **Trust** | Composite trust, Secure-boot attestation, Integrity verification |

### Integration mandate

Every new enterprise pillar must integrate with:

| Spine capability | Role in enterprise |
|------------------|-------------------|
| **Readiness** | Gates for provision, deploy, OTA promote, operator actions |
| **Assurance** | Anomaly, prognostics, assurance cases in Control Center |
| **Diagnosis** | Root-cause reports, drift correlation, incident context |
| **Recovery** | Operator approval, failover chains, alert escalation |
| **Trust** | Provisioning validation, package trust, device quarantine |
| **Health** | Fleet policies, SRE uptime, degraded/offline lifecycle |
| **Device Registry** | Device Pool inventory, discovery ingest, assignment |
| **Configuration** | Cascading TOML, snapshots, drift baselines |
| **Traceability** | Capability matrices, digital thread links |
| **Audit** | Mutation logging, compliance evidence, reporting |
| **Security** | RBAC, secrets, encryption, tamper response |
| **Packages** | Discovery transports, alert channels, SDK bindings |

### Lean-core rule (repeated)

Contracts and orchestration live in focused Rust crates (`spanda-api`, `spanda-config`, `spanda-security`, `spanda-ops`, `spanda-telemetry-store`, …). Vendor SDKs, transport adapters, time-series backends, alert channel integrations, and industry compliance packs ship in **optional packages**.

---

## 1. Platform pillar classification

Enterprise operations pillars compose existing engines — they do **not** replace Language, Runtime, Compiler, Verification, Safety, Simulation, Health, Fleet, Packages, or the maturity/differentiation roadmaps.

| # | Pillar | Lifecycle phase(s) | Tier | Primary outcome |
|---|--------|-------------------|------|-----------------|
| 1 | **Control Center** | Operate, Observe, Govern | Experimental | Web-based operational visibility |
| 2 | **Device Pool** | Deploy, Operate | Experimental | Central device inventory and lifecycle |
| 3 | **Device Discovery** | Deploy | Experimental | Host-backed core probes (`discovery_live`) + optional registry packages |
| 4 | **Provisioning** | Deploy, Verify | Experimental | Discover → verify → assign → ready workflow |
| 5 | **Configuration Management** | Deploy, Operate | Experimental | Versioned cascading TOML with snapshots |
| 6 | **RBAC** | Govern, Operate | Experimental | Role-based access for humans and services |
| 7 | **Secret Management** | Deploy, Govern, Security | Experimental | Encrypted credentials contract with rotation metadata |
| 8 | **Telemetry** | Operate, Observe | Experimental | Time-series health, readiness, mission data |
| 9 | **Alerting** | Operate, Recover | Experimental | Multi-channel incident notifications |
| 10 | **Configuration Drift** | Operate, Verify | Experimental | Expected vs actual parity across dimensions |
| 11 | **OTA & Rollback** | Deploy | Experimental | Canary, blue/green, phased rollout |
| 12 | **Package Trust** | Verify, Build | Experimental | Signature, reputation, vulnerability, coverage, compatibility scoring |
| 13 | **SDKs** | Build, Operate | Experimental | Python SDK, REST v1, JSON-RPC gateway, WebSocket stream; CLI as reference SDK |
| 14 | **Operator Workflows** | Operate, Recover | Experimental | Mission approval, takeover, quarantine, device trust |
| 15 | **SRE** | Operate, Observe | Experimental | SLO/SLA, MTTR/MTBF, incident reporting |
| 16 | **Reporting** | Govern, Audit | Experimental | Fleet, mission, compliance, executive exports (incl. PDF) |
| 17 | **Compliance** | Verify, Govern, Audit | Experimental | Evidence packs, immutable audit trails |
| 18 | **APIs** | All | Experimental | REST v1 + OpenAPI; JSON-RPC gateway; native gRPC (tonic) **Experimental** — `--grpc-bind`, 60 RPCs (full REST parity except `/v1/rpc`) |
| 19 | **Observability** | Operate, Observe | Experimental | Metrics, logs, traces, events; OTLP export; correlation IDs |
| 20 | **Digital Thread** | Build → Retire | Experimental | End-to-end traceability chain (v1 query) |

### Tier definitions

| Tier | Meaning |
|------|---------|
| **Experimental** | Shipped with caveats; CLI or partial UI; not enterprise-hardened |
| **Planned** | Design spec + integration contracts agreed; implementation scheduled |
| **Future** | Depends on Planned foundations; larger scope or external integrations |

### Existing foundations (do not rebuild)

| Capability | Current home | Enterprise reuse |
|------------|--------------|------------------|
| Device identity registry | `spanda-config::DeviceRegistry` | Device Pool v1 |
| Cascading configuration | `spanda-config::ConfigResolver` | Configuration Management |
| Config drift | `spanda-config::detect_config_drift` | Drift pillar (config dimension) |
| Device discovery CLI | `spanda device discover` (subnet, mdns, ble, usb, can, mqtt, ros2), `network scan`, `POST /v1/devices/discover` | Discovery v1 + pool ingest |
| Telemetry store | `spanda-telemetry-store` | Telemetry pillar |
| OTA deploy | `spanda-ota`, `deploy rollout` | OTA & Rollback |
| Package trust | `spanda-package::trust`, `spanda-trust` | Package Trust |
| Readiness / Health | `spanda-readiness`, health policies | Control Center, SRE, Alerting |
| Assurance / Diagnosis | `spanda-assurance` | Control Center, Reporting |
| Recovery | recovery planner + fleet mesh | Operator Workflows, Alerting |
| Security / Audit | `spanda-security`, `spanda-audit` | RBAC, Secrets, Compliance |
| Compliance profiles | `spanda-compliance` | Compliance pillar |
| Operations dashboard | `packages/web` Operations view | Control Center seed UI |
| Fleet agents / mesh | `spanda-fleet` | APIs, Telemetry ingest, Provisioning |
| Tamper / Trust | `spanda-tamper`, `spanda-trust` | Provisioning trust validation |

---

## 2. Core vs package ownership

### Core (lean workspace crates)

| Pillar | Core crate(s) | Responsibility |
|--------|---------------|----------------|
| Device Pool | extends `spanda-config` | Lifecycle state machine, inventory schema, assignment, trust, failover chains |
| Device Discovery | `spanda-config::discovery_live` + trait in `spanda-providers` | Host-backed probes; registry packages optional |
| Provisioning | `spanda-readiness` + `spanda-config` | Workflow orchestration, gate composition |
| Configuration Management | `spanda-config` | Versioning, snapshots, diff, rollback metadata |
| RBAC | `spanda-security` | Roles, permissions, policy evaluation |
| Secret Management | `spanda-security` | Secret store contract, encryption, rotation hooks |
| Telemetry | `spanda-telemetry-store` | Storage, query, aggregation APIs |
| Alerting | `spanda-ops` | Alert rules, routing, deduplication |
| Configuration Drift | `spanda-config` + `spanda-readiness` | Multi-dimensional drift reports |
| OTA & Rollback | `spanda-ota` | Rollout strategies, approval gates |
| Package Trust | `spanda-package`, `spanda-trust` | Scoring engine (exists) |
| Operator Workflows | `spanda-fleet` + `spanda-assurance` | Approval queues, takeover dispatch |
| SRE | extends `spanda-readiness` | SLO computation, incident aggregation |
| Reporting | `spanda-audit` + report composers | Report generation from engine outputs |
| Compliance | `spanda-compliance` | Evidence pack assembly (exists) |
| APIs | `spanda-api` | REST v1 + OpenAPI over CLI engines; JSON-RPC gateway |
| Observability | `spanda-telemetry-store` + OTLP | Trace log, OTLP export, WebSocket telemetry |
| Digital Thread | `spanda-capability` + `spanda-audit` | Traceability query v1 (`GET /v1/digital-thread/query`) |
| Control Center backend | `spanda-api` | Serves all UI modules |

### Packages (optional, vendor-specific)

| Package family | Examples | Pillar |
|----------------|----------|--------|
| Discovery transports | `spanda-discovery-mdns`, `spanda-discovery-ble`, `spanda-discovery-opcua`, `spanda-discovery-modbus` | Device Discovery |
| Alert channels | `spanda-alert-slack`, `spanda-alert-pagerduty`, `spanda-alert-teams` | Alerting |
| Secret backends | `spanda-secrets-vault`, `spanda-secrets-aws-sm`, `spanda-secrets-k8s` | Secret Management |
| Telemetry backends | `spanda-telemetry-timescale`, `spanda-telemetry-influx` | Telemetry |
| Observability exporters | `spanda-otel-collector` | Observability |
| SDK language bindings | `spanda-sdk-python` (official package) | SDKs |
| Control Center UI | `@spanda/web` (`ControlCenterPanel`) + embedded serve HTML | Control Center |
| Compliance industry packs | `spanda-compliance-medical`, `spanda-compliance-defense` | Compliance |
| Reporting templates | `spanda-report-executive`, `spanda-report-fleet` | Reporting |

**Rule:** If it requires a vendor SDK, cloud account, or third-party SaaS, it is a package — not core.

---

## 3. UI architecture (Control Center)

### Stack

| Layer | Technology | Notes |
|-------|------------|-------|
| UI framework | React + TypeScript | Extends existing `packages/web` |
| State | React Query + context | Server state from APIs; optimistic operator actions |
| Styling | Existing design tokens from web playground | Consistent with WASM demo |
| Desktop | Tauri (`@spanda/control-center-desktop`) | Wraps `ControlCenterPanel`; API via `spanda control-center serve` |
| Build | Vite | Shared with `@spanda/web` |

### Module map

```mermaid
flowchart TB
  subgraph ui ["Control Center UI (@spanda/web)"]
    DASH["Dashboard"]
    FLEET["Fleet View"]
    MISSION["Mission View"]
    POOL["Device Pool"]
    RDY["Readiness"]
    HLTH["Health"]
    ASR["Assurance"]
    DIAG["Diagnosis"]
    REC["Recovery"]
    SEC["Security"]
    CFG["Configuration"]
    SIM["Simulation"]
    RPL["Replay"]
    AUD["Audit"]
    ADM["Administration"]
  end

  subgraph api ["spanda-api (Rust)"]
    REST["REST /v1/*"]
    WS["WebSocket /v1/stream"]
    GRPC["gRPC SpandaService"]
  end

  subgraph engines ["Existing engines"]
    RDY_E["spanda-readiness"]
    ASR_E["spanda-assurance"]
    TEL["spanda-telemetry-store"]
    FLT["spanda-fleet"]
    CFG_E["spanda-config"]
    SEC_E["spanda-security"]
    AUD_E["spanda-audit"]
  end

  ui --> api
  api --> engines
```

### Module responsibilities

| Module | Data sources | Operator actions |
|--------|-------------|------------------|
| **Dashboard** | Readiness rollup, fleet health, active alerts, mission count | Navigate to detail views |
| **Fleet View** | Fleet mesh, agent registry, swarm state | Orchestrate, remote commands |
| **Mission View** | Mission plans, progress, contracts, continuity state | Approve, pause, resume, cancel |
| **Device Pool** | `DeviceRegistry`, lifecycle state, assignments | Assign, quarantine, retire |
| **Readiness** | `spanda readiness`, trends, gates | Run readiness, record snapshot |
| **Health** | Health policies, fault timeline | View degraded devices |
| **Assurance** | Assurance cases, anomaly reports | Run assure, view prognostics |
| **Diagnosis** | `spanda diagnose`, root-cause reports | Trigger diagnosis |
| **Recovery** | Recovery planner, knowledge store | Approve recovery, execute heal |
| **Security** | Trust scores, tamper alerts, secure-boot status | View integrity, quarantine |
| **Configuration** | Resolved config, diff, history, approval queue | Approve, rollback config |
| **Simulation** | Active sim sessions, twin state | Launch sim, inject faults |
| **Replay** | Trace library, deterministic playback | Replay, time-travel scrub |
| **Audit** | Decision audit trail, compliance evidence | Export evidence packs |
| **Administration** | RBAC, secrets metadata, API keys | Manage users, roles, integrations |

### Evolution from current UI

`packages/web` Operations view (readiness scoring, live agent fetch, continuity panel, WASM telemetry) becomes the **Dashboard + Readiness + Health** seed. New modules add incrementally without replacing the playground.

---

## 4. Backend API architecture

### Design principles

1. **CLI parity** — every `spanda` command maps to an API endpoint; no CLI-only capabilities.
2. **Versioned** — `/v1/` prefix; breaking changes require `/v2/`.
3. **Engine delegation** — APIs are thin wrappers over existing crates; no duplicate business logic.
4. **Auth at boundary** — RBAC middleware on all mutating endpoints.
5. **Audit on mutation** — every deploy, approve, override, recover writes to `spanda-audit`.

### Proposed crate: `spanda-api`

```
spanda-api/
  src/
    rest/          # axum or actix-web handlers
    grpc/          # tonic service definitions
    ws/            # WebSocket streaming (telemetry, alerts)
    auth/          # RBAC middleware
    openapi/       # OpenAPI 3.1 spec generation
```

### API surface (representative)

| Domain | REST | gRPC | CLI equivalent |
|--------|------|------|----------------|
| Readiness | `GET /v1/readiness/{robot}` | `EvaluateReadiness` | `spanda readiness` |
| Health | `GET /v1/health/{robot}` | `GetHealth` | health_check runtime |
| Assurance | `POST /v1/assure` | `RunAssurance` | `spanda assure` |
| Diagnosis | `POST /v1/diagnose` | `Diagnose` | `spanda diagnose` |
| Recovery | `POST /v1/recovery/execute` | `ExecuteRecovery` | `spanda recover` |
| Fleet | `GET /v1/fleet/agents` | `ListAgents` | `spanda fleet agent list` |
| Devices | `GET /v1/devices` | `ListDevices` | `spanda device discover` |
| Provisioning | `POST /v1/provision` | `ProvisionDevice` | (new workflow) |
| Config | `GET /v1/config/resolve` | `ResolveConfig` | `spanda config resolve` |
| Drift | `GET /v1/drift` | `DetectDrift` | `spanda drift` |
| Deploy | `POST /v1/deploy/rollout` | `Rollout` | `spanda deploy rollout` |
| Trust | `GET /v1/trust/{target}` | `EvaluateTrust` | `spanda trust` |
| Telemetry | `GET /v1/telemetry` | `QueryTelemetry` | `spanda telemetry` |
| Alerts | `GET /v1/alerts` | `StreamAlerts` | (new) |
| Audit | `GET /v1/audit` | `QueryAudit` | `spanda audit` |
| Missions | `POST /v1/missions/{id}/approve` | `ApproveMission` | operator workflow |
| Secrets | `POST /v1/secrets` | `RotateSecret` | (new) |

### Deployment modes

| Mode | Description |
|------|-------------|
| **Embedded** | `spanda control-center serve` — single-process API + static UI |
| **Fleet mesh** | Extends existing fleet mesh coordinator with API routes |
| **Standalone** | `spanda-api` container for enterprise deployments |
| **Edge agent** | Lightweight agent on robot; syncs to central Control Center |

Existing fleet agent endpoints (`/v1/status`, `/v1/recovery/execute`, `/v1/continuity/execute`, `/v1/fleet/telemetry/ingest`) remain the edge contract; Control Center aggregates them.

---

## 5. Integration map

Every enterprise pillar integrates with the platform spine:

```mermaid
flowchart LR
  subgraph spine ["Platform spine"]
    RDY["Readiness"]
    ASR["Assurance"]
    DIAG["Diagnosis"]
    REC["Recovery"]
    TRST["Trust"]
    HLTH["Health"]
    DEV["Device Registry"]
    CFG["Configuration"]
    TRACE["Traceability"]
    AUD["Audit"]
    SEC["Security"]
    PKG["Packages"]
  end

  subgraph enterprise ["Enterprise pillars"]
    CC["Control Center"]
    DP["Device Pool"]
    DISC["Discovery"]
    PROV["Provisioning"]
    CFM["Config Mgmt"]
    RBAC["RBAC"]
    SECM["Secrets"]
    TEL["Telemetry"]
    ALT["Alerting"]
    DRFT["Drift"]
    OTA["OTA"]
    PTR["Package Trust"]
    SDK["SDKs"]
    OPS["Operator Workflows"]
    SRE["SRE"]
    RPT["Reporting"]
    CMP["Compliance"]
    API["APIs"]
    OBS["Observability"]
    DT["Digital Thread"]
  end

  enterprise --> spine
```

### Cross-pillar integration matrix

| Pillar | Readiness | Assurance | Diagnosis | Recovery | Trust | Health | Device Reg | Config | Trace | Audit | Security | Packages |
|--------|-----------|-----------|-----------|----------|-------|--------|------------|--------|-------|-------|----------|----------|
| Control Center | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Device Pool | ✓ | | | | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | |
| Discovery | ✓ | | | | ✓ | | ✓ | ✓ | ✓ | | ✓ | ✓ |
| Provisioning | ✓ | ✓ | ✓ | | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Config Mgmt | ✓ | | | | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| RBAC | ✓ | ✓ | | ✓ | ✓ | ✓ | ✓ | ✓ | | ✓ | ✓ | |
| Secrets | | | | | ✓ | | ✓ | ✓ | | ✓ | ✓ | ✓ |
| Telemetry | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | | ✓ | ✓ | ✓ | |
| Alerting | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | | ✓ | ✓ | |
| Drift | ✓ | | ✓ | | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| OTA | ✓ | ✓ | | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Package Trust | ✓ | | | | ✓ | | | ✓ | ✓ | ✓ | ✓ | ✓ |
| SDKs | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Operator Workflows | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | |
| SRE | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | | ✓ | ✓ | ✓ | |
| Reporting | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Compliance | ✓ | ✓ | | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| APIs | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Observability | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | | ✓ | ✓ | ✓ | |
| Digital Thread | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

---

## 6. Pillar specifications

### 6.1 Control Center

Web-based operational visibility for robots, fleets, swarms, devices, sensors, missions, readiness, health, trust, security, and diagnostics.

| Layer | Stack |
|-------|-------|
| UI | React + TypeScript (`ControlCenterPanel` in `@spanda/web`) |
| State | React Query + context |
| Desktop | Tauri (`@spanda/control-center-desktop`) — CI signing scaffold + env-gated auto-update ([desktop-release-runbook.md](./desktop-release-runbook.md)) |
| Backend | Rust `spanda-api` (`spanda control-center serve`) |
| Build | Vite (shared with `@spanda/web`) |

**Modules:** Dashboard, Fleet View, Mission View, Device Pool, Readiness, Health, Assurance, Diagnosis, Recovery, Security, Configuration, Simulation, Replay, Audit, Administration.

### 6.2 Device Pool

Central inventory extending `DeviceRegistry` with lifecycle states:

**Device types:** Robots, Sensors, Actuators, Accessories, Compute Modules, Controllers, Gateways, Cameras, GPS, Lidar, Radar, BLE Devices, WiFi Devices, LTE Devices, 5G Devices, USB Devices, CAN Devices, EtherCAT Devices, PLC Devices.

**Lifecycle states:**

```
Discovered → Quarantined → Verified → Assigned → Active → Healthy ⇄ Degraded → Offline → Failed → Retired
```

| State | Meaning |
|-------|---------|
| **Discovered** | Seen by discovery or manual entry; not yet validated |
| **Quarantined** | Held pending trust/health review; no mission assignment |
| **Verified** | Identity, firmware, and capability gates passed |
| **Assigned** | Bound to robot/fleet; config layer applied |
| **Active** | Deployed and reporting heartbeat (runtime substate) |
| **Healthy** | Health policy pass; readiness above threshold |
| **Degraded** | Partial fault; may continue with constraints |
| **Offline** | No heartbeat within policy window |
| **Failed** | Unrecoverable fault or repeated recovery failure |
| **Retired** | End of life; audit record retained |

**Operations:** `assign` / `unassign`, `quarantine`, `retire`, **`trust`** (API `POST /v1/devices/{id}/trust`, CLI `spanda device trust`, Control Center Trust/Approve); failover chains wired into recovery (`enrich_recovery_plan_with_failover`).

**Core schema:** extends `[[devices]]` in `spanda.toml` with `lifecycle_state`, `assigned_robot`, `last_seen`, `provisioning_id`, `trust_level`.

### 6.3 Device Discovery

Core host-backed probes (`discovery_live`) with optional registry packages; discovery POST ingests matches into the device pool (`ingest_discovery_matches`).

| Transport | Core / package | Status |
|-----------|----------------|--------|
| IP subnet | core (`network scan`) | **Experimental** |
| Manual | core (`[[devices]]`) | **Experimental** |
| mDNS | core (`dns-sd` / `avahi-browse`) + `spanda-discovery-mdns` registry | **Experimental** (host probe + runtime package wrap when `packages/registry/spanda-discovery-mdns` present) |
| DNS-SD | core (same probe path as mDNS) + `spanda-discovery-mdns` | **Experimental** |
| USB | core (`lsusb` probe) + `spanda-discovery-usb` | **Experimental** (host probe) / **Planned** (package) |
| Bluetooth / BLE | core (`bluetoothctl` probe) + `spanda-discovery-ble` | **Experimental** (host probe) / **Planned** (package) |
| WiFi | core (subnet + mDNS correlation) + `spanda-discovery-wifi` | **Experimental** (registry stub) |
| LTE / 5G | modem status via device agent report + `spanda-discovery-cellular` | **Experimental** (registry stub) |
| CAN / EtherCAT | core (socketcan probe) + `spanda-discovery-can`, `spanda-discovery-ethercat` | **Experimental** (host probe) / **Planned** (package) |
| ROS2 / DDS | core (`ros2 topic list` probe) + `spanda-ros2` | **Experimental** |
| MQTT | core (broker probe) + `spanda-mqtt` | **Experimental** |
| OPC-UA | `spanda-opcua`, `spanda-discovery-opcua` | **Experimental** (stubs) |
| Modbus TCP / RTU | `spanda-modbus`, `spanda-discovery-modbus` | **Experimental** (stubs) |
| Serial | `spanda-discovery-serial` | **Experimental** (registry stub) |

### 6.4 Provisioning workflow

```
Discover → Verify Identity → Trust Validation → Firmware Validation
  → Health Validation → Capability Validation → Assign → Ready
```

Each gate composes existing engines:

| Step | Engine | Gate |
|------|--------|------|
| Verify Identity | `DeviceRegistry` + network validation | Duplicate IP/MAC/serial check |
| Trust Validation | `spanda-trust`, `spanda-tamper` | Composite trust ≥ threshold |
| Firmware Validation | `spanda-tamper::secure_boot` | Attestation match |
| Health Validation | `spanda-readiness` | Health policy pass |
| Capability Validation | `spanda-verify` | Capability matrix match |
| Assign | `DeviceRegistry` | Robot/fleet binding |
| Ready | `spanda-readiness` | Deployment gate pass |

### 6.5 Configuration Management

Extends cascading TOML ([cascading-config.md](./cascading-config.md)):

| Feature | Status | Notes |
|---------|--------|-------|
| Environment overrides | **Experimental** | `[extends]` layers |
| Deployment / robot / fleet overrides | **Experimental** | `ConfigResolver` |
| Versioning | **Experimental** | Config snapshot IDs |
| Rollback | **Experimental** | Point-in-time restore via snapshots API |
| Snapshots | **Experimental** | `.spanda/config-snapshots/`, `GET|POST /v1/config/snapshots` |
| History | Planned | Audit-linked change log |
| Diff | **Experimental** | `spanda config diff` |
| Approval | **Experimental** | `GET/POST /v1/config/approvals`, approve/reject subpaths |

### 6.6 RBAC

| Role | Deploy | Operate | Approve | Override | Shutdown | Recover | Delete | Provision |
|------|--------|---------|---------|----------|----------|---------|--------|-----------|
| Administrator | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Developer | ✓ | ✓ | | | | | | |
| Operator | | ✓ | | | ✓ | ✓ | | |
| Supervisor | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | | ✓ |
| Safety Officer | | ✓ | ✓ | | ✓ | | | |
| Auditor | | | | | | | | |
| Guest | | | | | | | | |

Core: `spanda-security::rbac` with JWT/API-key auth at API boundary.

### 6.7 Secret Management

Secures: API keys, certificates, private keys, robot credentials, cloud credentials, provider credentials.

Features: rotation, expiration, audit trail, encryption at rest (AES-256-GCM, existing wire crypto).

Package backends: HashiCorp Vault, AWS Secrets Manager, Kubernetes secrets.

### 6.8 Telemetry

Builds on [telemetry-store.md](./telemetry-store.md):

| Signal | Status |
|--------|--------|
| Health, Readiness | **Experimental** |
| CPU, Memory, Battery, Temperature | **Experimental** |
| GPS, Connectivity | **Experimental** |
| Events, Diagnostics | **Experimental** |
| Mission Progress, Recovery Events | **Experimental** |
| Time-series history | Planned (package backend) |
| Trend analysis | **Experimental** (`readiness trends`) |
| Forecasting | Planned |

### 6.9 Alerting

| Channel | Package |
|---------|---------|
| Email | `spanda-alert-email` |
| Slack | `spanda-alert-slack` |
| Microsoft Teams | `spanda-alert-teams` |
| Discord | `spanda-alert-discord` |
| SMS | `spanda-alert-sms` |
| Webhook | core (generic HTTP POST) |
| PagerDuty | `spanda-alert-pagerduty` |

**Alert types:** Mission Failure, Robot Offline, Crash, Reboot, Memory Leak, Tamper, Security, Low Battery, Health Critical, Readiness Failed, Recovery Failed.

Core: `spanda-ops::alerting` — rule engine, per-severity deduplication, severity routing. PagerDuty bi-directional sync via `POST /v1/integrations/pagerduty/webhook`.

### 6.10 Configuration Drift

Extends [drift-detection.md](./drift-detection.md):

| Dimension | Detector | Status |
|-----------|----------|--------|
| Configuration | `detect_config_drift` | **Experimental** |
| Firmware | attestation vs baseline | **Experimental** |
| Package | lockfile vs agent report | **Experimental** (operational drift API) |
| Provider | resolved vs runtime dispatch | **Experimental** |
| Capability | matrix vs runtime grants | **Experimental** |
| Policy | declared vs enforced | **Experimental** |
| Safety | certify hash vs runtime | **Experimental** |

Scheduled scans: `SPANDA_DRIFT_SCAN_INTERVAL_SECS`, `GET /v1/drift/scans`, `POST /v1/drift/scan`.

### 6.11 OTA & Rollback

Extends existing `spanda deploy plan|rollout|rollback|status`:

| Strategy | Status |
|----------|--------|
| Version pinning | **Experimental** |
| Rollback | **Experimental** |
| Approval gates (`--require-certify`) | **Experimental** |
| Canary | **Experimental** | `POST /v1/ota/plan` dry-run |
| Blue/Green | **Experimental** | `RolloutStrategy::BlueGreen` dry-run |
| Phased rollout | **Experimental** | staged strategy in OTA plan API |

### 6.12 Package Trust

Evaluates packages before install and deploy; composes with deployment gates and Control Center Security module.

| Signal | Weight factor | Source |
|--------|---------------|--------|
| Package signature | Ed25519 verify | `spanda-package` registry |
| Maintainer reputation | Historical publish record | registry metadata |
| Security scan | Static analysis pass/fail | `spanda-package` scan hook |
| Known vulnerabilities | CVE/advisory match | vulnerability DB package |
| Test coverage | Declared + CI evidence | package manifest |
| Compatibility | Target profile match | `spanda verify` matrix |

**Output:** composite **Trust Score** (0–100) via `evaluate_package_trust` / `GET /v1/trust/package` / `spanda trust <package>`.

### 6.13 SDKs

Official SDK surfaces for external systems to interact with Readiness, Assurance, Diagnosis, Recovery, Health, Mission, and Fleet:

| SDK | Status | Notes |
|-----|--------|-------|
| **CLI** | **Stable** | Reference implementation; all capabilities |
| **REST** | **Experimental** | `/v1/*` + OpenAPI 3.1 (`GET /v1/openapi.json`) |
| **gRPC** | **Experimental** | Native tonic `ControlCenter` service (`--grpc-bind`, 16 RPCs); JSON-RPC gateway (`POST /v1/rpc`) also ships |
| **WebSocket** | **Experimental** | `WS /v1/stream/telemetry` live telemetry |
| **Python** | **Experimental** | `packages/sdk-python` (`pip install spanda-sdk`) |

### 6.14 Operator Workflows

| Workflow | Integration |
|----------|-------------|
| Mission Approval | `requires approval`, RBAC, audit |
| Mission Pause / Resume / Cancel | runtime + fleet mesh |
| Manual Takeover | `spanda takeover`, continuity framework |
| Emergency Stop | kill switch + safety engine |
| Recovery Approval | `SPANDA_OPERATOR_APPROVAL`, RBAC |
| Device Assignment | Device Pool |
| Device Trust / Approve | `POST /v1/devices/{id}/trust`, `spanda device trust`, Control Center |
| Device Quarantine | Device Pool lifecycle + trust downgrade |

### 6.15 SRE

| Metric | Source | Status |
|--------|--------|--------|
| SLO / SLA | readiness history + telemetry | **Experimental** (`slo` on `GET /v1/sre/summary`, `SPANDA_SRE_SLO_PERCENT`) |
| Availability / Uptime | health_check + agent heartbeat | **Experimental** (`GET /v1/sre/summary`) |
| MTTR | resolved incidents | **Experimental** (`mttr_hint_ms` on summary) |
| MTBF | fault-class alert timeline | **Experimental** (`mtbf_hint_ms` on summary) |
| Crash / Recovery statistics | `spanda-runtime-faults` | Planned |
| Incident workflow | `GET/POST /v1/sre/incidents`, ack/resolve, auto-open from critical alerts | **Experimental** |
| Health trends | device pool + readiness history | **Experimental** (`health_trends`, `readiness_trends` on summary) |

### 6.16 Reporting

| Report | Engines | Export formats |
|--------|---------|----------------|
| Fleet | readiness, health, fleet mesh | HTML, Markdown, JSON, PDF, CSV |
| Mission | assurance, contracts, continuity | HTML, Markdown, JSON, PDF, CSV |
| Health / Readiness | readiness, health | HTML, Markdown, JSON, PDF, CSV |
| Security / Trust | tamper, trust, security assurance | HTML, Markdown, JSON, PDF, CSV |
| Compliance | compliance profiles, audit | HTML, Markdown, JSON, PDF, CSV |
| Configuration | config resolver, drift | HTML, Markdown, JSON, PDF, CSV |
| Recovery | recovery planner, knowledge | HTML, Markdown, JSON, PDF, CSV |
| Executive Dashboard | scorecard | HTML, PDF |

### 6.17 Compliance

Evidence packs for regulatory and internal audit workflows:

| Capability | Status | Integration |
|------------|--------|-------------|
| Evidence packs | **Experimental** | `GET /v1/compliance/export` |
| Approval history | **Experimental** | audit + operator workflow records |
| Audit trails | **Stable** | `spanda-audit`, decision audit trail |
| Digital signatures | **Experimental** | signed message + export bundles |
| Immutable evidence | **Experimental** | append-only `.spanda/evidence-append.jsonl`; `GET /v1/compliance/evidence` |
| Policy compliance | **Experimental** | `spanda verify --policy` |
| Safety compliance | **Experimental** | safety coverage + certify metadata |
| Mission compliance | **Experimental** | mission contracts + assurance cases |

### 6.18 APIs

REST and gRPC APIs with **CLI parity** — every `spanda` command maps to an endpoint; no CLI-only capabilities.

| Surface | Status |
|---------|--------|
| REST `/v1/*` | **Experimental** |
| OpenAPI 3.1 | **Experimental** |
| JSON-RPC gateway | **Experimental** (`POST /v1/rpc`) |
| Native gRPC (tonic) | **Experimental** | 60 RPCs; `--grpc-bind` |
| API versioning | `/v1/` prefix; breaking changes require `/v2/` |

### 6.19 Observability

| Signal | Status | Integration |
|--------|--------|-------------|
| Metrics | **Experimental** | OTLP metrics preview/export (`GET/POST /v1/observability/otlp/metrics`) |
| Logs | **Experimental** | trace log + structured audit |
| Traces | **Experimental** | OTLP export to Jaeger; correlation IDs (`X-Correlation-ID`); HA trace log persistence |
| Events | **Experimental** | telemetry store + alerting |
| Health / Readiness | **Experimental** | Control Center + SRE rollup |
| Distributed tracing | **Experimental** | `spanda-otel-collector` package + `SPANDA_OTEL_COLLECTOR_URL`; `GET /v1/observability/backend` |
| OpenTelemetry | **Experimental** | `POST /v1/observability/otlp/export` |

### 6.20 Digital Thread

End-to-end traceability chain (v1 query shipped; full lifecycle graph UI planned):

Requirement → Mission → Capability → Hardware → Device → Provider → Package → Simulation → Verification → Deployment → Runtime → Recovery → Evidence → Audit → Retirement

Builds on `spanda-capability` traceability matrices + `spanda-audit` + mission contracts.

---

## 7. Phased implementation plan

### Priority horizons

| Horizon | Timeline | Pillars |
|---------|----------|---------|
| **NOW** | 0–6 months (v0.5–v0.6) | Control Center, Device Pool, Provisioning, Telemetry, Alerting, RBAC, Secrets — **E1 shipped** (experimental) |
| **NEXT** | 6–12 months (v0.6–v0.7) | SDKs, Configuration Drift (full), OTA strategies, Package Trust (UI), Observability — **E2–E3 shipped** (experimental) |
| **LATER** | 12–18 months (v0.8–v1.0) | Compliance Packs, Executive Dashboards, Digital Thread (full graph UI), **Predictive Analytics** (readiness forecasting, anomaly trends) — **E4 shipped** (experimental; Tauri scaffold) |

### Phase E1 — Control plane foundation (v0.5+, Q3–Q4 2026)

**Theme:** Operators can see and govern the fleet from a browser.

| Deliverable | Component | Depends on |
|-------------|-----------|------------|
| `spanda-api` REST v1 | `spanda-api` crate | existing engines |
| Control Center shell | `ControlCenterPanel` in `@spanda/web` + embedded HTML | `spanda-api` |
| Dashboard + Fleet + Readiness modules | UI modules | telemetry, readiness APIs |
| Device Pool schema + lifecycle | extends `spanda-config` | `DeviceRegistry` |
| RBAC v1 (API keys + 4 roles) | `spanda-security` | audit |
| Secret store contract | `spanda-security` | encryption |
| Alerting core (webhook + email) | `spanda-ops` | telemetry events |

**Exit criteria:** `spanda control-center serve` + `scripts/enterprise_ops_smoke.sh` — **shipped** (wired into `scripts/showcase_smoke.sh`).

### Phase E2 — Provision and observe (v0.6, Q1 2027)

**Theme:** Devices enter the fleet through a verified pipeline; operators get notified.

| Deliverable | Component |
|-------------|-----------|
| Provisioning workflow API | readiness + trust + verify gates |
| Device Pool UI | assign, quarantine, lifecycle |
| Discovery packages (mDNS, BLE, OPC-UA) | optional packages |
| Telemetry time-series backend package | `spanda-telemetry-timescale` |
| Alerting packages (Slack, PagerDuty) | optional packages |
| Health + Assurance + Diagnosis UI modules | Control Center |
| Config versioning + snapshots | `spanda-config` |

**Exit criteria:** End-to-end provision demo; alert on readiness failure — **shipped** (`scripts/enterprise_ops_smoke.sh`).

### Phase E3 — Deploy and integrate (v0.7, Q2 2027)

**Theme:** External systems integrate; deployments are safe and reversible.

| Deliverable | Component |
|-------------|-----------|
| Python SDK + REST OpenAPI | `spanda-sdk-python`, OpenAPI spec |
| gRPC service | `spanda-api::grpc` — **shipped** (16 RPCs through E4 summaries + OTLP metrics) |
| OTLP metrics export | `spanda-ops::otlp_metrics`, `GET /v1/observability/otlp/metrics` — **shipped** |
| Full drift detection (7 dimensions) | `detect_operational_drift_full` — **shipped** (config + program + agents + policy) |
| OTA canary + phased rollout | `spanda-ota` |
| Package Trust UI | Control Center Security module |
| Observability (OpenTelemetry export) | OTLP + correlation IDs |
| Operator Workflows UI | mission approve, takeover, quarantine |
| SRE dashboard | MTTR/MTBF, incident reports |

**Exit criteria:** SDK integration test; canary deploy demo; correlation trace API — **shipped** (`scripts/enterprise_ops_smoke.sh`, `packages/sdk-python`). Full OTLP trace export to Jaeger and WebSocket telemetry SDK — **shipped** (`POST /v1/observability/otlp/export`, `WS /v1/stream/telemetry`).

### Phase E4 — Govern and trace (v1.0, 2027)

**Theme:** Enterprise audit, compliance, and executive visibility.

| Deliverable | Component |
|-------------|-----------|
| Compliance evidence packs (UI) | `spanda-compliance` + Control Center Audit |
| Executive dashboards | scorecard + reporting templates |
| Digital Thread v1 | capability traceability graph + interactive UI |
| Predictive analytics | readiness forecasting + anomaly trends |
| Reporting exports (PDF) | report composer |
| Tauri desktop packaging | `@spanda/control-center-desktop` |
| WebSocket SDK | real-time telemetry stream |

**Exit criteria:** Compliance report export; signed profile catalog; scheduled report delivery; digital thread lifecycle graph — **shipped** (`scripts/enterprise_ops_smoke.sh`). PDF executive export — **shipped** (`format=pdf`). Tauri desktop CI/signing scaffold — **shipped** (`scripts/control_center_desktop_smoke.sh`, `scripts/build_control_center_desktop.sh`, `.github/workflows/desktop-release.yml`). Stable promotion gates — [stable-hardening-enterprise-ops.md](./stable-hardening-enterprise-ops.md).

---

## 8. Risks and mitigation

| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|
| UI scope creep — 15 modules at once | High | High | Phase E1 ships Dashboard + Fleet + Readiness only; add modules incrementally |
| API/CLI divergence | High | Medium | OpenAPI generated from same handler traits; contract tests |
| RBAC bypass on fleet agents | High | Medium | Agent auth tokens; mesh mTLS; audit all mutations |
| Secret leakage in logs/traces | High | Low | Redaction middleware; secret references only (never values) |
| Telemetry storage at scale | Medium | High | Package backends (Timescale/Influx); retention policies exist |
| Discovery false positives | Medium | Medium | Quarantine lifecycle; human verification step in provisioning |
| Alert fatigue | Medium | High | Deduplication, severity routing, SLO-based thresholds |
| OTA bricking robots | High | Low | Canary + rollback; `--require-certify`; readiness gate before promote |
| Package trust gaming | Medium | Medium | Transparent scoring; community appeals (existing) |
| Compliance liability | Medium | Low | Template-only disclaimer (existing); not accredited regulatory approval |
| Tauri packaging complexity | Low | Medium | Scaffold shipped; web-first; installers not published |
| Duplicate drift logic | Medium | Medium | Single `spanda-config::drift` module; dimensions as plugins |
| Breaking JSON schemas | High | Low | Version fields on all API responses |
| Docs ahead of code | Low | Medium | Enterprise ops pillars marked **Experimental** once `enterprise_ops_smoke.sh` passes; **Planned** only for not-yet-shipped scope |

---

## 9. Success criteria

Spanda becomes a **complete Autonomous Systems Platform** covering:

| Phase | Question | Answered by |
|-------|----------|-------------|
| Build | Can I compose and verify a mission? | Language + verify + packages |
| Verify | Is it safe and capable? | Safety + readiness + assurance |
| Simulate | Can I test without hardware? | Sim + replay + twins |
| Deploy | Can I provision and roll out safely? | Provisioning + OTA + gates |
| Operate | Can I run missions with oversight? | Control Center + operator workflows |
| Observe | Can I see health and trends? | Telemetry + observability + SRE |
| Recover | Can failures self-heal safely? | Recovery + continuity + alerting |
| Govern | Who can do what? | RBAC + secrets + audit |
| Audit | Can I prove compliance? | Compliance + reporting + digital thread |
| Improve | Can I learn from operations? | Trends + forecasting + scorecards + predictive analytics |

**Without losing:** safety-first language, lean-core architecture, and package extensibility.

### Lifecycle coverage map

| Phase | Enterprise pillars | Existing engines |
|-------|-------------------|------------------|
| Build | SDKs, Package Trust, Digital Thread | Language, packages, verify |
| Verify | Provisioning gates, Drift, Compliance | Safety, readiness, assurance, tamper |
| Simulate | Control Center Simulation module | Sim, replay, twins |
| Deploy | Provisioning, OTA, Config Mgmt, Secrets | OTA CLI, deployment gates, cascading config |
| Operate | Control Center, Operator Workflows, RBAC | Fleet, continuity, health |
| Observe | Telemetry, Observability, SRE, Alerting | Telemetry store, runtime faults |
| Recover | Operator Workflows, Alerting | Recovery planner, continuity |
| Govern | RBAC, Secrets, Administration | Security, policy engine |
| Audit | Reporting, Compliance, Audit module | Audit, decision trail, explain |
| Continuously Improve | SRE, Reporting, Predictive Analytics | Readiness trends, scorecards, anomaly |

---

## Related documents

- [configuration.md](./configuration.md) · [cascading-config.md](./cascading-config.md) · [device-tree.md](./device-tree.md)
- [telemetry-store.md](./telemetry-store.md) · [drift-detection.md](./drift-detection.md) · [deployment-gates.md](./deployment-gates.md)
- [package-trust.md](./package-trust.md) · [trust-framework.md](./trust-framework.md) · [compliance-profiles.md](./compliance-profiles.md)
- [platform-maturity-roadmap.md](./platform-maturity-roadmap.md) · [differentiation-roadmap.md](./differentiation-roadmap.md)
- [stable-hardening-enterprise-ops.md](./stable-hardening-enterprise-ops.md) · [security-architecture.md](./security-architecture.md) · [readiness.md](./readiness.md) · [self-healing.md](./self-healing.md)

# Changelog

All notable changes to Spanda are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Control Center auth docs:** [control-center.md](docs/control-center.md) documents UI access paths, API key setup (`SPANDA_API_KEY`, `SPANDA_API_KEYS_FILE`), role matrix, and which endpoints require Bearer auth; [getting-started.md](docs/getting-started.md) cross-links the guide.

- **H6 HRI depth (experimental):** vendor live backends (`SPANDA_LIVE_HEALTHKIT`, `SPANDA_LIVE_HOLOLENS`); `[[twins]]` and `[[mission_approvals]]` config; `GET /v1/humans/twins`, `GET /v1/operator/mission/approvals`; mission approval queue persistence; Humans tab mission approval UI; `hri_field_soak_init.sh` and `hri_security_audit_prep.sh` for Stable promotion ops.

- **H5 HRI engineering expansion (experimental):** `[[hazard_zones]]` config; `GET /v1/humans/readiness` team rollup; `GET /v1/hri/collaboration` participant graph; `GET /v1/hri/context` hazard and location snapshot; Control Center Humans tab panels for team readiness, collaboration, and context awareness.

- **HRI stable promotion gate:** `scripts/hri_stable_promotion_gate.sh` and [stable-hardening-human-interaction.md](docs/stable-hardening-human-interaction.md); `@spanda/web` `ControlCenterPanel` Humans tab parity with embedded Control Center.

- **H4 Control Center human UI (experimental):** Humans tab in Control Center; `GET /v1/humans`, `/v1/wearables`, `/v1/human-health/policy`; `HumanHealthGate` opt-in (`SPANDA_HUMAN_HEALTH_ENABLED` + `[security.human_health]`); VR training continuity example.

- **H3 HRI & collaboration (experimental):** `spanda-voice`, `spanda-gesture`, `spanda-eye-tracking` registry packages; `HriInputProvider` and `OverlayProvider` wiring; `[[spatial_sessions]]` config; Control Center `/v1/hri/sessions` API; collaborative continuity in spatial-computing blueprint examples.

- **Official package provenance binding:** built-in provider bootstrap and trust scoring require registry provenance — registry lockfile source or version constraint, or a path to the canonical `packages/registry/<name>` tree; path/git overrides of official names no longer wire built-in providers; `official_provenance` validation warning; `OfficialProvenance` API in `spanda-package::official`.
- **Production deploy gates:** `spanda deploy gate --policy production` hard-fails on `official_provenance` (official name path/git squatting) and `registry_signatures` (`SPANDA_REGISTRY_REQUIRE_SIGNATURE=1` plus verified lockfile registry signatures); `spanda-package::provenance_gate` helpers.

- **H2 Wearables & AR (experimental):** nine registry packages (`spanda-smartwatch`, `spanda-industrial-wearables`, `spanda-bodycam`, `spanda-hololens`, `spanda-arkit`, `spanda-arcore`, `spanda-vision-pro`, `spanda-magic-leap`, `spanda-openxr`); `WearableTelemetryProvider` and `SpatialSessionProvider` traits; provider dispatch and package stubs; spatial-computing blueprint device tree wired to H2 providers.

- **H1 Human Interaction (experimental):** `HumanRegistry` and fleet device tree nodes for humans, wearables, AR/VR/drones; operator capability registry; `human_collaboration` readiness profile and compliance template; `spanda demo spatial`; `./scripts/spatial_computing_smoke.sh`.

- **Human Interaction & Spatial Computing roadmap:** platform pillar for humans, wearables, AR/VR/XR, and collaborative autonomy — composes Device Registry, Capability Framework, Readiness, Continuity, Trust, and Control Center without core language extensions; phased delivery H1–H4 ([docs/human-interaction-spatial-computing-roadmap.md](docs/human-interaction-spatial-computing-roadmap.md)).
- **Spatial Computing Solution Blueprint (scaffold):** `examples/solutions/spatial-computing/` with six reference workflows (warehouse AR, remote maintenance, VR training, SAR AR, wearable health, operator approval); device tree with humans, wearables, AR/VR nodes.
- **HRI documentation:** [human-interaction.md](docs/human-interaction.md), [wearables.md](docs/wearables.md), [spatial-computing.md](docs/spatial-computing.md), [ar-vr-xr.md](docs/ar-vr-xr.md), [human-readiness.md](docs/human-readiness.md), [hri.md](docs/hri.md), [remote-expert.md](docs/remote-expert.md), [operator-capabilities.md](docs/operator-capabilities.md), [hri-packages.md](docs/hri-packages.md), [solutions/spatial-computing.md](docs/solutions/spatial-computing.md).
- **Control Center human interaction dashboard** (planned panels documented) and **website/solutions.html** Human Interaction & Spatial Computing blueprint section.
- **Master roadmap** updated with Human Interaction & Spatial Computing platform pillar.

- **ADAS & Autonomous Driving Solution Blueprint:** `examples/solutions/adas/` with highway pilot reference, five ADAS function examples, automotive device tree, readiness/assurance/security configs; `spanda demo adas`; `./scripts/adas_smoke.sh`.
- **ADAS documentation:** [docs/solutions/adas.md](docs/solutions/adas.md), [automotive-device-tree.md](docs/automotive-device-tree.md), [adas-readiness.md](docs/adas-readiness.md), [adas-assurance.md](docs/adas-assurance.md), [adas-security.md](docs/adas-security.md), [adas-replay.md](docs/adas-replay.md), [demo-plan-adas.md](docs/demo-plan-adas.md).
- **Control Center ADAS tab** and **website/solutions.html** Official Solution Blueprints page.

- **Compliance profiles (ISO 13849, IEC 61508):** signed catalog templates `iso13849` and `iec61508`; showcase programs `automotive_rover.sd`, `machinery_rover.sd`, `iec61508_rover.sd`; Control Center compliance tab profile selector; readiness scoring in compliance verify uses deploy target.
- **Compliance CLI and demo:** `spanda compliance list` (built-in + signed catalog); `spanda demo compliance` runs all five profile showcases.
- **WebSocket reconnect contract:** resume offsets, heartbeat, backpressure (`SPANDA_WS_MAX_PENDING_FRAMES`, `SPANDA_WS_HEARTBEAT_INTERVAL_MS`).
- **SIEM mutation audit export:** `GET /v1/audit/mutations/export?format=cef|jsonl`; registry package `spanda-audit-siem`.
- **On-call escalation templates:** registry package `spanda-alert-escalation` with tier routing guidance.
- **Device pool scale gate:** 1000-device list/summary perf test (`device_pool_scale`) + `scripts/device_pool_perf_bench.sh`.
- **Ops docs:** [control-center-rate-limits.md](docs/control-center-rate-limits.md), [otlp-collector-ha.md](docs/otlp-collector-ha.md); idempotent reprovision policy in [device-provisioning.md](docs/device-provisioning.md). complete `openapi.json` for all `/v1/*` routes; `openapi_routes.rs` registry + `openapi_parity_tests.rs` CI guard.
- **Scheduled drift scans:** background scheduler (`SPANDA_DRIFT_SCAN_INTERVAL_SECS`), `GET /v1/drift/scans`, `POST /v1/drift/scan`, `ConfigDrift` alerts with critical auto-incidents; CLI `control-center drift scan|scans`.
- **gRPC proto semver policy:** `grpc_policy.rs`, proto semver `1.0.0` on `control_center.proto`, `GET /v1/version` → `grpc` block; Health status includes proto semver and RPC count.
- **OTA readiness rollback:** `rollback_on_readiness_fail` on `POST /v1/ota/execute` (env `SPANDA_OTA_ROLLBACK_ON_READINESS_FAIL`); auto-rollback deploy agents when post-deploy readiness fails.
- **Multi-approver config approvals:** `required_approvals` on submit + `SPANDA_CONFIG_APPROVALS_REQUIRED`; distinct approver votes with `quorum` metadata; publish-on-approve when quorum met.
- **OTA fleet soak:** `scripts/ota_fleet_soak.sh` — multi-agent version bumps and canary→full progression tests.
- **Remote CLI OpenAPI parity:** `control_center_openapi_parity.rs` verifies CLI routes exist in `REST_V1_ROUTES`.
- **Encrypted config snapshots:** AES-256-GCM at rest via `encrypt` on `POST /v1/config/snapshots` or `SPANDA_CONFIG_SNAPSHOT_ENCRYPT=1` + `SPANDA_CONFIG_SNAPSHOT_KEY`.
- **Failover drill smoke:** `scripts/failover_drill_smoke.sh` validates redundant chain selection and recovery actions.
- **SLO burn-rate rollup:** `burn_rate` object on `GET /v1/sre/summary` with `fast_burn` when fault alert rate exceeds budget (`SPANDA_SRE_BURN_RATE_FAST`, `SPANDA_SRE_BURN_WINDOW_HOURS`).
- **SLO burn-rate monitor:** background fast-burn alert dispatch (`SPANDA_SRE_BURN_SCAN_INTERVAL_SECS`) with deduplicated `HealthCritical` alerts.
- **PagerDuty bi-directional sync:** inbound `POST /v1/integrations/pagerduty/webhook` (ack/resolve → incidents); outbound ack/resolve events on incident workflow; `incident_id` in PD custom details.
- **Digital thread lifecycle graph:** requirement → design → deploy → operate → retire phases on `GET /v1/digital-thread/query` (`lifecycle_phase` filter, `lifecycle_rows` / `lifecycle_summary`); lifecycle layout in graph UI.
- **Grafana dashboard templates:** registry package `spanda-grafana-dashboards` (SRE + OTA JSON dashboards).
- **Python SDK publish scaffold:** `packages/sdk-python/VERSIONING.md`, `sdk-python-v*` tag workflow, version `0.4.0`.
- **npm `@spanda/web` publish scaffold:** `PUBLISHING.md`, `npm-web-v*` tag workflow with PR dry-run.
- **30-day field soak gate:** `scripts/field_soak_gate.sh` + [docs/field-soak-gate.md](docs/field-soak-gate.md).
- **OTA production certify policy:** `SPANDA_OTA_REQUIRE_CERTIFY` / `SPANDA_PRODUCTION_POLICY=production` enforce certification proof on `POST /v1/ota/plan` and `/v1/ota/execute`.
- **Signed compliance profile catalog:** Ed25519-verified defense/medical/ISO 26262 templates; `GET /v1/compliance/profiles`; `cargo run -p spanda-compliance --bin sign_catalog`.
- **Scheduled report delivery:** `GET/POST /v1/reports/schedules` with webhook delivery (`SPANDA_REPORT_SCHEDULE_INTERVAL_SECS`).
- **Discovery TLS policy:** `SPANDA_DISCOVERY_REQUIRE_TLS`, `SPANDA_DISCOVERY_TLS_CA_BUNDLE`, registry package `spanda-discovery-tls`; `tls` summary on discovery responses.
- **Security audit prep:** `scripts/security_audit_prep.sh`, registry package `spanda-security-audit`, [docs/security-audit-third-party.md](docs/security-audit-third-party.md).
- **Desktop release runbook:** signed/notarized macOS CI (`.github/workflows/desktop-release.yml`), env-gated Tauri updater (`TAURI_UPDATER_PUBKEY`, `SPANDA_DESKTOP_UPDATER_ACTIVE`), [docs/desktop-release-runbook.md](docs/desktop-release-runbook.md).
- **SDK publish verify:** `scripts/verify_sdk_publish_ready.sh` for PyPI/npm readiness checks.
- **Docs:** enterprise ops stable-hardening checklist, control-center API reference, field soak, security audit, and desktop release runbooks synced.
- **Digital thread graph UI:** interactive SVG graph in `ControlCenterPanel` and embedded Control Center HTML — filter by capability/device, click-to-highlight neighbors.
- **Stable hardening checklist:** [docs/stable-hardening-enterprise-ops.md](docs/stable-hardening-enterprise-ops.md) — Experimental → Stable promotion gates per pillar.
- **Python SDK expansion:** executive scorecard, digital thread, reports export, OTA execute/status, config snapshots, audit mutations.
- **Enterprise ops doc sync:** roadmap/feature-status/product-strategy updated for 60 RPCs, remote CLI, and publish-on-approve.
- **Config publish-on-approve:** approving a config request applies the snapshot to runtime and persists device pool fields when `--config` is set.
- **gRPC parity:** `ListConfigApprovals`, `SubmitConfigApproval`, `ApproveConfigApproval`, `RejectConfigApproval`, `ListComplianceEvidence` (60 RPCs total).
- **Config approval queue:** `GET/POST /v1/config/approvals`, approve/reject subpaths; RBAC-gated publish workflow.
- **Immutable compliance evidence:** append-only `.spanda/evidence-append.jsonl` on export; `GET /v1/compliance/evidence`.
- **Alert channels:** PagerDuty and Teams webhook dispatch (`SPANDA_ALERT_PAGERDUTY_*`, `SPANDA_ALERT_TEAMS_URL`); registry packages `spanda-alert-slack`, `spanda-alert-pagerduty`, `spanda-alert-teams`.
- **SRE MTBF and health trends:** `mtbf_hint_ms`, `health_trends`, and `readiness_trends` on `GET /v1/sre/summary`; critical alerts auto-open incidents.
- **Control Center React parity:** drift, alerts, security/trust, OTA, compliance, audit, executive, and digital-thread tabs in `ControlCenterPanel`.
- **Discovery hardening:** cellular `mmcli` probe; `SPANDA_DISCOVERY_NO_STUB` to skip transport stubs.
- **SRE SLO rollup:** `SPANDA_SRE_SLO_PERCENT` and `slo` object on `GET /v1/sre/summary`.
- **Incident workflow UI:** SRE tab in embedded Control Center and `@spanda/web` `ControlCenterPanel`.
- **Python SDK incidents:** `list_incidents`, `create_incident`, `ack_incident`, `resolve_incident`.
- **macOS codesign scaffold:** `scripts/sign_tauri_macos.sh` with optional CI secrets.
- **Operational drift policy/safety rollup:** `DriftDimension::Policy` and `Safety` route firmware, attestation, and assurance findings into all seven `by_dimension` buckets.
- **SRE incident workflow:** `IncidentStore` in `spanda-ops`; `GET/POST /v1/sre/incidents`, ack/resolve subpaths; HA persistence; gRPC `ListSreIncidents`/`CreateSreIncident`/`AckSreIncident`/`ResolveSreIncident`; MTTR hint on `/v1/sre/summary`.
- **Discovery registry expansion:** `spanda-discovery-wifi`, `spanda-discovery-cellular`, `spanda-discovery-serial` packages; BLE/USB registry wrap; host probes via env overrides.
- **Tauri signed release scaffold:** `build.rs` injects `TAURI_UPDATER_PUBKEY`; macOS CI uploads bundle artifacts.
- **Multi-tenant isolation:** `SPANDA_TENANT_ID` scopes Control Center instances; API keys carry `tenant_id` (`SPANDA_API_KEYS_FILE` JSON); `GET /v1/tenant`; gRPC `GetTenant`; `403` on tenant mismatch for authenticated requests.
- **HA persistence:** alerts and trace log hydrate/persist under `.spanda/` (`SPANDA_CONTROL_CENTER_STATE_DIR`); survives restarts for operator dashboards.
- **Distributed trace backend:** registry package `spanda-otel-collector`; `SPANDA_OTEL_COLLECTOR_URL`; `GET /v1/observability/backend`; gRPC `GetObservabilityBackend`.
- **Tauri auto-update scaffold:** `tauri-plugin-updater` in `@spanda/control-center-desktop` (inactive until signing pubkey is configured).
- **Mutation audit trail:** append-only audit on successful REST/gRPC mutations via `spanda-audit`; `GET /v1/audit/mutations`; JSONL persist at `.spanda/control-center-mutations.jsonl` (`SPANDA_MUTATION_AUDIT_PATH`); gRPC `ListAuditMutations`.
- **API versioning policy:** `GET /v1/version`; `X-Spanda-Api-Version: v1` header enforcement on REST and gRPC.
- **Live OTA fleet execute:** `SPANDA_DEPLOY_AGENTS` registry path for `POST /v1/ota/execute`; `scripts/ota_fleet_execute_smoke.sh`.
- **gRPC full REST parity (read paths):** `GetDeviceTree`, `GetDeviceReports`, `GetFailoverChains`, `ListSecrets`, `GetRbacMatrix`, `GetAnalyticsReadiness`, `ExportReports`, `GetObservabilityTraces`, `GetOtlpTraces`, `ExportOtlpTraces`, `ExportOtlpMetrics` RPCs (39 total).
- **gRPC operator/provision parity:** `DiscoverDevices`, `RunDiscovery`, `ProvisionDevice`, `PlanOta`, `ExecuteOta`, `ListRobots`, `ListFleets`, `ListAlerts`, `ListConfigSnapshots`, `OperatorQuarantine`, `OperatorMissionApprove`, `ExportCompliance` RPCs (28 total); Bearer/`x-api-key` metadata for mutation RBAC.
- **Registry discovery runtime:** `discovery_registry` wraps installed `spanda-discovery-mdns`, `spanda-discovery-ble`, and `spanda-discovery-usb` transports; `installed_packages` on discovery API responses.
- **OTA fleet execute:** `POST /v1/ota/execute` runs remote rollout via deploy agents (`execute_remote_rollout`); dry-run parity with plan.

- **gRPC E2/E4 expansion:** `GetHealthSummary`, `GetAssuranceSummary`, `GetDiagnosisSummary`, `GetExecutiveScorecard`, `QueryDigitalThread`, `GetOtaStatus`, `GetOtlpMetrics` RPCs (16 total).
- **OTLP metrics (Control Center):** `spanda-ops::otlp_metrics`, `GET /v1/observability/otlp/metrics`, `POST /v1/observability/otlp/export-metrics`; enterprise ops smoke probe.

- **gRPC expansion:** `ListDevices`, `ListFleetAgents`, `EvaluateReadiness`, `GetSreSummary`, `GetTrustPackage`, `GetOpenApi` RPCs on tonic `ControlCenter`; REST parity helpers in `handlers`; live probe `grpc_live_probe.rs`; `scripts/enterprise_ops_smoke.sh` gRPC section.
- **Tauri CI:** `control-center-desktop` job (Linux compile check); `control-center-desktop-bundle` job (`TAURI_BUILD=1` on macOS main pushes).
- **Fleet agent interpreter recovery (Stable):** `scripts/fleet_agent_recovery_smoke.sh`; wired into `scripts/fleet_field_validation.sh`; mesh integration test coverage.

- **Self-healing runtime (Stable):** auto-trigger recovery during `run`/`sim` via `try_invoke_recovery_for_event` and `issue_to_recovery_issue`; approval polling every trigger maintenance tick with deferred retry; fleet mesh failure events (`fleet_mesh_recovery_failed`); mission health-critical recovery hook; tests `recovery_auto_triggers_during_run_on_health_fault`; extended `scripts/self_healing_smoke.sh`.
- **Enterprise ops hardening:** native tonic gRPC (`--grpc-bind`, `ControlCenter` service with Health/GetDashboard/DetectDrift); full operational drift (`detect_operational_drift_full` with program + agent findings, `GET /v1/drift`); Tauri desktop build script (`scripts/build_control_center_desktop.sh`, `TAURI_BUILD=1` for installers); gRPC test `crates/spanda-api/tests/grpc_tests.rs`.
- **Field validation:** `scripts/fleet_field_validation.sh` — multi-process fleet agents, mesh orchestrate, recovery/continuity mesh tests; wired into `scripts/showcase_smoke.sh`; golden-path robot names aligned (ScoutA/ScoutB).

- **Policy engine readiness integration:** `spanda readiness --policy <name>` merges operational policy evaluation into readiness scoring (OperationalPolicy factor, violation issues, mission_ready gating); `spanda deploy gate --operational-policy <name>` adds an operational policy deployment gate; `evaluate_policy_with_options` shares readiness options with `min_readiness_score` rules; `scripts/policy_smoke.sh` extended; docs [policy-engine.md](docs/policy-engine.md), [readiness.md](docs/readiness.md), [deployment-gates.md](docs/deployment-gates.md), [test-plan.md](docs/test-plan.md), [roadmap.md](docs/roadmap.md), [README.md](README.md).

- **Enterprise operations roadmap expansion:** [docs/enterprise-operations-roadmap.md](docs/enterprise-operations-roadmap.md) — platform context (§0), 20-pillar classification with Control Center module map, Device Pool lifecycle, discovery transport matrix (WiFi/LTE/5G/DNS-SD/Serial/Modbus RTU), Package Trust scoring criteria, SDK surfaces (CLI/REST/gRPC/WebSocket/Python), Compliance/APIs/Observability specs, lifecycle coverage map, and stable-hardening matrix; [docs/roadmap.md](docs/roadmap.md) — Complete Autonomous Systems Platform table, enterprise integration spine diagram, deliverable index; [docs/feature-status.md](docs/feature-status.md) — 20-pillar enterprise operations matrix; [docs/platform-overview.md](docs/platform-overview.md) — Enterprise Operations layer.

- **Device Pool & Provisioning (enterprise operations):** extended `spanda-config` with `Active` lifecycle state, health/calibration readiness gates (`device_health`), quarantine policy (`device_quarantine`), pool operations (`assign`/`unassign`/`quarantine`/`retire`/`trust`), identity anomaly detection, host-backed multi-transport discovery (`discovery_live` for mDNS/BLE/USB/CAN/MQTT/ROS2 with stub fallback), discovery pool ingestion (`ingest_discovery_matches`), device report bundle, config persistence (`device_config_persist`), failover chains (`device_failover`) integrated with recovery via `enrich_recovery_plan_with_failover`; CLI `spanda device provision|assign|unassign|quarantine|trust|retire` with `--write`; Control Center API `GET /v1/devices/{id}`, `POST /v1/devices/discover`, per-device provision/assign/quarantine/trust, `GET /v1/robots|fleets|device-tree|failover/chains|device-reports`, `POST /v1/readiness/run`; expanded `@spanda/web` Control Center panel with trust/provision/assign/quarantine actions and robot selector; registry package `spanda-discovery-mdns`; docs [device-pool.md](docs/device-pool.md), [device-provisioning.md](docs/device-provisioning.md), [device-discovery.md](docs/device-discovery.md), [device-quarantine.md](docs/device-quarantine.md), [calibration.md](docs/calibration.md). OTLP/JSON trace export to Jaeger (`GET /v1/observability/otlp/traces`, `POST /v1/observability/otlp/export`, `SPANDA_OTLP_TRACES_ENDPOINT`, `SPANDA_OTLP_TRACE_AUTO_PUSH`); WebSocket live telemetry stream (`WS /v1/stream/telemetry`); Python `TelemetryStream` (`pip install spanda-sdk[stream]`); smoke probes `scripts/mock_otlp_traces_collector.py`, `scripts/ws_telemetry_probe.py`.
- **Control Center desktop (Tauri):** `@spanda/control-center-desktop` wraps `ControlCenterPanel` in a Tauri v2 shell; `npm run control-center:desktop:dev`; compile smoke `scripts/control_center_desktop_smoke.sh`.
- **Docs:** roadmap alignment — E1–E4 reframed as shipped **Experimental** with stable-hardening matrix; enterprise-operations discovery/trust/snapshots/OTA tables synced to codebase; feature-status device pool depth.
- **Executive PDF reports:** `GET /v1/reports/export?format=pdf` returns base64-encoded PDF via `spanda-ops::render_text_pdf`.
- **Phase E4 — Govern and trace:** digital thread query (`query_digital_thread`, `GET /v1/digital-thread/query`); compliance accreditation export (`GET|POST /v1/compliance/export`); executive scorecard (`GET /v1/executive/scorecard`); readiness analytics (`GET /v1/analytics/readiness`); executive report composer (`GET /v1/reports/export`); Control Center `--program` flag; Audit, Digital Thread, and Executive UI tabs; extended `scripts/enterprise_ops_smoke.sh`.
- **Phase E3 — Deploy and integrate:** operational drift API (`detect_operational_drift`, `GET /v1/drift?baseline_id=`); OTA rollout planning (`POST /v1/ota/plan` with canary/staged/blue_green, `GET /v1/ota/status`); package trust API (`GET /v1/trust/package`); SRE summary and observability trace log (`GET /v1/sre/summary`, `GET /v1/observability/traces`); operator workflows (`POST /v1/operator/quarantine`, `POST /v1/operator/mission/approve`); gRPC-compatible JSON gateway (`POST /v1/rpc`); OpenAPI 3.1 spec (`GET /v1/openapi.json`); `X-Correlation-ID` request tracing; Control Center UI tabs for Drift, OTA, Security, SRE, Operator; Python SDK (`packages/sdk-python`); `RolloutStrategy::BlueGreen` in `spanda-ota`; extended `scripts/enterprise_ops_smoke.sh`.
- **Phase E2 — Provision and observe:** provisioning workflow (`run_provision_workflow`, `POST /v1/provision`) with readiness-failure alerting and quarantine; configuration snapshots (`.spanda/config-snapshots/`, `GET|POST /v1/config/snapshots`); discovery transport contract (`DeviceDiscoveryTransport`, subnet + mock mDNS); `GET /v1/health|assurance|diagnosis/summary`; Control Center UI tabs for Health, Assurance, Diagnosis, Provision, Config; Slack webhook payload formatting in `spanda-ops`; registry stub `spanda-discovery-mdns`; extended `scripts/enterprise_ops_smoke.sh`.
- **Phase E1 — Control Center (enterprise operations):** new `spanda-api` crate (REST `/v1/*`, embedded Control Center HTML UI); `spanda control-center serve [--bind] [--config]`; `spanda-ops` alerting core (webhook, email dry-run, log channel); Device Pool lifecycle in `spanda-config` (`DeviceLifecycleState`, `DevicePoolEntry`, `pool_summary`, `set_lifecycle`); RBAC v1 in `spanda-security` (`Role`, `ApiKeyStore`, `RbacAction`, `SPANDA_API_KEY`); `ManagedSecretVault` secret store contract with rotation metadata; Control Center view in `@spanda/web` (`ControlCenterPanel`); `scripts/enterprise_ops_smoke.sh`.
- **Enterprise operations roadmap:** [docs/enterprise-operations-roadmap.md](docs/enterprise-operations-roadmap.md) — 20 platform pillars for production enterprise deployments (Control Center, Device Pool, Device Discovery, Provisioning, Configuration Management, RBAC, Secret Management, Telemetry, Alerting, Configuration Drift, OTA & Rollback, Package Trust, SDKs, Operator Workflows, SRE, Reporting, Compliance, APIs, Observability, Digital Thread); core vs package ownership; React/TypeScript UI + Rust `spanda-api` backend architecture; integration map with Readiness, Assurance, Diagnosis, Recovery, Trust, Health, Device Registry, Configuration, Traceability, Audit, Security, Packages; phased implementation (E1–E4); risks and mitigation; priority horizons NOW/NEXT/LATER.
- **Roadmap sync:** [docs/roadmap.md](docs/roadmap.md) Enterprise Operations section; [docs/feature-status.md](docs/feature-status.md) planned matrix; [docs/platform-maturity-roadmap.md](docs/platform-maturity-roadmap.md) cross-reference; [docs/README.md](docs/README.md) index entry.
- **Cascading TOML configuration:** new `spanda-config` crate with `SpandaManifest`, `ConfigResolver`, `ResolvedSystemConfig`, `DeviceTree`, `LogicalPhysicalMap`, and `ConfigValidationReport`; root `spanda.toml` `[project]` / `[config]` / `[extends]` / `[merge]` sections; fragment files (`spanda.devices.toml`, `spanda.providers.toml`, etc.); JSON config compatibility; CLI `spanda config resolve|validate|graph|diff|report`, `spanda device-tree inspect|graph`, `spanda map verify`; `spanda readiness --config`; docs [configuration.md](docs/configuration.md), [cascading-config.md](docs/cascading-config.md), [device-tree.md](docs/device-tree.md), [config-validation.md](docs/config-validation.md); updated [spanda-toml.md](docs/spanda-toml.md).
- **Configuration runtime integration:** `ResolvedSystemConfig` wired into `run`/`sim`/`fleet run`/`verify`/`readiness`/`assure`/`diagnose`/`heal`/`recover`/`replay`; shared `resolve_for_source`; lockfile- and device-tree-aligned provider bootstrap; readiness policy and per-robot `[health]` fault injection from config; assurance/mission/recovery thresholds from `[assurance]`/`[mission]`/`[recovery]`; TypeScript CLI delegation with `cargo run` and lightweight fallback for `config validate|resolve`.
- **Device identity mapping:** `[[devices]]` registry with network/bus identity fields (`ip`, `mac`, `serial`, `endpoint`, `protocol`, `can_id`, `trust_level`, …); `DeviceRegistry` on `ResolvedSystemConfig`; duplicate IP/MAC/serial validation; redundant failover rules; CLI `spanda device discover|inspect`, `spanda network scan`, `spanda config report --network`; readiness connectivity issues from device identity gaps; assurance/diagnosis traceability when `--config` is set.
- **Configuration drift detection:** `detect_config_drift` / `ConfigDriftReport` in `spanda-config`; CLI `spanda drift` and `spanda config drift` with `--baseline` and `--agent`; agent `/v1/status` fields (`program_hash`, `hardware_profile`, `firmware_version`, `packages`); `spanda readiness --baseline` and `spanda readiness --agent` for config and attestation drift gates; docs [drift-detection.md](docs/drift-detection.md).
- **Dependency graphs:** new `spanda-graph` crate; CLI `spanda graph <file.sd>` with `--format json|mermaid|dot|text`; composes capability traceability and resolved config; docs [dependency-graphs.md](docs/dependency-graphs.md).
- **Deployment gates:** `evaluate_deployment_gates` in `spanda-readiness`; CLI `spanda deploy gate` with `--policy default|production`; docs [deployment-gates.md](docs/deployment-gates.md).
- **Package trust:** `evaluate_package_trust` in `spanda-package`; CLI `spanda trust <package>` with `--version`, `--project`, `--json`; package trust gate in deployment gates when `--config` declares packages; docs [package-trust.md](docs/package-trust.md).
- **Composite program trust:** new `spanda-trust` crate with `evaluate_composite_trust`; CLI `spanda trust <file.sd>` with `--json` and `--format markdown`; weighted categories (package, device, firmware, configuration, identity, safety); secure-boot import detection (`trust.jetson` / `trust.pi`); `scripts/trust_program_smoke.sh`; docs [trust-framework.md](docs/trust-framework.md).
- **Secure-boot contract integration:** `spanda-tamper::secure_boot` scores `trust.jetson` / `trust.pi` in `tamper-check` and `integrity`; optional `SPANDA_ATTESTATION_ENDPOINT` live attestation; optional `SPANDA_TPM_BACKEND` backends including `vendor` SDK adapters, tpm2-tools PCR quote with `tpm2_checkquote` verification, remote AK cert chain validation (`SPANDA_ATTESTATION_TRUST_STORE`), and `SPANDA_TPM2_PCR0_EXPECT` policy; bundled trust registry for offline demos; deploy agent `/v1/status` attestation fields; drift detection for attestation posture; `examples/showcase/secure_boot/`; `scripts/secure_boot_smoke.sh`, `scripts/attestation_smoke.sh`, `scripts/gaps_smoke.sh`, `scripts/lib/registry_env.sh`; docs [hardware-attestation.md](docs/hardware-attestation.md).
- **Composite trust in gates and scorecard:** `spanda deploy gate` adds `composite_trust` and `secure_boot` gates; scorecard security pillar blends threat risk, composite trust, and secure-boot coverage; `spanda explain` includes `composite_trust` and `secure_boot` sections; `spanda demo maturity` runs program trust.
- **Explain polish:** `spanda explain <file.sd>` supports `--config` and `--baseline` for configuration validation, deployment gates preview, package trust, and drift sections; docs [explainability.md](docs/explainability.md).
- **Platform maturity demo:** `spanda demo maturity` runs graph, explain, trust, and deploy gate on `examples/showcase/readiness/rover.sd`; `scripts/maturity_smoke.sh` (wired into `scripts/showcase_smoke.sh`).
- **Threat modeling:** new `spanda-threat` crate with `analyze_threat_model`; CLI `spanda threat-model <file.sd> [--json]`; docs [threat-modeling.md](docs/threat-modeling.md).
- **Mission diff:** new `spanda-diff` crate with `diff_programs`; CLI `spanda diff <baseline.sd> <candidate.sd> [--json]`; `scripts/diff_smoke.sh`; docs [mission-diff.md](docs/mission-diff.md).
- **Scorecard:** new `spanda-score` crate with `evaluate_scorecard`; CLI `spanda score <file.sd> [--json] [--format markdown]`; docs [scorecards.md](docs/scorecards.md).
- **Policy engine (verify-time):** `policy { }` declarations in AST/parser; new `spanda-policy` crate with `evaluate_policy`; CLI `spanda verify --policy <Name> [--json]`; showcase `examples/showcase/policy/warehouse.sd`; `scripts/policy_smoke.sh`; docs [policy-engine.md](docs/policy-engine.md).
- **Chaos engineering:** new `spanda-chaos` crate with `run_chaos_experiment`; CLI `spanda chaos <file.sd> [--inject gps-failure,...] [--json]`; `scripts/chaos_smoke.sh`; docs [chaos-engineering.md](docs/chaos-engineering.md).
- **Readiness trends:** history store in `spanda-readiness` (`record_readiness_snapshot`, `analyze_readiness_trends`); CLI `spanda readiness --record` and `spanda readiness trends <file.sd> [--forecast 7d] [--json]`; `scripts/readiness_trends_smoke.sh`; docs [readiness-trends.md](docs/readiness-trends.md).
- **Mission resource estimation:** new `spanda-estimate` crate with `estimate_mission`; CLI `spanda estimate <file.sd> [--target <profile>] [--json]`; `scripts/estimate_smoke.sh`; docs [resource-estimation.md](docs/resource-estimation.md).
- **Compliance profiles:** new `spanda-compliance` crate with built-in industry templates; CLI `spanda verify --profile <name>` and `spanda readiness --profile <name>`; `spanda compliance report <file.sd> --profile <name>` accreditation export bundles with evidence checklist and audit export ID; defense/medical secure-boot contract requirements; defense showcase at `examples/showcase/compliance/defense_rover.sd`; `scripts/compliance_smoke.sh`; docs [compliance-profiles.md](docs/compliance-profiles.md).
- **Architecture decision records:** new `spanda-adr` crate with `generate_adrs`; CLI `spanda adr <file.sd> [--json] [--out <dir>]`; `scripts/adr_smoke.sh`; docs [architecture-decision-records.md](docs/architecture-decision-records.md).
- **Tamper / integrity (verify-time):** new `spanda-tamper` crate with `generate_tamper_check`; CLI `spanda tamper-check <file.sd> [--json]`; `scripts/tamper_smoke.sh`; docs [tamper-detection.md](docs/tamper-detection.md).
- **Runtime tamper (trace analysis):** `generate_runtime_tamper_check` in `spanda-tamper`; `spanda tamper-check <file.trace> [--runtime]`; trust showcase `examples/showcase/runtime_intrusion/`; `scripts/trust_showcase_smoke.sh`.
- **Tamper diagnosis:** `diagnose_tamper_trace` in `spanda-tamper`; CLI `spanda diagnose tamper <file.trace> [--json]`; `scripts/tamper_diagnose_smoke.sh`.
- **Fleet tamper correlation:** `correlate_fleet_tamper` in `spanda-tamper`; CLI `spanda tamper-check --fleet <manifest.json>`; showcase `examples/showcase/fleet_tamper/`; `scripts/fleet_tamper_smoke.sh`.
- **Security assurance rollup:** `generate_security_assurance` in `spanda-tamper`; CLI `spanda security assurance <file.sd>`; `scripts/security_assurance_smoke.sh`.
- **Tamper policy runtime:** `tamper_policy` blocks in `.sd` programs; `spanda-tamper::policy` matching; runtime dispatch via recovery actions; showcase `examples/showcase/tamper_policy/`; `scripts/tamper_policy_smoke.sh`.
- **Live fleet mesh tamper:** `POST /v1/fleet/tamper/ingest`, `GET /v1/fleet/tamper`; CLI `spanda tamper-check --mesh-url <url>`; runtime ingest via `SPANDA_FLEET_MESH_URL`; `scripts/fleet_mesh_tamper_smoke.sh`.
- **Spoofing ML backend:** optional `SPANDA_SPOOFING_ML_ENDPOINT` HTTP merge plus `SPANDA_SPOOFING_ML_BACKEND` stub backends (`mock`, `file`, `script`) and `SPANDA_SPOOFING_ML_MIN_CONFIDENCE` filtering for trace spoof-check; global `SPANDA_SPOOFING_MIN_CONFIDENCE` trace filter and operator confirmation gates for High/Critical alerts.
- **Secure boot package stubs:** `spanda-trust-jetson`, `spanda-trust-pi` attestation contracts in hosted registry index.
- **Compliance template hardening:** accreditation notice on profiles; defense/medical require `tamper_policy`; Critical tamper actions require operator approval.
- **Platform maturity gap closure:** `spanda demo gaps` walkthrough; `secure_boot_status_line` in explain and deploy gates; attestation and compliance smokes extended for vendor TPM and AK chain validation.
- **Trust showcase demos:** `examples/showcase/package_tampering/`, `mission_tampering/`, `runtime_intrusion/` with tamper and integrity workflows; `spanda demo trust` one-command walkthrough; bundled registry slice (trust + gps/fusion packages) in CLI crate via `scripts/sync_bundled_registry.sh`; CLI defaults `SPANDA_REGISTRY_URL` to bundled registry when unset; `scripts/bundled_trust_smoke.sh`.
- **Integrity verification (verify-time):** `generate_integrity_report` in `spanda-tamper`; CLI `spanda integrity <file.sd> [--baseline <file.sd>] [--agent <Robot@Hardware>] [--json]`; `scripts/integrity_smoke.sh`; docs [integrity-verification.md](docs/integrity-verification.md).
- **Decision trace explainability:** `spanda explain decision <mission.trace> [--json]` with evidence, safety checks, and rejected alternatives; `scripts/decision_explain_smoke.sh`.
- **Runtime policy enforcement:** `spanda-policy` runtime monitor with `max_speed` and `operation_hours` gates; `spanda run|sim --enforce-policy <name>`; `scripts/policy_runtime_smoke.sh`.
- **AI-assisted development (mock-first):** new `spanda-generate` crate with template scaffolds and `suggest_program`; CLI `spanda generate mission|robot|health-policy` and `spanda suggest <file.sd>`; optional `--backend llm` via `SPANDA_LLM_ENDPOINT`; `scripts/generate_smoke.sh`; docs [ai-assisted-development.md](docs/ai-assisted-development.md).
- **Spoofing detection:** new `spanda-spoofing` crate with program coverage and trace plausibility analysis; CLI `spanda spoof-check <file.sd|file.trace> [--json]`; showcase `examples/showcase/gps_spoofing/`; `spanda demo spoof` focused walkthrough; `scripts/spoof_smoke.sh`; package backends in `spanda-gps` and `spanda-fusion`; `scripts/package_spoofing_smoke.sh`; docs [spoofing-detection.md](docs/spoofing-detection.md).
- **Runtime fault detection:** new `spanda-runtime-faults` crate with `RuntimeFault`, `CrashEvent`, `RebootEvent`, `MemoryLeakEvent`, `OOMEvent`, `WatchdogTimeout`, `DeadlockEvent`, `RestartLoop`, `ResourcePressure`, `HeartbeatStatus`, `ProcessHealth`, `RuntimeHealth`, `FaultEvidence`, and `FaultTimeline` types; AST declarations (`heartbeat`, `memory_watch`, `resource_watch`, `restart_policy`, `on runtime crash` triggers); CLI `spanda fault scan|report`, `spanda runtime health|diagnose`, and `spanda replay --show-faults`; readiness/assurance/diagnosis/recovery/replay integration; showcase examples under `examples/showcase/runtime_faults/`; docs [runtime-fault-detection.md](docs/runtime-fault-detection.md), [crash-detection.md](docs/crash-detection.md), [reboot-detection.md](docs/reboot-detection.md), [memory-leak-detection.md](docs/memory-leak-detection.md), [runtime-health.md](docs/runtime-health.md). `spanda-contract`, `spanda-explain`, `spanda-decision` crates; CLI `spanda contract verify`, `spanda explain`, `spanda audit decisions`, `spanda safety-coverage`, `spanda recovery-coverage`; `spanda demo differentiation`; `examples/showcase/differentiation/warehouse.sd`; `scripts/differentiation_smoke.sh` (wired into `showcase_smoke.sh`).
- **Philosophy & pronunciation doc sync:** README pronunciation guide (*SPUN-duh* /ˈspʌndə/) and expanded Sanskrit etymology propagated to [overview/philosophy.md](docs/overview/philosophy.md), [vision.md](docs/vision.md), [product-strategy.md](docs/product-strategy.md), [website-content.md](docs/website-content.md), [getting-started.md](docs/getting-started.md), [spanda-for-dummies](docs/spanda-for-dummies/), [tutorials index](docs/tutorials/README.md) (audience paths), [docs/README.md](docs/README.md), and mdBook [introduction](docs-site/src/introduction.md).
- **Differentiation roadmap:** [docs/differentiation-roadmap.md](docs/differentiation-roadmap.md) — 15 signature platform areas (mission contracts, explainability, decision audit trail, safety/recovery coverage, what-if, risk, trust graph, scorecards, digital mission twin, certification packs, mission time travel, human/robot teaming, autonomous governance); priority horizons NOW/NEXT/LATER; architecture impact; package vs core ownership; integration mapping; documentation, demo, and adoption plans.
- **Signature capabilities:** Safety-Typed AI, Mission Contracts, Readiness Engine, Continuity & Takeover, Trust Framework, Explainability & Audit Trail — documented in [product-strategy.md](docs/product-strategy.md) and [README.md](README.md).
- **Differentiation topic guides:** [mission-contracts.md](docs/mission-contracts.md), [decision-audit-trail.md](docs/decision-audit-trail.md), [safety-coverage.md](docs/safety-coverage.md), [recovery-coverage.md](docs/recovery-coverage.md), [what-if-analysis.md](docs/what-if-analysis.md), [mission-risk-analysis.md](docs/mission-risk-analysis.md), [digital-mission-twin.md](docs/digital-mission-twin.md), [certification-packs.md](docs/certification-packs.md), [mission-time-travel.md](docs/mission-time-travel.md), [human-robot-teaming.md](docs/human-robot-teaming.md).
- **Roadmap sync:** [docs/roadmap.md](docs/roadmap.md) Differentiation section; [docs/feature-status.md](docs/feature-status.md) planned matrix; [docs/README.md](docs/README.md) index entries.
- **Mission continuity framework:** `spanda-assurance` continuity module with `MissionContinuityManager`, `MissionDelegationManager`, `TakeoverCoordinator`, `SuccessionPlanner`, `MissionCheckpointManager`, `MissionStateTransferManager`, `MissionRecoveryPlanner`, and `ContinuationDecisionEngine`; takeover modes (resume, restart, partial restart, shadow/hot/cold/human); state transfer models; successor ranking with trust/readiness gates.
- **Continuity CLI:** `spanda continuity`, `spanda takeover`, `spanda delegate`, `spanda succession` with `--failed`, `--progress`, `--trigger`, `--scope`, and report formats.
- **`continuity_policy` syntax:** parser, AST, and takeover mode inference from declared actions; TypeScript parser parity.
- **`spanda demo continuity`:** showcase demo and `scripts/continuity_smoke.sh` (wired into `showcase_smoke.sh`).
- **Fleet runtime takeover dispatch:** interpreter `execute_continuity_on_program`, fleet agent `/v1/continuity/execute`, mesh `POST /v1/fleet/continuity`, `fleet_takeover` peer topic.
- **Continuity diagnostics:** `collect_continuity_diagnostics` in `spanda-assurance` and `src/continuity-diagnostics.ts`; merged into `spanda check --readiness-json` and LSP readiness diagnostics.
- **Official package:** `spanda-mission-continuity` (`assurance.continuity`).
- **Recovery handoff bridge:** fleet recovery `reassign mission` actions also relay continuity takeover when `SPANDA_FLEET_MESH_URL` is set.
- **Continuity man page and docs:** `spanda man continuity`; README and getting-started continuity section; roadmap/feature-status sync.
- **LSP continuity quick-fixes:** readiness/recovery/continuity diagnostics cached for code actions; VS Code snippets for `continuitypolicy` and `recoverypolicy`.
- **Continuity docs and editor polish:** fleet-distributed continuity APIs, syntax highlighting for `continuity_policy`, LSP completions/hover, robot-aware approval quick-fixes, package count sync (38).
- **`continuity:mission` diagnostic:** warns when resume/checkpoint actions lack `mission_plan`; [continuity-policies.md](docs/continuity-policies.md) policy guide; `continuity_policy` promoted to **Stable** in roadmap.
- **Documentation sync (continuity):** overview CLI, differentiators, layers, getting-started showcase table, platform workflow, feature-status, spanda-language, man cross-links, mdBook SUMMARY, VS Code README.
- **Continuity runtime hardening:** mode-specific interpreter takeover (resume/restart/partial/shadow/hot/cold/human), durable checkpoint store (`.spanda/mission-checkpoints.json`), swarm `--failed` handoff in `spanda swarm coordinate`, operations dashboard continuity panel, interpreter/CLI/TS tests; runtime promoted to **Stable**.
- **Continuity auto-trigger and polish:** automatic takeover during `run`/`sim` on health faults and recovery; parser action spacing fix; TS checkpoint disk I/O; `spanda-mission-continuity` provider exports; swarm+mesh integration test.
- **Continuity examples:** `examples/showcase/continuity/`, `takeover/`, `delegation/`, `swarm_takeover/`, `fleet_succession/`.
- **Docs:** [mission-continuity.md](docs/mission-continuity.md).
- **Persistent telemetry store:** HTTP scrape server (`spanda telemetry serve`); OTLP/JSON export and **`spanda telemetry push`** to remote collectors (`SPANDA_OTLP_ENDPOINT`); **`spanda telemetry push --watch`** and **`SPANDA_OTLP_AUTO_PUSH`** session-end auto-push; **`spanda telemetry fleet-push`** with fleet mesh ingest (`/v1/fleet/telemetry/ingest`, merged `GET /v1/fleet/telemetry`) and **`SPANDA_FLEET_TELEMETRY_AUTO_INGEST`** session-end agent ingest; per-event `session_id` tagging; Prometheus export; SQLite backend with JSONL migration; `sessions` / `replay --session`; `telemetry info`; TypeScript parity for sessions, metrics, retention, SQLite (Node 22+), serve, push, fleet-push, replay inspect/playback/**deterministic verify**, and Rust trace JSON normalization; vitest coverage in `tests/telemetry-store.test.ts`, `tests/telemetry-replay.test.ts`, `tests/telemetry-push.test.ts`, and `tests/telemetry-fleet.test.ts`; **TS `runtime_metrics` aligned with Rust `RuntimeTelemetry` (scheduler, task, execution, pipeline, watchdog, trigger, topic, provider maps)**; **topic QoS deadline tracking and provider-call metrics in the TS interpreter**; **in-memory telemetry store (`memory` module) with WASM bindings** (`wasm_telemetry_clear|append|stats|prometheus|otlp`) with **web playground telemetry panel**; optional `sqlite` feature on `spanda-telemetry-store` for wasm32 builds; **CI wasm32 `cargo check` for `spanda-wasm` and `spanda-telemetry-store` (no default features)** plus **`scripts/telemetry_store_golden_path.sh` job**; **TS interpreter routes official package imports through `dispatchOfficialPackageCall`**; **event handler trigger metrics**; **`wasm_run` auto-appends `runtime_metrics` to the in-memory buffer**; **TypeScript `TriggerRegistry` with timer/when/while/state/system triggers, storm limits, missed-deadline metrics, and scheduler integration**
- **Platform maturity topic guides:** dependency graphs, threat modeling, drift detection, policy engine, compliance profiles, explainability, chaos engineering, resource estimation, readiness trends, package trust, deployment gates, scorecards, tamper detection, integrity verification, trust framework, spoofing detection, security assurance.
- **Roadmap sync:** [docs/roadmap.md](docs/roadmap.md) Platform Maturity section; [docs/product-strategy.md](docs/product-strategy.md) maturity focus; [docs/README.md](docs/README.md) index entries.
- **Structured docstring standard:** [docs/coding-standards.md](docs/coding-standards.md) defines Description / Inputs / Outputs / Example for all Rust, TypeScript, Python, and Spanda APIs; [docs/documentation-coverage.md](docs/documentation-coverage.md) tracks coverage; `scripts/validate_documentation.py`, `add_structured_api_docs.py`, and `migrate_legacy_inline_docs.py` enforce and bulk-apply the format; CI emits warnings (non-blocking) on pull requests.
- **Documentation tooling (follow-up):** getting-started and README links for `spanda doc`, `spanda man`, and `docs/language-reference/`; feature-status matrix updated for docgen and man pages.
- **Documentation tooling:** `///` doc comments in `.sd` files (parsed into AST); `spanda doc --html`, `spanda doc examples/`, `spanda man`; expanded man pages with EXIT STATUS and FILES; `docs/language-reference/` topic index; mdBook site sections; CI gates for `cargo doc`, `spanda doc`, and mdBook build.
- **Self-healing docs sync:** man page (`spanda-recovery`), CLI overview, fleet-distributed recovery HTTP APIs, verification/readiness diagnostic categories, feature-status matrix, and deploy-http `fleet_recovery` module docs. validated recovery actions execute at runtime (mode transitions, speed caps, connectivity restart, mission pause, fleet coordination) with assurance gating.
- **Recovery knowledge store:** persistent `.spanda/recovery_knowledge.json` records outcomes; planner and `evaluate_recovery` use merged knowledge when policies are absent; `spanda recovery knowledge` inspects the store.
- **Operator approval hooks:** `SPANDA_OPERATOR_APPROVAL`, `SPANDA_GRANT_RECOVERY_APPROVAL`, `Approval` comm-topic polling, `RunOptions.inbound_comm_messages` test hook, and mission `requires approval Operator` runtime gating for high-risk recovery and mission steps.
- **Fleet recovery mesh signal:** runtime publishes `/fleet/recovery` Command messages and relays to fleet mesh (`POST /v1/fleet/recovery`) when `SPANDA_FLEET_MESH_URL` is set; mesh coordinator fans out `fleet_recovery` peer deliveries to registered agents; agents expose `recovery_active` on `/v1/status`.

- **Self-healing and recovery framework:** `spanda-assurance` recovery module with `RecoveryPlan`, `RecoveryPlanner`, failure classification, recovery levels (0–4), safe mode transitions, self-correction actions, validation gates (safety, hardware, capability, readiness), human approval integration, recovery audit/traceability, fleet recovery, recovery knowledge base, and assurance metrics.
- **Recovery policy syntax:** `recovery_policy Name { on condition { actions; } }` declarations.
- **Recovery CLI:** `spanda heal`, `spanda recover`, `spanda recovery-report`, `spanda recovery plan|knowledge`; `spanda sim --inject-failure <kind>`; `spanda analyze-failure --with-recovery`.
- **Examples:** `examples/showcase/self_healing/`, `self_correction/`, `fleet_recovery/`, `recovery_assurance/`.
- **Docs:** [self-healing.md](docs/self-healing.md), [self-correction.md](docs/self-correction.md), [recovery-planning.md](docs/recovery-planning.md), [recovery-assurance.md](docs/recovery-assurance.md), [recovery-policies.md](docs/recovery-policies.md).
- **Mission assurance platform:** new `spanda-assurance` crate with core interfaces (knowledge model, state estimation, anomaly detection, diagnosis, prognostics, mitigation, mode management, mission planning, resilience, assurance evidence) and static analysis integrated with readiness, health, traceability, and hardware verification.
- **Mission assurance language:** `knowledge_model`, `state_estimator`, `anomaly_detector`, `on anomaly`, `prognostics`, `mitigation`, `operating_mode`, `mission_plan`, `resilience_policy`, and `assurance_case` declarations.
- **Mission assurance CLI:** `spanda assure`, `spanda anomaly scan`, `spanda prognostics`, `spanda mission verify`, and `spanda resilience check`; enhanced `spanda diagnose` for traces and programs.
- **Official packages:** `spanda-assurance`, `spanda-knowledge-model`, `spanda-anomaly`, `spanda-diagnosis`, `spanda-prognostics`, `spanda-mission-planning`, `spanda-mission-continuity`, `spanda-resilience`, `spanda-fusion`.
- **Examples:** `examples/assurance/`, `examples/anomaly/`, `examples/diagnostics/`, `examples/prognostics/`, `examples/resilience/`, `examples/mission/`.
- **Docs:** [mission-assurance.md](docs/mission-assurance.md), [knowledge-models.md](docs/knowledge-models.md), [anomaly-detection.md](docs/anomaly-detection.md), [diagnostics.md](docs/diagnostics.md), [prognostics.md](docs/prognostics.md), [resilience.md](docs/resilience.md), [assurance-cases.md](docs/assurance-cases.md).
- **Learned anomaly detectors:** `learned backend <module>;` on `anomaly_detector`; anomaly scan reports include learned model metadata.
- **State estimation CLI:** `spanda state estimate` reports estimators and belief state; included in `spanda assure` summary.
- **Readiness state estimation:** empty `state_estimator` inputs reduce Assurance factor score with span-aware IDE diagnostics.
- **Assurance demo:** `spanda demo assurance` runs the full mission assurance CLI suite on `examples/showcase/assurance/rover.sd`.
- **Self-healing demo:** `spanda demo self-healing` runs heal, recover, recovery knowledge, sim inject-failure, and fleet recovery showcase paths.
- **Self-healing CI smoke:** `scripts/self_healing_smoke.sh` exercises recovery CLI, runtime tests, diagnostics, and demo.
- **Fleet agent assurance recovery:** deployed fleet agents run assurance plan/validate/apply on `fleet_recovery` peer commands and `POST /v1/recovery/execute`; status reports `recovery_validation`.
- **Fleet agent interpreter recovery:** deployed agents run live interpreter recovery dispatch (`execute_recovery_on_program`) for mode transitions, speed caps, and mission pause; `recovery_engine` on `/v1/status` reports `interpreter` vs `assurance` fallback.
- **TypeScript recovery diagnostics:** `src/recovery-diagnostics.ts` mirrors Rust `collect_recovery_diagnostics` for LSP/readiness JSON fallbacks.
- **Learned anomaly runtime:** health polling invokes `assurance.anomaly::scan_learned` for detectors with `learned backend`; package stub scores observations below 0.85 as anomalies.
- **Weighted sensor fusion:** runtime and `spanda state estimate` use type-weighted confidence; `fusion.read()` exposes `sources` and `state_estimate`.
- **Learned anomaly EMA:** runtime tracks per-detector EMA volatility and passes it to `scan_learned` for drift detection.
- **`spanda-fusion` package:** `assurance.fusion` import path with `weight_for_sensor` and `confidence_for_types` provider dispatch (extends lean-core weighted fusion).
- **ONNX anomaly inference:** `SPANDA_ANOMALY_ONNX_MODEL_PATH` (or `SPANDA_ONNX_MODEL_PATH`) enables ONNX-backed `scan_learned` via Python bridge; showcase rover links learned anomaly + fusion packages.
- **Documentation sync:** README, getting-started, platform-overview, tutorials index, examples library, mission-assurance guides, and registry catalog updated for mission assurance features and examples (37 hosted packages).
- **README split:** Slim root [README.md](README.md); expanded content moved to [docs/overview/](docs/overview/README.md) subpages.
- **Overview subpages:** Platform structure, components, architecture, layers, library, packages, web playground, and CLI reference under `docs/overview/`.

### Changed

- **Roadmap audit (2026-06-24):** [docs/roadmap.md](docs/roadmap.md) — v0.5 beta milestone, post-v0.4.0 continuity/telemetry notes, differentiation docs-vs-code honesty, tooling/web updates; [roadmap-codebase-audit-2026-06.md](docs/roadmap-codebase-audit-2026-06.md) refreshed; [feature-status.md](docs/feature-status.md) telemetry OTLP/fleet row.

- **Overview doc dedup:** Overview subpages link to canonical guides instead of repeating prose and tables; README and docs index consolidate navigation through [docs/overview/README.md](docs/overview/README.md).
- **Roadmap sync:** Package count updated to 37 across roadmap and product strategy (matches hosted registry index).

### Fixed

- **Trust import typecheck:** `trust.jetson` and `trust.pi` resolve in `spanda check` / `spanda verify` via framework package import paths.

- **Package trust signatures:** `evaluate_package_trust` verifies Ed25519 registry signatures using the embedded index public key when `SPANDA_REGISTRY_TRUST_KEY` is unset (matches install-time verification).

- **Readiness polish:** CI smoke (`scripts/readiness_smoke.sh`), agent `/v1/readiness` integration tests, `spanda deploy|fleet agent readiness` CLI, TypeScript fallbacks for all operational commands (`src/operational.ts`), span-aware LSP readiness diagnostics, `POST /v1/program` on deploy agents, bundled `root_cause_analysis` example, and fleet dashboard aggregates in the web Operations panel.

- **Readiness gap closure (phases 1–4):** `--target`, `--runtime`, `--inject-health-faults`, and `--agent-json` on `spanda readiness`; `spanda check --readiness-json` for IDE diagnostics; `spanda demo readiness`; bundled showcase examples synced to the CLI crate.
- **Agent readiness API:** `GET /v1/readiness` on deploy and fleet agents (`?runtime=true`, `?inject_health_faults=true`); TypeScript CLI fallback via `src/readiness.ts` when the native binary is unavailable.
- **Operations dashboard:** `packages/web` Operations view with local readiness scoring and live agent fetch.
- **LSP readiness diagnostics:** readiness issues surface in the language server via native CLI or TypeScript fallback.

- **Operational readiness engine:** new `spanda-readiness` crate composing hardware, capability, health, connectivity, safety, and mission gates into weighted `ReadinessReport` with `spanda readiness <file.sd>` (`--json`, `--markdown`, `--html`).
- **Mission assurance CLI:** `spanda verify mission`, `spanda analyze-failure`, `spanda safety-report`, `spanda diagnose`, `spanda audit`, `spanda verify-fleet`, `spanda verify-approval`, `spanda fleet readiness`, `spanda twin readiness`.
- **Mission approvals:** `requires approval <Actor> for: <action>` in mission blocks for human-in-the-loop verification.
- **Showcase examples:** `examples/showcase/readiness/`, `mission_verification/`, `failure_analysis/`, `safety_report/`, `fleet_readiness/`, `root_cause_analysis/`.
- **Docs:** [readiness.md](docs/readiness.md), [mission-verification.md](docs/mission-verification.md), [failure-analysis.md](docs/failure-analysis.md), [safety-reporting.md](docs/safety-reporting.md), [fleet-readiness.md](docs/fleet-readiness.md), [root-cause-analysis.md](docs/root-cause-analysis.md), [safety-auditor.md](docs/safety-auditor.md).
- **Platform positioning:** README reframed as Autonomous Systems Platform; Spanda Platform component map; "Why Spanda?" and "What makes Spanda different?" sections; homepage messaging (Build · Verify · Simulate · Deploy · Operate).
- **Docs:** [platform-overview.md](docs/platform-overview.md) (platform vs language); [platform-positioning-migration.md](docs/platform-positioning-migration.md) (messaging migration and GitHub metadata); roadmap reorganized by platform area; platform Mermaid diagram in [diagrams/README.md](docs/diagrams/README.md).
- **Bundled rover demo:** `spanda demo rover` ships `examples/showcase/autonomous_rover/` source in the crate (install fetches registry deps).
- **Registry index:** all 29 `packages/registry/*` scaffolds indexed in `registry/index.json`.
- **Phase 30 test crate:** `phase30_gaps.rs` for health polling during trigger loops.
- **Docs:** Phase 27+ language reference section; version narrative aligned to v0.4; Phase 25 marked complete; native deploy tier matches feature-status (experimental).

### Changed

- **Docs:** Platform positioning follow-up — docs-site intro, getting-started, spanda-architecture, hardware-compatibility, demo-script, robotics-platform, For Dummies ch.1; CLI man blurb in `spanda-docs`; regenerated `spanda-reference.md` and `docs/man/`.
- **Brand assets:** Logo in README, docs index, platform overview, mdBook intro; banner and favicon on `website/index.html`; VS Code extension icon from `assets/image/app_favicon.png`.
- Golden-path live AI example reference: `examples/features/live_openai.sd`.

### Fixed

- **Formatter spans:** parser `span_from` now uses an exclusive end offset so `spanda fmt` preserves closing braces on span-backed declarations (hardware, assurance, health, kill switch).
- **Robot safety formatting:** `max_speed` rules no longer duplicate velocity units when the value is already a unit literal.
- **State estimation runtime:** `state_estimator` declarations register `SensorFusion` bindings at robot setup; a single estimator aliases `fusion`.
- **Learned anomaly backends:** `learned_models()` detects `assurance.anomaly` imports for package-backed detectors.
- **Fleet and deploy agents:** per-robot/per-target state files in Rust and TypeScript so concurrent agents on one host do not inherit the wrong identity; fleet mesh relay no longer holds the coordinator mutex during outbound HTTP; fleet agents reject peer relays when startup identity is missing.
- **Agent hardening:** TypeScript per-identity state paths ignore `SPANDA_*_STATE` env overrides; stale deployment fields reset when loaded identity mismatches startup; HTTPS agents use read timeouts and connection shutdown; HTTP 400 responses shut down cleanly; TypeScript agent servers serialize concurrent requests; remote/mesh clients use 30s fetch timeouts; `spanda fleet mesh start` routes through the native CLI; integration test spawns use per-identity state files.

## [0.4.0] - 2026-06-22

### Added

- **Native deploy path:** `spanda deploy --target native` links LLVM binaries (same pipeline as `compile-native`); guide [native-deploy.md](docs/native-deploy.md).
- **ROS 2 polish:** `spanda ros2 check [--json]` validates `ROS_DISTRO`, rclpy, and bridge script before live transport.
- **Distributed fleet docs:** [fleet-distributed.md](docs/fleet-distributed.md) for `--remote` orchestration, agent registry, and OTA rollout.
- **CI:** `live-iot-golden-path` job runs `scripts/live_iot_golden_path.sh`.

## [0.3.0] - 2026-06-22

### Added

- **Install ergonomics:** crate renamed to `spanda` (`cargo install --path crates/spanda-cli` installs binary `spanda`); `spanda --version`; bundled showcase examples ship in the crate for `spanda demo` without a full clone; `scripts/sync_bundled_examples.sh`.
- **Productization (credibility & demos):** `spanda demo {rover,safety,verify,fleet,health}`; showcase directories under `examples/showcase/`; `scripts/install.sh`, `scripts/benchmark.sh`, `scripts/showcase_smoke.sh`; docs [benchmarks.md](docs/benchmarks.md), [known-limitations.md](docs/known-limitations.md), [demo-script.md](docs/demo-script.md), [diagrams/](docs/diagrams/); README trust table; improved `SafeAction` type errors with hints; VS Code snippets; CI `showcase-smoke` job.
- **LSP v0.3 polish:** keyword hover for `ActionProposal`, `SafeAction`, `safety.validate`, `deploy`, `health_check`, `kill_switch`; SafeAction quick-fix code action.

### Changed

- CI and golden-path scripts use `cargo build -p spanda` (package rename from `spanda-cli`).

### Fixed

- **Multi-robot fleet runtime:** interpreter now setup+executes each robot in isolation so `spanda fleet run` works when the last robot lacks member actuators (e.g. coordinator-only programs).

## [0.2.0] - 2026-06-23
### Added

- **Documentation audit:** new guides [verification-diagnostics.md](docs/verification-diagnostics.md), [typed-handler-io.md](docs/typed-handler-io.md), [testing.md](docs/testing.md); expanded [fleet-health.md](docs/fleet-health.md), [swarm-health.md](docs/swarm-health.md), [kill-switch.md](docs/kill-switch.md), [capability-traceability.md](docs/capability-traceability.md), [agentic-programming.md](docs/agentic-programming.md); example indexes under [examples/hardware/README.md](examples/hardware/README.md), [examples/iot/README.md](examples/iot/README.md).
- **Examples (Phase 27–35 coverage):** `features/kill_switch.sd`, `fleet_health_require.sd`, `typed_handler_returns.sd`, `agent_can_deny.sd`, `live_openai.sd`, `live_anthropic.sd`, `live_onnx.sd`, `security/remote_signed_kill_switch.sd`, `integration/debugger_every.sd`, `basics/12_compile_fail_tests.sd`, `iot/modbus_dispatch/`, `packages/publish_mirror_project/`.

### Added

- **Verification & DX (Phase 28):** `expect_compile_error { }` blocks in test bodies — validated at test-run time; module function return type enforcement in the typechecker; TypeScript parser mirror for Phase 27 syntax (`kill_switch`, `health_check`, `health_policy`, `requires_capability`, hardware components, robot `uses hardware` / `exposes capabilities`); IoT protocol package stubs (`spanda-opcua`, `spanda-modbus`, `spanda-zigbee`, `spanda-lora`, `spanda-matter`, `spanda-canbus`); integration tests in `p1_features.rs` and `tests/capability-parser.test.ts`.
- **Verification & DX (Phase 29):** span-aware verification diagnostics for capability, traceability, minimum-hardware, health, and kill-switch checks; `spanda check --verification-json`; LSP integration; runtime health evaluation wired to `HardwareMonitor`; kill-switch and health-fault sim integration tests.
- **Verification & DX (Phase 30):** `suggested_fix` hints on verification diagnostics; LSP quick-fix code actions; continuous runtime health polling during trigger maintenance; debugger pause events for kill switch (`kill_switch_activated`) and critical health (`health_critical`).
- **Verification & DX (Phase 31):** runtime `health_policy` enforcement; behavior `-> Type` return validation; agent plan `SafeAction` return checks; IoT package dispatch stubs; agent capability audit logging; DAP output events for health/kill-switch.
- **Verification & DX (Phase 32):** in-memory IoT hub; task return types; agent `can[]` default-deny; VS Code VSIX verify script.
- **Verification & DX (Phase 33):** trigger handler `-> Type` return validation; live Modbus TCP and OPC-UA bridge IoT paths; live OpenAI provider for `ai_model` when `OPENAI_API_KEY` is set.
- **Verification & DX (Phase 34):** event handler I/O verification; kill switch `remote_signed` runtime enforcement and `on kill_switch` handlers; VS Code extension CI; IoT protocol dispatch stubs; live Anthropic provider; fleet/swarm health runtime coordination.
- **Verification & DX (Phase 35):** TypeScript build parity; live IoT bridges for zigbee/lora/matter/canbus; fleet health `require` runtime; ONNX provider; registry mirror publish; kill-switch verify errors; debugger `every` entry.
- **Verification & DX (Phase 27):** `spanda-capability` crate — capability registry, hardware/robot capability inference, traceability matrices, minimum-hardware safety checks, health-check analysis; CLI commands `spanda trace {hardware|capabilities|health}`, `spanda health robot`, `spanda hardware capabilities`, `spanda robot capabilities`, `spanda safety check --capabilities`; verify flags `--traceability`, `--capabilities`, `--health`, `--minimum-capabilities`; hardened `spanda test` with file paths, `--json`, `--filter`, `--compile-fail`; language syntax for `kill_switch`, `health_check`, `health_policy`, `requires_capability`, `uses hardware`, `exposes capabilities`; sim flags `--trigger-kill-switch`, `--inject-health-faults`; IoT provider contracts in `spanda-runtime`; `spanda-iot-core` package stub; mdBook site at `docs-site/` + GitHub Pages workflow; guides for kill switch, health, capabilities, traceability, IoT, agentic programming, debugger. (`dispatch_official_package_call`); connectivity provider stubs for Wi-Fi/BLE/cellular; `--trace-providers` observability flag; `spanda update` command; flagship demo at `examples/showcase/autonomous_rover/`; guides [how-packages-work.md](docs/how-packages-work.md), [how-providers-work.md](docs/how-providers-work.md), [how-runtime-resolution-works.md](docs/how-runtime-resolution-works.md).
- **Platform integration (phase 2):** transitive dependency resolution; SLAM/vision/simulation provider dispatch; `provider_call` mission-trace frames; aligned provider capabilities in validation/security; TS `package_dispatch.ts` mirror; project-aware module registry for check/build/test/run/verify.
- **Phase 23 CI golden paths:** fleet `--remote` in `golden_path_deploy.sh`; live MQTT Mosquitto job (`scripts/mqtt_golden_path.sh`, `examples/communication/mqtt_live.sd`); twin cloud upload job (`scripts/twin_cloud_golden_path.sh`); LLVM job (`scripts/llvm_golden_path.sh`); `live-mqtt` / `live-transport` Cargo features on `spanda-cli`.
- **Phase 23 completion:** `world_model { }` parser + observe→fusion belief hook; ledger community scaffold (`packages/community/`, `scripts/ledger_golden_path.sh`); cpp-native and self-host lexer golden paths; Phase 23 marked complete in roadmap.
- **Phase 26 (in progress):** P1 adoption golden paths — `ci_verify_golden_path.sh`, `python_native_golden_path.sh`; `python-native` feature on `spanda-cli`; Phase 26 on roadmap. — `killer_demo_golden_path.sh`, `live_ai_golden_path.sh`, `ros2_golden_path.sh`, `registry_golden_path.sh`; CI jobs; [ros2_cmd_vel_ping.sd](examples/communication/ros2_cmd_vel_ping.sd); VS Code `vsce publish` on release when `VSCE_PAT` is set; P0 status table in [tier-3-priority-plan.md](docs/tier-3-priority-plan.md).
- **CI fix:** `cargo fmt` drift; TypeScript build parity (lexer tokens, compile stub, package dispatch); release workflow `VSCE_PAT` guard without invalid `secrets` in `if`.
- **Phase 24 (complete):** `world_model_patrol.sd` showcase; `fleet_field_trial.sd` three-agent layout; [tier-3-golden-paths.md](docs/tier-3-golden-paths.md) index; `world-model-golden-path` CI; typechecker support for `world_model.belief()` / `update()` / `export()`; [mqtt-nav2-reference-architecture.md](docs/mqtt-nav2-reference-architecture.md); [llvm-embedded-benchmark.md](docs/llvm-embedded-benchmark.md) + `llvm-embedded-golden-path` CI; TS mirror parser/checker parity for `world_model { }`.

- **Tier 3 priority plan:** [tier-3-priority-plan.md](docs/tier-3-priority-plan.md) documents P0–P4 ordering (v0.5 beta → Phase 23 hardening → v1.0 optional → post-v1.0 production); Phase 23 planned in [lean-core-roadmap.md](docs/lean-core-roadmap.md).

- **Phase 22 Tier 3 experimental:** world-model runtime (`world_model.update`/`belief`/`export`), ledger provider wired to `MockLedgerBackend`, cloud upload via `SPANDA_CLOUD_UPLOAD_URL`, LLVM golden path script, self-host bootstrap example, and [tier-3-experimental.md](docs/tier-3-experimental.md).
- **Phase 18 P2/P3 closure:** performance (slim CLI, bridge timeouts, `cargo audit`) and observability (pipeline benchmark) marked complete in docs.

- **Phase 21 hosted registry signing:** `registry-index-maintain` binary updates checksums and Ed25519 `version_signatures` in `registry/index.json`; CI verifies against `registry/TRUST_KEY`; `scripts/update_registry_checksums.py` delegates to the Rust tool.
- **Phase 21 embedder slimming:** optional `certify` and `bridge` features on `spanda-core` (`default-features = false` omits certification and FFI shims; `full` remains default).

- **Automated version bumps:** `scripts/bump_version.py` bumps `Cargo.toml`, npm packages, and finalizes `CHANGELOG.md`. **Auto release** runs after CI on `main` when a merged PR has `release:major`, `release:minor`, or `release:patch`; **Bump version** (manual Actions workflow) is available for ad-hoc releases. Both push `v*` tags that trigger cargo-dist **Release** builds.

- **Phase 18 security hardening:** registry tarball SHA-256 verification and tar-slip-safe extraction in `spanda-package`; deploy/fleet/mesh agents require `--token` on non-loopback binds; bridge subprocess timeouts; `cargo audit` CI job; slim CLI build (`--no-default-features --features slim`); pipeline benchmark test; [phase-18-security-hardening.md](docs/phase-18-security-hardening.md).
- **Phase 18b signed registry:** Ed25519 `version_signatures` on publish/install via `SPANDA_REGISTRY_SIGN_KEY` / `SPANDA_REGISTRY_TRUST_KEY`.
- **Phase 19 transport shim removal:** dropped `spanda_core::transport*` modules; `spanda-core` no longer depends on transport adapter crates directly.
- **Phase 20 test distribution + embedder features:** OTA, fleet, provider, and certify integration tests moved to owning crates; `spanda-core` exposes optional `ota` / `fleet` features (`default = ["full"]`; `--no-default-features` for minimal embedder builds).

- **Interpreter architecture docs:** [architecture.md](docs/architecture.md) documents the modular `spanda-interpreter` runtime tree, one-way `spanda-core` → `spanda-interpreter` dependency, and `CoreRuntimeHost` wiring (see [lean-core-roadmap.md](docs/lean-core-roadmap.md)).
- **Hosted registry (20 packages):** `registry/index.json` and tarballs for all official packages under `packages/registry/`; `./scripts/build-registry.sh` auto-discovers package scaffolds; [registry.md](docs/registry.md) curated table updated.
- **Killer demo:** flagship program at `examples/showcase/killer_demo.sd` with walkthrough in [killer-demo.md](docs/killer-demo.md) (check → verify → sim narrative).
- **Hosted registry tests:** `crates/spanda-package/tests/hosted_registry.rs` guards 20-package index, tarballs, and `file://` fetch; `killer_demo.sd` added to golden manifest.
- **Interpreter Phase 8 (partial):** moved `triggers`, `telemetry`, `replay`, `twin`, `events`, `state_machine`, `reliability_runtime`, and `serialize` into `spanda-runtime` with thin `spanda-core` shims; interpreter runtime routes imports through workspace crates; `RuntimeError` implements `Display`.
- **`spanda-comm` crate:** extracts `CommBus`, `InMemoryCommBus`, comm safety chain, and bandwidth helpers from `spanda-core` with a thin shim; interpreter runtime imports `spanda_comm::CommBus` directly.
- **`spanda-safety` crate:** extracts `SafetyMonitor`, zones, `Pose2d`, and motion validation from `spanda-core` with a thin shim; interpreter runtime imports `spanda_safety` directly.
- **`spanda-hal` crate:** extracts HAL simulation backend, SoC profiles, and hardware health monitoring from `spanda-core` with thin shims; interpreter runtime imports `spanda_hal` directly.
- **`spanda-transport` routing:** moves wire-frame encode/decode into `spanda-transport` and `RoutingCommBus` into new `spanda-transport-routing` (avoids adapter-backend cycle); thin `transport` / `transport_wire` shims; interpreter runtime imports `spanda_transport_routing::RoutingCommBus` directly.
- **`spanda-error` crate:** extracts `SpandaError` and diagnostic helpers from `spanda-core`; interpreter runtime imports `spanda_error::SpandaError` directly; `RunOptions` / `RunResult` remain in core.
- **`spanda-ai` crate:** extracts AI model registry, agent runtime, memory store, and mock inference helpers from `spanda-core` with a thin shim; interpreter runtime imports `spanda_ai` directly.
- **`spanda-providers` crate:** extracts official package bootstrap, stubs, and transport adapter wiring; interpreter imports `spanda_providers` for registry bootstrap and comm-bus sync.
- **`spanda-concurrency` crate:** extracts cooperative channels, spawn handles, and select; thin core shim; interpreter imports `spanda_concurrency` directly.
- **`spanda-debug` crate:** extracts debugger controller, breakpoints, and `stmt_line`; interpreter imports `spanda_debug` directly.
- **Phase 8 routing complete:** `spanda-regex-lang`, `spanda-lib-registry`, `spanda-connectivity-runtime`, `spanda-runtime-host`, and `spanda-ffi` extracted with thin shims; interpreter runtime has zero `crate::` imports.
- **Phase 11 SIR extraction:** `spanda-sir` crate owns AST lowering to typed intermediate representation; `spanda-core` re-exports via thin shim.
- **Phase 10 run pipeline:** `spanda-certify` and `spanda-bridge` extracted; `spanda-driver::run` owns compile + certify gate + FFI defaults + interpreter execution; `spanda-core` re-exports the public API.
- **Phase 12 tooling extraction:** `spanda-hardware` (full verify + adapter verify + connectivity validators), `spanda-format`, `spanda-lint`, `spanda-codegen`, `spanda-modules`, and `spanda-docs` extracted with thin core shims; `spanda-security::validate` owns static security audit; `spanda-driver::debug_session` owns the debugger machine; `spanda-fleet::swarm_coordinator` owns swarm coordination; connectivity-runtime re-exports hardware validators to preserve the public API.
- **Phase 13 facade slim-down:** `spanda-driver` now owns verify, SIR lowering, replay/playback, debug run, deploy plan (with certify proof), and type-check host wiring; `spanda-ota::plan` extracts deploy assignments from the AST; reliability validators live in `spanda-typecheck`; `spanda-core` re-exports the public API without local pipeline bodies.
- **Phase 14 shim consolidation:** `transport_live` RuntimeValue hooks live in `spanda-transport-routing`; lexer `tokenize` error mapping and FFI bridge alias moved to `spanda-driver` / `spanda-bridge`; `providers/` collapsed to a single facade module.
- **Phase 15 caller migration:** `spanda-cli` and `spanda-node` import workspace crates directly (`spanda-driver`, OTA/fleet/deploy-http, tooling crates); MQTT/DDS/WebSocket `RuntimeValue` live bridges consolidated in `spanda-transport-routing::live_bridges` with thin core shims retained for API stability.
- **Phase 16 caller migration:** `spanda-llvm`, `spanda-wasm`, and `spanda-dap` no longer depend on `spanda-core`; only the `spanda-core` facade crate itself pulls the full workspace graph for external API stability.
- **Phase 17 transport shim removal:** removed `spanda_core::transport_mqtt`, `transport_dds`, `transport_websocket`, and `transport_live`; use `spanda-transport-routing` or `spanda-transport-*` workspace crates directly.
- **Documentation refresh:** rewritten lean-core guide, workspace crate index, per-crate READMEs; **tutorials & examples hub** (`examples/README.md`, `examples/packages/README.md`, updated tutorial indexes and learning paths); **API doc hierarchy** ([api-documentation.md](docs/api-documentation.md), grouped [api-reference.md](docs/api-reference.md) with facade→crate mapping).

### Changed core `mission` (named steps + lifecycle), program-level `fleet`, `safety_zone`, and `certify` metadata (optional `level` block), extended `observe`/`fusion.read()` with `confidence` and `state_estimate`, `std.navigation` / `std.fusion` / `std.slam` namespaces, navigation runtime helpers; program-level safety zone speed caps (motion allowed in cap zones); TypeScript parser/type-checker and interpreter parity; Nav2 golden-path publish on `navigation.navigate()` when `/cmd_vel` is declared; **OTA deploy CLI** (`spanda deploy plan|rollout|rollback|status` with canary/staged strategies); **fleet orchestrator** (`spanda fleet orchestrate`); verify warning when deploy targets lack certification metadata; examples in `examples/robotics/`; tests in `crates/spanda-core/tests/` and `tests/robotics-platform.test.ts`
- **Robotics TS CLI parity:** TypeScript mirrors for OTA deploy service and fleet orchestrator; `spanda deploy plan|rollout|rollback|status` and `spanda fleet orchestrate` in the Node CLI without requiring the Rust binary; hardware verify warns on deploy-without-`certify` in the TS fallback
- **Robotics navigation sugar + adapter verify:** `navigate { goal: ... }` statement sugar (Rust + TS); `spanda verify` reports framework adapter mappings for imports like `navigation.nav2`; registry entries for `spanda-nav2`, `spanda-cartographer`, and `spanda-rtabmap`; example package `examples/packages/nav2_adapter_package/`
- **Remote OTA deploy agents:** HTTP deploy agent server (`spanda deploy agent start`), agent registry (`.spanda/deploy-agents.json`), `spanda deploy rollout|rollback --remote`; SLAM stub runtime (`slam.localize()` / `slam.map()`); fleet orchestrator peer-aware mode
- **OTA artifact integrity + HTTPS agents:** deploy plans include SHA-256 `program_hash`; remote rollouts send hash to agents; optional `--require-hash` on agents; HTTPS agent URLs and `--tls-cert` / `--tls-key` for deploy agents (Rust rustls + Node https); fleet peer handoff messages during orchestration; SLAM adapter example packages (`cartographer_adapter_package`, `rtabmap_adapter_package`); `examples/robotics/fleet_peer_missions.sd`
- **Signed OTA bundles + fleet mesh delivery:** Ed25519-signed deploy artifact bundles (`--sign-key`, `--bundle-out`); agents verify signatures with `--require-signature --trust-key`; fleet orchestrator delivers peer mission steps over the in-process comm mesh (`peer_mesh_mission`)
- **Distributed fleet agents + strict certify verify:** HTTP fleet peer relay agents (`spanda fleet agent start|register|list`, `.spanda/fleet-agents.json`); `spanda fleet orchestrate --remote` relays peer mission steps to registered agents (`distributed_peer_mesh`); `spanda verify --strict-certify` treats missing deploy certification, ISO13849 level gaps, and deployed-robot mission/safety metadata as errors; adapter registry metadata for Nav2/Cartographer/RTabMap packages
- **Fleet mesh coordinator + runtime certify gate + adapter production hooks:** `spanda fleet mesh start` centralizes multi-host peer relay (`--mesh-url`); fleet agents forward peer deliveries to downstream robots; `spanda run --enforce-certify` / `SPANDA_ENFORCE_CERTIFY=1` blocks uncertified deploy programs at runtime; `spanda verify-adapter` validates package `[adapter]` sections; optional `SPANDA_NAV2_CMD` / `SPANDA_SLAM_CMD` subprocess bridges for production Nav2/SLAM backends
- **Certification proof artifacts + TS adapter bridge parity:** `spanda certify prove [--strict] [--out proof.json]` emits structured checklist reports with `program_hash`; TypeScript interpreter invokes Nav2/SLAM subprocess bridges and enforces `--enforce-certify` on run; reference bridge scripts in `examples/adapters/`
- **Deploy certification gate:** deploy plans embed certification proof summaries; `spanda deploy rollout --require-certify` blocks OTA when strict proof fails; deploy agents accept `--require-certify` to reject rollouts missing strict proof in the payload; TypeScript deploy-service and `verify-adapter` Node fallback mirror Rust behavior
- **Swarm coordinator (experimental):** program-level `swarm` declarations with `round_robin`, `broadcast`, and `leader_follow` policies; `spanda swarm coordinate` runtime with persistent round-robin cursors in `.spanda/swarm-state.json`; TypeScript parser/checker/coordinator parity
- **Robotics golden path script:** `examples/robotics/golden_path_deploy.sh` now covers certify, deploy, verify-adapter, fleet orchestrate, and swarm coordinate
- **Swarm mesh relay:** `spanda swarm coordinate --mesh-url` relays leader-follow peer deliveries through the fleet mesh coordinator; CI `robotics-golden-path` job runs the golden-path script against the release CLI
- **Swarm peer mesh parity:** round_robin and broadcast policies collect peer-link deliveries for mesh relay; golden path covers mesh fleet/swarm, remote OTA dry-run, and Nav2/SLAM adapter bridge fixtures

### Fixed

- **Fleet mesh CLI routing:** `spanda fleet mesh start` now receives the correct subcommand args (was treating `mesh` as the subcommand and exiting with usage)
- **Fleet mesh registry reload:** mesh coordinator reloads `SPANDA_FLEET_AGENTS` on each relay request instead of snapshotting at startup; fleet agents honor the same env for downstream forwarding
- **Swarm mesh peer delivery:** round_robin/broadcast include peer-link deliveries; leader_follow avoids duplicate peer/member handoffs
- **Secure communication:** optional encrypted communication across buses, topics, services, and actions — `secure_comm` policy, `trust_boundary` declarations, `secrets` blocks (env/file), extended `secure { }` blocks with encryption/authentication/trusted sources, `EncryptedMessage`/`VerifiedMessage` types (AES-256-GCM), production transport wire frames with `source_id`, `spanda security check|audit`, `--secure` and `--inject-security-faults` CLI flags; docs in `docs/secure-communication.md`, `docs/identity.md`, `docs/secrets.md`, `docs/trust-boundaries.md`; examples in `examples/security/`
- **Secure comm TS parity:** TypeScript `RoutingCommBus` wire encryption, `secure_comm` configure fail-fast, inbound `source_id`, trust-boundary registry, static `security check|audit`, and integration tests in `tests/security-comm.test.ts`
- **Live MQTT (optional):** `live-mqtt` Cargo feature with rumqttc bridge; enable with `SPANDA_LIVE_MQTT=1`
- **Live WebSocket + DDS (optional):** `live-websocket` / `live-dds` Cargo features (or `live-transport` bundle); enable with `SPANDA_LIVE_WEBSOCKET=1` / `SPANDA_LIVE_DDS=1`
- **mTLS handshake (optional):** rustls client handshake when mutual auth + cert/key files + TLS broker URL; `SPANDA_MTLS_REQUIRED=1` fails hard; TypeScript mirror with `SPANDA_MTLS_HANDSHAKE=1`
- **Runtime trust-boundary enforcement:** publish/receive validates declared boundaries against transport-mapped crossing rules
- **Bus broker URL:** `url:` field on `bus { }` blocks and `SPANDA_BROKER_URL` env fallback for live transport and mTLS

### Fixed

- **Secure comm parser/runtime:** `secure_topic.publish` / `actuator.execute.safe` capability parsing, timed `fault … at T+10s` offsets, inbound trusted-source checks on receive/poll, TypeScript parser mirror for `secure_comm`, `trust_boundary`, `secrets`, bus blocks, and full `secure { }` fields
- **Example regression:** repaired 20 skipped `.sd` examples (regex, security, robotics, packages, hardware/modules); `scripts/check_all_examples.sh` resolves relative `SPANDA_BIN` from repo root for package checks — **162 pass, 2 expected-fail, 0 skips**
- **Lean-core transport shims:** ROS2/MQTT live bridge logic moved from `spanda-core/src/transport_live.rs` into `spanda-transport-ros2` and `spanda-transport-mqtt`; core retains a thin `RuntimeValue` shim with `lean_core_shims` guard tests
- **Lean-core transport adapters:** `TransportAdapter` implementations moved from `spanda-core/src/transport.rs` into `spanda-transport-{ros2,mqtt,dds,websocket}`; ROS2 rclrs consolidated in `spanda-transport-ros2`; Nav2/SLAM subprocess bridge moved to `spanda-connectivity::adapter_bridge`; unused TLS deps removed from `spanda-core` (TLS remains in `spanda-transport` and deploy crates)
- **Lean-core provider kernel:** `ProviderRegistry` and provider trait contracts moved to `spanda-runtime`; new `spanda-transport` crate for adapter traits and wire security; `spanda-interpreter` staging crate; fleet orchestration moved to `spanda-fleet`
- **Lean-core connectivity runtime split:** moved geofence math, connectivity/fault trigger mapping, GPS drift/spoof simulation, and link impairment checks from `spanda-core::connectivity_positioning` to `spanda-connectivity::runtime_sim`; core keeps compatibility wrappers for AST/runtime value conversions
- **Interpreter extraction staging:** expanded `spanda-runtime::RuntimeHost` with connectivity/geofence/GPS-fault hooks and routed `spanda-core::runtime` trigger/failover/geofence callsites through host methods to reduce direct core coupling
- **Interpreter host injection:** `Interpreter` now stores an injectable `RuntimeHost` (`InterpreterOptions::runtime_host`); remaining GPS reading and SIM identity paths route through host hooks; `spanda-interpreter` re-exports `RuntimeHost`
- **Interpreter module split:** connectivity trigger, geofence, and failover logic extracted from `runtime.rs` into `runtime_connectivity.rs` as a staging step toward `spanda-interpreter`
- **Interpreter submodule extraction:** navigation/SLAM (`runtime_navigation.rs`), robot methods (`runtime_robot.rs`), and trigger dispatch (`runtime_triggers.rs`) split out of `runtime.rs` with lean_core guard tests
- **Interpreter robotics/sensors/twin split:** AI/mission/fleet/safety (`runtime_robotics.rs`), sensor fusion (`runtime_sensors.rs`), and digital twin (`runtime_twin.rs`) extracted from `runtime.rs` (~580 lines); `runtime.rs` down to ~7670 lines
- **Interpreter builtins/audit/actuators split:** builtin dispatch (`runtime_builtins.rs`), audit/ledger (`runtime_audit.rs`), actuator motion (`runtime_actuators.rs`), and shared helpers (`runtime_helpers.rs`) extracted; `runtime.rs` down to ~6640 lines
- **Interpreter eval cluster split:** expression evaluation, member/call dispatch, regex methods, and binary operators moved to `runtime_eval.rs`; `runtime.rs` down to ~5750 lines
- **Interpreter spawn/async split:** module calls, future resolution, spawn targets, and task-handle queue processing moved to `runtime_spawn.rs`; `runtime.rs` down to ~5480 lines
- **Interpreter execution split:** statement execution (`runtime_execute.rs`), scheduling/contracts (`runtime_scheduler.rs`), robot setup (`runtime_setup.rs`), reliability (`runtime_reliability.rs`), declarations (`runtime_declarations.rs`), program/trigger glue (`runtime_program.rs`), and security helpers (`runtime_security.rs`) extracted; orchestrator down to ~1790 lines
- **Interpreter sources in `spanda-interpreter`:** all `runtime_*.rs` modules and `orchestrator.rs` now live under `crates/spanda-interpreter/src/runtime/`; `spanda-core/src/runtime.rs` is a thin `#[path]` include shim; smoke tests moved to `runtime_smoke.rs`

### Changed

- **Dependency security:** `cargo update` bumps `log` 0.4.33 and `quote` 1.0.46; npm upgrades `vitest` to 3.2.6 (critical Dependabot) with `vite` override to 6.4.3 (`npm audit` clean); removed unused TLS crates from `spanda-core` AES-256-GCM wire frames (`spanda/wire/v1:`), `TransportWireFrame` with `source_id`, TLS session negotiation from cert/key secrets, rustls PEM validation when cert files exist, broker URL TLS scheme auto-upgrade (`mqtts://`, `wss://`), session-key derivation from robot secrets for `EncryptedMessage`, and production wire crypto (replacing mock-session stubs)
- **Python native bridge runtime:** upgraded optional `pyo3` from 0.23 to 0.29 and migrated bridge GIL entrypoint to `Python::attach`; fixed embedded Python runner script syntax for native bridge tests
- **MQTT TLS dependency chain:** `spanda-transport-mqtt` now uses `rumqttc` 0.25.1 with `default-features = false` + `use-native-tls`, removing the old `rustls-webpki <0.103.13` path from the MQTT transport dependency graph
- **VS Code marketplace readiness:** bundled LSP in extension VSIX, deploy-target autocomplete, verify picker command, Spanda debug type (`editor/vscode/`)
- **Hosted package registry:** `registry/index.json` + `spanda-openai` / `spanda-ros2` tarballs; default `SPANDA_REGISTRY_URL`
- **Live AI provider:** OpenAI via Python bridge — `docs/live-ai-provider.md`, `examples/ffi_openai_live.sd`
- **Twin replay JSON export:** `spanda twin export` and `--twin-export` on run/sim
- **Web playground:** killer demo preset as default (`packages/web/`)
- **Debug workflow:** `docs/debugging.md` — step through `task every` in VS Code
- **Adoption docs:** `docs/adoption-path.md` (one-sprint Python + ROS2 wrap), `docs/ci-verify.md` (GitHub Actions / GitLab + `--json`), `docs/ros2-golden-path.md` (rclpy bridge golden path)
- **Flagship showcase index:** `examples/showcase/README.md` — three evaluator entry points (safety, verify, sim); README trimmed to match
- **End-to-end examples:** warehouse delivery, pick-and-place cell, fleet coordination, incident response, real-time patrol, validated telemetry, concurrent inspection (`examples/end_to_end/`)
- **Feature examples:** `examples/features/` (16 focused demos) plus coverage index mapping every capability to a runnable file
- **Tutorials index:** master catalog at `docs/tutorials/README.md` (all learning paths, topic guides, examples)
- **Spanda for Dummies:** plain-English guide in `docs/spanda-for-dummies/` (cheat sheet, glossary, common mistakes)
- **Spanda 101:** ten-lesson tutorial series in `docs/spanda-101/` (hello robot through end-to-end patrol)
- **Examples ladder:** `examples/basics/` (11 progressive tutorials), `examples/integration/`, and `examples/end_to_end/` (safe patrol package + replay mission)
- **Cross-platform installable packages:** cargo-dist release pipeline (Linux/macOS/Windows archives, shell/PowerShell installers, Windows MSI, Homebrew formula); see [docs/installation.md](docs/installation.md)
- Deadline-aware tasks: `deadline`, `jitter <=`, `priority`, `isolated`
- Latency pipelines: `pipeline name budget Nms { … }`
- Watchdogs, operating `mode` blocks, `recover from`, `retry`/`fallback`
- First-class regex: literals, `Regex` type, string methods, triggers, subscribe filters, `validate` rules
- Mission trace replay: `spanda replay`, `--record`, `--trace-realtime`, `--metrics-json`
- Runtime telemetry: `PipelineMetrics`, `WatchdogMetrics`
- Docs: `docs/realtime.md`, `docs/reliability.md`, `docs/watchdogs.md`, `docs/degraded-modes.md`, `docs/replay.md`, `docs/regex.md`
- **Language reference:** `spanda reference`, `docs/spanda-reference.md` (JavaDoc-style `std.*`, builtins, types), `docs/man/` (man-page CLI docs)
- **Compiler API index:** `docs/api-reference.md` (Rust/TypeScript modules and public functions)
- Examples under `examples/realtime/` and `examples/regex/`
- **GPS/GNSS positioning and wireless connectivity:** `requires_connectivity`, hardware `connectivity [ … ]`, WGS84 `geofence`, `connectivity_policy`, Bluetooth/BLE blocks, connectivity triggers (`on gps.lost`, `on network.disconnected`, `on gps.spoofed`), `std.positioning` / `std.connectivity` namespaces; TypeScript parser/runtime mirror with TS verify fallback and transport rebinding on failover; u-blox NEO-M8N UART GNSS stub in `lib_registry`; docs in `docs/positioning.md`, `docs/connectivity.md`, `docs/geofencing.md`, `docs/bluetooth.md`, `docs/cellular.md`; examples in `examples/connectivity/`
- **GPS fault simulation at runtime:** `GpsSpoofing` offsets coordinates and degrades fix quality; `GpsDrift` accumulates positional drift over sim time; applied to GPS sensor reads and geofence checks in Rust and TypeScript; triggers `on gps.spoofed` and `on gps.drift`
- **TypeScript hardware verify parity:** builtin profile registry, sensor/actuator/network/connectivity checks, timing and mission validation, resource budget, deploy resolution, AI model memory/GPU checks, adapter mapping, topic bandwidth estimation, and `simulate_compatibility` fault injection when Rust CLI is unavailable
- **Transport reconnect on connectivity failover:** active transport adapter connects and resubscribes topic paths when `connectivity_policy` switches links; inactive stub adapters disconnect
- **Cellular SIM identity:** `SimIdentity` type and `robot.sim_identity()` return ICCID/carrier/eSIM/attested fields; gated by `cellular.connect` under strict permissions
- **Satellite emergency backhaul:** `Satellite` connectivity token maps to websocket transport; `SatelliteOutage` fault and `emergency: satellite` failover policies; example in `examples/connectivity/satellite_backup.sd`
- **Cascade failover:** when fallback link is fault-impaired (`NetworkOutage`, `LteOutage`, etc.), runtime escalates to `emergency` link in the same step
- **Documentation sync:** migration and getting-started guides updated for TypeScript hardware verify fallback

### Changed

- `.gitignore` allows committed golden mission traces under `examples/` and `tests/golden/` while ignoring other runtime `.trace` files
- Canonical repository moved to [Davalgi/Spanda](https://github.com/Davalgi/Spanda) (transferred from `sujaydavalgi/Spanda`); docs and package metadata URLs updated accordingly
- Runtime now executes watchdogs (task heartbeats), `run_pipeline`, retry/fallback on injected faults, recovery handlers, jitter telemetry, and mission trace recording (`--record` writes `<file>.trace`)
- Operating `mode` blocks execute on enter; topic QoS `deadline` violations are detected at runtime
- `spanda replay --deterministic` re-runs the traced program and verifies frame parity
- TypeScript mirror syncs parse/typecheck for realtime, reliability, regex, and replay features
- Wall-clock RTOS scheduling via `--wall-clock`; frame-by-frame mission playback via `spanda replay --playback`
- Mission traces (v2) embed robot state snapshots for playback without re-running program logic

## [0.1.0-alpha] - 2026-06-20

First public alpha release. Spanda is ready for community evaluation.

### Added

**Language & runtime**
- Spanda language (`.sd`) with robot-centric syntax: sensors, actuators, safety, agents, tasks
- Physical unit type system (`m`, `s`, `rad`, `m/s`, compound units)
- AI-native agents with `ai_model`, `goal`, `memory`, and mock LLM/Vision providers
- `ActionProposal` → `safety.validate()` → `SafeAction` compile-time and runtime gate
- Safety zones, emergency stop, and behavioral `verify { }` assertions
- State machines, events, digital twins with replay buffer
- Communication primitives: `message`, `topic`, `service`, `action`
- Hardware profiles, `deploy` targets, and `requires_hardware` / `requires_network`
- Foundations: `module`, `struct`, `enum`, `trait`, `match`, generics, trait objects
- Deterministic task scheduler (`task every Nms`) with resource budgets
- Sensor fusion via `observe { }` and `fusion.read()`
- SoC/HAL profiles (Raspberry Pi, ESP32, STM32, Jetson, Arduino)
- Manufacturer sensor driver registry (Velodyne, Bosch, Intel, Hokuyo, and others)

**Tooling**
- Native CLI: `check`, `verify`, `run`, `sim`, `fmt`, `lint`, `doc`, `debug`, `ir`
- Hardware verification: `--target`, `--all-targets`, `--simulate`, `--json`
- Package manager: `init`, `build`, `test`, `install`, `add`, `remove`
- TypeScript CLI wrapper with Rust delegation
- Language Server (`@spanda/lsp`): diagnostics, completion, hover, rename, format
- Web playground (`@spanda/web`) with WASM bindings
- Debug Adapter Protocol server (`spanda-dap`)
- Experimental LLVM path: `llvm-ir`, `compile-native`

**Security & audit**
- Capability system, secrets, signed messages, audit records

**Examples**
- 72+ sample `.sd` programs including `examples/showcase/` curated demos
- Package examples under `examples/packages/`

**Documentation**
- README overhaul with positioning and architecture overview
- `docs/getting-started.md`, `docs/architecture.md`, `docs/vision.md`
- `docs/feature-status.md` with v0.1.0-alpha support matrix
- `docs/website-content.md` for future site
- Language reference, type system, hardware compatibility, packages, security docs

**Community**
- `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`
- GitHub issue templates: bug report, feature request, language proposal, package proposal
- CI: Rust tests, TypeScript tests, `cargo fmt`, `cargo clippy`, LSP, WASM, ROS2 rclrs native (Ubuntu 22.04 + Humble)

### Known limitations

- AI providers use mock backends by default; set `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, or `SPANDA_ONNX_MODEL_PATH` for live calls
- ROS2 integration requires manual ROS Humble setup (experimental)
- LLVM/native compilation is experimental; interpreter is the primary runtime
- `spanda publish` mirrors to `registry/packages/`; hosted index lists 20 curated packages until `./scripts/build-registry.sh` is run
- VS Code extension VSIX builds in CI; Marketplace publish pending maintainer `VSCE_PAT`
- Multi-robot examples run in-process by default; distributed orchestration uses HTTP fleet agents

### Roadmap (post-alpha)

- VS Code Marketplace publish
- Production LLVM backend and optimized native binaries
- In-process Python/C++ FFI (PyO3, cxx) as primary path
- ROS2 production adapter with zero-config deployment
- Self-hosting compiler
- Digital twin cloud SaaS backend
- Distributed multi-robot orchestration at scale

[0.1.0-alpha]: https://github.com/Davalgi/Spanda/releases/tag/v0.1.0-alpha

## [Unreleased]

Post-alpha improvements on `main` (2026-06-20).

### Added

**Triggers & concurrency**
- Unified trigger execution model: events, messages, timers, conditions (`when`/`while`), state, safety, hardware, AI, verification, and twin triggers
- `TriggerRegistry` with priority ordering, per-tick storm limits, and `TriggerMetrics` telemetry
- CLI trace flags: `--trace-triggers`, `--trace-events` on `run`, `sim`, and `fleet run`
- Cooperative concurrency runtime: `spawn`, `join`, `parallel`, channels, `select`, per-task `budget { }`
- `spanda fleet run` for in-process multi-robot fleet simulation with deploy/peer wiring output
- Runtime telemetry: `TaskMetrics`, `SchedulerMetrics`, `ExecutionMetrics` in `RunResult.metrics`
- TypeScript interpreter parity for concurrency and fleet peer messaging
- `examples/triggers_demo.sd`, `examples/concurrency.sd`, `examples/communication/multi_robot_fleet.sd`
- [docs/triggers.md](docs/triggers.md), [docs/concurrency.md](docs/concurrency.md), [docs/product-strategy.md](docs/product-strategy.md)

**ROS 2**
- Native `spanda-ros2-rclrs-native` cdylib for in-process ROS 2 I/O
- Persistent rclpy ROS2 daemon transport (`SPANDA_ROS2_RCLRS`)
- CI job `ros2-rclrs-native` on Ubuntu 22.04 with ROS Humble

**Developer experience**
- Inline API documentation across all Rust crates and TypeScript sources
- Contextual logic-block comments replacing generic placeholders
- Doc tooling: `scripts/add_inline_docs.py`, `scripts/add_logic_block_docs.py`, `scripts/normalize_inline_docs.py`
- VS Code extension scaffold operationalized (`editor/vscode`) with packaging workflow
- Remote registry tarball caching for offline `spanda install`

### Fixed

- Rust brace indentation after bulk inline doc insertion (`cargo fmt` compliance)
- CI: pin `ros-tooling/setup-ros@v0.7` (invalid `@v2` reference)
- Removed dead empty `if` block in type checker (clippy `needless_ifs`)

### Changed

- [CONTRIBUTING.md](CONTRIBUTING.md) documents inline documentation standards and doc scripts
- [docs/README.md](docs/README.md) indexes triggers, concurrency, and developer doc tooling
- [docs/roadmap.md](docs/roadmap.md) marks triggers and cooperative concurrency as completed

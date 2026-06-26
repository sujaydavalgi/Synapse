# Enterprise Operations — Stable Hardening Checklist

Phases **E1–E4** are shipped at **Experimental** tier with CI smoke (`scripts/enterprise_ops_smoke.sh`, `scripts/control_center_desktop_smoke.sh`). This checklist tracks what remains before promoting enterprise operations pillars to **Stable** (target: v0.5 beta → v1.0).

**Related:** [enterprise-operations-roadmap.md](./enterprise-operations-roadmap.md) · [feature-status.md](./feature-status.md) · [roadmap.md](./roadmap.md) · [control-center.md](./control-center.md)

---

## Promotion criteria (all pillars)

| Gate | Requirement | Status |
|------|-------------|--------|
| Smoke | `enterprise_ops_smoke.sh` green on `main` | **Shipped** |
| Docs | User-facing docs match behavior | **Shipped** (ongoing sync) |
| API contract | `/v1/*` version header + **full OpenAPI** for REST routes (parity CI test) | **Shipped** |
| RBAC | Mutations gated; matrix documented | **Shipped** |
| HA persistence | Alerts, traces, incidents survive restart | **Shipped** (`SPANDA_CONTROL_CENTER_STATE_DIR`) |
| Multi-tenant | `SPANDA_TENANT_ID` + key `tenant_id` mismatch → 403 | **Shipped** |
| Soak | 30-day field pilot without data-loss regressions | **Planned** |
| Security audit | Third-party review of auth + secret handling | **Planned** |

---

## Per-pillar checklist

### Control Center + APIs

| Item | Experimental (today) | Stable requires |
|------|---------------------|-----------------|
| REST v1 | Full E1–E4 surface | — (OpenAPI parity test in CI) |
| gRPC | 59 RPCs (tonic) | **Shipped:** published proto semver policy (`GET /v1/version` → `grpc`) |
| Remote CLI | `spanda control-center *` shortcuts | **Shipped:** OpenAPI parity test (`control_center_openapi_parity.rs`) |
| Rate limits | `SPANDA_API_RATE_LIMIT_PER_MINUTE` | Load-test defaults documented per tier |
| Mutation audit | Hash-chained JSONL | External SIEM export adapter (package) |

### Device Pool + Provisioning

| Item | Experimental (today) | Stable requires |
|------|---------------------|-----------------|
| Lifecycle | discover → active → quarantine → retire | Fleet-scale (1000+ devices) perf benchmark |
| Discovery | mDNS/BLE/USB/wifi/cellular/serial registry | Production transport certs per vendor |
| Provisioning | `POST /v1/provision`, per-device workflows | Idempotent reprovision + conflict policy doc |
| Failover | Chain enrichment in recovery | **Shipped:** automated failover drill smoke (`scripts/failover_drill_smoke.sh`) |

### Configuration Management

| Item | Experimental (today) | Stable requires |
|------|---------------------|-----------------|
| Cascading TOML | resolve, diff, validate, graph | — (already Stable-adjacent) |
| Snapshots | save/list under `.spanda/` | **Shipped:** AES-256-GCM encrypted snapshots (`encrypt`, `SPANDA_CONFIG_SNAPSHOT_KEY`) |
| Approvals | queue + publish-on-approve | **Shipped:** multi-approver quorum (`required_approvals`, `SPANDA_CONFIG_APPROVALS_REQUIRED`) |
| Drift | 7-dimension operational drift API | **Shipped:** scheduled scans (`SPANDA_DRIFT_SCAN_INTERVAL_SECS`), `GET /v1/drift/scans`, `POST /v1/drift/scan`, `ConfigDrift` alerts |

### OTA & Rollback

| Item | Experimental (today) | Stable requires |
|------|---------------------|-----------------|
| Plan | canary / staged / blue_green dry-run | — |
| Execute | `POST /v1/ota/execute` via deploy agents | **Shipped:** `rollback_on_readiness_fail`; **Shipped:** fleet soak (`scripts/ota_fleet_soak.sh`) |
| Certification | `--require-certify` gate in planner | Mandatory in default production policy |

### Observability + SRE

| Item | Experimental (today) | Stable requires |
|------|---------------------|-----------------|
| Traces | API log + OTLP export to Jaeger | Managed collector HA deployment guide |
| Metrics | OTLP metrics preview + export | Dashboard templates (Grafana package) |
| WebSocket | `/v1/stream/telemetry` | Backpressure + reconnect contract |
| SRE | SLO, MTTR/MTBF hints, incidents, auto-open from critical alerts | **Shipped:** SLO burn-rate rollup on `/v1/sre/summary` (`burn_rate.fast_burn`) |
| Incidents | ack/resolve workflow | PagerDuty bi-directional sync |

### Compliance + Digital Thread

| Item | Experimental (today) | Stable requires |
|------|---------------------|-----------------|
| Export | `GET /v1/compliance/export` + evidence log | Profile catalog (defense, medical, ISO) signed templates |
| Reports | markdown / JSON / PDF | Scheduled report delivery |
| Digital thread | query API + **interactive graph UI** | Full lifecycle graph (requirement → retirement) |

### SDKs

| Item | Experimental (today) | Stable requires |
|------|---------------------|-----------------|
| Python | REST client + stream extra | PyPI publish + semver policy |
| TypeScript | `ControlCenterPanel` in `@spanda/web` | npm publish + visual regression CI |
| Remote CLI | `spanda control-center` client | Documented in `getting-started.md` |

### Desktop (Tauri)

| Item | Experimental (today) | Stable requires |
|------|---------------------|-----------------|
| Dev shell | `@spanda/control-center-desktop` | — |
| Build | `TAURI_BUILD=1` macOS CI artifacts | Signed + notarized macOS/Windows installers |
| Auto-update | `tauri-plugin-updater` scaffold | Active updater + key rotation runbook |
| Security | glib RUSTSEC git patch | Upstream gtk4 migration (Tauri v3 track) |

### Alerting

| Item | Experimental (today) | Stable requires |
|------|---------------------|-----------------|
| Channels | webhook, email, Slack, PagerDuty, Teams packages | On-call rotation + escalation policies |
| Dedup | Core dispatcher | Configurable dedup windows per severity |

---

## CI gates for Stable promotion

When promoting a pillar from **Experimental** → **Stable**, verify:

1. `scripts/enterprise_ops_smoke.sh` — includes pillar-specific assertions
2. `scripts/showcase_smoke.sh` — still green
3. `docs/feature-status.md` — pillar row updated
4. `CHANGELOG.md` — `[Unreleased]` → versioned release note
5. No **Planned** items remain in that pillar's Stable column above

---

## Out of scope for Stable (remain Future)

- VS Code Marketplace publish (maintainer `VSCE_PAT` — optional)
- Full digital-twin SaaS backend
- Blockchain / ledger production adapters
- Predictive analytics (readiness forecasting) — differentiation **NEXT**

---

## Related documents

- [control-center.md](./control-center.md) — API and UI reference
- [device-pool.md](./device-pool.md) · [device-provisioning.md](./device-provisioning.md)
- [telemetry-store.md](./telemetry-store.md) · [drift-detection.md](./drift-detection.md)
- [platform-maturity-roadmap.md](./platform-maturity-roadmap.md) — Phase A–D stable hardening (separate track)

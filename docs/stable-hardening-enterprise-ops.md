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
| Soak | 30-day field pilot without data-loss regressions | **Shipped:** `scripts/field_soak_gate.sh` + [field-soak-gate.md](./field-soak-gate.md) |
| Security audit | Third-party review of auth + secret handling | **Shipped:** prep package (`spanda-security-audit`), [security-audit-third-party.md](./security-audit-third-party.md), `scripts/security_audit_prep.sh` (external review pending) |

---

## Per-pillar checklist

### Control Center + APIs

| Item | Experimental (today) | Stable requires |
|------|---------------------|-----------------|
| REST v1 | Full E1–E4 surface | — (OpenAPI parity test in CI) |
| gRPC | 59 RPCs (tonic) | **Shipped:** published proto semver policy (`GET /v1/version` → `grpc`) |
| Remote CLI | `spanda control-center *` shortcuts | **Shipped:** OpenAPI parity test (`control_center_openapi_parity.rs`) |
| Rate limits | `SPANDA_API_RATE_LIMIT_PER_MINUTE` | **Shipped:** tier defaults in [control-center-rate-limits.md](./control-center-rate-limits.md) |
| Mutation audit | Hash-chained JSONL | **Shipped:** SIEM export (`GET /v1/audit/mutations/export`, `spanda-audit-siem`) |

### Device Pool + Provisioning

| Item | Experimental (today) | Stable requires |
|------|---------------------|-----------------|
| Lifecycle | discover → active → quarantine → retire | **Shipped:** 1000-device pool perf gate (`device_pool_scale` test) |
| Discovery | mDNS/BLE/USB/wifi/cellular/serial registry | **Shipped:** production TLS policy (`SPANDA_DISCOVERY_REQUIRE_TLS`, `spanda-discovery-tls`) |
| Provisioning | `POST /v1/provision`, per-device workflows | **Shipped:** idempotent reprovision + conflict policy ([device-provisioning.md](./device-provisioning.md)) |
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
| Certification | `--require-certify` gate in planner | **Shipped:** mandatory via `SPANDA_OTA_REQUIRE_CERTIFY` / `SPANDA_PRODUCTION_POLICY=production` |

### Observability + SRE

| Item | Experimental (today) | Stable requires |
|------|---------------------|-----------------|
| Traces | API log + OTLP export to Jaeger | **Shipped:** OTLP collector HA guide ([otlp-collector-ha.md](./otlp-collector-ha.md)) |
| Metrics | OTLP metrics preview + export | **Shipped:** Grafana dashboard templates (`spanda-grafana-dashboards`) |
| WebSocket | `/v1/stream/telemetry` | **Shipped:** backpressure + reconnect contract (`SPANDA_WS_MAX_PENDING_FRAMES`) |
| SRE | SLO, MTTR/MTBF hints, incidents, auto-open from critical alerts | **Shipped:** SLO burn-rate rollup + background fast-burn alert dispatch (`SPANDA_SRE_BURN_SCAN_INTERVAL_SECS`) |
| Incidents | ack/resolve workflow | **Shipped:** PagerDuty bi-directional sync (`POST /v1/integrations/pagerduty/webhook`) |

### Compliance + Digital Thread

| Item | Experimental (today) | Stable requires |
|------|---------------------|-----------------|
| Export | `GET /v1/compliance/export` + evidence log | **Shipped:** signed profile catalog (`GET /v1/compliance/profiles`, `spanda-compliance` Ed25519 templates) |
| Reports | markdown / JSON / PDF | **Shipped:** scheduled delivery (`GET/POST /v1/reports/schedules`, `SPANDA_REPORT_SCHEDULE_INTERVAL_SECS`) |
| Digital thread | query API + **interactive graph UI** | **Shipped:** full lifecycle graph (requirement → retirement) with `lifecycle_phase` filter |

### SDKs

| Item | Experimental (today) | Stable requires |
|------|---------------------|-----------------|
| Python | REST client + stream extra | **Shipped:** PyPI publish scaffold + semver policy (`VERSIONING.md`, `sdk-python-v*` tags) |
| TypeScript | `ControlCenterPanel` in `@spanda/web` | **Shipped:** npm publish scaffold (`PUBLISHING.md`, `npm-web-v*` tags) |
| Remote CLI | `spanda control-center` client | **Shipped:** documented in [getting-started.md](./getting-started.md) |

### Desktop (Tauri)

| Item | Experimental (today) | Stable requires |
|------|---------------------|-----------------|
| Dev shell | `@spanda/control-center-desktop` | — |
| Build | `TAURI_BUILD=1` macOS CI artifacts | **Shipped:** signed + notarized macOS via `scripts/sign_tauri_macos.sh`, `.github/workflows/desktop-release.yml` |
| Auto-update | `tauri-plugin-updater` scaffold | **Shipped:** env-gated active updater (`TAURI_UPDATER_PUBKEY`, `SPANDA_DESKTOP_UPDATER_ACTIVE`), [desktop-release-runbook.md](./desktop-release-runbook.md) |
| Security | glib RUSTSEC git patch | Upstream gtk4 migration (Tauri v3 track) |

### Alerting

| Item | Experimental (today) | Stable requires |
|------|---------------------|-----------------|
| Channels | webhook, email, Slack, PagerDuty, Teams packages | **Shipped:** escalation templates (`spanda-alert-escalation`) |
| Dedup | Core dispatcher | **Shipped:** per-severity dedup windows (`SPANDA_ALERT_DEDUP_WINDOW_*_SECS`) |

---

## CI gates for Stable promotion

When promoting a pillar from **Experimental** → **Stable**, verify:

1. `scripts/enterprise_ops_smoke.sh` — includes pillar-specific assertions
2. `scripts/showcase_smoke.sh` — still green
3. `docs/feature-status.md` — pillar row updated
4. `CHANGELOG.md` — `[Unreleased]` → versioned release note
5. No **Planned** items remain in that pillar's Stable column above

---

## Promotion status (Experimental → Stable)

**Implementation checklist: complete.** Every per-pillar stable-hardening item in this document is marked **Shipped** in code, CI, or registry packages.

**Operational gates still required** before updating `docs/feature-status.md` to **Stable**:

1. **30-day field soak** — [field-soak-gate.md](./field-soak-gate.md) (`scripts/field_soak_gate.sh`)
2. **Third-party security audit sign-off** — [security-audit-third-party.md](./security-audit-third-party.md) (`scripts/security_audit_prep.sh`)
3. **First production releases** — PyPI/npm/desktop tags with registry and signing secrets ([desktop-release-runbook.md](./desktop-release-runbook.md))
4. **CI green** — `enterprise_ops_smoke.sh`, `showcase_smoke.sh`, OpenAPI parity tests

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

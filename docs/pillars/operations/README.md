# Pillar 6 — Operations Platform

[← Pillars index](../README.md) · [ROADMAP § Pillar 6](../../ROADMAP.md#pillar-6--operations-platform)

**Spanda Control Center** — observe, alert, report, provision, and operate fleets in production.

## Control Center

| Topic | Guide |
|-------|--------|
| Control Center | [control-center.md](../../control-center.md) |
| Rate limits | [control-center-rate-limits.md](../../control-center-rate-limits.md) |
| Desktop release | [desktop-release-runbook.md](../../desktop-release-runbook.md) |
| Enterprise operations (20 pillars) | [enterprise-operations-roadmap.md](../../enterprise-operations-roadmap.md) |
| Stable promotion checklist | [stable-hardening-enterprise-ops.md](../../stable-hardening-enterprise-ops.md) |

## Telemetry & observability

| Topic | Guide |
|-------|--------|
| Telemetry store | [telemetry-store.md](../../telemetry-store.md) |
| Observability / OTLP | `spanda-otel-collector` package |
| Readiness trends | [readiness-trends.md](../../readiness-trends.md) |
| Drift detection | [drift-detection.md](../../drift-detection.md) |
| Scorecards | [scorecards.md](../../scorecards.md) |

## SRE & reporting

| Topic | Guide |
|-------|--------|
| Chaos engineering | [chaos-engineering.md](../../chaos-engineering.md) |
| Resource estimation | [resource-estimation.md](../../resource-estimation.md) |
| Digital thread | Enterprise ops API lifecycle phases |

## Packages (alert channels)

| Package | Role |
|---------|------|
| `spanda-alert-slack` | Slack alerting |
| `spanda-alert-teams` | Teams alerting |
| `spanda-alert-pagerduty` | PagerDuty sync |
| `spanda-otel-collector` | OTLP backend |
| `spanda-grafana-dashboards` | Dashboard templates |

## Examples

| Path | Focus |
|------|--------|
| `spanda control-center serve` | Full ops UI |
| [examples/showcase/compliance/](../../../examples/showcase/compliance/) | Compliance export |
| [examples/end_to_end/incident_response.sd](../../../examples/end_to_end/incident_response.sd) | Incident workflow |

## Smoke gates

`scripts/enterprise_ops_smoke.sh` · `scripts/control_center_desktop_smoke.sh` · `scripts/maturity_smoke.sh` · `scripts/field_soak_gate.sh` · [scripts/gates/README.md](../../../scripts/gates/README.md)

## Stable promotion gates

| Gate | Doc |
|------|-----|
| 30-day field soak | [field-soak-gate.md](../../field-soak-gate.md) |
| Third-party audit | [security-audit-third-party.md](../../security-audit-third-party.md) |

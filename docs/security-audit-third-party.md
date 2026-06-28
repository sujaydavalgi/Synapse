# Third-party security audit — Control Center

Spanda enterprise operations promotion requires an independent review of authentication, authorization, and secret handling. This guide scopes the audit and links to automated prep artifacts.

## Scope

| Area | Evidence |
|------|----------|
| API authentication | `SPANDA_API_KEY`, Bearer tokens, tenant isolation (`SPANDA_TENANT_ID`) |
| RBAC | `GET /v1/rbac/matrix`, mutation gating on POST/PATCH |
| Secrets | `ManagedSecretVault`, no secret values in audit exports |
| Persistence | `SPANDA_CONTROL_CENTER_STATE_DIR`, encrypted snapshots |
| Mutation audit | Hash-chained JSONL + SIEM export |

## Auditor workflow

1. Run `./scripts/security_audit_prep.sh` and attach `.spanda/security-audit-prep.json`.
2. Review [security.md](./security.md) and [control-center.md](./control-center.md).
3. Exercise `scripts/enterprise_ops_smoke.sh` on a staging Control Center instance.
4. File findings against RBAC matrix rows and `/v1/audit/mutations` coverage.

## Out of scope

- VS Code extension marketplace supply chain
- Customer-managed OTLP collector infrastructure
- Physical device vendor firmware

## Human Interaction addendum (HRI)

For **Spatial Computing & Human-Robot Collaboration** promotion, also review:

| Area | Evidence |
|------|----------|
| Health opt-in | `GET /v1/human-health/policy`, `SPANDA_HUMAN_HEALTH_ENABLED` |
| Wearable telemetry | `SPANDA_LIVE_HEALTHKIT` gate, health mirror stripping on twins |
| AR session RBAC | `POST /v1/hri/sessions/{id}/annotate` requires Approve role |
| Mission approvals | `GET /v1/operator/mission/approvals`, persisted queue |

Run `./scripts/hri_security_audit_prep.sh` and attach `.spanda/hri-security-audit-prep.json` alongside the enterprise ops packet.

## Registry package

See [packages/registry/spanda-security-audit/README.md](../packages/registry/spanda-security-audit/README.md) for the audit checklist template.

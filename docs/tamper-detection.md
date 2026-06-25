# Tamper Detection

**Status:** Experimental (verify-time + trace runtime) · **Phase:** Verify, Operate, Recover · **Priority:** P3.1

Detect unauthorized modification, compromise, spoofing, tampering, or suspicious behavior in autonomous systems.

**Core question:** *Can this robot, device, fleet, mission, or provider still be trusted?*

## Threat types

Hardware tampering · Sensor spoofing · GPS spoofing · Firmware modification · Configuration tampering · Package tampering · Provider tampering · Unauthorized OTA · Network intrusion · Identity spoofing · Agent manipulation · Mission modification · Safety rule modification · Capability registry modification · Runtime injection · Replay attacks · Privilege escalation

## Framework types

| Type | Role |
|------|------|
| `TamperEvent` | Raw detection signal |
| `TamperAlert` | Operator-facing notification |
| `TamperEvidence` | Supporting data (hash, trace, telemetry) |
| `TamperSeverity` | Info · Low · Medium · High · Critical |
| `TamperPolicy` | Declarative response rules |
| `TamperDetectionResult` | Full analysis outcome |
| `TamperStatus` | Trusted · Suspicious · Tampered · Compromised · Unknown |

## CLI

```bash
spanda tamper-check rover.sd
spanda tamper-check rover.sd --json
spanda tamper-check mission.trace
spanda diagnose tamper mission.trace [--json]
spanda tamper-check --fleet fleet_tamper/manifest.json [--json]
spanda diagnose tamper --fleet fleet_tamper/manifest.json [--json]
spanda tamper-check --mesh-url http://127.0.0.1:8765 --fleet-name PatrolFleet [--json]
```

Verify-time `spanda tamper-check` composes threat modeling, safety audit, security analysis, and structural integrity signals. Runtime analysis accepts `.trace` files (or `--runtime`) for capability denials and tamper events. `spanda diagnose tamper <trace>` adds tamper source, affected components, impact, timeline, and recovery recommendations. `spanda tamper-check --fleet <manifest.json>` correlates tamper signals across fleet member traces (shared agents, simultaneous events, coordinated denials). **Live fleet mesh:** `POST /v1/fleet/tamper/ingest` on the mesh coordinator; `spanda tamper-check --mesh-url <url>` correlates ingested shards; runtime publishes shards when `SPANDA_FLEET_MESH_URL` is set.

**Tamper policies:** declare `tamper_policy` blocks with `on tamper severity Critical { ... }` or `on tamper signal capability_denied { ... }` branches. At runtime, matching signals dispatch recovery actions (`enter SafeMode`, `stop_all_actuators()`, `audit.record(...)`). **Critical** destructive actions (stop, kill switch, safe mode) require operator approval unless `SPANDA_OPERATOR_APPROVAL=1`. See `examples/showcase/tamper_policy/`.

## Integration

Readiness · Assurance · Diagnosis · Health · Security · Capability verification · Hardware verification · Trust score · Audit · Replay

## Crate

`spanda-tamper` — evidence collection, detection engine, trust scorer, response dispatcher.

See [integrity-verification.md](./integrity-verification.md) · [trust-framework.md](./trust-framework.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).

# Event Model

Common event schema for Spanda Platform subsystems.

**Parent:** [platform-architecture.md](./platform-architecture.md)

---

## Purpose

Every subsystem publishes events on a shared model. Events are the foundation for:

- **Telemetry** — persistence and export (`spanda-telemetry-store`)
- **Replay** — deterministic mission playback
- **Control Center** — live dashboards and alerts
- **Audit** — provenance and compliance (`spanda-audit`)
- **Notifications** — ops routing (`spanda-ops`)

---

## Namespace

```
spanda.events.<category>.<EventName>
```

JSON payloads use `type` (event name) and `timestamp` (RFC 3339) at minimum.

---

## Event categories

### Entity events

Published by entity registry mutations and config sync (`spanda-config`, `spanda-api`).

| Event | When | Key fields |
|-------|------|------------|
| `EntityCreated` | New entity registered | `entity_id`, `kind`, `source` |
| `EntityUpdated` | Entity record changed | `entity_id`, `fields` |
| `EntityDeleted` | Entity removed | `entity_id` |
| `EntityTagged` | Tag applied | `entity_id`, `tag` |
| `EntityRelated` | Relationship added | `from`, `to`, `relationship` |

See [entity-apis.md](./entity-apis.md).

---

### Health events

Published by runtime HAL, watchdogs, and entity health evaluation.

| Event | When | Key fields |
|-------|------|------------|
| `HealthChanged` | Health status transition | `entity_id`, `from`, `to`, `reason` |
| `HealthCheckFailed` | Scheduled check failed | `entity_id`, `check`, `detail` |
| `DegradedModeEntered` | System entered degraded mode | `entity_id`, `mode`, `trigger` |

See [entity-health.md](./entity-health.md), [runtime-health.md](./runtime-health.md).

---

### Readiness events

Published by `spanda-readiness` during evaluation.

| Event | When | Key fields |
|-------|------|------------|
| `ReadinessChanged` | Readiness score/status changed | `entity_id`, `score`, `gates` |
| `ReadinessGateFailed` | Gate blocked deployment | `entity_id`, `gate`, `evidence` |

See [readiness.md](./readiness.md).

---

### Mission events

Published by interpreter during program execution.

| Event | When | Key fields |
|-------|------|------------|
| `MissionStarted` | Mission execution began | `mission_id`, `program`, `entities` |
| `MissionCompleted` | Mission finished successfully | `mission_id`, `duration` |
| `MissionAborted` | Mission stopped early | `mission_id`, `reason` |
| `MissionPaused` | Mission paused | `mission_id`, `reason` |

Trace frames (`behavior_tick`, sensor samples) are recorded separately in mission trace format. See [replay.md](./replay.md).

---

### Recovery events

Published by runtime reliability and recovery policies.

| Event | When | Key fields |
|-------|------|------------|
| `RecoveryTriggered` | Recovery plan activated | `entity_id`, `plan`, `fault` |
| `RecoveryCompleted` | Recovery succeeded | `entity_id`, `plan`, `duration` |
| `RecoveryFailed` | Recovery exhausted | `entity_id`, `plan`, `error` |

See [reliability.md](./reliability.md), [recovery-policies.md](./recovery-policies.md).

---

### Trust events

Published by `spanda-trust` during evaluation.

| Event | When | Key fields |
|-------|------|------------|
| `TrustUpdated` | Trust score recalculated | `entity_id`, `score`, `dimensions` |
| `TrustGateFailed` | Trust below threshold | `entity_id`, `threshold`, `score` |

See [entity-trust.md](./entity-trust.md).

---

### Security events

Published by `spanda-security`, `spanda-tamper`, `spanda-spoofing`.

| Event | When | Key fields |
|-------|------|------------|
| `TamperDetected` | Integrity violation | `entity_id`, `component`, `evidence` |
| `SpoofingDetected` | Sensor/GPS spoof signal | `entity_id`, `sensor`, `confidence` |
| `SecretRotated` | Secret material rotated | `scope`, `id` |
| `AuthFailed` | Authentication rejected | `principal`, `reason` |

See [tamper-detection.md](./tamper-detection.md), [security-architecture.md](./security-architecture.md).

---

### Package events

Published by package install/verify workflows.

| Event | When | Key fields |
|-------|------|------------|
| `PackageInstalled` | Package installed | `name`, `version`, `provenance` |
| `PackageRemoved` | Package removed | `name`, `version` |
| `PackageVerified` | Package signature verified | `name`, `version`, `signer` |

See [how-packages-work.md](./how-packages-work.md).

---

### Telemetry events

Published by telemetry pipeline.

| Event | When | Key fields |
|-------|------|------------|
| `TraceFrameRecorded` | Mission trace frame appended | `mission_id`, `frame`, `seq` |
| `MetricEmitted` | Metric sample | `name`, `value`, `labels` |
| `LogEmitted` | Structured log line | `level`, `message`, `fields` |

See [telemetry-store.md](./telemetry-store.md).

---

### Fleet events

Published by fleet and OTA subsystems.

| Event | When | Key fields |
|-------|------|------------|
| `FleetMemberJoined` | Peer joined mesh | `fleet_id`, `member_id` |
| `FleetMemberLeft` | Peer left mesh | `fleet_id`, `member_id` |
| `OtaRolloutStarted` | OTA rollout began | `fleet_id`, `artifact`, `targets` |
| `OtaRolloutCompleted` | OTA rollout finished | `fleet_id`, `artifact`, `status` |

See [fleet-distributed.md](./fleet-distributed.md).

---

## Event envelope

Recommended JSON shape for REST, telemetry store, and audit. Rust implementations should use `spanda_audit::PlatformEvent` (`crates/spanda-audit/src/platform_event.rs`).

```json
{
  "type": "ReadinessChanged",
  "timestamp": "2026-06-28T12:00:00Z",
  "source": "spanda-readiness",
  "entity_id": "robot/warehouse-alpha",
  "payload": {
    "score": 0.92,
    "gates": []
  }
}
```

gRPC events mirror the same fields in proto messages where defined.

---

## Publishers and consumers

| Publisher | Primary events | Consumers |
|-----------|----------------|-----------|
| `spanda-config` / API | Entity* | Control Center, audit (`PlatformEvent` via `record_platform_event`) |
| `spanda-readiness` | ReadinessChanged, ReadinessGateFailed, HealthChanged, HealthCheckFailed | CLI, API, assurance, telemetry |
| `spanda-interpreter` | Mission*, Recovery*, DegradedModeEntered | Telemetry, replay |
| `spanda-package` / CLI | PackageInstalled, PackageVerified, PackageRemoved | Telemetry, audit |
| `spanda-providers` | PackageInstalled | Telemetry, audit (runtime bootstrap) |
| `spanda-trust` | TrustUpdated, TrustGateFailed | Explain, scorecard, telemetry |
| `spanda-tamper` | TamperDetected | Trust, ops alerts, telemetry |
| `spanda-spoofing` | SpoofingDetected | Trust, ops alerts, telemetry |
| `spanda-security` | AuthFailed, SecretRotated | Control Center, audit, telemetry |
| `spanda-fleet` | FleetMemberJoined, FleetMemberLeft | Control Center, telemetry |
| `spanda-ota` | OtaRolloutStarted, OtaRolloutCompleted | Control Center, telemetry |
| `spanda-telemetry-store` | All (persisted) | Replay, Grafana, audit |
| `spanda-ops` | Alert routing | PagerDuty, webhooks |

---

## Adding a new event

1. Choose category and name from existing taxonomy.
2. Add to `scripts/architecture-manifest.yaml` → `event_types`.
3. Document payload fields in this file.
4. Emit through telemetry store for persistence.
5. Add Control Center or CLI surfacing if user-visible.

Do not create parallel event schemas in blueprint examples — use platform event types.

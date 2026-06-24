# Persistent Telemetry Store

Append-only local storage for device metrics, sensor readings, and task heartbeats.

## What gets stored

| Event kind | Source | When |
|------------|--------|------|
| `device` | `iot.telemetry.publish`, IoT hub dispatch | Each published device metric |
| `sensor` | Robot `sensor.read()` / fusion inputs | Each sensor read during runtime |
| `topic` | Robot `publish` on declared topics | Recorded as device telemetry (`device_id=robot`, `metric=topic path`) |
| `heartbeat` | Task scheduler watchdog heartbeats | Latest index on every tick; history throttled (5s per task) |
| `device_heartbeat` | IoT device register, liveness metrics (`heartbeat`/`liveness`/`alive`/`ping`), fleet agent health | Latest index updated each touch; history throttled (5s per device) |
| `health` | Health status transitions (overall + per-check) | Runtime health polling |
| `session` | Run boundaries when `--persist-telemetry` is enabled | `start` at run begin; `end` at completion with optional mission trace path |
| `runtime_metrics` | End-of-run scheduler snapshot | Same payload as `--metrics-json` / run JSON `metrics` field |

Runtime scheduler metrics are also captured as a `runtime_metrics` event at the end of each persisted run. Mission traces (`--record`) can be linked via the `session` end event's `mission_trace_path`.

## Enable persistence

Per run:

```bash
spanda run rover.sd --persist-telemetry
spanda sim rover.sd --persist-telemetry
```

Or globally for the process:

```bash
export SPANDA_TELEMETRY_STORE=1
spanda run rover.sd
```

## Storage layout

| File | Purpose |
|------|---------|
| `.spanda/telemetry-store.jsonl` | Append-only event log (JSONL) |
| `.spanda/telemetry-heartbeats.json` | Latest heartbeat timestamp per task and device |

Override paths:

| Variable | Purpose |
|----------|---------|
| `SPANDA_TELEMETRY_STORE_PATH` | Event log file |
| `SPANDA_TELEMETRY_HEARTBEAT_PATH` | Heartbeat index file |
| `SPANDA_TELEMETRY_MAX_EVENTS` | Trim oldest events when the log exceeds this count |

Files live under `.spanda/` (gitignored) like deploy and fleet state.

## CLI

```bash
spanda telemetry list [--device <id>] [--sensor <id>] [--task <name>] [--session <id>] [--kind device|sensor|heartbeat|device_heartbeat|health|session|runtime_metrics] [--since <ms>] [--limit <n>] [--json]
spanda telemetry latest [--device <id> [--metric <name>] | --sensor <id> | --task <name>] [--json]
spanda telemetry heartbeats [--json]
spanda telemetry devices [--json]
spanda telemetry stats [--json]
spanda telemetry export [--out <file.jsonl>]
```

## Example workflow

```bash
spanda sim examples/end_to_end/validated_telemetry.sd --persist-telemetry --record
spanda telemetry list --kind session --json
spanda telemetry list --kind runtime_metrics --json
spanda telemetry stats
spanda telemetry list --kind sensor --json
spanda telemetry latest --device TelemetryRover --metric /telemetry
spanda telemetry heartbeats
spanda telemetry devices
```

Or run the scripted golden path: `./scripts/telemetry_store_golden_path.sh`

Device liveness is recorded when:
- `iot.device.register` succeeds
- `iot.telemetry.publish` uses metric `heartbeat`, `liveness`, `alive`, or `ping`
- `spanda fleet agent` health checks succeed (`protocol=fleet-agent`)
- Deploy OTA agents respond on `/v1/health` (`protocol=deploy-agent`)
- Robot `publish` on declared topics (metric = topic path, e.g. `/telemetry`)

## Crate

Implementation: `crates/spanda-telemetry-store` (`TelemetryEvent`, `PersistentTelemetryStore`).

TypeScript mirror: `src/telemetry-store.ts` records sensor reads, topic publishes, task/device heartbeats, and simplified health transitions when `persistTelemetry` is set on `run()` or `SPANDA_TELEMETRY_STORE=1`. `spanda telemetry` falls back to `src/telemetry-cli.ts` when the native CLI is unavailable.

See also [iot.md](./iot.md), [watchdogs.md](./watchdogs.md), [replay.md](./replay.md).

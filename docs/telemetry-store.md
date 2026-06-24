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

During an active run, every recorded event is tagged with `session_id` so `spanda telemetry list --session <id>` returns exactly that run's sensor reads, heartbeats, and publishes (legacy events without `session_id` still match by timestamp window).

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

### JSONL (default)

| File | Purpose |
|------|---------|
| `.spanda/telemetry-store.jsonl` | Append-only event log (JSONL) |
| `.spanda/telemetry-heartbeats.json` | Latest heartbeat timestamp per task and device |

### SQLite

Set `SPANDA_TELEMETRY_BACKEND=sqlite` to use an indexed SQLite database instead of JSONL:

| File | Purpose |
|------|---------|
| `.spanda/telemetry-store.db` | SQLite database (`telemetry_events` + `heartbeat_liveness` tables) |

Heartbeat liveness is stored in the `heartbeat_liveness` table (no JSON sidecar in SQLite mode).

On first open, an empty SQLite database automatically imports a sibling `telemetry-store.jsonl` (and `telemetry-heartbeats.json` beside it), then renames the JSONL file to `telemetry-store.jsonl.bak`.

Override paths:

| Variable | Purpose |
|----------|---------|
| `SPANDA_TELEMETRY_BACKEND` | `sqlite` for SQLite; omit for JSONL (default) |
| `SPANDA_TELEMETRY_STORE_PATH` | Event log file (`.jsonl`) or database (`.db`) |
| `SPANDA_TELEMETRY_HEARTBEAT_PATH` | Heartbeat index file (JSONL mode only) |
| `SPANDA_TELEMETRY_MAX_EVENTS` | Trim oldest events when the log exceeds this count |

Files live under `.spanda/` (gitignored) like deploy and fleet state.

## CLI

```bash
spanda telemetry list [--device <id>] [--sensor <id>] [--task <name>] [--session <id>] [--kind device|sensor|heartbeat|device_heartbeat|health|session|runtime_metrics] [--since <ms>] [--limit <n>] [--json]
spanda telemetry latest [--device <id> [--metric <name>] | --sensor <id> | --task <name>] [--json]
spanda telemetry heartbeats [--json]
spanda telemetry devices [--json]
spanda telemetry stats [--json]
spanda telemetry info [--json]
spanda telemetry sessions [--json]
spanda telemetry replay --session <id> [--from T+mm:ss] [--deterministic] [--playback] [--json]
spanda telemetry export [--out <file.jsonl>]
spanda telemetry prometheus [--out <file.prom>]
spanda telemetry otlp [--out <file.json>]
spanda telemetry push --endpoint <url> [--token <t>]
spanda telemetry serve [--bind <addr>] [--once]
```

## Example workflow

```bash
spanda sim examples/end_to_end/validated_telemetry.sd --persist-telemetry --record
spanda telemetry list --kind session --json
spanda telemetry sessions --json
spanda telemetry replay --session <id> --deterministic
spanda telemetry list --kind runtime_metrics --json
spanda telemetry stats
spanda telemetry list --kind sensor --json
spanda telemetry latest --device TelemetryRover --metric /telemetry
spanda telemetry heartbeats
spanda telemetry devices
```

```bash
export SPANDA_TELEMETRY_BACKEND=sqlite
spanda run rover.sd --persist-telemetry
spanda telemetry stats
```

Or run the scripted golden path: `./scripts/telemetry_store_golden_path.sh`

## Prometheus export

Scrape the store as Prometheus text exposition (no server required):

```bash
spanda telemetry prometheus
spanda telemetry prometheus --out metrics.prom
```

Exports event totals, heartbeat timestamps, latest `runtime_metrics` scheduler/task counters, numeric device metrics, and health scores. Point Prometheus at a file written by `--out`, or pipe stdout into your collector.

## OTLP export

Emit OTLP/JSON metrics for OpenTelemetry collectors:

```bash
spanda telemetry otlp
spanda telemetry otlp --out metrics.otlp.json
```

## Remote OTLP push

POST the current store snapshot to an OpenTelemetry HTTP collector:

```bash
export SPANDA_OTLP_ENDPOINT=http://collector:4318/v1/metrics
spanda telemetry push
spanda telemetry push --endpoint https://collector.example/v1/metrics --token "$OTEL_TOKEN"
```

The payload matches `spanda telemetry otlp` (OTLP/JSON `resourceMetrics` shape). Use with `spanda telemetry serve` for local collector testing.

## HTTP scrape server

Run a local metrics endpoint for Prometheus or OTLP polling:

```bash
spanda telemetry serve
spanda telemetry serve --bind 0.0.0.0:9090
```

| Path | Format |
|------|--------|
| `GET /metrics` | Prometheus text |
| `GET /otlp/v1/metrics` | OTLP/JSON |
| `GET /healthz` | Liveness (`ok`) |

Device liveness is recorded when:
- `iot.device.register` succeeds
- `iot.telemetry.publish` uses metric `heartbeat`, `liveness`, `alive`, or `ping`
- `spanda fleet agent` health checks succeed (`protocol=fleet-agent`)
- Deploy OTA agents respond on `/v1/health` (`protocol=deploy-agent`)
- Robot `publish` on declared topics (metric = topic path, e.g. `/telemetry`)

## Crate

Implementation: `crates/spanda-telemetry-store` (`TelemetryEvent`, `PersistentTelemetryStore`, in-memory `MemoryTelemetryStore` for WASM).

TypeScript mirror: `src/telemetry-store.ts` records sensor reads, topic publishes, task/device heartbeats, health transitions, session boundaries, and `runtime_metrics` (scheduler, task, execution, pipeline, watchdog, trigger, topic QoS deadline misses, and provider-call counters) when `persistTelemetry` is set on `run()` or `SPANDA_TELEMETRY_STORE=1`. The TS interpreter uses `src/runtime/trigger-registry.ts` for unified trigger dispatch (`every`, `when`, `while`, and legacy `on` handlers) with per-tick storm limits and missed timer-deadline metrics aligned with Rust. With `SPANDA_TELEMETRY_BACKEND=sqlite`, Node.js 22+ (`node:sqlite`) is required for the TS path; otherwise use the native Rust CLI. `spanda telemetry` tries the native Rust CLI first, then falls back to `src/telemetry-cli.ts` (including `serve`, `push`, `replay --session` inspect/playback/**deterministic verification**, and `info`). Mission traces written by the Rust CLI use snake_case JSON fields; the TS replay loader normalizes them for inspect and verify.

### WASM (browser)

`wasm_telemetry_clear`, `wasm_telemetry_append` (JSONL line), `wasm_telemetry_stats`, `wasm_telemetry_prometheus`, and `wasm_telemetry_otlp`. Successful `wasm_run` calls also append a `runtime_metrics` snapshot to the in-memory buffer. TypeScript wrappers live in `packages/web/src/spanda-wasm.ts` (`telemetryClear`, `telemetryAppend`, `telemetryStats`, `telemetryPrometheus`, `telemetryOtlp`). The WASM crate depends on `spanda-telemetry-store` with `default-features = false` (no SQLite/rusqlite).

See also [iot.md](./iot.md), [watchdogs.md](./watchdogs.md), [replay.md](./replay.md).

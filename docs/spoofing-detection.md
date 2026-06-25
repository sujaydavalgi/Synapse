# Spoofing Detection

**Status:** Experimental · **Phase:** Operate, Recover · **Priority:** P3.5

Detect GPS and sensor spoofing through plausibility checks, cross-sensor fusion coverage, and mission-trace analysis.

## CLI

```bash
spanda spoof-check examples/showcase/gps_spoofing/rover.sd
spanda spoof-check examples/showcase/gps_spoofing/spoof.trace --json
```

| Input | Analysis |
|-------|----------|
| `.sd` program | Static coverage — GPS sensor, state-estimator fusion, `on gps.spoofed` handler, health bounds, geofence |
| `.trace` mission file | Runtime plausibility — impossible GPS jumps, explicit spoof events, degraded fix quality |

## Detection examples

| Signal | Detection |
|--------|-----------|
| GPS impossible movement | Velocity/acceleration bounds between trace samples |
| GPS vs IMU conflict | IMU near-zero motion after recent GPS jump (trace payloads) |
| Sensor out of bounds | Declared `health_check` ranges (program coverage) |
| Explicit spoof event | `gps.spoofed` or `spoof` in trace events |

## Output

- `SpoofingAlert` — sensor, severity, confidence, evidence
- **Confidence score** (0–1) — never binary-only for Critical actions
- Program reports include **coverage score** (0–100) and per-check gaps

## Response

Integrates with `tamper_policy` and `recovery_policy` — default: alert + audit; Critical may require human approval before kill switch. Declare `on gps.spoofed { ... }` in mission programs to react at runtime when connectivity simulation or live agents emit spoof events.

## Implementation

**Crate:** `spanda-spoofing` — `analyze_spoofing_coverage`, `analyze_trace_spoofing`, `generate_program_spoof_check`, `generate_trace_spoof_check`.

**Package-backed extensions** — `spanda-gps` (`positioning.gps`) and `spanda-fusion` (`assurance.fusion`) export spoofing backend contracts; core heuristics live in `spanda-spoofing` and `spanda-connectivity` (`haversine_m`, `GpsSpoofing` fault simulation).

## Demo

`examples/showcase/gps_spoofing/` — program with fusion + spoof handler passes coverage; `spoof.trace` demonstrates impossible GPS jump and explicit spoof alert.

`scripts/spoof_smoke.sh` (wired into `scripts/showcase_smoke.sh`).

See [tamper-detection.md](./tamper-detection.md) · [state-estimation.md](./state-estimation.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).

# ADAS Solution Blueprint — Demo Plan

Demonstration plan for the Official ADAS & Autonomous Driving Solution Blueprint.

**Duration:** 15 minutes (full) · 5 minutes (executive summary) · **Command:** `spanda demo adas`

---

## Prerequisites

```bash
cd examples/solutions/adas
spanda install
export SPANDA_ROOT=/path/to/Spanda  # if not in repo root
```

---

## Executive summary (5 min)

| Step | Command | Talking point |
|------|---------|---------------|
| 1 | `spanda demo adas` | One command runs the full ADAS blueprint |
| 2 | Show `src/highway_drive.sd` | Capabilities, health, assurance — no core language changes |
| 3 | `spanda readiness src/highway_drive.sd --profile iso26262` | Go/no-go before ADAS activates |
| 4 | `spanda replay src/highway_drive.trace --deterministic` | Behavior-loop golden trace (20 ticks) |
| 5 | `spanda diagnose src/highway_drive.sd fixtures/camera_failure_recovery.trace` | Sensor failure → recovery narrative |

**Key message:** Spanda provides safety-first ADAS operations through composition, not core bloat.

---

## Full demo (15 min)

### 1. Blueprint overview (2 min)

- Open `examples/solutions/adas/README.md`
- Show device tree: `spanda device-tree inspect vehicle-001 --config spanda.toml`
- Explain nine vehicle applications, twelve ADAS functions

### 2. Verification & readiness (3 min)

```bash
spanda verify src/highway_drive.sd --profile iso26262 --capabilities --traceability --json
spanda readiness src/highway_drive.sd --profile iso26262 --json
spanda trace capabilities src/highway_drive.sd
```

Talking points: ISO 26262 template, capability traceability, sensor/calibration gates.

### 3. ADAS function walkthrough (4 min)

```bash
spanda check lane_keeping/lane_keeping.sd
spanda check adaptive_cruise/adaptive_cruise.sd
spanda check automatic_emergency_braking/aeb.sd
```

Show mission continuity:

```bash
spanda continuity sensor_failure_recovery/camera_failure.sd \
  --failed front_camera --trigger sensor_failed
```

### 4. Diagnosis & replay (3 min)

```bash
spanda replay src/highway_drive.trace --deterministic
spanda replay sim_record/lane_keep_task.trace --deterministic
spanda diagnose src/highway_drive.sd fixtures/camera_failure_recovery.trace
spanda explain driver_takeover/driver_takeover.sd fixtures/driver_takeover.trace
```

Talking points: explainable emergency braking, camera obstruction, driver takeover. `highway_drive.trace` captures `behavior_tick` frames; `fixtures/` hold narrative scenarios; `sim_record/` demonstrates `task` scheduler ticks.

### 5. Assurance & Control Center (3 min)

```bash
spanda compliance report src/highway_drive.sd --profile iso26262
spanda control-center serve --config spanda.toml --program src/highway_drive.sd
```

Open ADAS tab — vehicle health, sensor health, readiness, trust, alerts, OTA, replay viewer.

---

## Simulation scenarios (optional extension)

```bash
spanda sim src/highway_drive.sd
spanda sim sim_record/lane_keep_task.sd --record
spanda replay fixtures/aeb_activation.trace --playback
```

Scenarios: heavy rain, snow, fog, night, camera/radar/LiDAR failure, GPS spoofing, CAN failure, emergency vehicle.

---

## Security demo (optional extension)

```bash
spanda demo trust
```

Cross-reference: GPS spoofing, tamper policy, secure boot — see [adas-security.md](./adas-security.md).

---

## CI validation

```bash
./scripts/adas_smoke.sh
```

---

## Audience-specific paths

| Audience | Focus | Skip |
|----------|-------|------|
| OEM engineering | Device tree, capabilities, readiness | Control Center |
| Safety / compliance | ISO 26262, assurance, replay | Sim scenarios |
| Operations | Control Center ADAS tab, diagnosis | Language syntax |
| Executive | 5-min summary + architecture diagram | CLI details |

---

## Related

- [solutions/adas.md](./solutions/adas.md) — Architecture
- [killer-demo.md](./killer-demo.md) — Flagship rover demo
- [compliance-profiles.md](./compliance-profiles.md) — ISO 26262 profile

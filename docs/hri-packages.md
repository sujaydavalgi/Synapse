# HRI & Spatial Computing Packages

Optional packages for wearables, AR/VR/XR, and HRI modalities. **None ship in core** — install per deployment via `spanda install`.

**Related:** [how-packages-work.md](./how-packages-work.md) · [provider-interfaces.md](./provider-interfaces.md) · [wearables.md](./wearables.md) · [ar-vr-xr.md](./ar-vr-xr.md)

---

## Package catalog

| Package | Category | Provides | Requires | Status |
|---------|----------|----------|----------|--------|
| `spanda-vision-pro` | AR | `spatial_anchors`, `hand_tracking`, `robot_overlay` | visionOS SDK (host) | **Experimental** |
| `spanda-hololens` | AR | `spatial_anchors`, `hand_tracking`, `annotation` | Windows + HoloLens SDK | **Experimental** |
| `spanda-magic-leap` | AR | `spatial_anchors`, `robot_overlay` | Magic Leap SDK | **Experimental** |
| `spanda-arkit` | AR | `spatial_anchors`, `plane_detection`, `robot_overlay` | iOS ARKit | **Experimental** |
| `spanda-arcore` | AR | `spatial_anchors`, `plane_detection` | Android ARCore | **Experimental** |
| `spanda-smartwatch` | Wearable | `heart_rate`, `battery_level`, `connectivity_status` | HealthKit / Health Connect | **Experimental** |
| `spanda-industrial-wearables` | Wearable | `fall_detection`, `proximity_alert`, `gas_detection` | BLE industrial devices | **Experimental** |
| `spanda-bodycam` | Wearable | `video_stream`, `gps_location` | RTSP / USB body cameras | **Experimental** |
| `spanda-voice` | HRI | `voice_command` | Platform speech API | **Experimental** |
| `spanda-gesture` | HRI | `gesture_recognition`, `hand_tracking` | Camera / depth sensor | **Experimental** |
| `spanda-eye-tracking` | HRI | `eye_tracking`, `gaze_target` | AR headset eye sensors | **Experimental** |
| `spanda-openxr` | VR/XR | `vr_training`, `mission_replay_view` | OpenXR runtime | **Experimental** |

---

## Package manifest template

Each package follows the standard `spanda.toml` adapter pattern:

```toml
[package]
name = "spanda-hololens"
version = "0.1.0"
description = "Microsoft HoloLens spatial session provider for Spanda"
license = "Apache-2.0"

[adapter]
provides = ["spatial_anchors", "hand_tracking", "annotation", "robot_overlay"]
requires = ["telemetry_streaming"]
backend = "hololens"  # env: SPANDA_HOLOLENS_SESSION=1

[capabilities]
registers = [
  "spatial_anchors",
  "hand_tracking",
  "annotation",
]
```

---

## Provider traits (experimental)

| Trait | Responsibility |
|-------|----------------|
| `SpatialSessionProvider` | Start/stop AR/XR sessions, anchor sync (`spanda-runtime/src/providers/hri.rs`) |
| `WearableTelemetryProvider` | Heart rate, battery, connectivity (`spanda-runtime/src/providers/hri.rs`) |
| `HriInputProvider` | Voice, gesture, eye, pose events (`spanda-voice`, `spanda-gesture`, `spanda-eye-tracking`) |
| `OverlayProvider` | Subscribe to robot/mission/readiness overlay layers (`spanda-hololens` stub) |

Traits live in `spanda-runtime` provider dispatch; packages implement backends via registry stubs until vendor SDK bindings ship.

---

## Installation

```bash
spanda install spanda-hololens spanda-smartwatch spanda-voice
export SPANDA_HOLOLENS_SESSION=1
export SPANDA_LIVE_WEARABLE=1
spanda verify examples/solutions/spatial-computing/remote-maintenance/repair.sd --capabilities
```

---

## Registry status

H2 wearable and spatial packages ship in the curated registry (`registry/index.json`) as **experimental stubs** — vendor SDK bindings remain out of core. Enable live session/telemetry shims with:

```bash
export SPANDA_LIVE_WEARABLE=1
export SPANDA_LIVE_HEALTHKIT=1      # HealthKit-style fields on spanda-smartwatch
export SPANDA_LIVE_HOLOLENS=1       # HoloLens session metadata
export SPANDA_HOLOLENS_SESSION=1   # or SPANDA_SPATIAL_SESSION=1
export SPANDA_LIVE_VISION_PRO=1    # Vision Pro overlay enrichment
```

Existing registry packages used by blueprints today:

- `spanda-opencv` — camera streams for remote expert
- `spanda-mqtt` — spatial session sync transport
- `spanda-ble` — wearable discovery (complements `spanda-smartwatch` / `spanda-industrial-wearables`)
- `spanda-mission-continuity` — takeover and delegation

---

## Lean-core rule

| In core | In packages |
|---------|-------------|
| Provider trait definitions | Vendor SDK bindings |
| Capability registry entries | Device-specific telemetry parsers |
| Device tree schema | Session render loops |
| Readiness dimension hooks | ML models for gesture/voice |

Do **not** add AR/VR rendering, wearable SDKs, or speech models to workspace crates.

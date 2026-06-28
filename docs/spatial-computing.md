# Spatial Computing

Spatial computing in Spanda covers **shared workspaces**, **spatial anchors**, and **overlay workflows** that connect humans, wearables, AR/VR/XR devices, and robots — implemented through optional packages and solution blueprints, not core rendering engines.

**Related:** [ar-vr-xr.md](./ar-vr-xr.md) · [hri.md](./hri.md) · [solutions/spatial-computing.md](./solutions/spatial-computing.md)

---

## Concepts

| Concept | Description | Implementation |
|---------|-------------|----------------|
| **Spatial anchor** | Fixed point in physical or digital space | Package provider; synced via MQTT/ROS2/WebSocket |
| **Shared workspace** | Multi-user spatial session | Control Center session registry + package bridge |
| **Robot overlay** | Live robot pose, path, sensor frustum in AR | `spanda-arkit` / `spanda-hololens` + twin mirror |
| **Mission overlay** | Waypoints, tasks, approval state in AR | Mission continuity + readiness state export |
| **Sensor overlay** | Lidar/camera frustums, hazard zones | Device tree + assurance anomaly hints |
| **Readiness overlay** | Go/no-go indicators per asset | Readiness API → AR provider |
| **Digital twin overlay** | Twin state rendered in XR | Existing twin mirror + replay buffer |

---

## Spatial session lifecycle

```
1. Operator readiness passes
2. AR device registers session (package provider)
3. Spatial anchors published to shared workspace
4. Robot / mission state streamed to overlay
5. Context events (geofence, hazard) update overlay + robot
6. Session ends → audit + replay capture
```

Sessions are **not** managed by the core runtime — packages implement `SpatialSessionProvider` (planned provider trait) and register with the provider dispatch table.

---

## Shared workspaces

A shared workspace binds:

- One or more **humans** (operators, remote experts)
- **AR/VR/XR devices** per participant
- **Robots / drones** under collaborative mission
- **Control Center** as session authority

```toml
[[spatial_sessions]]
id = "maint-session-001"
workspace = "bay-3-repair"
participants = [
  { human_id = "tech-001", device_id = "hololens-001", role = "field" },
  { human_id = "expert-002", device_id = "desktop-cc", role = "remote_expert" },
]
robot_id = "arm-001"
mission_id = "guided-repair-17"
```

---

## Context awareness

Spatial computing composes with fleet geofencing and readiness alerts:

```
Operator enters hazard zone
  → Wearable alert (spanda-industrial-wearables)
  → AR warning overlay (spanda-hololens)
  → Robot slows (safety zone + mission update)
  → Mission state updated (continuity checkpoint)
  → Control Center incident (optional)
```

Supported context types:

- Restricted areas and danger zones
- Geofencing (GPS, UWB, beacon)
- Proximity alerts (robot ↔ human)
- Fatigue alerts (optional health)
- Emergency evacuation workflows

---

## Device tree

```toml
[[fleet.ar_devices]]
id = "hololens-001"
type = "ARHeadset"
provider = "spanda-hololens"
human_id = "tech-001"
capabilities = ["spatial_anchors", "hand_tracking", "robot_overlay", "annotation"]

[[fleet.vr_devices]]
id = "quest-training-001"
type = "VRHeadset"
provider = "spanda-openxr"
capabilities = ["vr_training", "mission_replay", "digital_twin_view"]
```

---

## Control Center

- **AR Session Viewer** — active sessions, participants, anchor list
- **Live Collaboration** — participant graph, mission allocation

See [control-center.md](./control-center.md#human-interaction-dashboard).

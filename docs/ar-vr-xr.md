# AR, VR & XR

Spanda supports **augmented**, **virtual**, and **mixed** reality workflows through optional packages — not built-in AR/VR engines. Reference integrations target common platforms; industrial and vendor-specific glasses ship as additional packages.

**Related:** [spatial-computing.md](./spatial-computing.md) · [hri-packages.md](./hri-packages.md) · [remote-expert.md](./remote-expert.md)

---

## AR (Augmented Reality)

### Reference integrations

| Platform | Package | Status |
|----------|---------|--------|
| Apple Vision Pro | `spanda-vision-pro` | **Planned** |
| Microsoft HoloLens | `spanda-hololens` | **Planned** |
| Magic Leap | `spanda-magic-leap` | **Planned** |
| Apple ARKit (iOS/iPad) | `spanda-arkit` | **Planned** |
| Google ARCore (Android) | `spanda-arcore` | **Planned** |
| Industrial AR glasses | `spanda-industrial-wearables` | **Planned** |

### Use cases

| Use case | Platform integration |
|----------|---------------------|
| Remote maintenance | HoloLens + live robot camera + remote expert |
| Warehouse picking | ARKit/ARCore + pick path overlay |
| Robot visualization | Spatial anchor + twin pose stream |
| Mission guidance | Mission continuity state → AR overlay |
| Hazard visualization | Geofence + assurance anomaly → AR warning |
| Inspection | Checklist overlay + replay capture |
| Training | AR guided steps + readiness gate |

---

## VR (Virtual Reality)

VR workflows reuse **simulation**, **replay**, and **digital twin** — no separate VR engine in core.

| Use case | Spanda integration |
|----------|-------------------|
| Operator training | `spanda sim` + VR package viewport |
| Simulation | Physics-lite sim + VR headset package |
| Mission replay | `spanda replay --playback` + VR session |
| Digital twin | Twin mirror fields → VR scene graph (package) |
| Remote robot operation | Teleoperation + VR stereo camera stream |
| Mission planning | Mission plan visualization in VR workspace |

Example: `examples/solutions/spatial-computing/vr-training/`

---

## XR (Mixed Reality)

Mixed reality combines live physical context with digital overlays:

| Overlay type | Data source |
|--------------|-------------|
| Live robot overlay | Robot pose, path, safety zone |
| Mission overlay | Mission plan, task state, approvals |
| Sensor overlay | Lidar point cloud summary, camera ROI |
| Health overlay | Optional wearable vitals (privacy-gated) |
| Readiness overlay | Per-asset go/no-go from readiness API |
| Recovery overlay | Active recovery plan, degraded mode |
| Digital twin overlay | Twin mirror + replay buffer |

XR sessions register as `XRSession` device-tree nodes with `capabilities` listing supported overlay types.

---

## Package architecture

```
.spanda program / mission state
        ↓
Control Center / MQTT / ROS2 topic
        ↓
spanda-arkit | spanda-hololens | spanda-vision-pro (package)
        ↓
Platform SDK (ARKit, OpenXR, visionOS)
        ↓
Headset / glasses display
```

Packages implement provider traits for:

- Session start/stop
- Spatial anchor CRUD
- Overlay layer subscribe
- Annotation publish (remote expert)
- Hand / eye / pose event stream (see [hri.md](./hri.md))

---

## Verification

AR/VR devices appear in capability traceability like any hardware:

```bash
spanda verify remote_maintenance.sd --capabilities --traceability
```

Programs may require AR capabilities:

```sd
requires_capability spatial_anchors {
  device_type = "ARHeadset"
}
```

---

## Privacy

Camera and health overlays require explicit deployment policy:

- `SPANDA_AR_CAMERA_ENABLED=1`
- `SPANDA_HUMAN_HEALTH_ENABLED=1`
- RBAC `ar:stream` and `health:read` permissions

Audit trail records session start, annotation, and override events.

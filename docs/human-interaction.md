# Human Interaction

Humans are **first-class entities** in Spanda's device registry and device tree — operators, technicians, supervisors, and other roles collaborate with robots, wearables, and spatial devices through configuration, capabilities, readiness, and Control Center workflows. No human-specific keywords are added to the `.sd` language.

**Related:** [human-interaction-spatial-computing-roadmap.md](./human-interaction-spatial-computing-roadmap.md) · [operator-capabilities.md](./operator-capabilities.md) · [human-readiness.md](./human-readiness.md) · [device-tree.md](./device-tree.md)

---

## Roles

| Role | Description | Typical assignment |
|------|-------------|-------------------|
| Operator | Runs robots and missions day-to-day | Fleet robot, active mission |
| Technician | Maintains and repairs equipment | Robot + AR glasses for remote assist |
| Supervisor | Approves missions and overrides | Fleet or site scope |
| Safety Officer | Enforces safety policy and evacuations | Site-wide hazard zones |
| Emergency Responder | SAR, hazmat, medical emergency | Incident mission |
| Healthcare Worker | Patient-facing care (Connected Healthcare) | Patient + medical wearables |
| Driver | Teleoperation or piloting | Robot or drone control |
| Patient | Health subject (optional health twin) | Care mission |
| Researcher | Observation and experiment control | Sim / replay sessions |
| Volunteer | Restricted operations under supervision | Limited robot scope |

Roles map to **RBAC permission tokens** and **operator capabilities** — see [operator-capabilities.md](./operator-capabilities.md).

---

## Human entity fields

Declare humans in `spanda.devices.toml` (or a dedicated `spanda.humans.toml` referenced from `[config]`):

```toml
[[humans]]
id = "operator-001"
role = "operator"
display_name = "Alex Chen"
capabilities = ["operate_robot", "approve_mission"]
certifications = [
  { id = "forklift-cert", expires = "2027-06-01" },
  { id = "drone-pilot", expires = "2026-12-15" },
]
assignments = { robot_id = "rover-001", mission_id = "warehouse-pick-42" }
availability = "available"
trust_level = "trusted"
location = { zone = "warehouse-a", lat = 37.7749, lon = -122.4194 }
permissions = ["mission:approve", "robot:operate"]
# health_status — optional; requires deployment opt-in (see human-readiness.md)
```

| Field | Description |
|-------|-------------|
| `id` | Unique identifier (device pool + digital thread) |
| `role` | Primary role token |
| `capabilities` | Operator capability tokens for verify/traceability |
| `certifications` | Expiring credentials checked by readiness |
| `assignments` | Active robot, mission, fleet, or AR session bindings |
| `health_status` | Optional summary from wearable packages (privacy-controlled) |
| `availability` | `available`, `busy`, `off_duty`, `unreachable` |
| `trust_level` | `unverified`, `verified`, `trusted`, `restricted` |
| `location` | Zone, GPS, or indoor beacon reference |
| `permissions` | RBAC tokens (extends Control Center RBAC v1) |

---

## Device tree hierarchy

Humans, wearables, and spatial devices sit alongside robots in the fleet tree:

```
fleet
├── robots / drones / IoT
├── wearables
├── ar_devices
├── vr_devices
├── humans
└── control_center (logical node)
```

Example:

```toml
[fleet]
id = "warehouse-fleet-a"

[[fleet.humans]]
id = "operator-001"
role = "operator"
capabilities = ["operate_robot", "forklift_operator"]

[[fleet.wearables]]
id = "watch-001"
type = "SmartWatch"
provider = "spanda-smartwatch"
human_id = "operator-001"
capabilities = ["heart_rate", "connectivity_status"]

[[fleet.ar_devices]]
id = "hololens-001"
type = "ARHeadset"
provider = "spanda-hololens"
human_id = "operator-001"
capabilities = ["spatial_anchors", "hand_tracking", "robot_overlay"]
```

See [wearables.md](./wearables.md) and [spatial-computing.md](./spatial-computing.md).

---

## Integration with platform spine

| Platform capability | Human interaction use |
|---------------------|----------------------|
| **Capability verification** | `spanda verify --capabilities` traces operator certs and role requirements |
| **Readiness** | Operator, team, and mission readiness gates before collaborative deploy |
| **Mission continuity** | Human takeover, robot takeover, delegation |
| **Trust** | Operator trust level affects approval workflows |
| **Digital twin** | Operator twin, team twin, training twin |
| **RBAC** | Role → permission mapping in Control Center |
| **Audit** | Human approval and override events in decision audit trail |

---

## Human digital twin

Operator, team, and training twins use the existing `twin` machinery — mirror fields track assignments, current task, mission state, equipment, safety status, optional health, and training history.

```toml
[[twins]]
id = "operator-twin-001"
entity_id = "operator-001"
entity_type = "human"
mirror = ["assignment", "current_task", "mission_state", "safety_status", "training_history"]
# mirror health only when SPANDA_HUMAN_HEALTH_ENABLED=1
```

Training twins link to VR replay sessions — see [ar-vr-xr.md](./ar-vr-xr.md).

---

## Collaborative stack

```
Human → Wearable → AR/VR/XR → Robot → Autonomous System → Control Center
```

Context-aware workflows (hazard zone entry → wearable alert → AR warning → robot slows) compose readiness, alerting, mission continuity, and package-backed spatial overlays — see [hri.md](./hri.md).

---

## Solution blueprint

Reference architecture: [solutions/spatial-computing.md](./solutions/spatial-computing.md) · `examples/solutions/spatial-computing/`

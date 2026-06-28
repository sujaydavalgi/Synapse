# Human-Robot Interaction (HRI)

HRI abstractions in Spanda are **provider-backed interaction modalities** — voice, gesture, hand tracking, eye tracking, pose tracking, spatial anchors, shared workspaces, approvals, teleoperation, and takeover — composed from existing mission continuity, safety, and capability frameworks.

**Related:** [human-interaction.md](./human-interaction.md) · [spatial-computing.md](./spatial-computing.md) · [mission-continuity.md](./mission-continuity.md)

---

## Interaction modalities

| Modality | Package | Provider capability |
|----------|---------|---------------------|
| Voice commands | `spanda-voice` | `voice_command` |
| Gesture recognition | `spanda-gesture` | `gesture_recognition` |
| Hand tracking | AR packages + `spanda-gesture` | `hand_tracking` |
| Eye tracking | `spanda-eye-tracking` | `eye_tracking` |
| Pose tracking | Wearable / AR packages | `pose_tracking` |
| Spatial anchors | `spanda-arkit`, `spanda-hololens`, … | `spatial_anchors` |

Packages register capabilities; programs consume them through existing `sensor` / provider dispatch — no new HRI language syntax.

---

## Approval workflows

Human approval reuses **mission continuity** and **recovery** approval paths:

| Action | Mechanism |
|--------|-----------|
| Mission start | `requires approval` + `approve_mission` capability |
| Recovery execution | Recovery planner + `approve_recovery` |
| Emergency override | `emergency_override` + audit trail |
| Config publish | Control Center approval queue (E1) |

```sd
continuity_policy warehouse_policy {
  on sensor_loss -> degraded_mode
  on human_takeover -> pause_mission
  requires approval for resume
}
```

Operator approval topics and env gates (`SPANDA_MISSION_APPROVAL_REQUIRED`) are **Stable** — see [self-healing.md](./self-healing.md).

---

## Teleoperation & takeover

| Pattern | Continuity mode | Description |
|---------|-----------------|-------------|
| Remote teleoperation | `human` takeover | Operator assumes direct control via VR/teleop package |
| Human takeover | `human` | Safety-gated pause + state snapshot |
| Robot takeover | `hot` / `shadow` | Autonomous system resumes after human releases |
| Collaborative task allocation | `delegate` | Subtasks assigned to human or robot agents |

```bash
spanda takeover --mode human --mission warehouse-pick-42
spanda delegate --to robot-002 --task inspect-aisle-7
```

Fleet mesh relays takeover when `SPANDA_FLEET_MESH_URL` is set.

---

## Collaborative missions

Collaborative missions bind heterogeneous participants:

```
Human → Robot → Drone → IoT → AI Agent → Fleet
```

Integration points:

| Concern | Platform feature |
|---------|------------------|
| Task allocation | `spanda swarm coordinate` + mission plan |
| Approval gates | Operator capabilities + RBAC |
| State sync | Mission continuity checkpoints |
| Shared awareness | Spatial workspace + twin mirror |
| Audit | Decision audit trail + replay |

Example: `examples/solutions/spatial-computing/operator-approval/`

---

## Shared workspaces

Multi-participant sessions bind humans, devices, and robots — see [spatial-computing.md](./spatial-computing.md#shared-workspaces).

Remote experts join via Control Center WebSocket + AR annotation stream — see [remote-expert.md](./remote-expert.md).

---

## Safety

All HRI commands pass through the existing safety engine:

- Voice/gesture intents → `ActionProposal` → `SafeAction`
- Emergency gestures map to `kill_switch` handlers
- Teleoperation respects `max_speed` and safety zones
- Takeover requires readiness + capability verification

---

## Example programs

| Example | Demonstrates |
|---------|--------------|
| `warehouse-ar/` | AR picking + robot coordination |
| `remote-maintenance/` | Telepresence + guided repair |
| `operator-approval/` | Mission approval workflow |
| `search-and-rescue-ar/` | Multi-human collaborative SAR |

Path: `examples/solutions/spatial-computing/`

# Feature examples — full coverage index

Runnable `.sd` programs mapped to Spanda capabilities. Use this when you need **one file per feature** or want to see what the language supports.

**Learning paths:** [Spanda 101](../../docs/spanda-101/README.md) · [Spanda for Dummies](../../docs/spanda-for-dummies/README.md) · [Tutorials index](../../docs/tutorials/README.md)

```bash
spanda check examples/features/dyn_trait_object.sd
spanda run examples/features/enum_payload.sd
```

---

## Language core

| Feature | Example |
|---------|---------|
| Minimal robot | `basics/01_minimal_robot.sd` |
| Physical units | `types/units.sd`, `basics/02_sensors_and_safety.sd` |
| `if` / `match` / loops | `basics/03_control_flow.sd` |
| `Result` / `Option` | `basics/04_result_and_option.sd` |
| Modules / `export` | `modules/path_planning.sd`, `modules/navigation.sd` |
| Traits + `impl` | `basics/05_traits_and_impl.sd` |
| `dyn Trait` objects | [`features/dyn_trait_object.sd`](./dyn_trait_object.sd) |
| Generic structs | [`features/generic_struct.sd`](./generic_struct.sd) |
| Generic module fn | [`features/generic_module_fn.sd`](./generic_module_fn.sd) |
| Struct literals | [`features/struct_literals.sd`](./struct_literals.sd) |
| Enum payloads | [`features/enum_payload.sd`](./enum_payload.sd) |
| `serialize` / `deserialize` | `basics/06_serialize_telemetry.sd` |
| `test` blocks | `basics/07_in_language_tests.sd` |
| `async` / `await` | `basics/08_async_await.sd` |
| Behavior `requires` / `ensures` | `basics/09_behavior_contracts.sd` |
| State machines + `enter` | `basics/10_state_machine.sd` |
| `observe` / fusion | `basics/11_observe_and_fusion.sd`, `types/fusion.sd` |

---

## Robotics & safety

| Feature | Example |
|---------|---------|
| Lidar avoidance | `lidar_avoidance.sd` |
| Safety zones | [`features/safety_zones.sd`](./safety_zones.sd), `patrol_with_zones.sd` |
| `max_speed` / `stop_if` | `basics/02_sensors_and_safety.sd` |
| Differential drive | `differential_drive.sd` |
| Arm / gripper | `robotic_arm_pick_place.sd`, `vision_pick_place.sd` |
| Drone | `drone_patrol.sd`, `drone_altitude_hold.sd` |
| Humanoid | `humanoid_assistant.sd` |
| Warehouse / logistics | `warehouse_logistics.sd`, `showcase/warehouse_robot.sd` |
| Outdoor / rover nav | `outdoor_navigation.sd`, `showcase/rover_navigation.sd` |

---

## AI & agents

| Feature | Example |
|---------|---------|
| AI safety gate | `showcase/rover_navigation.sd`, `end_to_end/safe_patrol/` |
| Compile-time AI rejection | `showcase/ai_safety_violation.sd` |
| LLM assistant (sensor-only) | `llm_robot_assistant.sd` |
| Vision pick-place | `vision_pick_place.sd` |
| Agent goals | `types/goals.sd` |
| Agent memory | `types/memory.sd` |
| Skills + capabilities | [`features/agent_capabilities.sd`](./agent_capabilities.sd) |
| Jetson / edge vision | `jetson_inspection.sd` |

---

## Tasks, triggers, concurrency

| Feature | Example |
|---------|---------|
| `task every Nms` | `types/multitask.sd`, `concurrency.sd` |
| Task `requires` clause | [`features/task_requires.sd`](./task_requires.sd) |
| Task `budget { }` | `concurrency.sd`, `realtime/resource_ceiling.sd` |
| Event `on` triggers | `integration/triggers_minimal.sd` |
| Full trigger catalog | `triggers_demo.sd` |
| `while` trigger | [`features/while_trigger.sd`](./while_trigger.sd) |
| `spawn` / `join` | [`features/join_and_spawn.sd`](./join_and_spawn.sd) |
| `parallel` blocks | [`features/parallel_block.sd`](./parallel_block.sd), `concurrency.sd` |
| Channels / `select` | `integration/concurrency_minimal.sd` |

---

## Real-time & reliability

| Feature | Example |
|---------|---------|
| `deadline` / `jitter` | `realtime/deadline_tasks.sd` |
| Latency `pipeline` | `realtime/latency_budget.sd` |
| Watchdogs | `realtime/watchdog.sd` |
| Operating `mode` | `realtime/degraded_mode.sd` |
| `recover from` | `realtime/recovery.sd` |
| Mission trace / replay | `end_to_end/replay_mission.sd`, `realtime/deterministic_replay.sd` |
| `mission { duration }` | [`features/mission_duration.sd`](./mission_duration.sd) |

---

## Communication & fleet

| Feature | Example |
|---------|---------|
| Pub/sub + QoS | `communication/basic_publish_subscribe.sd` |
| Topic QoS + deadline | [`features/topic_qos.sd`](./topic_qos.sd) |
| Services | `communication/service_calls.sd` |
| Actions | `communication/actions.sd` |
| Agent-to-agent | `communication/agent_to_agent.sd` |
| Multi-robot fleet | `communication/multi_robot_fleet.sd` |
| Digital twin sync | `communication/digital_twin_sync.sd` |
| Human interaction | `communication/human_interaction.sd` |
| ROS2 bridge surface | `ros2_bridge.sd` |

---

## Hardware, deploy, verify

| Feature | Example |
|---------|---------|
| `hardware` + `deploy` | `hardware/rover_deploy.sd`, `integration/verify_walkthrough.sd` |
| `requires_hardware` | [`features/requires_hardware.sd`](./requires_hardware.sd) |
| `requires_network` | [`features/requires_network.sd`](./requires_network.sd) |
| Full compatibility matrix | `hardware/full_compat.sd`, `showcase/hardware_compatibility.sd` |
| Fault simulation | [`features/simulate_fault.sd`](./simulate_fault.sd) |
| HAL / SoC (ESP32, Pi, STM32) | `esp32_sensors.sd`, `raspberry_pi_hal.sd`, `stm32_motor_control.sd` |
| Vendor sensors | `environmental_sensors.sd`, `ouster_os1.sd` |

---

## Digital twins

| Feature | Example |
|---------|---------|
| Twin mirror + replay | `digital_twin.sd`, `showcase/digital_twin_demo.sd` |
| Twin types | `types/digital_twin.sd` |

---

## Regex

| Feature | Example |
|---------|---------|
| Literals + `.matches()` | `regex/basic_regex.sd` |
| Command triggers | `regex/command_trigger.sd` |
| Log filters | `regex/log_filter.sd` |
| Subscribe filters | `regex/message_filter.sd` |
| `validate` rules | `regex/validation.sd` |

---

## Security & audit

| Feature | Example |
|---------|---------|
| Capabilities + signed topics | `std/security.sd` |
| Audit records | `std/audit_log.sd` |
| Device identity | `std/device_identity.sd` |
| Provenance | `std/provenance.sd` |

---

## FFI & ecosystem

| Feature | Example |
|---------|---------|
| Python `extern` | `ffi_python_extern.sd` |
| C++ `extern` | `ffi_cpp_extern.sd` |
| FFI bridge overview | `ffi_bridge.sd` |
| ROS2 adapter package | `packages/ros2_adapter_package/` |

---

## Packages & stdlib

| Feature | Example |
|---------|---------|
| Package manifest | `packages/basic_project/` |
| Path dependencies | `packages/local_dependency/` |
| Standard library | `std/*.sd` (robotics, spatial, time, ai, …) |
| Type snippets | `types/*.sd` |

---

## Showcase & end-to-end

| Feature | Example |
|---------|---------|
| Killer demo | `showcase/killer_demo.sd`, [killer-demo.md](../../docs/killer-demo.md) |
| Safe patrol package | `end_to_end/safe_patrol/` |
| Warehouse delivery package | [`end_to_end/warehouse_delivery/`](../../examples/end_to_end/warehouse_delivery/) |
| Pick-and-place cell | [`end_to_end/pick_and_place_cell/`](../../examples/end_to_end/pick_and_place_cell/) |
| Fleet coordination | [`end_to_end/fleet_coordination.sd`](../../examples/end_to_end/fleet_coordination.sd) |
| Incident response | [`end_to_end/incident_response.sd`](../../examples/end_to_end/incident_response.sd) |
| Real-time patrol | [`end_to_end/realtime_patrol.sd`](../../examples/end_to_end/realtime_patrol.sd) |
| Validated telemetry | [`end_to_end/validated_telemetry.sd`](../../examples/end_to_end/validated_telemetry.sd) |
| Concurrent inspection | [`end_to_end/concurrent_inspection.sd`](../../examples/end_to_end/concurrent_inspection.sd) |
| Replay mission | `end_to_end/replay_mission.sd` |

---

## CI golden fixtures

Key examples are guarded in `tests/golden/manifest.json`. Run:

```bash
npm test -- tests/golden/rust.test.ts
```

---

## Adding a feature example

1. Add `examples/features/<feature>.sd` with header comment + `spanda check` command
2. Add a row to this README
3. Optional: add to `tests/golden/manifest.json`
4. Update [feature-status.md](../../docs/feature-status.md) if stability tier changed

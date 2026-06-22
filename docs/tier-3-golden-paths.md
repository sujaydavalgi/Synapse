# Tier 3 golden paths (CI)

Runnable scripts and CI jobs that validate **experimental** Tier 3 capabilities and **v0.5 beta** P0 blockers. Phase 23 added platform wiring; Phase 24 extends v1.0-optional showcases; Phase 25 adds beta golden paths.

See [tier-3-experimental.md](./tier-3-experimental.md) for feature status and [tier-3-priority-plan.md](./tier-3-priority-plan.md) for P0–P4 ordering.

---

## v0.5 beta (P0)

| Capability | Script | CI job | Example |
|------------|--------|--------|---------|
| **Killer demo** | [killer_demo_golden_path.sh](../scripts/killer_demo_golden_path.sh) | `killer-demo-golden-path` | [killer_demo.sd](../examples/showcase/killer_demo.sd) |
| **Live AI (OpenAI mock/live)** | [live_ai_golden_path.sh](../scripts/live_ai_golden_path.sh) | `live-ai-golden-path` | [ffi_openai_live.sd](../examples/ffi_openai_live.sd) |
| **ROS2 rclpy `/cmd_vel`** | [ros2_golden_path.sh](../scripts/ros2_golden_path.sh) | `ros2-golden-path` | [ros2_cmd_vel_ping.sd](../examples/communication/ros2_cmd_vel_ping.sd) |
| **Hosted registry install** | [registry_golden_path.sh](../scripts/registry_golden_path.sh) | `registry-golden-path` | `spanda-openai`, `spanda-ros2` |
| **CI verify `--json` gate** | [ci_verify_golden_path.sh](../scripts/ci_verify_golden_path.sh) | `ci-verify-golden-path` | [hardware_compatibility.sd](../examples/showcase/hardware_compatibility.sd) |
| **Python PyO3 in-process FFI** | [python_native_golden_path.sh](../scripts/python_native_golden_path.sh) | `python-native-golden-path` | [ffi_python_extern.sd](../examples/ffi_python_extern.sd) |

---

## Tier 3 experimental

| Capability | Script | CI job | Example |
|------------|--------|--------|---------|
| **Platform packages** | — | `rust` + `check_all_examples.sh` | [autonomous_rover](../examples/showcase/autonomous_rover/) |
| **Robotics deploy/fleet** | [golden_path_deploy.sh](../examples/robotics/golden_path_deploy.sh) | `robotics-golden-path` | [fleet_field_trial.sd](../examples/robotics/fleet_field_trial.sd) |
| **Live MQTT** | [mqtt_golden_path.sh](../scripts/mqtt_golden_path.sh) | `mqtt-golden-path` | [mqtt_live.sd](../examples/communication/mqtt_live.sd) |
| **Twin cloud export** | [twin_cloud_golden_path.sh](../scripts/twin_cloud_golden_path.sh) | `twin-cloud-golden-path` | [twin_replay_golden.sd](../examples/communication/twin_replay_golden.sd) |
| **LLVM native codegen** | [llvm_golden_path.sh](../scripts/llvm_golden_path.sh) | `llvm-golden-path` | [hello_world.sd](../examples/hello_world.sd) |
| **LLVM aarch64 (Jetson/Pi slice)** | [llvm_embedded_golden_path.sh](../scripts/llvm_embedded_golden_path.sh) | `llvm-embedded-golden-path` | [hello_world.sd](../examples/hello_world.sd) |
| **C++ in-process FFI** | [cpp_native_golden_path.sh](../scripts/cpp_native_golden_path.sh) | `cpp-native-golden-path` | [ffi_cpp_extern.sd](../examples/ffi_cpp_extern.sd) |
| **Ledger scaffold** | [ledger_golden_path.sh](../scripts/ledger_golden_path.sh) | `ledger-golden-path` | [spanda-ledger](../packages/registry/spanda-ledger/) |
| **Self-host lexer** | [self_host_lexer_golden_path.sh](../scripts/self_host_lexer_golden_path.sh) | `self-host-lexer-golden-path` | [lexer_keywords.sd](../examples/self_host/lexer_keywords.sd) |
| **World model fusion** | [world_model_golden_path.sh](../scripts/world_model_golden_path.sh) | `world-model-golden-path` | [world_model_patrol.sd](../examples/showcase/world_model_patrol.sd) |

---

## Run locally

Build the CLI once:

```bash
cargo build -p spanda-cli --release
export PATH="$PWD/target/release:$PATH"
```

Then run any script from the repo root:

```bash
./scripts/killer_demo_golden_path.sh   # v0.5 beta flagship
./scripts/live_ai_golden_path.sh       # OpenAI bridge mock path
./scripts/registry_golden_path.sh      # file:// registry install
./scripts/ros2_golden_path.sh          # requires ROS 2 Humble + rclpy
./scripts/world_model_golden_path.sh   # no extra deps
./scripts/llvm_embedded_golden_path.sh # aarch64 triple when clang supports it
./examples/robotics/golden_path_deploy.sh
```

### Feature flags

| Feature | Build flag | Runtime env |
|---------|------------|-------------|
| Live MQTT | `--features live-mqtt` | `SPANDA_LIVE_MQTT=1` |
| Live transport bundle | `--features live-transport` | MQTT + WebSocket + DDS env vars |
| C++ in-process | `--features cpp-native` | unset `SPANDA_CPP_SUBPROCESS` |
| LLVM codegen | `--features llvm` (default) | `clang` on PATH |

---

## Platform integration docs

| Guide | Topic |
|-------|-------|
| [how-packages-work.md](./how-packages-work.md) | `spanda.toml`, lock, vendor, `spanda install` / `update` |
| [how-providers-work.md](./how-providers-work.md) | Provider registry, traits, bootstrap |
| [how-runtime-resolution-works.md](./how-runtime-resolution-works.md) | Import → dispatch → telemetry / replay |

---

## Related

- [lean-core-roadmap.md](./lean-core-roadmap.md) — Phase 23–24 (complete), Phase 25 (in progress)
- [mqtt-nav2-reference-architecture.md](./mqtt-nav2-reference-architecture.md) — ROS2 + Nav2 + MQTT stack
- [llvm-embedded-benchmark.md](./llvm-embedded-benchmark.md) — Jetson/Pi cross-compile slice
- [ffi-and-ecosystem.md](./ffi-and-ecosystem.md) — Python/C++/ROS2 bridges
- [replay.md](./replay.md) — mission trace and twin export
- [robotics-platform.md](./robotics-platform.md) — fleet orchestration and field trials

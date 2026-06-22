# ROS2 golden path (rclpy bridge)

Spanda does **not** replace ROS2. This document is the **single supported interop path** for v0.5 beta: publish and subscribe on live ROS2 topics via the **rclpy subprocess bridge**, without rewriting drivers or navigation stacks.

**Golden path choice:** rclpy bridge (`SPANDA_ROS2_LIVE=1`).  
**Alternative (advanced):** native rclrs cdylib — see [Advanced: rclrs native](#advanced-rclrs-native). Pick one path per deployment; do not mix in the same process without understanding the transport priority chain.

## What works today

| Capability | Status | How |
|------------|--------|-----|
| `publish` on `/cmd_vel`, `/scan`, etc. | **Live with rclpy** | `SPANDA_ROS2_LIVE=1` + sourced ROS2 distro |
| `subscribe` / topic read | **Live with rclpy** | Same bridge; mock when rclpy absent |
| Spanda `topic` / `service` / `action` syntax | Type-checked | [`examples/ros2_bridge.sd`](../examples/ros2_bridge.sd) |
| Replace existing ROS2 nodes | **Not required** | Bridge only — keep `nav2`, drivers, SLAM as-is |

Without `SPANDA_ROS2_LIVE`, transport calls log through the simulator (mock mode). This is fine for `spanda check`, `spanda verify`, and `spanda sim`.

## Prerequisites

- ROS 2 **Humble** (or compatible distro) installed and sourced
- Python 3 with **rclpy** available in the same environment
- Spanda CLI built: `cargo build -p spanda-cli --release`

```bash
# Ubuntu 22.04 example
source /opt/ros/humble/setup.bash
python3 -c "import rclpy; print('rclpy OK')"
```

## Step 1 — Type-check the bridge program

```bash
spanda check examples/ros2_bridge.sd
```

The program declares ROS2-shaped topics and services:

```spanda
topic cmd_vel: Velocity publish on "/cmd_vel";
sensor lidar: Lidar on "/scan";
```

## Step 2 — Enable live transport

```bash
export SPANDA_ROS2_LIVE=1
export PATH="$PWD/target/release:$PATH"
```

Optional overrides:

| Variable | Purpose |
|----------|---------|
| `SPANDA_PYTHON_BRIDGE` | Path to `scripts/spanda_python_bridge.py` (default: repo script) |
| `SPANDA_ROS2_DAEMON_SCRIPT` | Persistent daemon for `SPANDA_ROS2_RCLRS=1` mode |

## Step 3 — Manual validation on `/cmd_vel`

**Terminal A — ROS2 echo (validation):**

```bash
source /opt/ros/humble/setup.bash
ros2 topic echo /cmd_vel
```

**Terminal B — run Spanda with live bridge:**

```bash
source /opt/ros/humble/setup.bash
export SPANDA_ROS2_LIVE=1
spanda run examples/ros2_bridge.sd
```

**Expected:** `ros2 topic echo` receives messages when the program publishes `cmd_vel`. Without rclpy, the bridge returns `mode: mock` in internal logs and echo stays quiet — install `ros-humble-rclpy` and re-source.

### Validate `/scan` subscribe path

**Terminal A — publish a test scan topic:**

```bash
source /opt/ros/humble/setup.bash
ros2 topic pub /scan std_msgs/msg/String "data: test-scan" --once
```

**Terminal B:**

```bash
export SPANDA_ROS2_LIVE=1
spanda run examples/ros2_bridge.sd
```

The program reads lidar on `/scan` and republishes on `scan_out`. Use `ros2 topic list` and `ros2 topic echo /scan_out` to confirm.

## Transport priority (reference)

When multiple backends are enabled, `spanda-core` resolves in this order:

1. `SPANDA_ROS2_NATIVE=1` — `ros2` CLI subprocess (best-effort)
2. `SPANDA_ROS2_RCLRS=1` — persistent rclpy daemon (`scripts/spanda_ros2_daemon.py`)
3. `SPANDA_ROS2_LIVE=1` — per-call rclpy via `scripts/spanda_python_bridge.py`
4. Simulator / mock (default)

**For adoption, use only `SPANDA_ROS2_LIVE=1`** unless you need the persistent daemon.

## Bridge handlers (Python)

`scripts/spanda_python_bridge.py` registers:

- `ros2_publish(topic, data)`
- `ros2_subscribe(topic)`
- `ros2_service_call(service, service_type, request)`

Handlers use `std_msgs/String` for generic payloads today. Typed `geometry_msgs/Twist` for `/cmd_vel` is planned; the golden path validates topic **connectivity**, not message-type parity with Nav2.

## CI status

| Check | CI job | What it proves |
|-------|--------|----------------|
| rclpy live bridge | `ros2-golden-path` | Publish on `/cmd_vel` with `SPANDA_ROS2_LIVE=1` |
| rclrs native crate | `ros2-rclrs-native` | Native library loads under ROS2 Humble |

Run locally: `./scripts/ros2_golden_path.sh` (requires ROS 2 Humble and rclpy; skips gracefully when absent).

## Advanced: rclrs native

For in-process native transport (lower latency, no per-call subprocess):

```bash
source /opt/ros/humble/setup.bash
cargo build --manifest-path crates/spanda-ros2-rclrs-native/Cargo.toml --release
export SPANDA_ROS2_RCLRS_LIB="$PWD/target/release/libspanda_ros2_rclrs_native.so"
export SPANDA_ROS2_RCLRS=1
```

This path is tested in CI but requires building the native cdylib. Prefer the rclpy golden path for first integration.

## Adoption checklist

- [ ] `spanda check` passes on your `.sd` program with ROS2 topic declarations
- [ ] `SPANDA_ROS2_LIVE=1` set in the runtime environment
- [ ] ROS2 distro sourced in the same shell as `spanda run`
- [ ] `ros2 topic echo /cmd_vel` (or `/scan`) confirms live traffic
- [ ] Existing ROS2 nodes unchanged — Spanda is a coordinator, not a replacement

## Related

- [ffi-and-ecosystem.md](./ffi-and-ecosystem.md) — full FFI and bridge strategy
- [adoption-path.md](./adoption-path.md) — one-sprint Python + ROS2 integration
- [`examples/ros2_bridge.sd`](../examples/ros2_bridge.sd) — reference program
- [`examples/packages/ros2_adapter_package/`](../examples/packages/ros2_adapter_package/) — package layout sketch

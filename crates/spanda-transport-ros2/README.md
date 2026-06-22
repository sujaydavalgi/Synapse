# spanda-transport-ros2

ROS 2 transport backend for Spanda. Extracted from `spanda-core` as part of the lean-core architecture.

## Backends

1. **Native rclrs** — dynamically loads `libspanda_ros2_rclrs_native` when `SPANDA_ROS2_RCLRS=1`
2. **rclpy daemon** — persistent subprocess via `scripts/spanda_ros2_daemon.py`
3. **Live bridge** — optional `ros2` CLI (`SPANDA_ROS2_NATIVE=1`) or per-call Python bridge (`SPANDA_ROS2_LIVE=1`) via `live_bridge`

Spanda core retains a thin `RuntimeValue` compatibility shim in `transport_live.rs`.

## Related crates

- `spanda-ros2-rclrs-native` — build the native cdylib (requires ROS 2 Humble)
- `spanda-mqtt` — official package under `packages/registry/spanda-ros2`

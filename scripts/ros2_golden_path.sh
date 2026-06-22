#!/usr/bin/env bash
# Golden path for live ROS2 /cmd_vel publish via rclpy (SPANDA_ROS2_LIVE=1).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SPANDA="${SPANDA_BIN:-$ROOT/target/release/spanda}"
SOURCE="${ROOT}/examples/communication/ros2_cmd_vel_ping.sd"
TOPIC="/cmd_vel"
OUT="${TMPDIR:-/tmp}/spanda-ros2-golden.txt"

if [[ -z "${ROS_DISTRO:-}" ]]; then
  if [[ -f /opt/ros/humble/setup.bash ]]; then
    # shellcheck disable=SC1091
    source /opt/ros/humble/setup.bash
  else
    echo "ROS 2 not sourced and /opt/ros/humble/setup.bash missing; skip ROS2 golden path" >&2
    exit 0
  fi
fi

if ! python3 -c "import rclpy" 2>/dev/null; then
  echo "rclpy not available; skip ROS2 golden path" >&2
  exit 0
fi

echo "== build spanda-cli with live-transport =="
cargo build -p spanda-cli --release --features live-transport
SPANDA="${ROOT}/target/release/spanda"

echo "== check ros2 cmd_vel ping example =="
"${SPANDA}" check "${SOURCE}"

echo "== live publish on ${TOPIC} =="
export SPANDA_ROS2_LIVE=1
: >"${OUT}"
(
  timeout 12 ros2 topic echo "${TOPIC}" --once >"${OUT}"
) &
ECHO_PID=$!
sleep 2
"${SPANDA}" run "${SOURCE}"
wait "${ECHO_PID}" || true

if ! grep -qE 'data:|linear|published' "${OUT}"; then
  echo "ros2 topic echo did not receive /cmd_vel traffic:" >&2
  cat "${OUT}" >&2 || true
  exit 1
fi

echo "ROS2 golden path complete."

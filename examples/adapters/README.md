# Reference adapter bridges

Use these stub scripts with Spanda production bridge environment variables:

```bash
export SPANDA_NAV2_CMD="$PWD/examples/adapters/nav2_bridge.sh {goal}"
export SPANDA_SLAM_CMD="$PWD/examples/adapters/slam_bridge.sh {op}"

spanda run examples/robotics/nav2_bridge.sd
spanda run examples/robotics/slam_integration.sd
```

Replace the scripts with wrappers around your Nav2 action client or Cartographer/RTAB-Map CLI.

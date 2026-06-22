# spanda-runtime

Runtime kernel pieces extracted from `spanda-core` for the Phase 4 lean-core split:

- **scheduler** — sim vs wall-clock tick helpers
- **providers** — provider trait contracts, `ProviderRegistry`, lean `TransportConfig`
- **robot_state** — `RobotState`, `PoseState`, `VelocityState`
- **hal_config** — `HalMemberConfig` for HAL provider contracts
- **classification** — module ownership audit table
- **robotics** — `MissionRuntime`, `FleetRegistry`, zone registries
- **value** — `RuntimeValue`, `MotionCommand`, pose/velocity helpers
- **environment** — interpreter variable bindings
- **error** — `RuntimeError`
- **host** — `RuntimeHost` trait for domain hook extraction

Bootstrap wiring (`bootstrap`, `package_stubs`, `TransportAdapterProvider`) remains in `spanda-core` as compatibility shims; `CoreRuntimeHost` wires SLAM/navigation import detection.

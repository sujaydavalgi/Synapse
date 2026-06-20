# Reliability Model

Spanda combines compile-time contracts with runtime enforcement for autonomous systems.

## Components

| Feature | Syntax | Status |
|---------|--------|--------|
| Periodic tasks | `task … every Nms` | Stable |
| Deadlines | `deadline Nms` on tasks | Stable |
| Jitter bounds | `jitter <= Nms` | Stable |
| Resource budgets | `budget { cpu <= … }` | Stable |
| Pipelines | `pipeline name budget Nms { … }` | Stable |
| Watchdogs | `watchdog Name timeout Nms { … }` | Stable |
| Recovery | `recover from Error { … }` | Stable |
| Retry/fallback | `retry N times backoff Nms { … } fallback { … }` | Parsed; runtime policy partial |
| Operating modes | `mode name { … }` | Stable |

## Diagnostics

Invalid timing and regex patterns produce line/column diagnostics with suggested fixes, for example:

- `deadline (20ms) must be <= period (10ms)`
- `Invalid regex syntax: …`
- `Watchdog 'W' target task 'Missing' not found`

## Safety expectations

Recovery from `RuntimeError` should stop actuators or enter a degraded mode. The type checker warns when recovery blocks omit these actions.

See [realtime](realtime.md) and [degraded-modes](degraded-modes.md).

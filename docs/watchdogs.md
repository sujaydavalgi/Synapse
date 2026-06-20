# Watchdogs

Watchdog handlers run when a monitored task misses its heartbeat or exceeds a timeout.

```sd
watchdog SafetyMonitor timeout 50ms {
    enter safe_mode();
    stop_all_actuators();
}
```

Optional syntax monitors a named task:

```sd
task SafetyMonitor critical every 10ms { … }

watchdog SafetyMonitor timeout 50ms {
    stop_all_actuators();
}
```

The compiler verifies the target task exists. Runtime telemetry tracks `WatchdogMetrics.timeouts`.

See [realtime](realtime.md).

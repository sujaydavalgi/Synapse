# Real-Time Execution in Spanda

Spanda schedules periodic robot tasks on a **deterministic simulation clock**. Tasks declare timing contracts; the compiler validates them; the runtime records deadline misses and jitter.

## Deadline-aware tasks

```sd
task safety_monitor critical every 5ms deadline 2ms {
    check_emergency_stop();
}

task control_loop every 10ms deadline 8ms jitter <= 1ms {
    perceive();
    reason();
    act();
}
```

Rules enforced at compile time:

- Period must be positive
- `deadline <= period`
- `jitter <= deadline` slack
- `critical` tasks receive highest scheduler priority
- `critical isolated` tasks cannot be starved by lower-priority work

## Priority isolation

```sd
task SafetyMonitor critical isolated {
    check_safety();
}

task AIPlanner low {
    plan_route();
}
```

Isolated tasks are sorted ahead of non-isolated tasks at the same priority tier.

## CLI tracing

```bash
spanda run rover.sd --trace-realtime
spanda sim rover.sd --trace-realtime --record
spanda run rover.sd --metrics-json
```

`--trace-realtime` enables scheduler, task, trigger, and event traces. `--metrics-json` emits JSON metrics (same payload as `--json`).

See also: [reliability](reliability.md), [watchdogs](watchdogs.md), [replay](replay.md).

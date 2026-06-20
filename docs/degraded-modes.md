# Degraded Operating Modes

Declare operating modes and transition with `enter`:

```sd
mode normal {
    max_speed = 1.5 m/s;
}

mode degraded {
    max_speed = 0.3 m/s;
}

mode emergency {
    stop_all_actuators();
}
```

Hardware fault handlers can switch modes:

```sd
on hardware LidarFailure {
    use camera_depth;
    enter degraded_mode;
}
```

Runtime tracks the active mode (`normal` by default).

See [reliability](reliability.md).

# Regular Expressions

Spanda provides first-class regex literals and typed APIs.

## Literals and types

```sd
let pattern: Regex = /ERROR|WARN/;
let id_pattern: Regex = /^robot-[0-9]+$/;
```

Types: `Regex`, `Match`, `Capture`, `CaptureGroup`.

## String methods

| Method | Returns |
|--------|---------|
| `"text".matches(regex)` | `Bool` |
| `"text".find(regex)` | `String` or null |
| `"text".replace(regex, "x")` | `String` |
| `"text".split(regex)` | split list |
| `"text".capture(regex)` | `Capture` with named groups |

Flags: `/pattern/i`, `/pattern/m`, `/pattern/s`.

## Triggers

```sd
on log matches /EMERGENCY_STOP|MOTOR_FAULT/ {
    stop_all_actuators();
}

on message.text matches /help|stop|cancel/i {
    emergency_stop;
}
```

## Subscription filters

```sd
subscribe diagnostics
where message.text matches /WARN|ERROR/;
```

## Validation rules

```sd
validate RobotId {
    value matches /^robot-[0-9]{3}$/;
}
```

Invalid patterns are rejected at compile time with line/column diagnostics.

See examples in `examples/regex/`.

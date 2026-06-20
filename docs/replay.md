# Deterministic Replay

Record and replay simulation traces for regression and incident analysis.

## Recording

```bash
spanda sim rover.sd --record
```

Produces a JSON mission trace (`.trace`) with scheduler events, sensor snapshots, and twin frames.

## Replay

```bash
spanda replay mission.trace
spanda replay mission.trace --from T+00:30
spanda replay mission.trace --deterministic
```

Offsets accept milliseconds or `T+mm:ss` / `T+hh:mm:ss` forms.

Twin replay integrates with existing `twin { replay true; }` blocks.

See [realtime](realtime.md).

# Deterministic Replay

Record and replay simulation traces for regression and incident analysis.

## Recording

```bash
spanda sim rover.sd --record
```

Produces a JSON mission trace (`.trace`, version 2 when state snapshots are present) with scheduler events and embedded robot state (pose, velocity, e-stop, active mode) on each recorded frame.

## Replay modes

**Inspect frames** (default):

```bash
spanda replay mission.trace
spanda replay mission.trace --from T+00:30
```

**Deterministic verification** — re-run the source program and compare event sequences:

```bash
spanda replay mission.trace --deterministic
```

**Frame-by-frame playback** — apply recorded state snapshots without re-executing program logic:

```bash
spanda replay mission.trace --playback
```

Playback uses wall-clock pacing between frames by default. Offsets accept milliseconds or `T+mm:ss` / `T+hh:mm:ss` forms.

Twin replay integrates with existing `twin { replay true; }` blocks.

See [realtime](realtime.md).

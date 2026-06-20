# spanda-replay(1)

## NAME

replay — Replay or deterministically verify a recorded mission trace.

## SYNOPSIS

```
spanda replay <mission.trace> [--from T+mm:ss] [--deterministic] [--playback]
```

## DESCRIPTION

Replay or deterministically verify a recorded mission trace.

## OPTIONS

`--from` — start offset
`--deterministic` — verify reproducibility
`--playback` — frame-by-frame playback

## EXAMPLES

```bash
spanda replay mission.trace --deterministic
spanda replay mission.trace --playback --from T+00:30
```

## SEE ALSO

spanda-sim(1), spanda-run(1), [spanda(1)](./spanda.md), [spanda-reference.md](../spanda-reference.md)

# spanda-run(1)

## NAME

run — Execute a Spanda program on the interpreter backend.

## SYNOPSIS

```
spanda run [--json] [--verbose] [--trace-*] [--record] <file.sd>
```

## DESCRIPTION

Execute a Spanda program on the interpreter backend.

## OPTIONS

`--trace-scheduler`, `--trace-tasks`, `--trace-triggers`, `--trace-events` — scheduler telemetry
`--trace-realtime`, `--metrics-json` — realtime metrics
`--record` — write mission trace

## EXAMPLES

```bash
spanda run examples/rover.sd
spanda run robot.sd --trace-realtime --metrics-json
```

## SEE ALSO

spanda-sim(1), spanda-replay(1), [spanda(1)](./spanda.md), [spanda-reference.md](../spanda-reference.md)

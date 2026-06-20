# spanda-verify(1)

## NAME

verify — Verify hardware compatibility and safety constraints for a deploy target.

## SYNOPSIS

```
spanda verify [--json] [--target <profile>] [--all-targets] [--simulate] <file.sd>
```

## DESCRIPTION

Verify hardware compatibility and safety constraints for a deploy target.

## OPTIONS

`--target` — hardware profile name
`--all-targets` — compatibility matrix
`--simulate` — include simulator checks
`--json` — JSON report

## EXAMPLES

```bash
spanda verify robot.sd --target RoverV1
spanda verify robot.sd --all-targets --simulate
```

## SEE ALSO

spanda-check(1), spanda-run(1), [spanda(1)](./spanda.md), [spanda-reference.md](../spanda-reference.md)

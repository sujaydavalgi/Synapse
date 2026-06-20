# spanda-fleet(1)

## NAME

fleet — Run a multi-robot fleet program with peer communication.

## SYNOPSIS

```
spanda fleet run [--json] [--trace-*] <file.sd>
```

## DESCRIPTION

Run a multi-robot fleet program with peer communication.

## OPTIONS

Same trace flags as `spanda run`.

## EXAMPLES

```bash
spanda fleet run examples/communication/multi_robot_fleet.sd
```

## SEE ALSO

spanda-run(1), [spanda(1)](./spanda.md), [spanda-reference.md](../spanda-reference.md)

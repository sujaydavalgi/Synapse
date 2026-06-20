# spanda-debug(1)

## NAME

debug — Start an interactive debug session.

## SYNOPSIS

```
spanda debug [--break <line>] <file.sd>
```

## DESCRIPTION

Start an interactive debug session.

## OPTIONS

`--break` — initial breakpoint line

## EXAMPLES

```bash
spanda debug robot.sd --break 42
```

## SEE ALSO

spanda-run(1), [spanda(1)](./spanda.md), [spanda-reference.md](../spanda-reference.md)

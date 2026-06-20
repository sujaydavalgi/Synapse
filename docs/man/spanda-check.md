# spanda-check(1)

## NAME

check — Type-check and parse a Spanda program or project.

## SYNOPSIS

```
spanda check [--json] [<file.sd> | --project]
```

## DESCRIPTION

Type-check and parse a Spanda program or project.

## OPTIONS

`--json` — machine-readable diagnostics
`--project` — check all modules in the current project

## EXAMPLES

```bash
spanda check examples/rover.sd
spanda check --project
```

## SEE ALSO

spanda-verify(1), spanda-run(1), [spanda(1)](./spanda.md), [spanda-reference.md](../spanda-reference.md)

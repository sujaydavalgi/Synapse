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
`--readiness-json` — readiness + recovery + continuity policy hints
`--project` — check all modules in the current project

## EXAMPLES

```bash
spanda check examples/rover.sd
spanda check --project
```

## EXIT STATUS

0 on success; 1 on parse, type, or lint errors.

## FILES

`spanda.toml` — project manifest when using `--project`

## SEE ALSO

spanda-verify(1), spanda-run(1), spanda-continuity(1), [spanda(1)](./spanda.md), [spanda-reference.md](../spanda-reference.md)

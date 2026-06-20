# spanda(1)

## NAME

spanda — Spanda autonomous systems programming language toolchain

## SYNOPSIS

```
spanda <command> [options] [arguments]
```

## DESCRIPTION

The Spanda CLI compiles, checks, verifies, runs, simulates, and documents `.sd` programs.

## COMMANDS

- **check** — Type-check and parse a Spanda program or project.
- **verify** — Verify hardware compatibility and safety constraints for a deploy target.
- **run** — Execute a Spanda program on the interpreter backend.
- **sim** — Run a program in the built-in simulator with optional trace recording.
- **replay** — Replay or deterministically verify a recorded mission trace.
- **fleet** — Run a multi-robot fleet program with peer communication.
- **fmt** — Format Spanda source to canonical style.
- **lint** — Run linter rules beyond parse/type checking.
- **doc** — Generate JavaDoc-style API docs for a single `.sd` module.
- **reference** — Emit the full Spanda language reference (this document).
- **codegen** — Generate deployable artifacts from a Spanda program.
- **debug** — Start an interactive debug session.

Package commands: `init`, `build`, `test`, `add`, `remove`, `install`, `publish`, `registry search`, `registry info`.

## SEE ALSO

[spanda-reference.md](../spanda-reference.md), [getting-started.md](../getting-started.md)

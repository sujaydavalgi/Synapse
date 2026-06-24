# spanda(1)

## NAME

spanda — Spanda autonomous systems platform toolchain

## SYNOPSIS

```
spanda <command> [options] [arguments]
```

## DESCRIPTION

The Spanda CLI drives the autonomous systems platform: check, verify, simulate, replay, fleet, and document `.sd` programs.

## COMMANDS

- **check** — Type-check and parse a Spanda program or project.
- **verify** — Verify hardware compatibility and safety constraints for a deploy target.
- **run** — Execute a Spanda program on the interpreter backend.
- **sim** — Run a program in the built-in simulator with optional trace recording.
- **replay** — Replay or deterministically verify a recorded mission trace.
- **test** — Run in-language `test` blocks and package test suites for a Spanda project.
- **readiness** — Evaluate operational readiness: health, safety, fleet, and deployment gates.
- **assure** — Run assurance workflows: anomaly coverage, prognostics, and assurance cases.
- **diagnose** — Diagnose failures from static analysis and optional mission traces.
- **heal** — Execute self-healing and recovery policies declared in the program.
- **continuity** — Mission continuity, takeover, delegation, and succession planning.
- **fleet** — Run a multi-robot fleet program with peer communication.
- **package** — Manage Spanda packages: manifests, dependencies, builds, and registry operations.
- **trace** — Record scheduler, task, trigger, and event traces from a program run.
- **security** — Validate security policies, identities, and audit configuration.
- **fmt** — Format Spanda source to canonical style.
- **lint** — Run linter rules beyond parse/type checking.
- **doc** — Generate JavaDoc-style API docs for `.sd` modules (markdown or HTML).
- **man** — Display man-page style documentation for Spanda CLI commands.
- **reference** — Emit the full Spanda language reference and optional man pages.
- **codegen** — Generate deployable artifacts from a Spanda program.
- **debug** — Start an interactive debug session.

Package commands: `init`, `build`, `test`, `add`, `remove`, `install`, `publish`, `registry search`, `registry info`.

## SEE ALSO

[spanda-reference.md](../spanda-reference.md), [getting-started.md](../getting-started.md)

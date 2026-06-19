# Migration Guide

## From legacy `.syn` files

The `.syn` extension is no longer supported. Rename files to `.sd` and update tooling paths.

## From behavior-only programs

Existing programs continue to work unchanged. New constructs are additive:

| Old pattern | New pattern |
|-------------|-------------|
| `behavior loop()` with `loop every` inside | `task name every 20ms { ... }` |
| Ad-hoc state variables | `enum` + `match` |
| Agent tools only | Add `skill` and `can [ ... ]` for capability clarity |
| Implicit modules | Optional `module name;` + `import path;` |
| Deploy without validation | `hardware` profile + `deploy Robot to Target;` + `spanda verify` |
| Runtime-only safety checks | Add `verify { }` for post-run behavioral assertions |

## Import paths

Library imports (`import bosch.bno055;`) still resolve to sensor drivers.

Code module imports (`import sensors.lidar;`) resolve against the module registry in `foundations.rs`. Add new paths there when splitting source files.

## Breaking changes in v0.2

- New keywords: `module`, `struct`, `enum`, `trait`, `match`, `fn`, `state`, `transition`, `task`, `skill`, `event`, `twin`, `can`, `requires`, `ensures`, `invariant`, `verify`, `observe`, `remember`
- Hardware keywords: `hardware`, `deploy`, `requires_hardware`, `requires_network`, `simulate_compatibility`, `budget`, `mission`, `fault`, `sensors`, `actuators`, `network`, `bandwidth`, `latency`, `timing`, `min_period`, `duration`
- `match` expression arms use `=>` and require `;` after single-statement bodies
- `plan`, `state`, `goal`, `mission`, and other keywords may appear as identifiers in binding positions (parser allows keyword-as-name where unambiguous)
- `to` is a keyword for `deploy Robot to Target` but remains valid as a named argument (`trajectory(from: a, to: b)`)
- `battery` is a keyword in hardware profiles; HAL bindings named `battery` still parse (`adc battery on channel 0`)

## Dual backend note

The **Rust CLI** is canonical for new syntax including hardware verification:

```bash
npm run build:rust
spanda verify program.sd --target RoverV1
```

TypeScript `verifyViaCli()` delegates to the Rust CLI. The TS parser mirror does not yet parse all hardware constructs locally; use Rust for `check`/`verify` on programs with `hardware`, `deploy`, or `requires_*` blocks.

## Adding hardware verification to existing robots

1. Pick or declare a `hardware` profile (or use builtins: `RoverV1`, `ESP32`, …).
2. Add `deploy YourRobot to ProfileName;`.
3. Run `spanda verify your_program.sd`.
4. Optionally add `requires_hardware { }` for minimum platform requirements and `mission { duration: N h; }` for power checks.

See [hardware-compatibility.md](./hardware-compatibility.md).

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

## Import paths

Library imports (`import bosch.bno055;`) still resolve to sensor drivers.

Code module imports (`import sensors.lidar;`) resolve against the module registry in `foundations.rs`. Add new paths there when splitting source files.

## Breaking changes in v0.2

- New keywords: `module`, `struct`, `enum`, `trait`, `match`, `fn`, `state`, `transition`, `task`, `skill`, `event`, `twin`, `can`, `requires`, `ensures`, `invariant`
- `match` expression arms use `=>` and require `;` after single-statement bodies
- `plan`, `state`, `goal`, and other keywords may appear as identifiers in label positions

## Dual backend note

Until the TypeScript mirror catches up, use the **Rust CLI** (`cargo run -p spanda-cli` or `npm run spanda:native`) for programs using new syntax.

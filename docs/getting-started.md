# Getting Started with Spanda

Build and run your first autonomous robot program in under 10 minutes.

---

## Installation

Install prebuilt packages for Linux, macOS, and Windows from [GitHub Releases](https://github.com/Davalgi/Spanda/releases), or build from source below.

See [installation.md](./installation.md) for shell/MSI/PowerShell installers and platform archives.

### Build from source

#### 1. Clone the repository

```bash
git clone https://github.com/Davalgi/Spanda.git
cd Spanda
```

#### 2. Install dependencies

```bash
npm install
```

#### 3. Build the native CLI

```bash
npm run build:rust
```

This produces `target/release/spanda`. Verify:

```bash
./target/release/spanda check examples/hello_world.sd
```

Optional: add `target/release` to your `PATH` so you can run `spanda` directly.

---

## Your first project

### Initialize a project

```bash
spanda init my_rover
cd my_rover
```

This creates a `spanda.toml` manifest and `src/main.sd` starter file.

### Edit `src/main.sd`

```spanda
robot MyRover {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  safety {
    max_speed = 1.0 m/s;
    stop_if lidar.nearest_distance < 0.5 m;
  }

  behavior patrol() {
    loop every 100ms {
      wheels.drive(linear: 0.3 m/s, angular: 0.0 rad/s);
    }
  }
}
```

---

## Learn by example (basics → end-to-end)

Progressive tutorials live under [`examples/basics/`](examples/basics/README.md):

```bash
spanda check examples/basics/01_minimal_robot.sd
spanda run examples/basics/02_sensors_and_safety.sd
spanda test examples/basics/07_in_language_tests.sd
```

| Tier | Directory | Topics |
|------|-----------|--------|
| Basics | `examples/basics/` | Robot syntax, safety, control flow, Result/Option, traits, async, contracts |
| Integration | `examples/integration/` | Triggers, concurrency, hardware verify |
| End-to-end | `examples/end_to_end/` | Full patrol package, record/replay mission |

After the ladder, try [`examples/showcase/killer_demo.sd`](examples/showcase/killer_demo.sd) and [killer-demo.md](./killer-demo.md).

---

## Core commands

### Type-check

```bash
spanda check src/main.sd
spanda check --project          # check entire project
spanda check src/main.sd --json # machine-readable diagnostics
```

### Run simulation

```bash
spanda run src/main.sd
spanda sim src/main.sd          # verbose simulation output
```

### Hardware verification

```bash
spanda verify src/main.sd
spanda verify src/main.sd --target RoverV1
spanda verify src/main.sd --all-targets --json
```

Add a deploy target to your program first:

```spanda
hardware RoverV1 {
  memory: 4 GB;
  sensors [ Lidar ];
  actuators [ DifferentialDrive ];
}

deploy MyRover to RoverV1;
```

### Run tests

```bash
spanda test
```

Add test blocks in your `.sd` files:

```spanda
test "safety stops near obstacles" {
  // test body
}
```

### Format and lint

```bash
spanda fmt src/main.sd
spanda lint src/main.sd
```

### Trace runtime behavior

Debug scheduler, task, and trigger execution with trace flags:

```bash
spanda run src/main.sd --trace-scheduler --trace-tasks
spanda run src/main.sd --trace-triggers --trace-events
spanda sim src/main.sd --replay --trace-scheduler
```

Trace output appears in the runtime log stream with prefixes like `trace-scheduler:`, `trace-task:`, and `trace-trigger:`.

### Record and replay mission traces

Record a simulation run, then inspect, verify, or play it back:

```bash
spanda sim examples/realtime/deterministic_replay.sd --record
spanda replay mission.trace
spanda replay mission.trace --deterministic   # re-run source and verify frame parity
spanda replay mission.trace --playback --from T+00:30
```

Real-time telemetry and wall-clock scheduling:

```bash
spanda run examples/realtime/deadline_tasks.sd --trace-realtime --metrics-json
spanda sim examples/realtime/latency_budget.sd --wall-clock
```

See [realtime.md](./realtime.md), [replay.md](./replay.md), and [reliability.md](./reliability.md).

---

## Try the showcase examples

The `examples/showcase/` directory contains curated demos for v0.1.0-alpha:

```bash
spanda check examples/showcase/rover_navigation.sd
spanda run examples/showcase/rover_navigation.sd
spanda verify examples/showcase/hardware_compatibility.sd --json
spanda check examples/showcase/ai_safety_violation.sd   # expect compile error
```

| Example | Command to try |
|---------|----------------|
| `rover_navigation.sd` | `spanda run examples/showcase/rover_navigation.sd` |
| `warehouse_robot.sd` | `spanda run examples/showcase/warehouse_robot.sd` |
| `hardware_compatibility.sd` | `spanda verify examples/showcase/hardware_compatibility.sd` |
| `communication_demo.sd` | `spanda run examples/showcase/communication_demo.sd` |
| `digital_twin_demo.sd` | `spanda run examples/showcase/digital_twin_demo.sd` |
| `triggers_demo.sd` | `spanda run examples/triggers_demo.sd --trace-triggers` |

### Triggers and concurrency

Reactive programs use unified triggers — see [triggers.md](./triggers.md):

```bash
spanda run examples/triggers_demo.sd --trace-triggers
spanda run examples/concurrency.sd --trace-scheduler --trace-tasks
spanda fleet run examples/communication/multi_robot_fleet.sd
```

### Real-time, reliability, and regex

Deadline-aware tasks, watchdogs, and degraded modes:

```bash
spanda check examples/realtime/deadline_tasks.sd
spanda run examples/realtime/watchdog.sd --trace-realtime
spanda run examples/realtime/degraded_mode.sd
```

Regex triggers and validation:

```bash
spanda check examples/regex/basic_regex.sd
spanda run examples/regex/command_trigger.sd --trace-triggers
```

### Killer demo (5 minutes)

Flagship safety walkthrough — compile-time AI gate, hardware verify, and sim:

```bash
spanda check examples/showcase/killer_demo.sd
spanda verify examples/showcase/killer_demo.sd --json
spanda sim examples/showcase/killer_demo.sd
```

Full script: [killer-demo.md](./killer-demo.md)

---

## Build your first robot (step by step)

### Step 1 — Minimal robot

Start with `examples/hello_world.sd` or `examples/rover.sd`:

```bash
spanda check examples/rover.sd
spanda run examples/rover.sd
```

### Step 2 — Add safety

Study `examples/lidar_avoidance.sd` for `stop_if` and emergency stop patterns.

### Step 3 — Add AI with safety gate

Study `examples/showcase/rover_navigation.sd`:

```bash
spanda run examples/showcase/rover_navigation.sd
```

The agent proposes actions; `safety.validate()` gates them before `wheels.execute()`.

### Step 4 — Verify hardware fit

Study `examples/showcase/hardware_compatibility.sd`:

```bash
spanda verify examples/showcase/hardware_compatibility.sd --json
```

### Step 5 — Package your project

```bash
spanda init warehouse_bot
cd warehouse_bot
# edit src/main.sd
spanda build
spanda test
```

---

## TypeScript fallback (no Rust build)

If you have not built the native CLI, the npm wrapper falls back to the TypeScript interpreter:

```bash
npm run spanda -- check examples/rover.sd
npm run spanda -- run examples/rover.sd
```

Hardware verification (`spanda verify`) requires the native Rust CLI.

---

## Web playground

```bash
npm run build:wasm
npm run web:dev
```

Open http://localhost:5173 to type-check and run programs in the browser.

---

## Language Server (LSP)

Build the LSP for editor integration:

```bash
npm run build:rust
npm run build --workspace=@spanda/lsp
```

Spanda now includes a first-party VS Code extension scaffold at `editor/vscode` that wires to `spanda-lsp`.

### Editor support roadmap (v0.1.0-alpha)

| Capability | Status |
|------------|--------|
| Syntax highlighting | Via TextMate grammar (community); LSP semantic tokens planned |
| Autocomplete | LSP — symbols, comm keywords, transports |
| Diagnostics | LSP — type-check + hardware verify (requires native CLI) |
| Go to definition | LSP |
| Format on save | LSP `textDocument/formatting` |
| VS Code extension package | **Experimental** — build from `editor/vscode` |

To configure VS Code manually, add to `.vscode/settings.json`:

```json
{
  "spanda.languageServerPath": "${workspaceFolder}/packages/lsp/dist/server.js"
}
```

To build the extension locally:

```bash
cd editor/vscode
npm install
npm run build
```

Then run it in VS Code Extension Development Host. The extension setting `spanda.languageServerPath` can point to your built `packages/lsp/dist/server.js`.

To package and install a local VSIX:

```bash
cd editor/vscode
npm install
npm run package
code --install-extension spanda-vscode-0.1.0.vsix
```

---

## Next steps

- [spanda-language.md](./spanda-language.md) — full language reference
- [killer-demo.md](./killer-demo.md) — 5-minute safety + verify walkthrough
- [realtime.md](./realtime.md) — deadline-aware tasks and wall-clock mode
- [replay.md](./replay.md) — mission trace record and playback
- [regex.md](./regex.md) — regex literals and filters
- [triggers.md](./triggers.md) — trigger-driven execution
- [concurrency.md](./concurrency.md) — tasks, spawn, channels, fleet CLI
- [hardware-compatibility.md](./hardware-compatibility.md) — deploy profiles
- [architecture.md](./architecture.md) — how the compiler works
- [feature-status.md](./feature-status.md) — what is stable vs experimental

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| `spanda: command not found` | Run `npm run build:rust` or use `./target/release/spanda` |
| `verify` not available | Native CLI required; TS fallback does not support verify |
| Compile error on `wheels.execute(proposal)` | Expected — use `safety.validate(proposal)` first |
| Tests fail after clone | Run `npm install` then `npm run build:rust` before `npm test` |

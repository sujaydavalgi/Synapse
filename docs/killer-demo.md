# Killer Demo: The Unsafe Planner

**Duration:** under 5 minutes  
**Audience:** robotics engineers, safety reviewers, technical evaluators  
**Message:** Spanda blocks unsafe AI at compile time, verifies hardware fit before deploy, and simulates safe execution with runtime safety rules.

This is the flagship demonstration for Spanda v0.5 beta. Run it locally or adapt the script for conference talks and README videos.

---

## What you will show

| Step | Command | What the audience sees |
|------|---------|------------------------|
| 1 | Read source | A readable robot program with AI, safety, and deploy |
| 2 | `spanda check` (unsafe) | Compile error: `ActionProposal` cannot reach actuators |
| 3 | `spanda check` (safe) | Type check passes with `safety.validate()` |
| 4 | `spanda verify --json` | Hardware compatibility report (memory, sensors, timing, battery) |
| 5 | `spanda sim` | Robot moves; `stop_if` triggers near simulated obstacles |
| 6 | Fault injection | Verify warns when battery degradation is simulated |

---

## Prerequisites

Build the native CLI (recommended):

```bash
npm run build:rust
export PATH="$PWD/target/release:$PATH"
```

Or use the TypeScript CLI wrapper:

```bash
npm install
npm run build
```

---

## Step 1 — Show the program (~30 seconds)

Open [`examples/showcase/killer_demo.sd`](../examples/showcase/killer_demo.sd).

Highlight four blocks:

1. **`ai_model planner`** — AI proposes motion; output is untrusted.
2. **`safety { stop_if ... }`** — runtime rules including emergency stop.
3. **`safety.validate(proposal)`** — only validated output reaches actuators.
4. **`deploy SafePatrol to RoverV1`** — program is bound to a hardware profile before ship.

Key lines:

```spanda
let proposal = planner.reason(prompt: "Plan safe forward motion", input: scene);
let action = safety.validate(proposal);
wheels.execute(action);
```

---

## Step 2 — Unsafe AI blocked at compile time (~45 seconds)

Show the minimal failure case (15 lines):

```bash
spanda check examples/showcase/ai_safety_violation.sd
```

**Expected:** non-zero exit; diagnostic mentions `ActionProposal` and `SafeAction`.

The unsafe program passes a raw proposal to the actuator:

```spanda
wheels.execute(proposal);  // error: requires SafeAction
```

**Talking point:** This fails in CI before hardware exists — not at runtime on a factory floor.

---

## Step 3 — Safe program type-checks (~15 seconds)

```bash
spanda check examples/showcase/killer_demo.sd
```

**Expected:** exit code 0.

The fix is one semantic step: validate before execute.

---

## Step 4 — Hardware verification (~60 seconds)

Human-readable report:

```bash
spanda verify examples/showcase/killer_demo.sd
```

JSON for CI integration:

```bash
spanda verify examples/showcase/killer_demo.sd --json
```

**Expected:** compatible with `RoverV1`; checks include memory, sensors, actuators, task timing, mission battery estimate, and AI model requirements.

**Talking point:** *"Will this program run on this robot?"* is answered before flash/deploy — not after integration debugging.

---

## Step 5 — Simulation with safety stop (~60 seconds)

```bash
spanda sim examples/showcase/killer_demo.sd
```

**Expected:** interpreter runs the patrol loop; when simulated lidar range drops below `0.5 m`, the `stop_if` rule triggers emergency stop behavior.

**Talking point:** Design → check → verify → sim is a single toolchain, not four separate tools.

---

## Step 6 — Fault injection (~30 seconds)

The program declares:

```spanda
simulate_compatibility {
  fault BatteryDegradation;
}
```

Re-run verify and point out warnings or margin changes in the report when simulation mode evaluates degraded battery assumptions:

```bash
spanda verify examples/showcase/killer_demo.sd --simulate
```

**Talking point:** Deploy risk is visible in the verify report, not only in post-mortems.

---

## One-liner script (copy-paste)

```bash
set -e
spanda check examples/showcase/ai_safety_violation.sd && exit 1 || true
spanda check examples/showcase/killer_demo.sd
spanda verify examples/showcase/killer_demo.sd
spanda verify examples/showcase/killer_demo.sd --json
spanda sim examples/showcase/killer_demo.sd
spanda verify examples/showcase/killer_demo.sd --simulate
```

---

## What not to show in this demo

Keep the narrative focused. Do **not** include in the same 5-minute flow:

- Blockchain / ledger anchoring
- LLVM / `compile-native`
- World models or advanced agent frameworks
- Multi-robot fleet orchestration
- MQTT/DDS live transport

Those are documented elsewhere and dilute the core story: **safety-typed AI + hardware verify + sim-first development**.

---

## Files

| File | Role |
|------|------|
| [`examples/showcase/killer_demo.sd`](../examples/showcase/killer_demo.sd) | Safe hero program — check, verify, sim |
| [`examples/showcase/ai_safety_violation.sd`](../examples/showcase/ai_safety_violation.sd) | Minimal compile-time failure |
| [`docs/hardware-compatibility.md`](./hardware-compatibility.md) | Deep dive on `spanda verify` |
| [`docs/product-strategy.md`](./product-strategy.md) | Why this demo exists |

---

## CI smoke test

When the native CLI is built, the showcase test suite validates this demo:

```bash
npm test -- tests/showcase.test.ts
```

---

## Related GitHub issue

[v0.5 beta — Curate killer demo program](https://github.com/sujaydavalgi/Spanda/issues/4)

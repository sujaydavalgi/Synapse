# Video demo script (~3 minutes)

Use the **autonomous rover** showcase as the visual backbone. Terminal font ≥ 14pt; split screen optional (code + terminal).

---

## 0:00 — Hook (15s)

> "Spanda is a language where AI output is untrusted, hardware fit is checked before deploy, and safety is mandatory — not optional."

Show: `examples/showcase/autonomous_rover/src/rover.sd` — `safety { }`, `ai_model`, `deploy … to RoverV1`.

---

## 0:15 — Verify hardware (30s)

```bash
cd examples/showcase/autonomous_rover
spanda install
spanda verify src/rover.sd --json --target RoverV1
```

Narrate: memory, sensors, connectivity, task budgets — **before** flash or field trial.

---

## 0:45 — Simulate (30s)

```bash
spanda sim src/rover.sd --record
```

Point at patrol loop, `stop_if`, mock planner → `safety.validate()`.

---

## 1:15 — Inject fault / health (25s)

```bash
spanda sim examples/showcase/health_monitoring/rover.sd --inject-health-faults
```

Narrate: **Healthy → Degraded → Critical** and policy reactions.

---

## 1:40 — Safety gate (25s)

```bash
spanda check examples/showcase/unsafe_ai/unsafe.sd    # fails
spanda check examples/showcase/unsafe_ai/safe.sd    # passes
```

Highlight diagnostic: **Expected SafeAction — Found: ActionProposal — Hint: safety.validate()**.

---

## 2:05 — Kill switch (20s)

Briefly open `examples/features/kill_switch.sd` or `examples/security/remote_signed_kill_switch.sd`.

```bash
spanda sim examples/features/kill_switch.sd --trigger-kill-switch EmergencyStop
```

---

## 2:25 — Replay (25s)

```bash
spanda replay src/rover.trace
spanda replay src/rover.trace --deterministic
```

Narrate: incident review and regression without hardware.

---

## 2:50 — Close (10s)

> "Install, verify, simulate, and replay in under fifteen minutes. Start with `spanda demo rover`."

On-screen: [github.com/Davalgi/Spanda](https://github.com/Davalgi/Spanda) · `docs/getting-started.md`

---

## One-command alternatives

| Segment | Command |
|---------|---------|
| Full flagship | `spanda demo rover` |
| Safety only | `spanda demo safety` |
| Hardware verify | `spanda demo verify` |
| Health | `spanda demo health` |

Related: [killer-demo.md](./killer-demo.md) · [examples/showcase/README.md](../examples/showcase/README.md)

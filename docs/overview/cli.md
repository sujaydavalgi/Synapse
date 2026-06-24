# CLI quick reference

[← Overview](./README.md) · Full reference: [spanda-reference.md](../spanda-reference.md) · Man pages: [man/](../man/)

## Core workflow

| Command | Description |
|---------|-------------|
| `spanda init [name]` | Create a new Spanda project |
| `spanda check <file.sd>` | Type-check |
| `spanda verify <file.sd>` | Hardware compatibility verification |
| `spanda run <file.sd>` | Run with simulated backend |
| `spanda sim <file.sd>` | Simulation with detailed output |
| `spanda test` | Run project tests |
| `spanda fmt <file.sd>` | Format source |
| `spanda lint <file.sd>` | Lint source |

## Demos

| Command | Description |
|---------|-------------|
| `spanda demo rover` | Flagship autonomous rover |
| `spanda demo safety` | Unsafe AI blocked; safe path passes |
| `spanda demo verify` | Hardware verification showcase |
| `spanda demo fleet` | Multi-robot fleet simulation |
| `spanda demo health` | Health checks + fault injection |
| `spanda demo readiness` | Operational go/no-go scoring |
| `spanda demo assurance` | Mission assurance CLI suite |

## Mission assurance & readiness

| Command | Description |
|---------|-------------|
| `spanda assure <file.sd>` | Assurance report |
| `spanda anomaly scan <file.sd>` | Anomaly detector analysis |
| `spanda state estimate <file.sd>` | State estimators and fusion previews |
| `spanda diagnose <file.sd\|trace>` | Fault diagnosis |
| `spanda prognostics <file.sd>` | RUL and degradation warnings |
| `spanda mission verify <file.sd>` | Mission plan achievability |
| `spanda resilience check <file.sd>` | Resilience policies |
| `spanda mitigation plan <file.sd>` | Recovery actions |
| `spanda readiness <file.sd>` | Operational go/no-go score |

## Fleet, replay, packages

| Command | Description |
|---------|-------------|
| `spanda fleet run <file.sd>` | In-process multi-robot simulation |
| `spanda replay <mission.trace>` | Inspect or replay mission trace |
| `spanda build` / `install` / `update` | Package workflow |
| `spanda publish` | Mirror bundle to registry |
| `spanda ros2 check` | Validate ROS 2 bridge setup |
| `spanda twin export <file.sd> --out <replay.json>` | Export twin replay buffer |

## Common flags

**Verify:** `--target <Profile>`, `--all-targets`, `--simulate`, `--json`

**Run / sim / fleet:** `--trace-scheduler`, `--trace-tasks`, `--trace-triggers`, `--trace-events`, `--trace-providers`, `--trace-realtime`, `--metrics-json`, `--record`, `--wall-clock`

**Replay:** `--from T+mm:ss`, `--deterministic`, `--playback`

Topic guides: [realtime.md](../realtime.md) · [replay.md](../replay.md) · [mission-assurance.md](../mission-assurance.md)

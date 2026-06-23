# Benchmarks

Reproducible compile and simulation timings for Spanda **v0.2.0**. Numbers vary by machine; use the same hardware when comparing releases.

## Quick run

From the repository root (after `cargo build -p spanda-cli --release`):

```bash
./scripts/benchmark.sh
```

Set `SPANDA_BIN` to override the CLI path.

## What is measured

| Stage | Input | Command |
|-------|--------|---------|
| Parse | `examples/showcase/killer_demo.sd` | tokenize + parse |
| Type check | same | `spanda check` |
| Verification | same | `spanda verify` |
| Sim startup | same | `spanda sim` (wall time to completion) |

The script prints median of 5 runs (parse/check/verify) using `/usr/bin/time -f '%e'`.

## Manual commands

```bash
SPANDA=target/release/spanda
FILE=examples/showcase/killer_demo.sd

# Parse + type check
/usr/bin/time -f 'check %e s' $SPANDA check "$FILE"

# Hardware verification
/usr/bin/time -f 'verify %e s' $SPANDA verify "$FILE" --json

# Simulation
/usr/bin/time -f 'sim %e s' $SPANDA sim "$FILE"
```

## Larger workloads

| File | Purpose |
|------|---------|
| `examples/showcase/autonomous_rover/src/rover.sd` | Package imports + providers |
| `examples/end_to_end/fleet_coordination.sd` | Multi-robot program size |
| `examples/hardware/capability_verification.sd` | Verification + capabilities |

## CI

Benchmarks are **not** gated in CI (machine variance). The `scripts/benchmark.sh` script is for local regression tracking.

## Related

- [known-limitations.md](./known-limitations.md)
- [feature-status.md](./feature-status.md)

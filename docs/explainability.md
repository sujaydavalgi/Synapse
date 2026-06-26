# Explainability

**Status:** Stable (static v1) · **Phase:** Build, Operate, Recover · **Priority:** P0.3

Help engineers understand code, missions, verification results, and autonomous decisions.

## CLI

```bash
# Static analysis (Phase A)
spanda explain rover.sd
spanda explain rover.sd --config spanda.toml --baseline approved/
spanda explain readiness --file rover.sd
spanda explain verify --file rover.sd
spanda explain safety --file rover.sd

# Trace decisions (Phase D)
spanda explain decision <mission.trace> [--json]
spanda explain <mission.trace> [--json]
```

With `--config`, reports add **configuration validation**, **deployment gates preview** (including `composite_trust` and `secure_boot` when trust contracts are imported), and **package trust** sections. All program explains include a **composite_trust** category breakdown when source is available; programs importing `trust.jetson` or `trust.pi` also include a **secure_boot** section. With `--baseline`, adds a **drift** section comparing approved vs live configuration.

## Capabilities by phase

| Phase | Explains |
|-------|----------|
| A (static) | Source structure, readiness failures, verify failures, safety rule violations, composite trust, secure boot (when `trust.jetson` / `trust.pi` imported), config validation, deployment gates, package trust, drift (with `--baseline`) |
| B | Policy violations, drift deltas |
| D (trace) | Decision, reason, evidence, safety checks, chosen action, rejected actions |

## AI-assisted development (Area 1)

**Experimental** — see [ai-assisted-development.md](./ai-assisted-development.md).

```bash
spanda generate mission [--out patrol.sd]
spanda generate robot
spanda generate health-policy
spanda suggest rover.sd
```

## Crate

`spanda-explain` — composes `spanda-assurance`, `spanda-readiness`, `spanda-hardware`, and `spanda-trust` diagnostics.

See [diagnostics.md](./diagnostics.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).

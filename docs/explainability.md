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
spanda explain decision.trace
```

With `--config`, reports add **configuration validation**, **deployment gates preview**, and **package trust** sections. With `--baseline`, adds a **drift** section comparing approved vs live configuration.

## Capabilities by phase

| Phase | Explains |
|-------|----------|
| A (static) | Source structure, readiness failures, verify failures, safety rule violations, config validation, deployment gates, package trust, drift (with `--baseline`) |
| B | Policy violations, drift deltas |
| D (trace) | Decision, reason, evidence, safety checks, chosen action, rejected actions |

## AI-assisted development (Area 1)

Related commands (Phase D, guardrailed):

```bash
spanda generate mission
spanda generate robot
spanda generate health-policy
spanda suggest rover.sd
```

All generated output must pass `spanda check` and `spanda verify` before deploy. No auto-deploy.

## Crate

`spanda-explain` — composes `spanda-assurance`, `spanda-readiness`, `spanda-hardware` diagnostics.

See [diagnostics.md](./diagnostics.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).

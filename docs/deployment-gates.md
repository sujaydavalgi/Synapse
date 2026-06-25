# Deployment Gates

**Status:** Experimental · **Phase:** Deploy · **Priority:** P0.2

Prevent unsafe deployment when operational gates fail.

## Types

- `DeploymentGatePolicy` — named thresholds (`default`, `production`)
- `DeploymentGate` — single pass/fail check with message
- `DeploymentGateReport` — composite gate result

## Example gates

| Gate | Condition |
|------|-----------|
| Readiness | Score ≥ threshold and `mission_ready` |
| Safety | Safety audit has no critical/high findings |
| Capability | Capability traceability matrix PASS |
| Package trust | Configured packages meet trust threshold (with `--config`) |
| Health | No high-severity health readiness issues |

## CLI

```bash
spanda deploy gate rover.sd
spanda deploy gate rover.sd --policy production
spanda deploy gate rover.sd --json --config spanda.toml
```

Deployment is **blocked** (exit code 1) when any gate fails.

## Foundation

Extends `spanda-readiness` (`evaluate_deployment_gates`), safety auditor, and capability traceability. Complements `deploy rollout --require-certify`.

## Integration

Composes `spanda-readiness`, capability verification, health framework, and assurance evidence.

See [readiness.md](./readiness.md) · [ci-verify.md](./ci-verify.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).

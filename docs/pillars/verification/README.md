# Pillar 3 — Verification Platform

[← Pillars index](../README.md) · [ROADMAP § Pillar 3](../../ROADMAP.md#pillar-3--verification-platform)

Prove hardware fit, mission safety, readiness, assurance, diagnosis, recovery, trust, and explainability.

## Verify & safety

| Topic | Guide |
|-------|--------|
| Hardware verification | [hardware-compatibility.md](../../hardware-compatibility.md) · [man/spanda-verify.md](../../man/spanda-verify.md) |
| Capability traceability | [capability-traceability.md](../../capability-traceability.md) |
| Verification diagnostics | [verification-diagnostics.md](../../verification-diagnostics.md) |
| Minimum-hardware safety | [minimum-hardware-safety.md](../../minimum-hardware-safety.md) |
| Safety gate / agents | [agentic-programming.md](../../agentic-programming.md) · [kill-switch.md](../../kill-switch.md) |
| CI verify | [ci-verify.md](../../ci-verify.md) |

## Readiness & assurance

| Topic | Guide |
|-------|--------|
| Readiness engine | [readiness.md](../../readiness.md) · [man/spanda-readiness.md](../../man/spanda-readiness.md) |
| Mission assurance | [mission-assurance.md](../../mission-assurance.md) |
| State estimation | [state-estimation.md](../../state-estimation.md) |
| Anomaly detection | [anomaly-detection.md](../../anomaly-detection.md) |
| Prognostics | [prognostics.md](../../prognostics.md) |
| Diagnostics | [diagnostics.md](../../diagnostics.md) · [man/spanda-diagnose.md](../../man/spanda-diagnose.md) |
| Mission verification | [mission-verification.md](../../mission-verification.md) |
| Assurance cases | [assurance-cases.md](../../assurance-cases.md) |

## Recovery & continuity

| Topic | Guide |
|-------|--------|
| Self-healing | [self-healing.md](../../self-healing.md) · [recovery-policies.md](../../recovery-policies.md) |
| Mission continuity | [mission-continuity.md](../../mission-continuity.md) · [continuity-policies.md](../../continuity-policies.md) |

## Simulation & replay

| Topic | Guide |
|-------|--------|
| Simulation | [man/spanda-sim.md](../../man/spanda-sim.md) · [killer-demo.md](../../killer-demo.md) |
| Replay | [replay.md](../../replay.md) |
| Realtime sim | [realtime.md](../../realtime.md) |

## Differentiation & maturity

| Topic | Guide |
|-------|--------|
| Mission contracts | [mission-contracts.md](../../mission-contracts.md) |
| Explainability | [explainability.md](../../explainability.md) |
| Decision audit trail | [decision-audit-trail.md](../../decision-audit-trail.md) |
| Safety / recovery coverage | [safety-coverage.md](../../safety-coverage.md) · [recovery-coverage.md](../../recovery-coverage.md) |
| Platform maturity (16 areas) | [platform-maturity-roadmap.md](../../platform-maturity-roadmap.md) |
| Signature capabilities | [differentiation-roadmap.md](../../differentiation-roadmap.md) |

## Examples

| Directory | Focus |
|-----------|--------|
| [examples/showcase/](../../../examples/showcase/) | Flagship verify/sim demos |
| [examples/showcase/assurance/](../../../examples/showcase/assurance/) | Assurance CLI suite |
| [examples/showcase/self_healing/](../../../examples/showcase/self_healing/) | Recovery policies |
| [examples/showcase/continuity/](../../../examples/showcase/continuity/) | Takeover, delegation |
| [examples/showcase/readiness/](../../../examples/showcase/readiness/) | Go/no-go scoring |
| [examples/assurance/](../../../examples/assurance/) | Domain examples |

## Smoke gates

`scripts/differentiation_smoke.sh` · `scripts/assurance_smoke.sh` · `scripts/continuity_smoke.sh` · `scripts/self_healing_smoke.sh` · [scripts/gates/README.md](../../../scripts/gates/README.md)

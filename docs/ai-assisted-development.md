# AI-Assisted Development

**Status:** Experimental (mock-first) · **Phase:** Build, Operate · **Priority:** P3.3

Guardrailed scaffolding and static suggestions for Spanda programs. All generated output passes parse and typecheck before emission. No auto-deploy.

## CLI

```bash
spanda generate mission [--backend template|llm] [--robot Rover] [--mission Patrol] [--out patrol.sd]
spanda generate robot [--backend template|llm] [--robot Rover] [--hardware RoverV1] [--json]
spanda generate health-policy [--backend template|llm] [--health-policy RoverPolicy] [--out health.sd]
spanda suggest examples/showcase/readiness/rover.sd [--json]
```

## Guardrails

| Gate | Behavior |
|------|----------|
| Parse + typecheck | Every `spanda generate` scaffold is validated before reporting success |
| Suggest-only | `spanda suggest` never writes files — recommendations only |
| No auto-deploy | Generated source must pass `spanda verify` separately before deploy |
| Mock-first | Template-based generation by default |
| Optional LLM | `--backend llm` posts the template to `SPANDA_LLM_ENDPOINT` (falls back to template when unset) |

## Suggestions

`spanda suggest` composes readiness scoring, safety audit findings, and policy-gap hints into actionable recommendations.

## Crate

`spanda-generate` — template scaffolds, validation, and rule-based suggestions.

See [explainability.md](./explainability.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).

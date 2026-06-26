# Compliance profile showcases

Template compliance profiles (`defense`, `medical`) with secure boot, tamper policies, and assurance cases — not legal accreditation.

## Commands

```bash
spanda verify examples/showcase/compliance/defense_rover.sd --profile defense
spanda verify examples/showcase/compliance/medical_rover.sd --profile medical
spanda compliance report examples/showcase/compliance/defense_rover.sd --profile defense
spanda deploy gate examples/showcase/compliance/defense_rover.sd
spanda demo gaps
```

Smoke: `scripts/compliance_smoke.sh`, `scripts/gaps_smoke.sh`

See [docs/compliance-profiles.md](../../../docs/compliance-profiles.md).

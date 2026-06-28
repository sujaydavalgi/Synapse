# Pillar 5 — Security Platform

[← Pillars index](../README.md) · [ROADMAP § Pillar 5](../../ROADMAP.md#pillar-5--security-platform)

Encryption, identity, certificates, RBAC, secrets, tamper detection, trust, threat modeling, policy, and compliance.

## Architecture & contracts

| Topic | Guide |
|-------|--------|
| Security architecture | [security-architecture.md](../../security-architecture.md) |
| Secure communication | [secure-communication.md](../../secure-communication.md) |
| Identity | [identity.md](../../identity.md) |
| Secrets | [secrets.md](../../secrets.md) |
| Trust framework | [trust-framework.md](../../trust-framework.md) |
| Trust boundaries | [trust-boundaries.md](../../trust-boundaries.md) |

## Detection & assurance

| Topic | Guide |
|-------|--------|
| Tamper detection | [tamper-detection.md](../../tamper-detection.md) |
| Integrity verification | [integrity-verification.md](../../integrity-verification.md) |
| Spoofing detection | [spoofing-detection.md](../../spoofing-detection.md) |
| Hardware attestation | [hardware-attestation.md](../../hardware-attestation.md) |
| Security assurance | [security-assurance.md](../../security-assurance.md) |
| Audit provenance | [audit-provenance.md](../../audit-provenance.md) |

## Policy & compliance

| Topic | Guide |
|-------|--------|
| Threat modeling | [threat-modeling.md](../../threat-modeling.md) |
| Policy engine | [policy-engine.md](../../policy-engine.md) |
| Compliance profiles | [compliance-profiles.md](../../compliance-profiles.md) |
| Package trust | [package-trust.md](../../package-trust.md) |
| Deployment gates | [deployment-gates.md](../../deployment-gates.md) |

## Examples

| Directory | Blueprint | Focus |
|-----------|-----------|--------|
| [examples/security/](../../../examples/security/) | Defense | Signed commands, invalid signature |
| [examples/showcase/secure_boot/](../../../examples/showcase/secure_boot/) | Defense | Secure-boot attestation |
| [examples/showcase/mission_tampering/](../../../examples/showcase/mission_tampering/) | Defense | Tamper diagnosis |
| [examples/showcase/gps_spoofing/](../../../examples/showcase/gps_spoofing/) | — | Spoofing detection |
| [examples/showcase/compliance/](../../../examples/showcase/compliance/) | Critical Infrastructure | ISO / IEC profiles |

## Smoke gates

`scripts/tamper_smoke.sh` · `scripts/secure_boot_smoke.sh` · `scripts/security_assurance_smoke.sh` · `scripts/compliance_smoke.sh` · [scripts/gates/README.md](../../../scripts/gates/README.md)

## Stable promotion

[security-audit-third-party.md](../../security-audit-third-party.md)

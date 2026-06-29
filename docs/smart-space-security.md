# Smart Space Security

Identity, access control, trust verification, and audit for [Smart Spaces & Ambient Intelligence](./solutions/smart-spaces.md).

**Status:** Experimental (scaffold) · **Config:** `examples/solutions/smart-spaces/spanda.security.toml`

---

## Principles

1. **Verify before trust** — Locks, cameras, and gateways must pass trust checks before access missions run.
2. **Least privilege** — Occupants, visitors, and operators have capability-scoped grants in the entity graph.
3. **Tamper awareness** — Device tamper signals block automation and escalate to operators.
4. **Audit everything** — Access decisions and overrides append to assurance and audit logs.
5. **No silent bypass** — Lockdown and emergency overrides require explicit approval or policy.

---

## Security dimensions

| Dimension | Mechanism |
|-----------|-----------|
| Identity | Human entities, certificates, API keys |
| Certificates | Device attestation via trust framework |
| Encryption | TLS on discovery, MQTT, REST bridges |
| Tamper detection | `spanda-tamper` integration, device tree flags |
| Threat detection | Anomaly on access patterns (`spanda-anomaly`) |
| Access control | `spanda-smart-locks`, capability `access_control` |
| Trust verification | `spanda trust`, package trust scores |
| Audit logging | Platform events, compliance export |

---

## Access control model

```text
Occupant / Visitor (human)
  → capabilities: [enter_zone, unlock_door, …]
  → certifications / expiry
  → trust_level

Door Lock (device)
  → provider: spanda-smart-locks | spanda-matter
  → requires_capability access_control
  → tamper_status
```

Missions that unlock doors or disable alarms require:

- Readiness: lock online, gateway healthy, camera record path (if policy)
- Trust: lock firmware in allowlist, no tamper flag
- Optional: operator approval for after-hours access

---

## Lockdown mission

Triggered by operator or automated threat signal:

1. Readiness — all perimeter locks reachable, cameras recording
2. Deny visitor capabilities
3. Notify security operator queue
4. Produce assurance bundle with timestamped access denials

Example: [examples/solutions/smart-spaces/smart-building/floor_readiness.sd](../examples/solutions/smart-spaces/smart-building/floor_readiness.sd)

---

## Integration with home ecosystems

Home Assistant, Apple Home, and similar systems may remain the **pairing authority**. Spanda:

- Reads lock/camera state via `spanda-home-assistant` or Matter
- Verifies trust before Spanda-orchestrated missions
- Does not store homeowner credentials in core — package-local config only

---

## Compliance profiles

| Deployment | Profile hook |
|------------|--------------|
| Hospital | Medical compliance template + access audit |
| Hotel | Guest access retention policy |
| Enterprise | SOC2-oriented audit export |
| Residential | Privacy-minimal retention |

See [compliance-profiles.md](./compliance-profiles.md)

---

## CLI

```bash
spanda trust --config examples/solutions/smart-spaces/spanda.toml
spanda verify examples/solutions/smart-spaces/smart-home/night_mode.sd \
  --capabilities --traceability \
  --config examples/solutions/smart-spaces/spanda.toml
```

---

## Related

- [smart-space-readiness.md](./smart-space-readiness.md) — Security readiness dimensions
- [security.md](./security.md) — Platform security model
- [trust.md](./trust.md) — Trust framework

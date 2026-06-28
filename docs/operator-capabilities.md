# Operator Capabilities

Operator capabilities extend the **existing capability framework** — the same traceability matrices, `requires_capability` checks, and `spanda verify` flows used for robots apply to human operators. No separate verification engine.

**Related:** [capability-traceability.md](./capability-traceability.md) · [human-interaction.md](./human-interaction.md) · [human-readiness.md](./human-readiness.md)

---

## Capability tokens

| Capability | Typical role | Mission use |
|------------|--------------|-------------|
| `operate_robot` | Operator, Driver | Teleoperation, manual override |
| `approve_mission` | Operator, Supervisor, Safety Officer | Mission start / resume |
| `approve_recovery` | Supervisor, Safety Officer | Recovery plan execution |
| `emergency_override` | Supervisor, Safety Officer | Kill switch bypass with audit |
| `drone_pilot` | Operator, Driver | UAV control |
| `medical_responder` | Healthcare Worker, Emergency Responder | Medical missions |
| `hazmat_certified` | Safety Officer, Emergency Responder | Hazmat zone entry |
| `remote_expert` | Technician, Researcher | Remote assist sessions |
| `maintenance_technician` | Technician | Guided repair workflows |
| `forklift_operator` | Operator | Warehouse forklift missions |
| `search_rescue_operator` | Emergency Responder | SAR collaborative missions |

Packages may register additional operator capabilities in the capability registry (same mechanism as `spanda-nav` → `obstacle_avoidance`).

---

## Verification

Operator capabilities are verified the same way as hardware capabilities:

```bash
spanda verify warehouse_pick.sd --capabilities --traceability --config spanda.toml
spanda check warehouse_pick.sd --verification-json
```

Traceability columns include:

| Capability | Required By | Provided By | Human / Cert | Package | Status |
|------------|-------------|-------------|--------------|---------|--------|

### Program requirements

Missions declare operator requirements with existing syntax:

```sd
requires_capability operate_robot {
  minimum_role = "operator"
  certification = "forklift-cert"
}

requires_capability approve_mission {
  minimum_role = "supervisor"
}
```

Programs do **not** use new HRI-specific keywords — `requires_capability` and `continuity_policy` cover collaborative gates.

---

## Certification tracking

Certifications are TOML entries on human entities with expiry dates. Readiness fails when:

- Certification is expired or missing
- Operator is `off_duty` or `unreachable`
- Trust level is `restricted`
- Required capability is not in the operator's capability list

```toml
[[humans]]
id = "operator-001"
capabilities = ["operate_robot", "forklift_operator"]
certifications = [
  { id = "forklift-cert", expires = "2027-06-01", issuer = "OSHA-12345" },
]
```

---

## Role → capability defaults

Blueprint profiles may define role defaults in `spanda.readiness.toml`:

```toml
[human_roles.operator]
default_capabilities = ["operate_robot"]
required_certifications = ["site-safety-induction"]

[human_roles.supervisor]
default_capabilities = ["operate_robot", "approve_mission", "approve_recovery"]
```

Deployment-specific overrides remain in `spanda.devices.toml`.

---

## Control Center

The **Operator Readiness** panel shows capability coverage per team member. The **Approval Queue** blocks mission start until `approve_mission` capability is satisfied by an available, certified operator.

See [control-center.md](./control-center.md#human-interaction-dashboard).

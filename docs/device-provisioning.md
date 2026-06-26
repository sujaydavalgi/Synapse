# Device Provisioning

Provisioning moves a device from **discovered** to **ready for operations** through a gated workflow.

## Workflow

1. **Discover** — device appears in pool (subnet, mDNS, BLE, USB, CAN, MQTT, ROS2)
2. **Inspect identity** — serial, MAC, IP, certificate fingerprint
3. **Verify trust** — trust level must be `verified` or `trusted`
4. **Validate capabilities** — remote actuators declare `move`, `stop`, `emergency_stop`
5. **Run health check** — lifecycle not quarantined/failed; calibration valid
6. **Assign** — bind to robot/fleet/swarm and logical entity
7. **Update device tree** — refresh `spanda.devices.toml` mapping
8. **Readiness verification** — readiness engine confirms mission impact

## CLI

```bash
spanda device provision <device-id> --robot rover-001 [--json]
spanda device trust <device-id> [--write] [--json]
```

Operator approval after quarantine or for `trust_level = unknown`:

```bash
spanda device trust camera-front-001 --write --config spanda.toml
```

API equivalent: `POST /v1/devices/{id}/trust` (requires Bearer token with **Approve** role).

Exit code `1` when any gate fails.

## API

```bash
curl -X POST http://127.0.0.1:8080/v1/provision \
  -H "Authorization: Bearer $SPANDA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"device_id":"gps-001","robot_id":"rover-001"}'

# Per-device route
curl -X POST http://127.0.0.1:8080/v1/devices/gps-001/provision \
  -H "Authorization: Bearer $SPANDA_API_KEY" \
  -d '{"robot_id":"rover-001"}'
```

Failed provisioning quarantines the device automatically.

## Idempotent reprovision and conflicts

Re-running provision for the same `device_id` is **idempotent** when:

- Lifecycle is already `active` or `assigned` with the same `robot_id`
- Trust level remains `verified` or `trusted`
- Firmware and calibration gates still pass

**Conflict policy** (operator action required):

| Situation | Behavior |
|-----------|----------|
| Device assigned to a different robot | API returns `ready: false`; update assignment via `PATCH /v1/devices/{id}` first |
| Device quarantined | Resolve quarantine cause, then `POST /v1/devices/{id}/trust` before reprovision |
| Stale provisioning_id | New run generates a fresh report; prior `provisioning_id` is superseded in audit only |
| Concurrent provision requests | Last successful write wins; use correlation IDs in mutation audit for ordering |

Treat `POST /v1/provision` as a **gate check**, not a destructive reset — it does not retire or delete pool entries.

## Logical mapping example

```toml
# logical sensor gps → physical device gps-ublox-001
[[devices]]
id = "gps-ublox-001"
logical_name = "gps"
assigned_robot = "rover-001"

# logical actuator wheels → physical sabertooth-drive-001
[[devices]]
id = "sabertooth-drive-001"
logical_name = "wheels"
type = "DifferentialDrive"
capabilities = ["move", "stop", "emergency_stop"]
```

Export mapping JSON via `GET /v1/device-tree`.

## Related

- [device-pool.md](./device-pool.md)
- [calibration.md](./calibration.md)
- [device-quarantine.md](./device-quarantine.md)

# Maritime — Official Solution Blueprint

Autonomous vessels, port logistics, and shore coordination — composed from redundant navigation, prognostics hooks, and mission continuity.

**Profile:** `maritime` · **Status:** Experimental (scaffold) · **Doc:** [docs/solutions/maritime.md](../../docs/solutions/maritime.md)

---

## Quick start

```bash
cd examples/solutions/maritime
spanda check harbor_patrol.sd
spanda verify harbor_patrol.sd --target CoastalVesselV1
spanda sim harbor_patrol.sd
spanda readiness harbor_patrol.sd --json
```

---

## Blueprint layout

```
maritime/
├── README.md
├── spanda.toml
├── harbor_patrol.sd         # Coastal patrol with collision avoidance
└── (planned) convoy_escort.sd, docking_assist.sd
```

---

## Platform pillars used

| Pillar | Capabilities |
|--------|--------------|
| Device & Fleet | Fleet mesh, continuity/takeover, OTA |
| Verification | Pre-departure readiness, prognostics, recovery |
| Security | Encrypted comms, RBAC for shore operators |
| Operations | Incident workflow, fleet map |

---

## Related

- [docs/solutions/maritime.md](../../docs/solutions/maritime.md)
- [docs/mission-continuity.md](../../docs/mission-continuity.md)
- [docs/positioning.md](../../docs/positioning.md)

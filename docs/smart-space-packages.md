# Smart Space Packages & Integration Strategy

Optional packages and ecosystem integration for [Smart Spaces & Ambient Intelligence](./solutions/smart-spaces.md).

**Rule:** All packages below are **optional**. The platform core has no smart-building dependencies.

---

## Package catalog

| Package | Category | Import path | Status |
|---------|----------|-------------|--------|
| `spanda-matter` | IoT | `iot.matter` | Experimental (stub + bridge) |
| `spanda-thread` | IoT | `iot.thread` | Experimental (stub) |
| `spanda-zigbee` | IoT | `iot.zigbee` | Experimental (stub) |
| `spanda-zwave` | IoT | `iot.zwave` | Experimental (stub) |
| `spanda-bacnet` | IoT | `iot.bacnet` | Experimental (bacpypes3 + env bridge) |
| `spanda-knx` | IoT | `iot.knx` | Experimental (xknx + env bridge) |
| `spanda-modbus` | IoT | `iot.modbus` | Experimental (stub) |
| `spanda-mqtt` | IoT | `iot.mqtt` | Experimental |
| `spanda-ble` | Connectivity | `connectivity.ble` | Experimental |
| `spanda-wifi` | Connectivity | `connectivity.wifi` | Experimental |
| `spanda-home-assistant` | Bridge | `bridge.home_assistant` | Experimental (REST + mock) |
| `spanda-energy` | Energy | `energy.solar`, `energy.storage` | Experimental (stub) |
| `spanda-building` | Blueprint | `building.entity` | Experimental (stub) |
| `spanda-smart-locks` | Access | `access.lock` | Experimental (stub) |
| `spanda-environment` | Sensors | `environment.aq` | Experimental (stub) |
| `spanda-mission-continuity` | Ops | continuity policies | Experimental |
| `spanda-smartwatch` | HRI | wearables | Experimental |
| `spanda-voice` | HRI | voice ingress | Experimental |

Source: `packages/registry/spanda-*/`

---

## Integration strategy

### Layer 1 — Device authority (external)

Home Assistant, OpenHAB, Apple Home, Google Home, Amazon Alexa, SmartThings, and vendor clouds remain **device pairing and scene authorities** for end users.

Spanda does not replicate their UX or device databases.

### Layer 2 — Provider bridge (Spanda packages)

Optional packages read state and send **verified commands** through provider interfaces:

```text
External hub (e.g. Home Assistant)
  ↔ spanda-home-assistant (REST/WebSocket)
  ↔ Spanda provider runtime
  ↔ .sd mission / readiness / assurance
```

**Field buses (BACnet / KNX):** install `requirements-bacnet.txt` or `requirements-knx.txt`, set `SPANDA_LIVE_*` env vars, and use package `scripts/read_*.sh` helpers — see [iot.md](../iot.md#live-hardware-optional) and each package README.

### Layer 3 — Orchestration (Spanda core)

Readiness, verify, mission continuity, assurance, trust, Control Center — unchanged platform pillars.

### Layer 4 — Evidence & operations

Assurance bundles, audit logs, operator dashboards, simulation/replay.

---

## Ecosystem matrix

| Ecosystem | Integration approach | Package | Spanda role |
|-----------|---------------------|---------|-------------|
| Home Assistant | REST + WebSocket state sync | `spanda-home-assistant` | Mission orchestration above HA |
| OpenHAB | REST bridge (planned) | — | Same pattern as HA |
| Apple Home | Matter devices via hub | `spanda-matter` | Readiness + assurance |
| Google Home | Voice trigger ingress (planned) | `spanda-voice` | Mission start only |
| Amazon Alexa | Skill → webhook (planned) | `spanda-mqtt` | Mission start only |
| SmartThings | Cloud API bridge (planned) | `spanda-mqtt` | State read + verified commands |
| BACnet BMS | Field bus | `spanda-bacnet` | Commercial building orchestration |
| KNX | Field bus | `spanda-knx` | EU building automation |

**Voice assistants** trigger Spanda missions; they do not receive unsafe actuation bypass.

---

## Provider interface contract

Packages implement traits documented in [provider-interfaces.md](./provider-interfaces.md) and [iot.md](./iot.md):

| Operation | Description |
|-----------|-------------|
| `telemetry.read` | Device state for readiness |
| `command.send` | Actuation after verify pass |
| `shadow.sync` | Desired vs reported (Matter-style) |
| `health.ping` | Gateway liveness |

Missions declare `requires capabilities`; providers map capabilities to device commands at runtime.

---

## Deployment patterns

| Pattern | Packages | Use case |
|---------|----------|----------|
| Home + Matter | `spanda-matter`, `spanda-energy` | Night mode, leak, solar |
| Home + HA bridge | `spanda-home-assistant`, `spanda-mission-continuity` | Existing HA install |
| Office BMS | `spanda-bacnet`, `spanda-environment` | Occupancy HVAC |
| Hospital wing | `spanda-bacnet`, `spanda-smartwatch` | Patient + clinical |
| Campus rollup | `spanda-building`, `spanda-mqtt` | Multi-building NOC |

---

## Install

```bash
spanda install spanda-matter spanda-energy spanda-building
# Edit examples/solutions/smart-spaces/spanda.providers.toml
spanda install --config examples/solutions/smart-spaces/spanda.toml
```

---

## Registry maintenance

Add or edit packages under `packages/registry/`, then:

```bash
./scripts/build-registry.sh
./scripts/sync_bundled_registry.sh
```

---

## Related

- [official-packages.md](./official-packages.md) — Full official catalog
- [iot.md](./iot.md) — IoT provider guide
- [building-automation.md](./building-automation.md) — Protocol selection by building type

# Unified Entity Model

The **Unified Entity Model** is a foundational platform pillar in Spanda. Every object managed by the platform — robots, fleets, humans, wearables, devices, providers, packages, missions, facilities, and control centers — is represented as an **Entity** with shared properties, relationships, health, readiness, trust, security, and lifecycle semantics.

## Why a unified model?

As Spanda expands across industries (ADAS, healthcare, search & rescue, industrial automation, spatial computing), dedicated top-level models for each object type become inconsistent. The entity model provides:

- One **registry** and **graph** for traversal, dependency analysis, and impact analysis
- One **query language** for operational questions (“which robots use firmware X?”)
- One **Control Center** browse path for health, readiness, trust, and relationships
- **Backward-compatible** APIs — existing `/v1/devices`, `/v1/robots`, `/v1/humans` routes remain unchanged

## Architecture

```text
TOML / runtime sources                Unified projection
─────────────────────                 ──────────────────
DeviceTree ──────────┐
DeviceRegistry ──────┼──► build_entity_registry() ──► EntityRegistry
HumanRegistry ───────┤                                      │
LogicalPhysicalMap ──┤                                      ├── EntityGraph
Packages / Providers ┘                                      └── EntityQuery
```

**Canonical implementation:** `crates/spanda-config/src/entity.rs`

**API surface:** `GET /v1/entities/*` in `crates/spanda-api/src/sdk_ops.rs`

**SDK:** `SpandaClient::list_entities`, `entity_graph`, `query_entities` in `crates/spanda-sdk`

## Entity hierarchy

The type taxonomy is **extensible**. Built-in kinds include:

| Category | Entity kinds |
|----------|----------------|
| People & teams | `human`, `team` |
| Autonomous systems | `robot`, `drone`, `vehicle`, `fleet`, `swarm`, `ai_agent` |
| Devices | `device`, `sensor`, `actuator`, `camera`, `gps`, `plc`, `gateway`, `controller`, `wearable`, `medical_device` |
| Spatial | `ar_device`, `vr_device`, `iot_device` |
| Software | `provider`, `package`, `edge_service`, `cloud_service` |
| Operations | `mission`, `incident`, `digital_twin` |
| Places | `facility`, `building`, `zone`, `hazard`, `organization` |
| Control | `command_center`, `control_center` |
| Custom | `custom` string via `EntityKind::Custom` |

Domain-specific TOML types (`HumanEntity`, `RobotNode`, `DeviceIdentityRecord`, …) remain the **source of truth**. They project into `EntityRecord` — they are not replaced.

## Common properties

Every `EntityRecord` carries:

| Property | Description |
|----------|-------------|
| `id` | Unique identifier |
| `name`, `display_name`, `description` | Human-facing labels |
| `entity_type` | Typed kind (`EntityKind`) |
| `parent_id`, `children_ids` | Hierarchy |
| `labels`, `tags` | Filtering and grouping |
| `version`, `manufacturer`, `model`, `serial_number` | Identity |
| `hardware_revision`, `firmware_version`, `software_version` | Revision tracking |
| `provider`, `package` | Software supply chain |
| `location` | Coordinates, zone references |
| `capabilities` | Operational capabilities |
| `health_status` | `healthy`, `warning`, `degraded`, `offline`, `critical`, `unknown` |
| `readiness_status` | `ready`, `not_ready`, `partial`, `unknown` |
| `trust_status` | `verified`, `trusted`, `untrusted`, `compromised`, `unknown` |
| `security` | Identity, certificates, permissions |
| `lifecycle_state` | `discovered` → `archived` |
| `owner`, `metadata`, `audit` | Governance |

Legacy API field **`kind`** is preserved as an alias of `entity_type.as_str()` for SDK compatibility.

## Entity capabilities

Capabilities are plain strings on the entity record. Examples:

| Entity | Capabilities |
|--------|----------------|
| Human | `operate_robot`, `approve_mission`, `emergency_override` |
| Robot | `navigate`, `pick`, `place`, `inspect` |
| Wearable | `heart_rate`, `gps`, `fall_detection` |
| Mission | `pause`, `resume`, `cancel` |
| Package | `install`, `update`, `validate` |

Capability requirements for missions continue to flow through readiness and assurance crates; entities expose the **inventory view**.

## Health, readiness, trust, security, lifecycle

| Dimension | Enum | Notes |
|-----------|------|-------|
| Health | `EntityHealthStatus` | Derived from device pool health and human health fields |
| Readiness | `EntityReadinessStatus` | Derived from lifecycle and operator availability |
| Trust | `EntityTrustStatus` | Maps legacy `trust_level` strings |
| Lifecycle | `EntityLifecycleState` | Maps `DeviceLifecycleState` and availability |
| Security | `EntitySecurityIdentity` | Certificates, permissions from TOML security sections |

See also: [entity-relationships.md](./entity-relationships.md), [entity-registry.md](./entity-registry.md), [entity-graph.md](./entity-graph.md), [entity-query-language.md](./entity-query-language.md).

## API (additive)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/v1/entities` | List entities (optional query filters) |
| GET | `/v1/entities/graph` | Full entity graph |
| POST | `/v1/entities/query` | Structured query body |
| GET | `/v1/entities/{id}` | Entity detail |
| GET | `/v1/entities/{id}/relationships` | Edges, impact analysis, dependency chain |
| GET | `/v1/entities/{id}/health` | Health snapshot |
| GET | `/v1/entities/{id}/readiness` | Readiness snapshot |
| GET | `/v1/entities/traceability` | Unified traceability (entity + program graph) |
| POST | `/v1/entities/register` | Register or update entity overlay (Bearer) |
| POST | `/v1/entities/{id}/tags` | Add or remove tags (Bearer) |
| POST | `/v1/entities/relationships` | Relate two entities (Bearer) |
| POST | `/v1/entities/sync` | Sync overlay to TOML fragments (Bearer) |

**gRPC (tonic):** same JSON payloads via `GetEntityGraph`, `GetEntityTraceability`, `QueryEntities`, `GetEntityRelationships`, `GetEntityReadiness`, `RegisterEntity`, `TagEntity`, `RelateEntities`, `SyncEntities` on `--grpc-bind`. Mutations require Bearer metadata (Rust `GrpcClient` reads `SPANDA_API_KEY`). JSON-RPC gateway exposes read-only entity methods via `POST /v1/rpc`.

Existing routes (`/v1/devices`, `/v1/robots`, `/v1/fleets`, `/v1/humans`, …) are unchanged.

## Control Center

The **Entities** tab in `@davalgi-spanda/web` uses the unified API:

- Browse and search the entity inventory
- Inspect health, readiness, trust, capabilities
- Traverse relationship edges and neighborhood graph

Component: `packages/web/src/EntityGraphPanel.tsx`

## Roadmap integration

Before adding a new top-level platform abstraction, ask:

> **Should this be modeled as a new Entity kind?**

If yes, extend `EntityKind`, add a projection in `build_entity_registry`, and document the mapping. See [../ROADMAP.md](../ROADMAP.md) — **Pillar 0 — Unified Entity Model**.

Cross-references:

| Roadmap item | Entity mapping |
|--------------|----------------|
| Device Registry (Pillar 4) | `device`, `sensor`, `actuator`, … |
| Human entity model (Pillar 4) | `human`, `wearable`, `digital_twin` |
| Fleet / swarm (Pillar 4) | `fleet`, `robot`, `swarm` |
| Provider registry (Pillar 2) | `provider` |
| Package loader (Pillar 2) | `package` |
| Digital thread (Pillar 6) | Graph edges complement dependency graph |
| Trust / security (Pillar 5) | `trust_status`, `security` on every entity |

## Migration plan

### Phase 1 — Foundation (shipped)

- [x] `EntityRecord`, `EntityRegistry`, `EntityGraph`, `EntityQuery` in `spanda-config`
- [x] `build_entity_registry(&ResolvedSystemConfig)` projects fleet tree, device registry, human registry, logical map, packages, providers
- [x] Expanded `/v1/entities/*` REST API (backward compatible `kind` field)
- [x] Control Center **Entities** tab
- [x] SDK typed fields on `Entity`

### Phase 2 — Runtime missions (Complete)

- [x] Project runtime `MissionRuntime` into entity registry during active programs
- [x] Link mission entities to robot/fleet entities via `participates_in` edges
- [x] Mission readiness overlays on entity readiness API

### Phase 3 — Graph unification (Complete)

- [x] Align `spanda-graph` dependency nodes with entity IDs
- [x] Merge digital-thread device links into entity relationship store
- [x] Unified traceability queries across program graph and entity graph (`GET /v1/entities/traceability`)

### Phase 4 — Industry extensions (Complete)

- [x] Facility, building, zone entities from solution blueprint TOML (`spanda.facilities.toml`, `[[entity_kinds]]`)
- [x] Medical device and ADAS-specific entity kinds with compliance metadata (`entity_kind`, `compliance_profile`, assurance/readiness/security profiles)
- [x] Custom entity kinds via package manifests (`[entity_kinds]` on `PackageManifest`) and blueprint `[[entity_kinds]]`

### Phase 5 — Write path (Complete)

- [x] Entity mutation APIs (register, tag, relate) with audit
- [x] Bi-directional sync from entity registry to TOML fragments (`POST /v1/entities/sync`)

### Stabilization (Complete)

- [x] CI smoke script `scripts/entity_model_smoke.sh` (graph, traceability, query, mutations, TypeScript + Python SDK)
- [x] Control Center **Entities** tab write UI (register, tag, relate, sync) with API key auth
- [x] SDK parity: `registerEntity` / `register_entity`, `tagEntity` / `tag_entity`, `relateEntities` / `relate_entities`, `syncEntities` / `sync_entities`, `entityGraph` / `entity_graph`, `entityTraceability` / `entity_traceability`, `queryEntities` / `query_entities` (TypeScript, Python, Rust REST + Rust `GrpcClient` gRPC)
- [x] Stable promotion gate: `scripts/entity_model_stable_promotion_gate.sh` + CI `entity-model-promotion-gate` — [entity-model-stable-promotion.md](./entity-model-stable-promotion.md)

### Promotion to Stable

Implementation is **complete**. Tier remains **Experimental** until:

1. Shared 30-day field soak ([field-soak-gate.md](./field-soak-gate.md))
2. Third-party security audit sign-off
3. `entity_model_stable_promotion_gate.sh` passes without skip flags
4. PyPI `spanda-sdk` **0.4.1** published (`sdk-python-v0.4.1` tag)
5. `docs/feature-status.md` **Unified Entity Model** row updated to **Stable**

### Compatibility guarantees

1. **No breaking changes** to existing REST routes or TOML schemas in Phase 1–3
2. **`kind` field** on list responses remains stable for SDK consumers
3. Domain crates (`HumanEntity`, `DeviceIdentityRecord`, …) stay authoritative for configuration authoring

### Developer checklist — adding a new industry object

1. Add or reuse an `EntityKind` variant (or `Custom` string)
2. Implement projection in `build_entity_registry` from your TOML/runtime source
3. Emit `EntityRelationship` edges to related entities
4. Add query filter fields if needed on `EntityQuery`
5. Update [feature-status.md](./feature-status.md) and roadmap cross-reference
6. Add Control Center filter label if user-facing

# Entity SDK

Official SDKs for the Unified Entity Model over Control Center **REST v1** and **gRPC**. All entity methods are thin HTTP/gRPC wrappers — evaluation logic lives in `spanda-readiness` and `spanda-trust`.

| SDK | Package | Source |
|-----|---------|--------|
| Rust | [`spanda-sdk`](https://crates.io/crates/spanda-sdk) | `crates/spanda-sdk/` |
| TypeScript | [`@davalgi-spanda/sdk`](https://www.npmjs.com/package/@davalgi-spanda/sdk) | `sdk/typescript/` |
| Python | [`spanda-sdk`](https://pypi.org/project/spanda-sdk/) | `sdk/python/` |

Publishing: [sdk-publishing.md](./sdk-publishing.md)

## Configuration

| Variable | Purpose | Default |
|----------|---------|---------|
| `SPANDA_CONTROL_CENTER_URL` | REST base URL | `http://127.0.0.1:8080` |
| `SPANDA_API_KEY` | Bearer token for mutations | unset (mutations fail without it) |
| `SPANDA_GRPC_BIND` | gRPC target (Rust `GrpcClient` only) | derived from URL host + port `50051` |

Start Control Center before calling SDK methods:

```bash
CONFIG=crates/spanda-config/tests/fixtures/warehouse/spanda.toml
spanda control-center serve --bind 127.0.0.1:8080 --config "$CONFIG"
```

Smoke test (all three SDKs): `scripts/entity_model_smoke.sh`

## Method parity

| Operation | Rust `SpandaClient` | TypeScript | Python | Rust `GrpcClient` |
|-----------|---------------------|------------|--------|-------------------|
| List entities | `list_entities()` | `listEntities()` | `list_entities()` | `list_entities()` |
| Get entity | `get_entity(id)` | `getEntity(id)` | `get_entity(id)` | `get_entity(id)` |
| Query | `query_entities(&json)` | `queryEntities(body)` | `query_entities(body)` | `query_entities(&json)` |
| Graph | `entity_graph()` | `entityGraph()` | `entity_graph()` | `entity_graph()` |
| Traceability | `entity_traceability(…)` | `entityTraceability(…)` | `entity_traceability(…)` | `entity_traceability(…)` |
| Relationships | `entity_relationships(id)` | `entityRelationships(id)` | `entity_relationships(id)` | `entity_relationships(id)` |
| Health | `entity_health(id)` | `getHealth(id)` | `get_health(id)` | `entity_health(id)` |
| Readiness | `entity_readiness(id)` | `entityReadiness(id)` | `entity_readiness(id)` | `entity_readiness(id)` |
| Trust | `entity_trust(id)` | `getTrust(id)` | `get_trust(id)` | `entity_trust(id)` |
| Verify | `entity_verify(id, body)` | `verifyEntity(id, body)` | `entity_verify(id, …)` | `entity_verify(id, &json)` |
| Register | `register_entity(&json)` | `registerEntity(body)` | `register_entity(body)` | `register_entity(&json)` |
| Tag | `tag_entity(id, &json)` | `tagEntity(id, body)` | `tag_entity(id, body)` | `tag_entity(id, &json)` |
| Relate | `relate_entities(&json)` | `relateEntities(body)` | `relate_entities(body)` | `relate_entities(&json)` |
| Sync | `sync_entities()` | `syncEntities()` | `sync_entities()` | `sync_entities()` |

### Smart Spaces (Control Center blueprint)

| Operation | Rust `SpandaClient` | TypeScript | Python |
|-----------|---------------------|------------|--------|
| Summary | `smart_spaces_summary()` | `smartSpacesSummary()` | `smart_spaces_summary()` |
| Facilities | `list_facilities()` | `listFacilities()` | `list_facilities()` |
| Readiness | `facility_readiness(id)` | `facilityReadiness(id)` | `facility_readiness(id)` |
| Occupancy | `zone_occupancy(id)` | `zoneOccupancy(id)` | `zone_occupancy(id)` |
| Devices | `smart_spaces_devices(facility)` | `smartSpacesDevices(id)` | `smart_spaces_devices(id)` |
| Health | `facility_health(id)` | `facilityHealth(id)` | `facility_health(id)` |
| Security | `facility_security(id)` | `facilitySecurity(id)` | `facility_security(id)` |
| Environment | `zone_environment(id)` | `zoneEnvironment(id)` | `zone_environment(id)` |
| Energy detail | `energy_system(id)` | `energySystem(id)` | `energy_system(id)` |
| Floor map | `facility_floor_map(id)` | `facilityFloorMap(id)` | `facility_floor_map(id)` |

gRPC: `smart_spaces_summary`, `list_facilities`, `facility_readiness`, `zone_occupancy`, `list_energy_systems`, `emergency_status` on `GrpcClient` (`grpc` feature).

API shapes: [entity-apis.md](./entity-apis.md)

## Rust — REST

```rust
use serde_json::json;
use spanda_sdk::SpandaClient;

let client = SpandaClient::local();

// Inventory
let entities = client.list_entities()?;
let rover = client.get_entity("rover-001")?;

// Graph and query
let graph = client.entity_graph()?;
let filtered = client.query_entities(&json!({ "kind": "robot" }))?;

// Evaluation (server-side engines when CC has --config)
let health = client.entity_health("rover-001")?;
let readiness = client.entity_readiness("rover-001")?;
let trust = client.entity_trust("rover-001")?;
let verify = client.entity_verify(
    "rover-001",
    Some(&json!({ "include_dependencies": true })),
)?;

// Mutations (requires SPANDA_API_KEY)
client.register_entity(&json!({
    "id": "bay-3",
    "entity_type": "calibration_station",
    "parent_id": "warehouse-a",
    "capabilities": ["calibrate"]
}))?;
client.tag_entity("bay-3", &json!({ "add": ["sdk"] }))?;
client.sync_entities()?;
```

Typed wrappers: `Entity`, `HealthReport`, `TrustReport` in `spanda_sdk::types`. Raw JSON is available on `.raw` fields.

**Example:** `cargo run -p spanda-sdk --example entity_mutations` (used by `scripts/entity_model_smoke.sh`)

## Rust — gRPC

Async client over tonic (`spanda_sdk::grpc::GrpcClient`):

```rust
use serde_json::json;
use spanda_sdk::grpc::GrpcClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GrpcClient::connect("http://127.0.0.1:50051").await?;
    let entities = client.list_entities().await?;
    let verified = client
        .entity_verify("rover-001", &json!({ "include_dependencies": true }))
        .await?;
    Ok(())
}
```

`SPANDA_API_KEY` is sent as Bearer metadata on mutation RPCs automatically.

## TypeScript

```typescript
import { SpandaClient } from "@davalgi-spanda/sdk";

const client = new SpandaClient({
  baseUrl: process.env.SPANDA_CONTROL_CENTER_URL,
  apiKey: process.env.SPANDA_API_KEY,
});

const entities = await client.listEntities();
const rover = await client.getEntity("rover-001");
const graph = await client.entityGraph();
const trace = await client.entityTraceability({ entityId: "rover-001" });
const health = await client.getHealth("rover-001");
const verify = await client.verifyEntity("rover-001", {
  includeDependencies: true,
});

await client.registerEntity({
  id: "bay-3",
  entity_type: "calibration_station",
  parent_id: "warehouse-a",
  capabilities: ["calibrate"],
});
await client.tagEntity("bay-3", { add: ["ts-sdk"] });
await client.syncEntities();
```

Build from source: `cd sdk/typescript && npm run build`

## Python

```python
from spanda_sdk import SpandaClient

client = SpandaClient()

entities = client.list_entities()
rover = client.get_entity("rover-001")
graph = client.entity_graph()
readiness = client.entity_readiness("rover-001")
verify = client.entity_verify("rover-001", include_dependencies=True)

client.register_entity({
    "id": "bay-3",
    "entity_type": "calibration_station",
    "parent_id": "warehouse-a",
    "capabilities": ["calibrate"],
})
client.tag_entity("bay-3", {"add": ["py-sdk"]})
client.sync_entities()
```

Install: `pip install spanda-sdk` or `pip install -e sdk/python`

## Local evaluation vs Control Center

| Surface | Config source | When to use |
|---------|---------------|-------------|
| `spanda entity *` CLI | `--config` / project `spanda.toml` | CI scripts, offline ops |
| SDK REST/gRPC | Control Center `--config` + `--program` | Fleet dashboards, integrations |
| In-process Rust | `build_entity_registry(&resolved)` + engine fns | Custom tools embedding `spanda-readiness` |

CLI and server paths call the **same engines** (`verify_entity`, `evaluate_entity_readiness`, etc.) with the same options structs.

## Error handling

| HTTP | SDK behavior |
|------|----------------|
| `404` | Entity not found |
| `401` / `403` | Missing or invalid `SPANDA_API_KEY` on mutations |
| Connection refused | Control Center not running |

Rust: `SpandaError::Connection`, `SpandaError::Api`  
TypeScript: `ConnectionError`, `SpandaError`  
Python: raises `SpandaError` subclasses

## Related docs

- [entity-apis.md](./entity-apis.md) — REST/gRPC reference
- [entity-verification.md](./entity-verification.md) — verify semantics
- [entity-readiness.md](./entity-readiness.md) — readiness semantics
- [entity-health.md](./entity-health.md) — health semantics
- [entity-trust.md](./entity-trust.md) — trust semantics
- [entity-best-practices.md](./entity-best-practices.md) — adoption patterns
- [examples/entity/](../examples/entity/) — runnable `.sd` programs

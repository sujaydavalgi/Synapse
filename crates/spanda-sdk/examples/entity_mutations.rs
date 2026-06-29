//! Entity graph and mutation helpers — used by `scripts/entity_model_smoke.sh`.
//!
//! Requires Control Center with warehouse fixture (started by the smoke script).

use serde_json::json;
use spanda_sdk::SpandaClient;

fn main() {
    let client = SpandaClient::local();

    let entities = client.list_entities().expect("list entities");
    if !entities.iter().any(|e| e.id == "smoke-bay") {
        panic!("smoke-bay missing after register");
    }

    client
        .tag_entity("smoke-bay", &json!({ "add": ["rust-sdk-smoke"] }))
        .expect("tag entity");

    let graph = client.entity_graph().expect("entity graph");
    if graph.get("graph").is_none() {
        panic!("entity graph missing");
    }

    let trace = client
        .entity_traceability(Some("rover-001"), None, None)
        .expect("entity traceability");
    if trace.get("traceability").is_none() {
        panic!("entity traceability missing");
    }

    println!("rust-sdk entity smoke ok");
}

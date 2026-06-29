//! Smart Spaces REST handlers.
use spanda_api::state::ControlCenterState;
use spanda_config::ConfigResolver;
use std::path::PathBuf;

fn smart_spaces_state() -> ControlCenterState {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/solutions/smart-spaces");
    let program = root.join("smart-building/floor_readiness.sd");
    let resolved = ConfigResolver::new()
        .with_validation(false)
        .resolve_from_dir(&root)
        .expect("resolve smart spaces blueprint");
    let mut state = ControlCenterState::new();
    state.resolved = Some(resolved);
    state.program_path = Some(program);
    state
}

#[test]
fn facilities_energy_and_emergency_from_smart_spaces_blueprint() {
    let state = smart_spaces_state();
    let facilities = spanda_api::smart_spaces::facilities_list(&state);
    assert_eq!(facilities.status, 200);
    assert!(facilities.body.contains("tower-demo"));
    assert!(facilities.body.contains("matter-hub-primary"));

    let readiness = spanda_api::smart_spaces::facility_readiness_get(&state, "tower-demo");
    assert_eq!(readiness.status, 200);
    assert!(readiness.body.contains("readiness_profile"));
    assert!(readiness.body.contains("factors"));
    assert!(readiness.body.contains("robots"));
    assert!(readiness.body.contains("wearables"));
    assert!(readiness.body.contains("trust_entries"));
    let readiness_json: serde_json::Value = serde_json::from_str(&readiness.body).unwrap();
    let score = readiness_json["score"].as_u64().unwrap_or(0);
    assert!(score > 0);
    let factors = readiness_json["factors"].as_array().unwrap();
    assert!(!factors.is_empty());

    let occupancy = spanda_api::smart_spaces::zone_occupancy_get(&state, "floor-12");
    assert_eq!(occupancy.status, 200);
    assert!(occupancy.body.contains("floor-12"));

    let energy = spanda_api::smart_spaces::energy_systems_list(&state);
    assert_eq!(energy.status, 200);
    assert!(energy.body.contains("solar-001"));

    let emergency = spanda_api::smart_spaces::emergency_status_get(&state);
    assert_eq!(emergency.status, 200);
    assert!(emergency.body.contains("continuity_pairs"));

    let summary = spanda_api::smart_spaces::smart_spaces_summary(&state);
    assert_eq!(summary.status, 200);
    assert!(summary.body.contains("smart_spaces"));
    assert!(summary.body.contains("readiness_rollups"));
    assert!(summary.body.contains("robots"));
}

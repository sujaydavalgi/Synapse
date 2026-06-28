//! Humans and wearables REST handlers.
use spanda_api::state::ControlCenterState;
use spanda_config::ConfigResolver;
use std::path::PathBuf;

fn spatial_blueprint_state() -> ControlCenterState {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/solutions/spatial-computing");
    let program = root.join("warehouse-ar/pick_mission.sd");
    let resolved = ConfigResolver::new()
        .with_validation(false)
        .resolve_from_dir(&root)
        .expect("resolve spatial blueprint");
    let mut state = ControlCenterState::new();
    state.resolved = Some(resolved);
    state.program_path = Some(program);
    state
}

#[test]
fn humans_and_wearables_list_from_spatial_blueprint() {
    let state = spatial_blueprint_state();
    let humans = spanda_api::humans::humans_list(&state);
    assert_eq!(humans.status, 200);
    assert!(humans.body.contains("operator-001"));
    let wearables = spanda_api::humans::wearables_list(&state);
    assert_eq!(wearables.status, 200);
    assert!(wearables.body.contains("watch-001"));
    let policy = spanda_api::humans::human_health_policy(&state);
    assert_eq!(policy.status, 200);
    assert!(policy.body.contains("human_health"));
}

#[test]
fn humans_readiness_team_and_hri_context_from_blueprint() {
    let state = spatial_blueprint_state();
    let team = spanda_api::humans::humans_readiness_team(&state);
    assert_eq!(team.status, 200);
    assert!(team.body.contains("operator-001"));
    assert!(team.body.contains("team_readiness"));
    let collab = spanda_api::hri::hri_collaboration_graph(&state);
    assert_eq!(collab.status, 200);
    assert!(collab.body.contains("repair-session-001"));
    assert!(collab.body.contains("assignment"));
    let context = spanda_api::hri::hri_context_snapshot(&state);
    assert_eq!(context.status, 200);
    assert!(context.body.contains("warehouse-a-restricted"));
    assert!(context.body.contains("human_locations"));
}

//! API tests for runtime mission entity overlay.
//!
use spanda_api::entity_runtime::runtime_missions_from_program;
use spanda_api::state::ControlCenterState;
use spanda_config::EntityKind;
use std::path::PathBuf;

#[test]
fn entity_registry_includes_program_missions() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/warehouse-ar");
    let program = root.join("pick_mission.sd");
    if !program.exists() {
        return;
    }
    let mut state = ControlCenterState::new();
    state.config_path = Some(root.join("spanda.toml"));
    state.program_path = Some(program);
    state.reload_config().expect("config");
    let (parsed, _, _) =
        spanda_api::program::parse_program_file(state.program_path.as_ref().unwrap())
            .expect("program");
    let missions =
        runtime_missions_from_program(&parsed, state.resolved.as_ref().and_then(|r| r.fleet_id()));
    assert!(!missions.is_empty());
    let registry = state.entity_registry();
    assert!(registry
        .list()
        .iter()
        .any(|entity| entity.entity_type == EntityKind::Mission));
}

//! Runtime mission projection into the unified entity registry overlay.
//!
use spanda_ast::foundations::MissionDecl;
use spanda_ast::nodes::{Program, RobotDecl};
use spanda_config::{mission_entity_id, RuntimeMissionEntity};
use spanda_runtime::robotics::MissionRuntime;

/// Extract mission entities from a loaded program and optional fleet id.
pub fn runtime_missions_from_program(
    program: &Program,
    fleet_id: Option<&str>,
) -> Vec<RuntimeMissionEntity> {
    program
        .robots()
        .iter()
        .filter_map(|robot| runtime_mission_for_robot(robot, fleet_id))
        .collect()
}

fn runtime_mission_for_robot(
    robot: &RobotDecl,
    fleet_id: Option<&str>,
) -> Option<RuntimeMissionEntity> {
    let RobotDecl::RobotDecl { mission, .. } = robot;
    let mission = mission.as_ref()?;
    let MissionDecl::MissionDecl {
        name,
        duration_hours,
        steps,
        required_capabilities,
        ..
    } = mission;
    let runtime = MissionRuntime::new(name.clone(), steps.clone(), *duration_hours);
    let mission_name = name
        .as_deref()
        .filter(|value| !value.is_empty())
        .unwrap_or("default");
    Some(RuntimeMissionEntity {
        id: mission_entity_id(robot.name(), mission_name),
        name: mission_name.to_string(),
        robot_id: Some(robot.name().to_string()),
        fleet_id: fleet_id.map(String::from),
        mission_state: runtime.state.as_str().to_string(),
        step_index: runtime.step_index,
        current_step: runtime.steps.get(runtime.step_index).cloned(),
        steps: runtime.steps.clone(),
        required_capabilities: required_capabilities.clone(),
        approval_pending: false,
    })
}

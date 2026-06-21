//! Distributed fleet orchestration beyond in-process `spanda fleet run`.
//!
//! Builds coordination plans from program-level `fleet` declarations and peer-robot wiring,
//! then executes a round-robin mission coordination pass across fleet members.

use crate::ast::{Program, RobotDecl};
use crate::foundations::MissionDecl;
use crate::robotics_platform::{FleetDecl, FleetRegistry, MissionRuntime};
use serde::{Deserialize, Serialize};

/// Per-member coordination state during orchestration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetMemberState {
    pub robot_name: String,
    pub mission_name: Option<String>,
    pub mission_state: String,
    pub current_step: String,
    pub has_peer_link: bool,
}

/// Orchestration report for one fleet group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetOrchestrationReport {
    pub fleet_name: String,
    pub members: Vec<FleetMemberState>,
    pub coordination_mode: String,
    pub steps_advanced: u32,
}

/// Full orchestration result for a program.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetOrchestrationResult {
    pub program: String,
    pub fleets: Vec<FleetOrchestrationReport>,
    pub success: bool,
}

fn robot_by_name<'a>(robots: &'a [RobotDecl], name: &str) -> Option<&'a RobotDecl> {
    robots.iter().find(|r| r.name() == name)
}

fn mission_for_robot(robot: &RobotDecl) -> Option<MissionRuntime> {
    let RobotDecl::RobotDecl { mission, .. } = robot;
    let mission = mission.as_ref()?;
    let MissionDecl::MissionDecl {
        name,
        duration_hours,
        steps,
        ..
    } = mission;
    Some(MissionRuntime::new(
        name.clone(),
        steps.clone(),
        *duration_hours,
    ))
}

/// Build fleet registry from program declarations.
pub fn fleet_registry_from_program(program: &Program) -> FleetRegistry {
    let Program::Program { fleets, .. } = program;
    let mut registry = FleetRegistry::default();
    for fleet in fleets {
        let FleetDecl::FleetDecl { name, members, .. } = fleet;
        registry.register(name, members.clone());
    }
    registry
}

/// Orchestrate fleet members by advancing missions in round-robin order.
pub fn orchestrate_fleets(program: &Program, program_path: &str) -> FleetOrchestrationResult {
    // Coordinate declared fleet groups using each member robot's mission controller.
    //
    // Parameters:
    // - `program` — parsed Spanda program
    // - `program_path` — source path for reporting
    //
    // Returns:
    // Orchestration report with per-member mission states.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = orchestrate_fleets(&program, "fleet.sd");

    let Program::Program { fleets, robots, .. } = program;
    let mut reports = Vec::new();

    for fleet in fleets {
        let FleetDecl::FleetDecl { name, members, .. } = fleet;
        let mut member_states = Vec::new();
        let mut steps_advanced = 0u32;

        for member_name in members {
            let Some(robot) = robot_by_name(robots, member_name) else {
                member_states.push(FleetMemberState {
                    robot_name: member_name.clone(),
                    mission_name: None,
                    mission_state: "MissingRobot".into(),
                    current_step: String::new(),
                    has_peer_link: false,
                });
                continue;
            };
            let RobotDecl::RobotDecl {
                peer_robots, mission, ..
            } = robot;
            let mut runtime = mission_for_robot(robot);
            let (mission_name, mission_state, current_step) = if let Some(ref mut m) = runtime {
                m.start();
                let step = m.advance().unwrap_or_default();
                if !step.is_empty() {
                    steps_advanced += 1;
                }
                (
                    m.name.clone(),
                    m.state.as_str().to_string(),
                    step,
                )
            } else {
                (None, "NoMission".into(), String::new())
            };
            let _ = mission;
            member_states.push(FleetMemberState {
                robot_name: member_name.clone(),
                mission_name,
                mission_state,
                current_step,
                has_peer_link: !peer_robots.is_empty(),
            });
        }

        reports.push(FleetOrchestrationReport {
            fleet_name: name.clone(),
            members: member_states,
            coordination_mode: "round_robin_mission".into(),
            steps_advanced,
        });
    }

    let success = reports.iter().all(|r| {
        r.members.iter().all(|m| m.mission_state != "MissingRobot")
    });

    FleetOrchestrationResult {
        program: program_path.to_string(),
        fleets: reports,
        success,
    }
}

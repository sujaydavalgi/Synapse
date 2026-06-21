//! Distributed fleet orchestration beyond in-process `spanda fleet run`.
//!
//! Builds coordination plans from program-level `fleet` declarations and peer-robot wiring,
//! then executes a round-robin mission coordination pass across fleet members.

use crate::ast::{Program, RobotDecl};
use crate::comm::{CommBus, InMemoryCommBus, PeerRobotDecl, TransportKind};
use crate::foundations::MissionDecl;
use crate::robotics_platform::{FleetDecl, FleetRegistry, MissionRuntime};
use crate::runtime::RuntimeValue;
use serde::{Deserialize, Serialize};

/// Per-member coordination state during orchestration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetMemberState {
    pub robot_name: String,
    pub mission_name: Option<String>,
    pub mission_state: String,
    pub current_step: String,
    pub has_peer_link: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub peer_handoffs: Vec<String>,
}

/// One peer message delivered over the in-process fleet mesh bus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PeerDelivery {
    pub from_robot: String,
    pub to_robot: String,
    pub topic: String,
    pub step: String,
    pub delivered: bool,
}

/// Orchestration report for one fleet group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetOrchestrationReport {
    pub fleet_name: String,
    pub members: Vec<FleetMemberState>,
    pub coordination_mode: String,
    pub steps_advanced: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub peer_messages: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub peer_deliveries: Vec<PeerDelivery>,
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

fn peer_handoffs(member_name: &str, step: &str, peer_robots: &[PeerRobotDecl]) -> Vec<String> {
    if step.is_empty() || peer_robots.is_empty() {
        return Vec::new();
    }
    peer_robots
        .iter()
        .map(|peer| {
            let PeerRobotDecl::PeerRobotDecl { name, .. } = peer;
            format!("{member_name}->{name}:step={step}")
        })
        .collect()
}

fn deliver_peer_steps(
    mesh_bus: &mut InMemoryCommBus,
    from_robot: &str,
    step: &str,
    peer_robots: &[PeerRobotDecl],
) -> Vec<PeerDelivery> {
    if step.is_empty() || peer_robots.is_empty() {
        return Vec::new();
    }
    let mut deliveries = Vec::new();
    for peer in peer_robots {
        let PeerRobotDecl::PeerRobotDecl { name, .. } = peer;
        mesh_bus.register_robot(name);
        mesh_bus.publish_peer(
            name,
            "mission_step",
            RuntimeValue::String {
                value: step.to_string(),
            },
            TransportKind::Sim,
            Some(from_robot),
        );
        let path = format!("/{name}/mission_step");
        let delivered = mesh_bus.published_messages().iter().any(|message| {
            message.topic_path == path && message.source_id.as_deref() == Some(from_robot)
        });
        deliveries.push(PeerDelivery {
            from_robot: from_robot.to_string(),
            to_robot: name.clone(),
            topic: "mission_step".into(),
            step: step.to_string(),
            delivered,
        });
    }
    deliveries
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
        let mut peer_messages = Vec::new();
        let mut peer_deliveries = Vec::new();
        let mut mesh_bus = InMemoryCommBus::new();

        for member_name in members {
            mesh_bus.register_robot(member_name);
        }

        for member_name in members {
            let Some(robot) = robot_by_name(robots, member_name) else {
                member_states.push(FleetMemberState {
                    robot_name: member_name.clone(),
                    mission_name: None,
                    mission_state: "MissingRobot".into(),
                    current_step: String::new(),
                    has_peer_link: false,
                    peer_handoffs: Vec::new(),
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
            let handoffs = peer_handoffs(member_name, &current_step, peer_robots);
            peer_messages.extend(handoffs.clone());
            peer_deliveries.extend(deliver_peer_steps(
                &mut mesh_bus,
                member_name,
                &current_step,
                peer_robots,
            ));
            let _ = mission;
            member_states.push(FleetMemberState {
                robot_name: member_name.clone(),
                mission_name,
                mission_state,
                current_step,
                has_peer_link: !peer_robots.is_empty(),
                peer_handoffs: handoffs,
            });
        }

        let has_peer_link = member_states.iter().any(|m| m.has_peer_link);
        let coordination_mode = if !peer_deliveries.is_empty() {
            "peer_mesh_mission".into()
        } else if has_peer_link {
            "peer_round_robin_mission".into()
        } else {
            "round_robin_mission".into()
        };
        reports.push(FleetOrchestrationReport {
            fleet_name: name.clone(),
            members: member_states,
            coordination_mode,
            steps_advanced,
            peer_messages,
            peer_deliveries,
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

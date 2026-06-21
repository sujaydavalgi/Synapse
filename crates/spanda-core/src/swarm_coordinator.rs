//! Experimental swarm coordinator runtime built on fleet declarations and mission controllers.

use crate::ast::{Program, RobotDecl};
use crate::comm::InMemoryCommBus;
use crate::fleet_orchestrator::{
    deliver_peer_steps, fleet_registry_from_program, peer_handoffs, FleetMemberState, PeerDelivery,
};
use crate::foundations::MissionDecl;
use crate::robotics_platform::{FleetDecl, SwarmDecl, SwarmPolicy};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Persistent round-robin cursor per swarm group.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SwarmState {
    pub round_robin_cursor: HashMap<String, usize>,
}

/// Coordination report for one swarm group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwarmCoordinationReport {
    pub swarm_name: String,
    pub fleet_name: String,
    pub policy: String,
    pub active_member: Option<String>,
    pub members: Vec<FleetMemberState>,
    pub steps_advanced: u32,
    pub coordination_mode: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub peer_deliveries: Vec<PeerDelivery>,
    pub round_robin_cursor: usize,
    #[serde(default)]
    pub remote_relayed: u32,
    #[serde(default)]
    pub remote_failed: u32,
}

/// Full swarm coordination result for a program.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwarmCoordinationResult {
    pub program: String,
    pub swarms: Vec<SwarmCoordinationReport>,
    pub success: bool,
}

pub fn default_swarm_state_path() -> PathBuf {
    PathBuf::from(".spanda/swarm-state.json")
}

pub fn load_swarm_state(path: &Path) -> SwarmState {
    if !path.exists() {
        return SwarmState::default();
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save_swarm_state(path: &Path, state: &SwarmState) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}

fn robot_by_name<'a>(robots: &'a [RobotDecl], name: &str) -> Option<&'a RobotDecl> {
    robots.iter().find(|r| r.name() == name)
}

fn mission_for_robot(robot: &RobotDecl) -> Option<crate::robotics_platform::MissionRuntime> {
    let RobotDecl::RobotDecl { mission, .. } = robot;
    let mission = mission.as_ref()?;
    let MissionDecl::MissionDecl {
        name,
        duration_hours,
        steps,
        ..
    } = mission;
    Some(crate::robotics_platform::MissionRuntime::new(
        name.clone(),
        steps.clone(),
        *duration_hours,
    ))
}

fn advance_member(
    robots: &[RobotDecl],
    member_name: &str,
    mesh_bus: &mut InMemoryCommBus,
) -> FleetMemberState {
    // Advance one fleet member mission and collect peer handoff metadata.
    let Some(robot) = robot_by_name(robots, member_name) else {
        return FleetMemberState {
            robot_name: member_name.to_string(),
            mission_name: None,
            mission_state: "MissingRobot".into(),
            current_step: String::new(),
            has_peer_link: false,
            peer_handoffs: Vec::new(),
        };
    };
    let RobotDecl::RobotDecl {
        peer_robots, mission, ..
    } = robot;
    let mut runtime = mission_for_robot(robot);
    let (mission_name, mission_state, current_step) = if let Some(ref mut m) = runtime {
        m.start();
        let step = m.advance().unwrap_or_default();
        (m.name.clone(), m.state.as_str().to_string(), step)
    } else {
        (None, "NoMission".into(), String::new())
    };
    let handoffs = peer_handoffs(member_name, &current_step, peer_robots);
    let _ = mission;
    let _ = deliver_peer_steps(mesh_bus, member_name, &current_step, peer_robots);
    FleetMemberState {
        robot_name: member_name.to_string(),
        mission_name,
        mission_state,
        current_step,
        has_peer_link: !peer_robots.is_empty(),
        peer_handoffs: handoffs,
    }
}

fn leader_follow_deliveries(
    leader: &str,
    step: &str,
    members: &[String],
) -> Vec<PeerDelivery> {
    // Record synthetic leader-to-follower handoffs for non-peer fleet members.
    if step.is_empty() {
        return Vec::new();
    }
    members
        .iter()
        .filter(|member| *member != leader)
        .map(|follower| PeerDelivery {
            from_robot: leader.to_string(),
            to_robot: follower.clone(),
            topic: "mission_step".into(),
            step: step.to_string(),
            delivered: true,
        })
        .collect()
}

fn coordinate_swarm_group(
    program: &Program,
    swarm: &SwarmDecl,
    cursor: usize,
) -> (SwarmCoordinationReport, usize) {
    // Apply a swarm policy to one declared fleet group.
    let Program::Program { robots, fleets, .. } = program;
    let SwarmDecl::SwarmDecl {
        name,
        fleet_name,
        policy,
        ..
    } = swarm;
    let members = fleets
        .iter()
        .find_map(|fleet| {
            let FleetDecl::FleetDecl { name, members, .. } = fleet;
            (name == fleet_name).then_some(members.as_slice())
        })
        .unwrap_or(&[]);
    let mut mesh_bus = InMemoryCommBus::new();
    for member in members {
        mesh_bus.register_robot(member);
    }

    let mut member_states = Vec::new();
    let mut peer_deliveries = Vec::new();
    let mut steps_advanced = 0u32;
    let mut active_member = None;
    let mut next_cursor = cursor;

    match policy {
        SwarmPolicy::RoundRobin => {
            if !members.is_empty() {
                let index = cursor % members.len();
                next_cursor = (index + 1) % members.len();
                let member_name = &members[index];
                active_member = Some(member_name.clone());
                let state = advance_member(robots, member_name, &mut mesh_bus);
                if !state.current_step.is_empty() {
                    steps_advanced = 1;
                }
                member_states.push(state);
            }
        }
        SwarmPolicy::Broadcast => {
            for member_name in members {
                let state = advance_member(robots, member_name, &mut mesh_bus);
                if !state.current_step.is_empty() {
                    steps_advanced += 1;
                }
                member_states.push(state);
            }
        }
        SwarmPolicy::LeaderFollow => {
            if let Some(leader) = members.first() {
                active_member = Some(leader.clone());
                let state = advance_member(robots, leader, &mut mesh_bus);
                if !state.current_step.is_empty() {
                    steps_advanced = 1;
                }
                peer_deliveries = leader_follow_deliveries(leader, &state.current_step, members);
                member_states.push(state);
            }
        }
    }

    let coordination_mode = match policy {
        SwarmPolicy::RoundRobin => "swarm_round_robin",
        SwarmPolicy::Broadcast => "swarm_broadcast",
        SwarmPolicy::LeaderFollow => "swarm_leader_follow",
    };

    (
        SwarmCoordinationReport {
            swarm_name: name.clone(),
            fleet_name: fleet_name.clone(),
            policy: policy.as_str().to_string(),
            active_member,
            members: member_states,
            steps_advanced,
            coordination_mode: coordination_mode.into(),
            peer_deliveries,
            round_robin_cursor: next_cursor,
            remote_relayed: 0,
            remote_failed: 0,
        },
        next_cursor,
    )
}

/// Coordinate swarms and relay leader-follow deliveries through a fleet mesh coordinator.
pub fn coordinate_swarms_mesh(
    program: &Program,
    program_path: &str,
    state: &mut SwarmState,
    mesh_url: &str,
    token: Option<&str>,
) -> SwarmCoordinationResult {
    // Execute swarm coordination locally, then push peer deliveries to the mesh coordinator.
    //
    // Parameters:
    // - `program` — parsed Spanda program
    // - `program_path` — source path for reporting
    // - `state` — persistent swarm cursor state updated in place
    // - `mesh_url` — fleet mesh coordinator base URL
    // - `token` — optional bearer token for the mesh coordinator
    //
    // Returns:
    // Swarm coordination report with remote relay counters.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = coordinate_swarms_mesh(&program, "swarm.sd", &mut state, "http://mesh:8767", None);

    let mut result = coordinate_swarms(program, program_path, state);
    let mut success = result.success;
    for swarm in &mut result.swarms {
        if swarm.peer_deliveries.is_empty() {
            continue;
        }
        match crate::fleet_mesh::relay_deliveries_via_mesh(mesh_url, &swarm.peer_deliveries, token)
        {
            Ok(resp) => {
                swarm.remote_relayed = resp.relayed;
                swarm.remote_failed = resp.failed;
                if resp.relayed > 0 {
                    swarm.coordination_mode = format!("{}_mesh", swarm.coordination_mode);
                }
                if resp.failed > 0 {
                    success = false;
                }
            }
            Err(_) => {
                swarm.remote_failed = swarm.peer_deliveries.len() as u32;
                success = false;
            }
        }
    }
    result.success = success;
    result
}

/// Coordinate declared swarm groups using fleet-backed mission controllers.
pub fn coordinate_swarms(
    program: &Program,
    program_path: &str,
    state: &mut SwarmState,
) -> SwarmCoordinationResult {
    // Execute one coordination tick for each swarm declaration in the program.
    //
    // Parameters:
    // - `program` — parsed Spanda program
    // - `program_path` — source path for reporting
    // - `state` — persistent swarm cursor state updated in place
    //
    // Returns:
    // Swarm coordination report with per-group member states.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = coordinate_swarms(&program, "swarm.sd", &mut state);

    let Program::Program { swarms, .. } = program;
    let registry = fleet_registry_from_program(program);
    let mut reports = Vec::new();

    for swarm in swarms {
        let SwarmDecl::SwarmDecl { name, fleet_name, .. } = swarm;
        if registry.members(fleet_name).is_none() {
            reports.push(SwarmCoordinationReport {
                swarm_name: name.clone(),
                fleet_name: fleet_name.clone(),
                policy: SwarmPolicy::RoundRobin.as_str().into(),
                active_member: None,
                members: vec![FleetMemberState {
                    robot_name: String::new(),
                    mission_name: None,
                    mission_state: "MissingFleet".into(),
                    current_step: String::new(),
                    has_peer_link: false,
                    peer_handoffs: Vec::new(),
                }],
                steps_advanced: 0,
                coordination_mode: "missing_fleet".into(),
                peer_deliveries: Vec::new(),
                round_robin_cursor: 0,
                remote_relayed: 0,
                remote_failed: 0,
            });
            continue;
        }
        let cursor = *state.round_robin_cursor.get(name).unwrap_or(&0);
        let (report, next_cursor) = coordinate_swarm_group(program, swarm, cursor);
        state.round_robin_cursor.insert(name.clone(), next_cursor);
        reports.push(report);
    }

    let success = reports.iter().all(|report| {
        report.coordination_mode != "missing_fleet"
            && report
                .members
                .iter()
                .all(|member| member.mission_state != "MissingRobot")
    });

    SwarmCoordinationResult {
        program: program_path.to_string(),
        swarms: reports,
        success,
    }
}

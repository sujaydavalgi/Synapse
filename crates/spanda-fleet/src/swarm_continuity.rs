//! Swarm member continuity handoff planning and mesh relay.

use serde::{Deserialize, Serialize};
use spanda_runtime::assurance_runtime::platform_assurance_runtime;
use spanda_runtime::{ContinuityContext, ContinuityTrigger, SuccessionScope};
use spanda_ast::nodes::Program;
use spanda_deploy_http::FleetContinuityRequest;

use crate::swarm_coordinator::SwarmCoordinationResult;

/// Continuity handoff planned when a swarm member is lost.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwarmContinuityHandoff {
    pub swarm_name: String,
    pub failed_member: String,
    pub successor: String,
    pub mission: String,
    pub progress_percent: f64,
    pub mode: String,
    pub relayed: u32,
    pub failed: u32,
}

/// Plan takeover when a swarm member goes offline.
pub fn plan_swarm_member_continuity(
    program: &Program,
    swarm_name: &str,
    failed_member: &str,
    progress_percent: f64,
    mission: Option<&str>,
) -> Option<SwarmContinuityHandoff> {
    let context = ContinuityContext {
        mission: mission.unwrap_or("default_mission").into(),
        failed_entity: failed_member.into(),
        trigger: ContinuityTrigger::SwarmMemberLost,
        progress_percent,
        scope: SuccessionScope::Swarm,
        current_step: None,
        checkpoints: Vec::new(),
    };
    let report = platform_assurance_runtime().plan_takeover(program, &context, None);
    if !report.succeeded {
        return None;
    }
    Some(SwarmContinuityHandoff {
        swarm_name: swarm_name.into(),
        failed_member: failed_member.into(),
        successor: report.successor,
        mission: report.mission,
        progress_percent,
        mode: format!("{:?}", report.mode),
        relayed: 0,
        failed: 0,
    })
}

/// Attach continuity handoff reports to swarm coordination output.
pub fn attach_swarm_continuity_handoff(
    program: &Program,
    result: &mut SwarmCoordinationResult,
    failed_member: &str,
    progress_percent: f64,
    mission: Option<&str>,
) {
    for swarm_report in &mut result.swarms {
        if !swarm_report
            .members
            .iter()
            .any(|member| member.robot_name == failed_member)
        {
            continue;
        }
        if let Some(handoff) = plan_swarm_member_continuity(
            program,
            &swarm_report.swarm_name,
            failed_member,
            progress_percent,
            mission,
        ) {
            swarm_report.continuity_handoff = Some(handoff);
        }
    }
}

/// Build a mesh continuity request from a swarm handoff report.
pub fn continuity_request_from_handoff(
    handoff: &SwarmContinuityHandoff,
    fleet_name: &str,
    members: &[String],
) -> FleetContinuityRequest {
    FleetContinuityRequest {
        failed_robot: handoff.failed_member.clone(),
        successor: Some(handoff.successor.clone()),
        mission: Some(handoff.mission.clone()),
        progress_percent: Some(handoff.progress_percent),
        trigger: Some("swarm_member_lost".into()),
        fleet_name: Some(fleet_name.into()),
        from_robot: None,
        members: members.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spanda_lexer::tokenize;
    use spanda_parser::parse;

    #[test]
    fn swarm_continuity_plans_successor_for_lost_member() {
        let source = include_str!("../../../examples/showcase/swarm_takeover/swarm.sd");
        let program = parse(tokenize(source).unwrap()).unwrap();
        let handoff = plan_swarm_member_continuity(
            &program,
            "ReconSwarm",
            "DroneTwo",
            55.0,
            Some("default_mission"),
        )
        .expect("handoff planned");
        assert_eq!(handoff.failed_member, "DroneTwo");
        assert!(!handoff.successor.is_empty());
    }
}

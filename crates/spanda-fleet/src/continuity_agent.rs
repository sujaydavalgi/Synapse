//! Assurance-backed and interpreter-backed takeover execution on fleet agents.

use crate::agent::{apply_continuity_takeover, FleetAgentState};
use spanda_deploy_http::FleetContinuityRequest;
use spanda_interpreter::{execute_continuity_on_program, ContinuityRunOptions};
use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_runtime::assurance_runtime::platform_assurance_runtime;
use spanda_runtime::{
    ContinuityContext, ContinuityEvidence, ContinuityTrigger, ContinuationDecision,
    MissionStateSnapshot, MissionStateTransfer, SuccessionScope, TakeoverMode, TakeoverReport,
};
use spanda_runtime::security_runtime::default_security_runtime_factory;
use spanda_transport_routing::runtime_bridge::routing_comm_bus_factory_fn;

fn parse_takeover_request(step: &str) -> Result<FleetContinuityRequest, String> {
    serde_json::from_str::<FleetContinuityRequest>(step).map_err(|e| e.to_string())
}

fn continuity_context_from_request(request: &FleetContinuityRequest) -> ContinuityContext {
    let trigger = request
        .trigger
        .as_deref()
        .map(|s| platform_assurance_runtime().parse_trigger(s))
        .unwrap_or(ContinuityTrigger::RobotFailed);
    ContinuityContext {
        mission: request
            .mission
            .clone()
            .unwrap_or_else(|| "default_mission".into()),
        failed_entity: request.failed_robot.clone(),
        trigger,
        progress_percent: request.progress_percent.unwrap_or(0.0),
        scope: SuccessionScope::Fleet,
        current_step: None,
        checkpoints: Vec::new(),
    }
}

fn sync_agent_from_interpreter(
    state: &mut FleetAgentState,
    outcome: &spanda_interpreter::ContinuityRunResult,
    request: &FleetContinuityRequest,
) {
    apply_continuity_takeover(state, &outcome.takeover, request);
    state.continuity_engine = Some("interpreter".into());
    state.continuity_validation = Some(if outcome.takeover.succeeded {
        "PASS".into()
    } else {
        "FAIL".into()
    });
    state.last_continuity_runtime_logs = outcome.logs.clone();
    state.last_continuity_evidence = serde_json::to_value(&outcome.takeover.evidence).ok();
}

/// Run live interpreter takeover dispatch on a deployed agent program.
pub fn execute_interpreter_continuity_on_agent(
    state: &mut FleetAgentState,
    program_source: &str,
    request: &FleetContinuityRequest,
) -> Result<TakeoverReport, String> {
    let tokens = tokenize(program_source).map_err(|e| e.to_string())?;
    let program = parse(tokens).map_err(|e| e.to_string())?;
    let context = continuity_context_from_request(request);
    let robot_name = (!state.robot_name.is_empty()).then(|| state.robot_name.clone());
    let outcome = execute_continuity_on_program(
        &program,
        &context,
        ContinuityRunOptions {
            robot_name,
            successor: request.successor.clone(),
            grant_operator_approval: std::env::var("SPANDA_OPERATOR_APPROVAL")
                .ok()
                .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true")),
            inbound_comm_messages: Vec::new(),
            assurance_runtime: Some(platform_assurance_runtime()),
            security_runtime_factory: Some(default_security_runtime_factory()),
            comm_bus_factory: Some(routing_comm_bus_factory_fn()),
        },
    )
    .map_err(|e| e.to_string())?;
    sync_agent_from_interpreter(state, &outcome, request);
    Ok(outcome.takeover)
}

/// Run assurance-only takeover planning and apply state on the agent.
pub fn execute_assurance_continuity_on_agent(
    state: &mut FleetAgentState,
    program_source: &str,
    request: &FleetContinuityRequest,
) -> Result<TakeoverReport, String> {
    let tokens = tokenize(program_source).map_err(|e| e.to_string())?;
    let program = parse(tokens).map_err(|e| e.to_string())?;
    let context = continuity_context_from_request(request);
    let report = platform_assurance_runtime().plan_takeover(&program, &context, request.successor.as_deref());
    apply_continuity_takeover(state, &report, request);
    state.continuity_engine = Some("assurance".into());
    state.continuity_validation = Some(if report.succeeded {
        "PASS".into()
    } else {
        "FAIL".into()
    });
    state.last_continuity_evidence = serde_json::to_value(&report.evidence).ok();
    Ok(report)
}

/// Handle an inbound fleet takeover peer command on a deployed agent.
pub fn handle_fleet_takeover_command(state: &mut FleetAgentState, step: &str) {
    let Ok(request) = parse_takeover_request(step) else {
        return;
    };
    state
        .last_continuity_commands
        .push(serde_json::to_string(&request).unwrap_or_else(|_| step.to_string()));

    if let Some(program) = state.program.clone() {
        if execute_interpreter_continuity_on_agent(state, &program, &request).is_ok() {
            return;
        }
        if execute_assurance_continuity_on_agent(state, &program, &request).is_ok() {
            return;
        }
    }

    state.continuity_engine = Some("fallback".into());
    apply_continuity_takeover(
        state,
        &TakeoverReport {
            mission: request.mission.clone().unwrap_or_default(),
            failed_entity: request.failed_robot.clone(),
            successor: request
                .successor
                .clone()
                .unwrap_or_else(|| "NoSuccessor".into()),
            mode: TakeoverMode::Resume,
            decision: ContinuationDecision::Continue,
            state_transfer: MissionStateTransfer {
                from_entity: request.failed_robot.clone(),
                to_entity: request.successor.clone().unwrap_or_default(),
                snapshot: MissionStateSnapshot {
                    mission: request.mission.clone().unwrap_or_default(),
                    completed_steps: Vec::new(),
                    current_goal: None,
                    progress_percent: request.progress_percent.unwrap_or(0.0),
                    checkpoints: Vec::new(),
                },
                transferable: true,
                transfer_notes: vec!["fallback takeover".into()],
            },
            safety_gates: Vec::new(),
            evidence: ContinuityEvidence {
                takeover_evidence: vec!["fallback".into()],
                delegation_evidence: Vec::new(),
                continuity_evidence: Vec::new(),
                safety_gates: Vec::new(),
                diagnosis: None,
                recovery_outcome: None,
            },
            succeeded: true,
            diagnosis: "fallback takeover".into(),
        },
        &request,
    );
    state.continuity_validation = Some("PARTIAL".into());
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLEET_PROGRAM: &str = r#"
hardware RoverV1 {
    sensors [GPS];
    actuators [DifferentialDrive];
    connectivity [WiFi];
}
continuity_policy FleetContinuity {
    on robot.failed { resume from checkpoint; reassign mission; }
}
fleet PatrolFleet { RoverAlpha; RoverBeta; }
robot RoverBeta {
    sensor gps: GPS;
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }
    behavior patrol() { wheels.drive(0.3 m/s); }
}
"#;

    #[test]
    fn interpreter_takeover_applies_on_successor_agent() {
        let mut state = FleetAgentState {
            robot_name: "RoverBeta".into(),
            program: Some(FLEET_PROGRAM.into()),
            ..Default::default()
        };
        let request = FleetContinuityRequest {
            failed_robot: "RoverAlpha".into(),
            successor: Some("RoverBeta".into()),
            mission: Some("Patrol".into()),
            progress_percent: Some(72.0),
            trigger: Some("robot_failed".into()),
            fleet_name: None,
            from_robot: Some("RoverAlpha".into()),
            members: vec!["RoverBeta".into()],
        };
        execute_interpreter_continuity_on_agent(&mut state, FLEET_PROGRAM, &request)
            .expect("takeover");
        assert_eq!(state.continuity_engine.as_deref(), Some("interpreter"));
        assert_eq!(state.continuity_successor.as_deref(), Some("RoverBeta"));
        assert!(!state.mission_paused);
    }
}

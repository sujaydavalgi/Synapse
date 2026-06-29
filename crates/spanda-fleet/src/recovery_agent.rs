//! Assurance-backed and interpreter-backed recovery execution on fleet agents.

use crate::agent::{apply_recovery_action, FleetAgentState};
use spanda_interpreter::{execute_recovery_on_program, RecoveryRunOptions, RecoveryRunResult};
use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_runtime::assurance_runtime::platform_assurance_runtime;
use spanda_runtime::recovery_primitives::{classify_failure, extract_recovery_policies};
use spanda_runtime::{RecoveryContext, RecoveryLevel, RecoveryReport, RecoveryStatus};
use spanda_runtime::security_runtime::default_security_runtime_factory;
use spanda_transport_routing::runtime_bridge::routing_comm_bus_factory_fn;

fn normalize_action(action: &str) -> String {
    // Description:
    //     Normalize action.
    //
    // Inputs:
    //     action: &str
    //         Caller-supplied action.
    //
    // Outputs:
    //     result: String
    //         Return value from `normalize_action`.
    //
    // Example:

    //     let result = spanda_fleet::recovery_agent::normalize_action(action);

    action
        .to_ascii_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

fn infer_recovery_issue(action: &str) -> String {
    // Description:
    //     Infer recovery issue.
    //
    // Inputs:
    //     action: &str
    //         Caller-supplied action.
    //
    // Outputs:
    //     result: String
    //         Return value from `infer_recovery_issue`.
    //
    // Example:

    //     let result = spanda_fleet::recovery_agent::infer_recovery_issue(action);

    let lower = action.to_ascii_lowercase();
    if lower.contains("gps") {
        return "gps.failed".into();
    }
    if lower.contains("lidar") {
        return "lidar.failed".into();
    }
    if lower.contains("lte") || lower.contains("wifi") || lower.contains("connect") {
        return "connectivity.failed".into();
    }
    if lower.contains("fleet")
        || lower.contains("reassign")
        || lower.contains("promote")
        || lower.contains("redistribute")
    {
        return "fleet.failed".into();
    }
    if lower.contains("mission") {
        return "mission.failed".into();
    }
    "fleet.failed".into()
}

fn recovery_issue_for_action(program: &spanda_ast::nodes::Program, action: &str) -> String {
    // Description:
    //     Recovery issue for action.
    //
    // Inputs:
    //     progra: &spanda_ast::nodes::Program
    //         Caller-supplied progra.
    //     action: &str
    //         Caller-supplied action.
    //
    // Outputs:
    //     result: String
    //         Return value from `recovery_issue_for_action`.
    //
    // Example:

    //     let result = spanda_fleet::recovery_agent::recovery_issue_for_action(progra, action);

    let normalized = normalize_action(action);
    for policy in extract_recovery_policies(program) {
        for (condition, actions) in &policy.triggers {
            for branch_action in actions {
                let branch_norm = normalize_action(branch_action);
                if branch_norm == normalized
                    || normalized.contains(&branch_norm)
                    || branch_norm.contains(&normalized)
                {
                    return condition.clone();
                }
            }
        }
    }
    infer_recovery_issue(action)
}

fn operator_approval_enabled() -> bool {
    // Description:
    //     Operator approval enabled.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: bool
    //         Return value from `operator_approval_enabled`.
    //
    // Example:

    //     let result = spanda_fleet::recovery_agent::operator_approval_enabled();

    std::env::var("SPANDA_OPERATOR_APPROVAL")
        .ok()
        .is_some_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
}

fn sync_agent_from_interpreter(state: &mut FleetAgentState, outcome: &RecoveryRunResult) {
    // Description:
    //     Sync agent from interpreter.
    //
    // Inputs:
    //     state: &mut FleetAgentState
    //         Caller-supplied state.
    //     outcome: &RecoveryRunResult
    //         Caller-supplied outcome.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_fleet::recovery_agent::sync_agent_from_interpreter(state, outcome);

    if let Some(last) = outcome.recovery.executed_actions.last() {
        state.recovery_active = Some(last.clone());
    }
    state
        .recovery_actions_applied
        .extend(outcome.recovery.executed_actions.clone());
    state.mission_paused = outcome.mission_paused;
    if outcome.active_mode != "normal" {
        state.recovery_mode = Some(outcome.active_mode.clone());
    }
    state.recovery_speed_cap = outcome.speed_cap;
    state.recovery_engine = Some("interpreter".into());
    state.last_recovery_runtime_logs = outcome.logs.clone();
}

fn validation_label_for_recovery(status: RecoveryStatus, report_passed: bool) -> &'static str {
    // Description:
    //     Validation label for recovery.
    //
    // Inputs:
    //     status: RecoveryStatus
    //         Caller-supplied status.
    //     report_passed: bool
    //         Caller-supplied report passed.
    //
    // Outputs:
    //     result: &'static str
    //         Return value from `validation_label_for_recovery`.
    //
    // Example:

    //     let result = spanda_fleet::recovery_agent::validation_label_for_recovery(status, report_passed);

    match status {
        RecoveryStatus::Success if report_passed => "PASS",
        RecoveryStatus::Success | RecoveryStatus::PartialSuccess => "PARTIAL",
        _ => "FAIL",
    }
}

/// Run live interpreter recovery dispatch on a deployed agent program.
pub fn execute_interpreter_recovery_on_agent(
    state: &mut FleetAgentState,
    program_source: &str,
    trigger_action: &str,
) -> Result<RecoveryReport, String> {
    // Description:
    //     Execute interpreter recovery on agent.
    //
    // Inputs:
    //     state: &mut FleetAgentState
    //         Caller-supplied state.
    //     program_source: &str
    //         Caller-supplied program source.
    //     rigger_action: &str
    //         Caller-supplied rigger action.
    //
    // Outputs:
    //     result: Result<RecoveryReport, String>
    //         Return value from `execute_interpreter_recovery_on_agent`.
    //
    // Example:

    //     let result = spanda_fleet::recovery_agent::execute_interpreter_recovery_on_agent(state, program_source, rigger_action);

    let tokens = tokenize(program_source).map_err(|e| e.to_string())?;
    let program = parse(tokens).map_err(|e| e.to_string())?;
    let issue = recovery_issue_for_action(&program, trigger_action);
    let robot_name = (!state.robot_name.is_empty()).then(|| state.robot_name.clone());
    let outcome = execute_recovery_on_program(
        &program,
        &issue,
        RecoveryRunOptions {
            robot_name,
            grant_operator_approval: operator_approval_enabled(),
            inbound_comm_messages: Vec::new(),
            assurance_runtime: Some(platform_assurance_runtime()),
            security_runtime_factory: Some(default_security_runtime_factory()),
            comm_bus_factory: Some(routing_comm_bus_factory_fn()),
        },
    )
    .map_err(|e| e.to_string())?;
    sync_agent_from_interpreter(state, &outcome);
    let report = platform_assurance_runtime().evaluate_recovery_program(&program);
    state.recovery_validation =
        Some(validation_label_for_recovery(outcome.recovery.status, report.passed).into());
    state.last_recovery_evidence = serde_json::to_value(&outcome.recovery.evidence).ok();
    Ok(report)
}

/// Run assurance-only recovery planning and validation, then apply gated actions on the agent.
pub fn execute_assurance_recovery_on_agent(
    state: &mut FleetAgentState,
    program_source: &str,
    trigger_action: &str,
) -> Result<RecoveryReport, String> {
    // Description:
    //     Execute assurance recovery on agent.
    //
    // Inputs:
    //     state: &mut FleetAgentState
    //         Caller-supplied state.
    //     program_source: &str
    //         Caller-supplied program source.
    //     rigger_action: &str
    //         Caller-supplied rigger action.
    //
    // Outputs:
    //     result: Result<RecoveryReport, String>
    //         Return value from `execute_assurance_recovery_on_agent`.
    //
    // Example:

    //     let result = spanda_fleet::recovery_agent::execute_assurance_recovery_on_agent(state, program_source, rigger_action);

    let tokens = tokenize(program_source).map_err(|e| e.to_string())?;
    let program = parse(tokens).map_err(|e| e.to_string())?;
    let issue = recovery_issue_for_action(&program, trigger_action);
    let context = RecoveryContext {
        issue: issue.clone(),
        diagnosis: None,
        classification: Some(classify_failure(&issue)),
        level: RecoveryLevel::Level3AutomaticWithValidation,
    };
    let rt = platform_assurance_runtime();
    let plan = rt.plan_recovery(&program, &context);
    let safe_actions = rt.validate_recovery_plan(&program, &plan);
    let mut executed_any = false;

    for safe in &safe_actions {
        let gates_ok = safe.safety_validation.passed
            && safe.hardware_verification.passed
            && safe.capability_verification.passed
            && safe.readiness_validation.passed;
        if !gates_ok {
            continue;
        }
        if safe.action.requires_approval && !safe.approved {
            continue;
        }
        apply_recovery_action(state, &safe.action.description);
        executed_any = true;
    }

    if !executed_any {
        apply_recovery_action(state, trigger_action);
    }

    let report = platform_assurance_runtime().evaluate_recovery_program(&program);
    state.recovery_engine = Some("assurance".into());
    state.recovery_validation = Some(if report.passed {
        "PASS".into()
    } else {
        "FAIL".into()
    });
    state.last_recovery_evidence = serde_json::to_value(&report.results).ok();
    Ok(report)
}

/// Handle an inbound fleet recovery peer command on a deployed agent.
pub fn handle_fleet_recovery_command(state: &mut FleetAgentState, action: &str) {
    // Description:
    //     Handle fleet recovery command.
    //
    // Inputs:
    //     state: &mut FleetAgentState
    //         Caller-supplied state.
    //     action: &str
    //         Caller-supplied action.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_fleet::recovery_agent::handle_fleet_recovery_command(state, action);

    state.last_recovery_commands.push(action.to_string());
    if let Some(program) = state.program.clone() {
        if execute_interpreter_recovery_on_agent(state, &program, action).is_ok() {
            return;
        }
        if execute_assurance_recovery_on_agent(state, &program, action).is_ok() {
            return;
        }
    }
    state.recovery_engine = Some("fallback".into());
    apply_recovery_action(state, action);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::FleetAgentState;

    const FLEET_PROGRAM: &str = r#"
recovery_policy FleetRecovery {
    on fleet.failed {
        enter degraded_mode;
        reduce_speed 0.5 m/s;
    }
}
robot RoverAlpha {
    sensor gps: GPS;
    actuator wheels: DifferentialDrive;
    mode degraded { }
    safety { max_speed = 1.0 m/s; }
    behavior patrol() {}
}
"#;

    #[test]
    fn interpreter_recovery_applies_runtime_dispatch() {
        // Description:
        //     Interpreter recovery applies runtime dispatch.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_fleet::recovery_agent::interpreter_recovery_applies_runtime_dispatch();

        std::env::set_var("SPANDA_OPERATOR_APPROVAL", "1");
        let mut state = FleetAgentState {
            robot_name: "RoverAlpha".into(),
            program: Some(FLEET_PROGRAM.into()),
            ..FleetAgentState::default()
        };
        handle_fleet_recovery_command(&mut state, "enter degraded_mode");
        assert_eq!(state.recovery_engine.as_deref(), Some("interpreter"));
        assert!(matches!(
            state.recovery_validation.as_deref(),
            Some("PASS") | Some("PARTIAL")
        ));
        assert_eq!(state.recovery_mode.as_deref(), Some("degraded"));
        std::env::remove_var("SPANDA_OPERATOR_APPROVAL");
    }

    #[test]
    fn assurance_recovery_applies_validated_actions() {
        // Description:
        //     Assurance recovery applies validated actions.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_fleet::recovery_agent::assurance_recovery_applies_validated_actions();

        let program = r#"
recovery_policy FleetRecovery {
    on fleet.failed { pause mission; }
}
robot RoverAlpha {
    sensor gps: GPS;
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }
    behavior patrol() {}
}
"#
        .to_string();
        let mut state = FleetAgentState {
            robot_name: "RoverAlpha".into(),
            program: Some(program.clone()),
            ..FleetAgentState::default()
        };
        execute_assurance_recovery_on_agent(&mut state, &program, "pause mission")
            .expect("assurance recovery");
        assert!(state.mission_paused);
        assert_eq!(state.recovery_engine.as_deref(), Some("assurance"));
    }
}

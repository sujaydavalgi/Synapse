//! Runtime recovery action dispatch, operator approval, and fleet coordination.

use super::super::super::fleet_http::{
    relay_continuity_via_mesh, relay_recovery_via_mesh, FleetContinuityRequest,
    FleetRecoveryRequest,
};
use super::super::super::options::{RecoveryRunOptions, RecoveryRunResult};
use super::super::super::simulator::{create_default_simulator, SimulatorConfig};
use super::{Interpreter, RobotBackend};
use serde::{Deserialize, Serialize};
use spanda_assurance::{
    classify_failure, default_knowledge_store_path, load_recovery_knowledge_store,
    merge_recovery_knowledge, record_recovery_outcome, save_recovery_knowledge_store,
    validate_recovery_plan, RecoveryContext, RecoveryLevel, RecoveryPlanner, RecoveryResult,
    RecoveryStatus,
};
use spanda_ast::nodes::{Program, RobotDecl};
use spanda_comm::CommBus;
use spanda_error::SpandaError;
use spanda_runtime::robotics::MissionState;
use spanda_runtime::value::RuntimeValue;
use std::cell::RefCell;
use std::rc::Rc;

fn action_triggers_continuity_handoff(action: &str) -> bool {
    // Description:
    //     Decide whether a recovery action should also relay fleet continuity.
    //
    // Inputs:
    //     action: &str
    //         Recovery action text from a policy branch.
    //
    // Outputs:
    //     result: bool
    //         True when the action reassigns or promotes fleet mission work.
    //
    // Example:
    //     let handoff = action_triggers_continuity_handoff("reassign mission");

    let lower = action.to_ascii_lowercase();
    lower.contains("reassign")
        || lower.contains("promote")
        || lower.contains("replace")
        || lower.contains("redistribute")
}

fn parse_speed_cap(action: &str) -> Option<f64> {
    // Description:
    //     Parse speed cap.
    //
    // Inputs:
    //     action: &str
    //         Caller-supplied action.
    //
    // Outputs:
    //     result: Option<f64>
    //         Return value from `parse_speed_cap`.
    //
    // Example:

    //     let result = spanda_interpreter::runtime_recovery::parse_speed_cap(action);

    action
        .split_whitespace()
        .find_map(|part| part.parse::<f64>().ok())
}

fn normalize_mode_name(action: &str) -> Option<&'static str> {
    // Description:
    //     Normalize mode name.
    //
    // Inputs:
    //     action: &str
    //         Caller-supplied action.
    //
    // Outputs:
    //     result: Option<&'static str>
    //         Return value from `normalize_mode_name`.
    //
    // Example:

    //     let result = spanda_interpreter::runtime_recovery::normalize_mode_name(action);

    let lower = action.to_lowercase();
    if lower.contains("degraded") {
        Some("degraded")
    } else if lower.contains("safe") {
        Some("safe")
    } else if lower.contains("recovery") {
        Some("recovery")
    } else if lower.contains("emergency") {
        Some("emergency")
    } else if lower.contains("normal") {
        Some("normal")
    } else {
        None
    }
}

impl<B: RobotBackend> Interpreter<B> {
    /// Return true when operator approval is granted for a recovery action.
    pub(super) fn operator_approval_granted(&self, action: &str) -> bool {
        // Description:
        //     Operator approval granted.
        //
        // Inputs:
        //     &self: value
        //         Caller-supplied &self.
        //     action: &str
        //         Caller-supplied action.
        //
        // Outputs:
        //     result: bool
        //         Return value from `operator_approval_granted`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_recovery::operator_approval_granted(&self, action);

        if self.granted_recovery_approvals.contains(action) {
            return true;
        }
        if let Ok(value) = std::env::var("SPANDA_OPERATOR_APPROVAL") {
            if value == "1" || value.eq_ignore_ascii_case("true") {
                return true;
            }
            if action.to_lowercase().contains(&value.to_lowercase()) {
                return true;
            }
        }
        if let Ok(value) = std::env::var("SPANDA_GRANT_RECOVERY_APPROVAL") {
            if action.to_lowercase().contains(&value.to_lowercase()) {
                return true;
            }
        }
        false
    }

    /// Record that operator approval is required before executing an action.
    pub(super) fn request_operator_approval(&mut self, action: &str) {
        // Description:
        //     Request operator approval.
        //
        // Inputs:
        //     &mut self: value
        //         Caller-supplied &mut self.
        //     action: &str
        //         Caller-supplied action.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_recovery::request_operator_approval(&mut self, action);

        self.pending_recovery_approvals.insert(action.to_string());
        self.log(format!(
            "recovery: operator approval required for '{action}'"
        ));
        self.record_mission_event(
            "recovery_approval_required",
            serde_json::json!({ "action": action }),
        );
    }

    /// Grant operator approval for a pending recovery action (comm topic / API hook).
    pub(super) fn grant_operator_approval(&mut self, action: &str) {
        // Description:
        //     Grant operator approval.
        //
        // Inputs:
        //     &mut self: value
        //         Caller-supplied &mut self.
        //     action: &str
        //         Caller-supplied action.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_recovery::grant_operator_approval(&mut self, action);

        self.pending_recovery_approvals.remove(action);
        self.granted_recovery_approvals.insert(action.to_string());
        self.log(format!(
            "recovery: operator approval granted for '{action}'"
        ));
        self.record_mission_event(
            "recovery_approval_granted",
            serde_json::json!({ "action": action }),
        );
    }

    /// Dispatch a single recovery action string at runtime.
    pub(super) fn dispatch_recovery_action(&mut self, action: &str) -> Result<bool, SpandaError> {
        // Description:
        //     Dispatch recovery action.
        //
        // Inputs:
        //     &mut self: value
        //         Caller-supplied &mut self.
        //     action: &str
        //         Caller-supplied action.
        //
        // Outputs:
        //     result: Result<bool, SpandaError>
        //         Return value from `dispatch_recovery_action`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_recovery::dispatch_recovery_action(&mut self, action);

        let lower = action.to_lowercase();

        if let Some(mode) = normalize_mode_name(action) {
            if lower.contains("enter") || lower.contains("mode") || lower.contains(mode) {
                self.enter_mode(mode)?;
                return Ok(true);
            }
        }

        if lower.contains("reduce_speed") {
            if let Some(cap) = parse_speed_cap(action) {
                if let Some(monitor) = &mut self.safety_monitor {
                    monitor.apply_speed_cap(cap);
                }
                self.recovery_speed_cap = Some(cap);
                self.log(format!("recovery: speed cap set to {cap} m/s"));
                return Ok(true);
            }
        }

        if lower.contains("restart") && lower.contains("connect") {
            self.restart_active_connectivity()?;
            return Ok(true);
        }

        if lower.contains("pause") && lower.contains("mission") {
            self.pause_active_mission();
            return Ok(true);
        }

        if lower.contains("reassign")
            || lower.contains("redistribute")
            || lower.contains("promote")
            || lower.contains("replace")
        {
            self.coordinate_fleet_recovery(action)?;
            return Ok(true);
        }

        if lower.contains("halt") || lower.contains("emergency_stop") || lower.contains("stop") {
            if let Some(monitor) = &mut self.safety_monitor {
                monitor.set_emergency_stop(true);
            }
            self.backend.set_emergency_stop(true);
            return Ok(true);
        }

        self.log(format!("recovery: recorded action '{action}'"));
        Ok(true)
    }

    /// Poll Approval topics and environment for operator grants.
    pub(super) fn poll_recovery_approvals(&mut self) {
        // Description:
        //     Poll recovery approvals.
        //
        // Inputs:
        //     &mut self: value
        //         Caller-supplied &mut self.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_recovery::poll_recovery_approvals(&mut self);

        for (path, text) in &self.options.inbound_comm_messages {
            self.comm_bus.push_inbound(
                path,
                RuntimeValue::String {
                    value: text.clone(),
                },
                None,
            );
        }

        let approval_topics: Vec<String> = self
            .topic_path_to_message_type
            .iter()
            .filter(|(_, message_type)| message_type.as_str() == "Approval")
            .map(|(path, _)| path.clone())
            .collect();

        for path in approval_topics {
            while let Some(envelope) = self.comm_bus.receive_envelope(&path) {
                if let RuntimeValue::String { value } = envelope.value {
                    self.grant_operator_approval(&value);
                } else {
                    for pending in self.pending_recovery_approvals.clone() {
                        self.grant_operator_approval(&pending);
                    }
                }
            }
        }

        for pending in self.pending_recovery_approvals.clone() {
            if self.operator_approval_granted(&pending) {
                self.grant_operator_approval(&pending);
            }
        }
    }

    /// Return true when a mission step or action requires operator approval.
    pub(super) fn mission_action_requires_approval(&self, action: &str) -> bool {
        // Description:
        //     Mission action requires approval.
        //
        // Inputs:
        //     &self: value
        //         Caller-supplied &self.
        //     action: &str
        //         Caller-supplied action.
        //
        // Outputs:
        //     result: bool
        //         Return value from `mission_action_requires_approval`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_recovery::mission_action_requires_approval(&self, action);

        self.mission_approval_actions.contains(action)
    }

    /// Block mission progression until operator approval is granted for an action.
    pub(super) fn ensure_mission_operator_approval(
        &mut self,
        action: &str,
        line: u32,
    ) -> Result<(), SpandaError> {
        // Description:
        //     Ensure mission operator approval.
        //
        // Inputs:
        //     &mut self: value
        //         Caller-supplied &mut self.
        //     action: &str
        //         Caller-supplied action.
        //     line: u32
        //         Caller-supplied line.
        //
        // Outputs:
        //     result: Result<(), SpandaError>
        //         Return value from `ensure_mission_operator_approval`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_recovery::ensure_mission_operator_approval(&mut self, action, line);

        if !self.mission_action_requires_approval(action) {
            return Ok(());
        }
        self.poll_recovery_approvals();
        if self.operator_approval_granted(action) {
            return Ok(());
        }
        self.request_operator_approval(action);
        self.record_mission_event(
            "mission_approval_required",
            serde_json::json!({ "action": action }),
        );
        Err(SpandaError::Runtime {
            message: format!("mission: operator approval required for '{action}'"),
            line,
        })
    }

    /// Execute a validated recovery plan at runtime for the given issue.
    pub(super) fn execute_recovery_runtime(
        &mut self,
        issue: &str,
    ) -> Result<RecoveryResult, SpandaError> {
        // Description:
        //     Execute recovery runtime.
        //
        // Inputs:
        //     &mut self: value
        //         Caller-supplied &mut self.
        //     issue: &str
        //         Caller-supplied issue.
        //
        // Outputs:
        //     result: Result<RecoveryResult, SpandaError>
        //         Return value from `execute_recovery_runtime`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_recovery::execute_recovery_runtime(&mut self, issue);

        self.poll_recovery_approvals();
        let Some(program) = self.health_program.clone() else {
            return Ok(RecoveryResult {
                plan: "none".into(),
                status: RecoveryStatus::Failed,
                executed_actions: vec![],
                failed_actions: vec![issue.into()],
                verification_outcome: "No recovery program cached".into(),
                evidence: spanda_assurance::RecoveryEvidence {
                    failure: issue.into(),
                    diagnosis: issue.into(),
                    plan: "none".into(),
                    safety_validation: "SKIP".into(),
                    recovery_actions: vec![],
                    outcome: "Failed".into(),
                    operator_approval: None,
                    verification: "No program".into(),
                },
            });
        };

        let context = RecoveryContext {
            issue: issue.into(),
            diagnosis: None,
            classification: Some(classify_failure(issue)),
            level: RecoveryLevel::Level3AutomaticWithValidation,
        };
        let plan = RecoveryPlanner::plan(&program, &context);
        let safe_actions = validate_recovery_plan(&program, &plan);
        let mut executed = Vec::new();
        let mut failed = Vec::new();
        let mut operator_approval = None;

        for safe in &safe_actions {
            let gates_ok = safe.safety_validation.passed
                && safe.hardware_verification.passed
                && safe.capability_verification.passed
                && safe.readiness_validation.passed;
            if !gates_ok {
                failed.push(safe.action.description.clone());
                continue;
            }
            if safe.action.requires_approval
                && !self.operator_approval_granted(&safe.action.description)
            {
                self.request_operator_approval(&safe.action.description);
                failed.push(format!("{} (approval required)", safe.action.description));
                operator_approval = Some("Operator".into());
                continue;
            }
            self.dispatch_recovery_action(&safe.action.description)?;
            executed.push(safe.action.description.clone());
        }

        let status = if failed.is_empty() && !executed.is_empty() {
            RecoveryStatus::Success
        } else if !executed.is_empty() {
            RecoveryStatus::PartialSuccess
        } else if safe_actions.iter().any(|a| !a.safety_validation.passed) {
            RecoveryStatus::Unsafe
        } else {
            RecoveryStatus::Failed
        };

        let evidence = spanda_assurance::RecoveryEvidence {
            failure: plan.failure.clone(),
            diagnosis: plan.diagnosis.clone(),
            plan: plan.name.clone(),
            safety_validation: if safe_actions.iter().all(|a| a.safety_validation.passed) {
                "PASS".into()
            } else {
                "FAIL".into()
            },
            recovery_actions: executed.clone(),
            outcome: format!("{status:?}"),
            operator_approval: operator_approval.clone(),
            verification: if status == RecoveryStatus::Success {
                "Recovery verified".into()
            } else {
                "Recovery incomplete".into()
            },
        };

        let result = RecoveryResult {
            plan: plan.name.clone(),
            status,
            executed_actions: executed,
            failed_actions: failed,
            verification_outcome: evidence.verification.clone(),
            evidence,
        };

        self.record_mission_event(
            "recovery_executed",
            serde_json::json!({
                "issue": issue,
                "status": format!("{:?}", result.status),
                "actions": result.executed_actions,
            }),
        );

        let persisted = load_recovery_knowledge_store(&self.recovery_knowledge_path);
        let mut knowledge = merge_recovery_knowledge(&program, &persisted);
        record_recovery_outcome(&mut knowledge, &result);
        let _ = save_recovery_knowledge_store(&self.recovery_knowledge_path, &knowledge);

        let _ = self.try_invoke_continuity_for_event(issue);

        Ok(result)
    }

    fn restart_active_connectivity(&mut self) -> Result<(), SpandaError> {
        // Description:
        //     Restart active connectivity.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //
        // Outputs:
        //     result: Result<(), SpandaError>
        //         Return value from `restart_active_connectivity`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_recovery::restart_active_connectivity(&mut self);

        let link = self.active_connectivity_link.clone();
        self.default_transport = self.host.connectivity_link_to_transport(&link);
        self.comm_bus.reconnect_transport(self.default_transport);
        self.log(format!("recovery: restarted connectivity on '{link}'"));
        self.record_mission_event(
            "recovery_connectivity_restart",
            serde_json::json!({ "link": link }),
        );
        Ok(())
    }

    fn pause_active_mission(&mut self) {
        // Description:
        //     Pause active mission.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_recovery::pause_active_mission(&mut self);

        let Some(RuntimeValue::MissionControl { mut runtime }) = self.env.get("mission").cloned()
        else {
            return;
        };
        runtime.pause();
        self.env
            .define("mission", RuntimeValue::MissionControl { runtime });
        self.log("recovery: mission paused".into());
        self.record_mission_event("recovery_mission_paused", serde_json::json!({}));
    }

    fn coordinate_fleet_recovery(&mut self, action: &str) -> Result<(), SpandaError> {
        // Description:
        //     Coordinate fleet recovery.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     action: &str
        //         Caller-supplied action.
        //
        // Outputs:
        //     result: Result<(), SpandaError>
        //         Return value from `coordinate_fleet_recovery`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_recovery::coordinate_fleet_recovery(&mut self, action);

        let fleet_names: Vec<String> = self.fleets.names().cloned().collect();
        let source = self.publish_source_id();
        self.comm_bus.publish(
            "/fleet/recovery",
            "Command",
            RuntimeValue::String {
                value: action.to_string(),
            },
            self.default_transport,
            Some(&source),
        );
        let mesh_url = std::env::var("SPANDA_FLEET_MESH_URL").ok();
        let mesh_token = std::env::var("SPANDA_FLEET_MESH_TOKEN").ok();
        for fleet_name in fleet_names {
            let Some(members) = self.fleets.members(&fleet_name).map(|m| m.to_vec()) else {
                continue;
            };
            self.log(format!(
                "fleet_recovery: {action} for fleet {fleet_name} members={members:?}"
            ));
            if let Some(url) = mesh_url.as_deref() {
                let request = FleetRecoveryRequest {
                    action: action.to_string(),
                    fleet_name: Some(fleet_name.clone()),
                    from_robot: members.first().cloned(),
                    members: members.clone(),
                };
                match relay_recovery_via_mesh(url, &request, mesh_token.as_deref()) {
                    Ok(resp) => {
                        self.log(format!(
                            "fleet_mesh: recovery '{}' relayed={} failed={}",
                            action, resp.relayed, resp.failed
                        ));
                        self.record_mission_event(
                            "fleet_mesh_recovery",
                            serde_json::json!({
                                "fleet": fleet_name,
                                "action": action,
                                "relayed": resp.relayed,
                                "failed": resp.failed,
                            }),
                        );
                    }
                    Err(err) => {
                        self.log(format!("fleet_mesh: recovery relay failed: {err}"));
                    }
                }
            }
            self.record_mission_event(
                "fleet_recovery",
                serde_json::json!({
                    "fleet": fleet_name,
                    "action": action,
                    "members": members,
                }),
            );

            if action_triggers_continuity_handoff(action) {
                let failed = members.first().cloned().unwrap_or_default();
                let request = FleetContinuityRequest {
                    failed_robot: failed.clone(),
                    successor: members.get(1).cloned(),
                    mission: None,
                    progress_percent: None,
                    trigger: Some("robot_failed".into()),
                    fleet_name: Some(fleet_name.clone()),
                    from_robot: Some(failed),
                    members: members.clone(),
                };
                let payload =
                    serde_json::to_string(&request).map_err(|e| SpandaError::Runtime {
                        message: e.to_string(),
                        line: 0,
                    })?;
                self.comm_bus.publish(
                    "/fleet/continuity",
                    "Command",
                    RuntimeValue::String { value: payload },
                    self.default_transport,
                    Some(&source),
                );
                if let Some(url) = mesh_url.as_deref() {
                    match relay_continuity_via_mesh(url, &request, mesh_token.as_deref()) {
                        Ok(resp) => {
                            self.log(format!(
                                "fleet_mesh: takeover '{}' relayed={} failed={}",
                                request.failed_robot, resp.relayed, resp.failed
                            ));
                            self.record_mission_event(
                                "fleet_mesh_continuity",
                                serde_json::json!({
                                    "fleet": fleet_name,
                                    "failed": request.failed_robot,
                                    "successor": request.successor,
                                    "relayed": resp.relayed,
                                    "failed_agents": resp.failed,
                                }),
                            );
                        }
                        Err(err) => {
                            self.log(format!("fleet_mesh: takeover relay failed: {err}"));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub(super) fn init_recovery_runtime(&mut self) {
        // Description:
        //     Init recovery runtime.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_recovery::init_recovery_runtime(&mut self);

        self.recovery_knowledge_path = default_knowledge_store_path();
        self.pending_recovery_approvals.clear();
        self.granted_recovery_approvals.clear();
        self.recovery_speed_cap = None;
    }

    /// Prepare a robot runtime for assurance-gated recovery dispatch.
    pub fn prepare_recovery_execution(
        &mut self,
        program: &Program,
        robot_name: Option<&str>,
    ) -> Result<(), SpandaError> {
        // Description:
        //     Prepare recovery execution.
        //
        // Inputs:
        //     &mut self: value
        //         Caller-supplied &mut self.
        //     progra: &Program
        //         Caller-supplied progra.
        //     robot_name: Option<&str>
        //         Caller-supplied robot name.
        //
        // Outputs:
        //     result: Result<(), SpandaError>
        //         Return value from `prepare_recovery_execution`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_recovery::prepare_recovery_execution(&mut self, progra, robot_name);

        let Program::Program {
            robots,
            geofences,
            fleets,
            program_safety_zones,
            certifications,
            connectivity_policies,
            ..
        } = program;
        self.load_program_metadata(program);
        self.cache_health_program(program);
        self.load_connectivity_metadata(geofences, connectivity_policies);
        self.load_robotics_platform_metadata(fleets, program_safety_zones, certifications);
        let robot = robots
            .iter()
            .find(|robot| {
                let RobotDecl::RobotDecl { name, .. } = robot;
                robot_name.is_none_or(|wanted| wanted == name)
            })
            .or_else(|| robots.first())
            .ok_or_else(|| SpandaError::Runtime {
                message: "no robot declared for recovery execution".into(),
                line: 0,
            })?;
        self.setup_robot(robot)?;
        Ok(())
    }

    /// Execute a validated recovery plan for a failure issue through the runtime dispatcher.
    pub fn run_recovery_issue(&mut self, issue: &str) -> Result<RecoveryResult, SpandaError> {
        // Description:
        //     Run recovery issue.
        //
        // Inputs:
        //     &mut self: value
        //         Caller-supplied &mut self.
        //     issue: &str
        //         Caller-supplied issue.
        //
        // Outputs:
        //     result: Result<RecoveryResult, SpandaError>
        //         Return value from `run_recovery_issue`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_recovery::run_recovery_issue(&mut self, issue);

        self.execute_recovery_runtime(issue)
    }

    /// Capture interpreter recovery side effects for fleet agent state sync.
    pub fn recovery_execution_snapshot(&self) -> RecoveryExecutionSnapshot {
        // Description:
        //     Recovery execution snapshot.
        //
        // Inputs:
        //     &self: value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: RecoveryExecutionSnapshot
        //         Return value from `recovery_execution_snapshot`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_recovery::recovery_execution_snapshot(&self);

        let mission_paused = self
            .env()
            .get("mission")
            .and_then(|value| {
                if let RuntimeValue::MissionControl { runtime } = value {
                    Some(runtime.state == MissionState::Paused)
                } else {
                    None
                }
            })
            .unwrap_or(false);
        RecoveryExecutionSnapshot {
            active_mode: self.active_mode.clone(),
            mission_paused,
            recovery_speed_cap: self.recovery_speed_cap,
        }
    }
}

/// Snapshot of interpreter state after recovery dispatch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryExecutionSnapshot {
    pub active_mode: String,
    pub mission_paused: bool,
    pub recovery_speed_cap: Option<f64>,
}

/// Run assurance-gated recovery actions through the live interpreter dispatcher.
pub fn execute_recovery_on_program(
    program: &Program,
    issue: &str,
    options: RecoveryRunOptions,
) -> Result<RecoveryRunResult, SpandaError> {
    // Description:
    //     Execute recovery on program.
    //
    // Inputs:
    //     progra: &Program
    //         Caller-supplied progra.
    //     issue: &str
    //         Caller-supplied issue.
    //     options: RecoveryRunOptions
    //         Caller-supplied options.
    //
    // Outputs:
    //     result: Result<RecoveryRunResult, SpandaError>
    //         Return value from `execute_recovery_on_program`.
    //
    // Example:

    //     let result = spanda_interpreter::runtime_recovery::execute_recovery_on_program(progra, issue, options);

    let sim = create_default_simulator(SimulatorConfig::default());
    let logs: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let logs_cb = logs.clone();
    let grant_approval = options.grant_operator_approval;
    let mut interp = Interpreter::new(
        sim,
        super::InterpreterOptions {
            max_loop_iterations: 1,
            inbound_comm_messages: options.inbound_comm_messages.clone(),
            on_log: Some(Rc::new(move |msg| logs_cb.borrow_mut().push(msg))),
            ..Default::default()
        },
    );
    if grant_approval {
        std::env::set_var("SPANDA_OPERATOR_APPROVAL", "1");
    }
    let run_result = (|| {
        interp.prepare_recovery_execution(program, options.robot_name.as_deref())?;
        let recovery = interp.run_recovery_issue(issue)?;
        let snapshot = interp.recovery_execution_snapshot();
        Ok(RecoveryRunResult {
            recovery,
            logs: logs.borrow().clone(),
            active_mode: snapshot.active_mode,
            mission_paused: snapshot.mission_paused,
            speed_cap: snapshot.recovery_speed_cap,
        })
    })();
    if grant_approval {
        std::env::remove_var("SPANDA_OPERATOR_APPROVAL");
    }
    run_result
}

#[cfg(test)]
mod recovery_execute_tests {
    use super::super::super::super::options::RecoveryRunOptions;
    use super::execute_recovery_on_program;
    use spanda_assurance::RecoveryStatus;
    use spanda_lexer::tokenize;
    use spanda_parser::parse;

    #[test]
    fn interpreter_recovery_enters_degraded_mode() {
        // Description:
        //     Interpreter recovery enters degraded mode.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_recovery::interpreter_recovery_enters_degraded_mode();

        let source = r#"
recovery_policy RoverRecovery {
    on gps.failed {
        enter degraded_mode;
        reduce_speed 0.4 m/s;
    }
}
robot Rover {
    sensor gps: GPS;
    actuator wheels: DifferentialDrive;
    mode degraded { }
    safety { max_speed = 1.0 m/s; }
    behavior patrol() {}
}
"#;
        let program = parse(tokenize(source).unwrap()).unwrap();
        let outcome = execute_recovery_on_program(
            &program,
            "gps.failed",
            RecoveryRunOptions {
                robot_name: Some("Rover".into()),
                grant_operator_approval: true,
                ..RecoveryRunOptions::default()
            },
        )
        .expect("recovery run");
        assert_eq!(outcome.recovery.status, RecoveryStatus::Success);
        assert_eq!(outcome.active_mode, "degraded");
        assert!(outcome
            .recovery
            .executed_actions
            .iter()
            .any(|action| action.contains("degraded") || action.contains("reduce_speed")));
    }
}

//! Runtime mission continuity takeover dispatch and fleet mesh coordination.

use super::super::super::fleet_http::{relay_continuity_via_mesh, FleetContinuityRequest};
use super::super::super::options::{ContinuityRunOptions, ContinuityRunResult};
use super::super::super::simulator::{create_default_simulator, SimulatorConfig};
use super::{Interpreter, RobotBackend};
use serde::{Deserialize, Serialize};
use spanda_assurance::{
    default_checkpoint_store_path, extract_continuity_policies, issue_to_continuity_trigger,
    load_checkpoint, load_checkpoint_store, parse_trigger, plan_takeover,
    program_has_continuity_for_trigger, record_checkpoint, save_checkpoint_store,
    ContinuityContext, SuccessionScope, TakeoverMode, TakeoverReport,
};
use spanda_ast::nodes::Program;
use spanda_comm::CommBus;
use spanda_error::SpandaError;
use spanda_runtime::robotics::MissionState;
use spanda_runtime::value::RuntimeValue;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

impl<B: RobotBackend> Interpreter<B> {
    fn checkpoint_store_path(&self) -> PathBuf {
        std::env::var("SPANDA_CONTINUITY_CHECKPOINTS")
            .map(PathBuf::from)
            .unwrap_or_else(|_| default_checkpoint_store_path())
    }

    fn persist_takeover_checkpoint(&self, report: &TakeoverReport) {
        let mut store = load_checkpoint_store(&self.checkpoint_store_path());
        record_checkpoint(
            &mut store,
            &report.mission,
            &report.failed_entity,
            report.state_transfer.snapshot.clone(),
        );
        if let Err(err) = save_checkpoint_store(&self.checkpoint_store_path(), &store) {
            self.log(format!("continuity: checkpoint persist failed: {err}"));
        } else {
            self.log(format!(
                "continuity: checkpoint persisted for '{}' at {:.0}%",
                report.failed_entity, report.state_transfer.snapshot.progress_percent
            ));
        }
    }

    fn loaded_progress_percent(&self, report: &TakeoverReport) -> f64 {
        let store = load_checkpoint_store(&self.checkpoint_store_path());
        load_checkpoint(&store, &report.mission, &report.failed_entity)
            .map(|snap| snap.progress_percent)
            .unwrap_or(report.state_transfer.snapshot.progress_percent)
    }

    fn mission_runtime_progress(&self) -> f64 {
        self.env
            .get("mission")
            .and_then(|v| {
                if let RuntimeValue::MissionControl { runtime } = v {
                    if runtime.steps.is_empty() {
                        Some(0.0)
                    } else {
                        Some((runtime.step_index as f64 / runtime.steps.len() as f64) * 100.0)
                    }
                } else {
                    None
                }
            })
            .unwrap_or(0.0)
    }

    fn mission_runtime_name(&self) -> String {
        self.env
            .get("mission")
            .and_then(|v| {
                if let RuntimeValue::MissionControl { runtime } = v {
                    runtime.name.clone()
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "default_mission".into())
    }

    /// Execute assurance-gated continuity when policies cover a runtime issue.
    pub(super) fn execute_continuity_runtime(
        &mut self,
        issue: &str,
    ) -> Result<Option<TakeoverReport>, SpandaError> {
        let Some(program) = self.health_program.clone() else {
            return Ok(None);
        };
        if extract_continuity_policies(&program).is_empty() {
            return Ok(None);
        }
        let Some(trigger) = issue_to_continuity_trigger(issue) else {
            return Ok(None);
        };
        if !program_has_continuity_for_trigger(&program, trigger) {
            return Ok(None);
        }
        let failed = self
            .active_robot_name
            .clone()
            .unwrap_or_else(|| issue.into());
        let context = ContinuityContext {
            mission: self.mission_runtime_name(),
            failed_entity: failed,
            trigger,
            progress_percent: self.mission_runtime_progress(),
            scope: if self.fleets.names().next().is_some() {
                SuccessionScope::Fleet
            } else {
                SuccessionScope::Robot
            },
            current_step: None,
            checkpoints: Vec::new(),
        };
        let options = ContinuityRunOptions {
            robot_name: self.active_robot_name.clone(),
            successor: None,
            grant_operator_approval: false,
            inbound_comm_messages: Vec::new(),
        };
        let report = self.run_continuity_takeover(&program, &context, &options)?;
        self.log(format!(
            "continuity: auto-triggered for '{issue}' -> successor '{}' mode {:?}",
            report.successor, report.mode
        ));
        Ok(Some(report))
    }

    /// Attempt continuity takeover for a hardware or health event during run/sim.
    pub(super) fn try_invoke_continuity_for_event(
        &mut self,
        event: &str,
    ) -> Result<(), SpandaError> {
        match self.execute_continuity_runtime(event) {
            Ok(Some(_)) => Ok(()),
            Ok(None) => Ok(()),
            Err(err) => {
                self.log(format!(
                    "continuity: auto-trigger skipped for '{event}': {err}"
                ));
                Ok(())
            }
        }
    }

    fn pause_mission_for_continuity(&mut self) {
        let Some(RuntimeValue::MissionControl { mut runtime }) = self.env.get("mission").cloned()
        else {
            return;
        };
        runtime.pause();
        self.env
            .define("mission", RuntimeValue::MissionControl { runtime });
        self.log("continuity: mission paused".into());
    }

    fn resume_mission_at_progress(&mut self, progress_percent: f64) {
        let Some(RuntimeValue::MissionControl { mut runtime }) = self.env.get("mission").cloned()
        else {
            return;
        };
        if !runtime.steps.is_empty() {
            let idx = ((progress_percent / 100.0) * runtime.steps.len() as f64).floor() as usize;
            runtime.step_index = idx.min(runtime.steps.len().saturating_sub(1));
        }
        if runtime.state == MissionState::Pending {
            runtime.start();
        } else {
            runtime.resume();
        }
        self.env
            .define("mission", RuntimeValue::MissionControl { runtime });
    }

    fn restart_mission_from_beginning(&mut self) {
        let Some(RuntimeValue::MissionControl { mut runtime }) = self.env.get("mission").cloned()
        else {
            return;
        };
        runtime.restart();
        self.env
            .define("mission", RuntimeValue::MissionControl { runtime });
        self.log("continuity: mission restarted from beginning".into());
    }

    fn partial_restart_mission_at_progress(&mut self, progress_percent: f64) {
        let Some(RuntimeValue::MissionControl { mut runtime }) = self.env.get("mission").cloned()
        else {
            return;
        };
        if !runtime.steps.is_empty() {
            let idx = ((progress_percent / 100.0) * runtime.steps.len() as f64).floor() as usize;
            runtime.step_index = idx.min(runtime.steps.len().saturating_sub(1));
        }
        runtime.restart_current_step();
        let step_index = runtime.step_index;
        self.env
            .define("mission", RuntimeValue::MissionControl { runtime });
        self.log(format!(
            "continuity: partial restart at step index {} ({:.0}%)",
            step_index, progress_percent
        ));
    }

    /// Dispatch takeover side effects for the local robot role in a fleet handoff.
    pub(super) fn dispatch_continuity_takeover(
        &mut self,
        report: &TakeoverReport,
        robot_name: Option<&str>,
    ) -> Result<(), SpandaError> {
        let name = robot_name.unwrap_or_default();
        if name == report.failed_entity {
            self.pause_mission_for_continuity();
            self.persist_takeover_checkpoint(report);
            self.log(format!(
                "continuity: failed robot '{name}' paused pending handoff"
            ));
            return Ok(());
        }
        if name != report.successor && !name.is_empty() {
            return Ok(());
        }

        let progress = self.loaded_progress_percent(report);
        match report.mode {
            TakeoverMode::Resume => {
                self.resume_mission_at_progress(progress);
                self.log(format!(
                    "continuity: successor '{}' resuming from checkpoint at {:.0}%",
                    report.successor, progress
                ));
            }
            TakeoverMode::Restart => {
                self.restart_mission_from_beginning();
                self.log(format!(
                    "continuity: successor '{}' restarting mission",
                    report.successor
                ));
            }
            TakeoverMode::PartialRestart => {
                self.partial_restart_mission_at_progress(progress);
                self.log(format!(
                    "continuity: successor '{}' partial restart at {:.0}%",
                    report.successor, progress
                ));
            }
            TakeoverMode::ShadowTakeover => {
                self.log(format!(
                    "continuity: shadow takeover — successor '{}' synchronized at {:.0}%",
                    report.successor, progress
                ));
                self.resume_mission_at_progress(progress);
            }
            TakeoverMode::HotTakeover => {
                self.log(format!(
                    "continuity: hot takeover — successor '{}' immediate resume at {:.0}%",
                    report.successor, progress
                ));
                self.resume_mission_at_progress(progress);
            }
            TakeoverMode::ColdTakeover => {
                self.log(format!(
                    "continuity: cold takeover — successor '{}' initializing from persisted checkpoint",
                    report.successor
                ));
                self.resume_mission_at_progress(progress);
            }
            TakeoverMode::HumanTakeover => {
                if self.operator_approval_granted("human takeover") {
                    self.log(format!(
                        "continuity: human takeover approved — successor '{}' resuming at {:.0}%",
                        report.successor, progress
                    ));
                    self.resume_mission_at_progress(progress);
                } else {
                    self.pause_mission_for_continuity();
                    self.log(
                        "continuity: human takeover awaiting operator approval (set SPANDA_OPERATOR_APPROVAL=1)"
                            .into(),
                    );
                }
            }
        }
        self.record_mission_event(
            "continuity_takeover",
            serde_json::json!({
                "successor": report.successor,
                "failed": report.failed_entity,
                "mode": format!("{:?}", report.mode),
                "progress": progress,
            }),
        );
        Ok(())
    }

    fn coordinate_fleet_takeover(
        &mut self,
        request: &FleetContinuityRequest,
    ) -> Result<(), SpandaError> {
        let payload = serde_json::to_string(request).map_err(|e| SpandaError::Runtime {
            message: e.to_string(),
            line: 0,
        })?;
        let source = self.publish_source_id();
        self.comm_bus.publish(
            "/fleet/continuity",
            "Command",
            RuntimeValue::String {
                value: payload.clone(),
            },
            self.default_transport,
            Some(&source),
        );
        let mesh_url = std::env::var("SPANDA_FLEET_MESH_URL").ok();
        let mesh_token = std::env::var("SPANDA_FLEET_MESH_TOKEN").ok();
        if let Some(url) = mesh_url.as_deref() {
            match relay_continuity_via_mesh(url, request, mesh_token.as_deref()) {
                Ok(resp) => {
                    self.log(format!(
                        "fleet_mesh: takeover '{}' relayed={} failed={}",
                        request.failed_robot, resp.relayed, resp.failed
                    ));
                    self.record_mission_event(
                        "fleet_mesh_continuity",
                        serde_json::json!({
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
        self.record_mission_event(
            "fleet_takeover",
            serde_json::json!({
                "failed": request.failed_robot,
                "successor": request.successor,
                "progress": request.progress_percent,
            }),
        );
        Ok(())
    }

    pub(super) fn run_continuity_takeover(
        &mut self,
        program: &Program,
        context: &ContinuityContext,
        options: &ContinuityRunOptions,
    ) -> Result<TakeoverReport, SpandaError> {
        let report = plan_takeover(program, context, options.successor.as_deref());
        self.dispatch_continuity_takeover(&report, options.robot_name.as_deref())?;

        let fleet_names: Vec<String> = self.fleets.names().cloned().collect();
        for fleet_name in fleet_names {
            let members = self
                .fleets
                .members(&fleet_name)
                .map(|m| m.to_vec())
                .unwrap_or_default();
            let request = FleetContinuityRequest {
                failed_robot: context.failed_entity.clone(),
                successor: Some(report.successor.clone()),
                mission: Some(context.mission.clone()),
                progress_percent: Some(context.progress_percent),
                trigger: Some(format!("{:?}", context.trigger).to_lowercase()),
                fleet_name: Some(fleet_name),
                from_robot: options.robot_name.clone(),
                members,
            };
            if std::env::var("SPANDA_FLEET_MESH_URL").ok().is_some() || !request.members.is_empty()
            {
                self.coordinate_fleet_takeover(&request)?;
            }
        }
        Ok(report)
    }

    pub(super) fn continuity_execution_snapshot(&self) -> ContinuityExecutionSnapshot {
        let mission_paused = self
            .env
            .get("mission")
            .and_then(|v| {
                if let RuntimeValue::MissionControl { runtime } = v {
                    Some(runtime.state == MissionState::Paused)
                } else {
                    None
                }
            })
            .unwrap_or(false);
        let mission_progress_percent = self
            .env
            .get("mission")
            .and_then(|v| {
                if let RuntimeValue::MissionControl { runtime } = v {
                    if runtime.steps.is_empty() {
                        Some(0.0)
                    } else {
                        Some((runtime.step_index as f64 / runtime.steps.len() as f64) * 100.0)
                    }
                } else {
                    None
                }
            })
            .unwrap_or(0.0);
        let checkpoint_store = load_checkpoint_store(&self.checkpoint_store_path());
        let checkpoint_count = checkpoint_store.entries.len();
        ContinuityExecutionSnapshot {
            mission_paused,
            mission_progress_percent,
            checkpoint_count,
        }
    }
}

/// Snapshot of interpreter state after continuity dispatch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContinuityExecutionSnapshot {
    pub mission_paused: bool,
    pub mission_progress_percent: f64,
    pub checkpoint_count: usize,
}

/// Run assurance-gated takeover through the live interpreter dispatcher.
pub fn execute_continuity_on_program(
    program: &Program,
    context: &ContinuityContext,
    options: ContinuityRunOptions,
) -> Result<ContinuityRunResult, SpandaError> {
    let sim = create_default_simulator(SimulatorConfig::default());
    let logs: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let logs_cb = logs.clone();
    let mut interp = Interpreter::new(
        sim,
        super::InterpreterOptions {
            max_loop_iterations: 1,
            inbound_comm_messages: options.inbound_comm_messages.clone(),
            on_log: Some(Rc::new(move |msg| logs_cb.borrow_mut().push(msg))),
            ..Default::default()
        },
    );
    if options.grant_operator_approval {
        std::env::set_var("SPANDA_OPERATOR_APPROVAL", "1");
    }
    let run_result = (|| {
        interp.prepare_recovery_execution(program, options.robot_name.as_deref())?;
        let takeover = interp.run_continuity_takeover(program, context, &options)?;
        let snapshot = interp.continuity_execution_snapshot();
        Ok(ContinuityRunResult {
            takeover,
            logs: logs.borrow().clone(),
            mission_progress_percent: snapshot.mission_progress_percent,
            handoff_from: Some(context.failed_entity.clone()),
            mission_paused: snapshot.mission_paused,
            checkpoint_count: snapshot.checkpoint_count,
        })
    })();
    if options.grant_operator_approval {
        std::env::remove_var("SPANDA_OPERATOR_APPROVAL");
    }
    run_result
}

/// Build a continuity context from mesh request fields.
pub fn continuity_context_from_request(
    failed_robot: &str,
    mission: Option<&str>,
    progress_percent: f64,
    trigger: Option<&str>,
) -> ContinuityContext {
    ContinuityContext {
        mission: mission.unwrap_or("default_mission").into(),
        failed_entity: failed_robot.into(),
        trigger: trigger
            .map(parse_trigger)
            .unwrap_or(spanda_assurance::ContinuityTrigger::RobotFailed),
        progress_percent,
        scope: SuccessionScope::Fleet,
        current_step: None,
        checkpoints: Vec::new(),
    }
}

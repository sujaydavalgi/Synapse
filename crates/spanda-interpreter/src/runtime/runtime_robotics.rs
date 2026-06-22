//! AI, mission, fleet, and safety validation method dispatch for the interpreter.
//!

use super::{get_string, IntoSpandaError, Interpreter, RobotBackend, RuntimeError, RuntimeValue};
use crate::ai::{
    proposal_confidence, proposal_from_value, safe_action_from_proposal, AI_CONFIDENCE_LOW_THRESHOLD,
};
use spanda_ast::nodes::{Expr, UnitKind};
use crate::error::SpandaError;
use crate::safety::{Pose2d, ValidateActionResult};
use spanda_runtime::triggers::SystemTriggerCategory;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn eval_ai_method(
        &mut self,
        target_name: &str,
        method: &str,
        args: &[Expr],
        named_args: &[spanda_ast::nodes::NamedArg],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Eval ai method.
        //
        // Parameters:
        // - `self` — method receiver
        // - `target_name` — input value
        // - `method` — input value
        // - `args` — input value
        // - `named_args` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.eval_ai_method(target_name, method, args, named_args, line);

        // Match on method and handle each case.
        match method {
            "reason" => {

                // Emit output when as deref provides a agent.
                if let Some(agent) = self.current_agent.as_deref() {
                    self.check_agent_capability(agent, "propose_motion", None, line)?;
                }
                let prompt = get_string(&self.get_named_arg_value(named_args, "prompt")?, "");
                let input = self.get_named_arg_value(named_args, "input")?;
                let input = if matches!(input, RuntimeValue::Void) {
                    None
                } else {
                    Some(input)
                };
                let goal_text = self.resolve_reason_goal(named_args, line)?;
                let goal_text = self.enrich_reason_goal(goal_text);
                let result = self
                    .ai_models
                    .get(target_name)
                    .ok_or_else(|| {
                        RuntimeError::new(format!("Unknown AI model '{target_name}'"), line)
                            .into_spanda()
                    })?
                    .reason(&prompt, input, goal_text.as_deref())
                    .map_err(|message| SpandaError::Runtime { message, line })?;
                self.log(format!("ai {target_name}.reason() -> ActionProposal"));
                let confidence = proposal_confidence(&result);

                // Take this path when confidence < AI CONFIDENCE LOW THRESHOLD.
                if confidence < AI_CONFIDENCE_LOW_THRESHOLD {

                    // Take the branch when ai confidence low active is false.
                    if !self.ai_confidence_low_active {
                        self.ai_confidence_low_active = true;
                        let _ = self.dispatch_system_trigger(
                            SystemTriggerCategory::Ai,
                            "ConfidenceLow",
                        );
                    }
                } else {
                    self.ai_confidence_low_active = false;
                }
                Ok(result)
            }
            "summarize" => {

                // Emit output when as deref provides a agent.
                if let Some(agent) = self.current_agent.as_deref() {
                    self.check_agent_capability(agent, "summarize", None, line)?;
                }
                let input = self.get_named_arg_value(named_args, "input")?;
                let input = if matches!(input, RuntimeValue::Void) {
                    None
                } else {
                    Some(input)
                };
                self.ai_models
                    .get(target_name)
                    .ok_or_else(|| {
                        RuntimeError::new(format!("Unknown AI model '{target_name}'"), line)
                            .into_spanda()
                    })?
                    .summarize(input)
                    .map_err(|message| SpandaError::Runtime { message, line })
            }
            "detect" => {

                // Emit output when as deref provides a agent.
                if let Some(agent) = self.current_agent.as_deref() {
                    self.check_agent_capability(agent, "detect", None, line)?;
                }
                let frame = if let Some(first) = args.first() {
                    self.eval_expr(first)?
                } else {
                    self.get_named_arg_value(named_args, "frame")?
                };
                self.ai_models
                    .get(target_name)
                    .ok_or_else(|| {
                        RuntimeError::new(format!("Unknown AI model '{target_name}'"), line)
                            .into_spanda()
                    })?
                    .detect(frame)
                    .map_err(|message| SpandaError::Runtime { message, line })
            }
            "drive" => Err(RuntimeError::new(
                "Unsafe AI action: LLM cannot drive actuators directly — use safety.validate() then wheels.execute()",
                line,
            )
            .into_spanda()),
            _ => Ok(RuntimeValue::Void),
        }
    }

    pub(super) fn eval_mission_method(
        &self,
        runtime: &mut spanda_runtime::robotics::MissionRuntime,
        property: &str,
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Dispatch mission lifecycle methods on the active mission controller.
        match property {
            "start" => {
                runtime.start();
                Ok(RuntimeValue::String {
                    value: runtime.state.as_str().into(),
                })
            }
            "pause" => {
                runtime.pause();
                Ok(RuntimeValue::String {
                    value: runtime.state.as_str().into(),
                })
            }
            "resume" => {
                runtime.resume();
                Ok(RuntimeValue::String {
                    value: runtime.state.as_str().into(),
                })
            }
            "advance" => {
                let step = runtime.advance().unwrap_or_default();
                Ok(RuntimeValue::String { value: step })
            }
            "complete" => {
                runtime.complete();
                Ok(RuntimeValue::String {
                    value: runtime.state.as_str().into(),
                })
            }
            "fail" => {
                runtime.fail();
                Ok(RuntimeValue::String {
                    value: runtime.state.as_str().into(),
                })
            }
            "state" => Ok(RuntimeValue::String {
                value: runtime.state.as_str().into(),
            }),
            "step" => Ok(RuntimeValue::String {
                value: runtime.current_step().unwrap_or("").into(),
            }),
            _ => Err(
                RuntimeError::new(format!("Unknown mission method '{property}'"), line)
                    .into_spanda(),
            ),
        }
    }

    pub(super) fn eval_fleet_method(
        &mut self,
        registry: &spanda_runtime::robotics::FleetRegistry,
        property: &str,
        args: &[Expr],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Dispatch fleet coordination helpers for member lookup.
        match property {
            "members" => {
                let fleet_name = if !args.is_empty() {
                    match self.eval_expr(&args[0])? {
                        RuntimeValue::String { value } => value,
                        RuntimeValue::Number { value, .. } => value.to_string(),
                        _ => String::new(),
                    }
                } else {
                    return Err(
                        RuntimeError::new("fleet.members() requires a fleet name", line)
                            .into_spanda(),
                    );
                };
                let members = registry.members(&fleet_name).unwrap_or(&[]);
                Ok(RuntimeValue::Number {
                    value: members.len() as f64,
                    unit: UnitKind::None,
                })
            }
            "names" => Ok(RuntimeValue::Number {
                value: registry.names().count() as f64,
                unit: UnitKind::None,
            }),
            _ => Err(
                RuntimeError::new(format!("Unknown fleet method '{property}'"), line).into_spanda(),
            ),
        }
    }

    pub(super) fn eval_safety_validate(
        &mut self,
        args: &[Expr],
        named_args: &[spanda_ast::nodes::NamedArg],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Eval safety validate.
        //
        // Parameters:
        // - `self` — method receiver
        // - `args` — input value
        // - `named_args` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.eval_safety_validate(args, named_args, line);

        // Compute arg for the following logic.
        let arg = if let Some(first) = args.first() {
            self.eval_expr(first)?
        } else {
            self.get_named_arg_value(named_args, "proposal")?
        };
        let proposal = proposal_from_value(&arg).ok_or_else(|| {
            RuntimeError::new("safety.validate() expects ActionProposal", line).into_spanda()
        })?;
        let state = self.backend.get_state();
        let pose2d = Pose2d {
            x: state.pose.x,
            y: state.pose.y,
        };
        let monitor = self.safety_monitor.as_ref().ok_or_else(|| {
            RuntimeError::new("Safety monitor not configured", line).into_spanda()
        })?;
        let result =
            monitor.validate_action_proposal(proposal.linear, proposal.angular, &self.env, &pose2d);

        // Match on result and handle each case.
        match result {
            ValidateActionResult::Ok(motion) => {
                self.log("safety.validate() approved ActionProposal".into());
                Ok(safe_action_from_proposal(motion.linear, motion.angular))
            }
            ValidateActionResult::Err { reason } => {
                Err(RuntimeError::new(reason, line).into_spanda())
            }
        }
    }
}

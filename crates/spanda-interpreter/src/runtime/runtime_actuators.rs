//! Actuator execute/safe-motion dispatch for the interpreter.
//!

use super::{
    get_number, get_trajectory_waypoints, IntoSpandaError, Interpreter, MotionCommand,
    RobotBackend, RuntimeError, RuntimeValue,
};
use crate::ai::{is_action_proposal, is_safe_action};
use spanda_ast::nodes::Expr;
use spanda_error::SpandaError;
use spanda_safety::Pose2d;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn execute_actuator_method(
        &mut self,
        name: &str,
        _actuator_type: &str,
        method: &str,
        args: &[Expr],
        named_args: &[spanda_ast::nodes::NamedArg],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Execute actuator method.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `_actuator_type` — input value
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
        // let result = instance.execute_actuator_method(name, _actuator_type, method, args, named_args, line);

        // Compute motion methods for the following logic.
        let motion_methods = [
            "drive",
            "move_to",
            "set_thrust",
            "grip",
            "release",
            "open",
            "hover",
            "follow",
        ];

        // Check membership before continuing.
        if (motion_methods.contains(&method) || method == "stop")
            && !self.check_safety_before_motion()
        {
            // Emit output when on motion blocked provides a cb.
            if let Some(cb) = &self.options.on_motion_blocked {
                cb("Safety rule triggered — motion blocked".into());
            }
            self.backend.execute_motion(MotionCommand::Stop {
                actuator: name.to_string(),
            });
            return Ok(RuntimeValue::Void);
        }

        // Match on method and handle each case.
        match method {
            "stop" => {
                self.backend.execute_motion(MotionCommand::Stop {
                    actuator: name.to_string(),
                });
            }
            "drive" => {
                let linear = get_number(&self.get_named_arg_value(named_args, "linear")?, 0.0);
                let angular = get_number(&self.get_named_arg_value(named_args, "angular")?, 0.0);
                let pose = self.backend.get_state().pose;
                let pose2d = Pose2d {
                    x: pose.x,
                    y: pose.y,
                };
                let max_speed = self
                    .safety_monitor
                    .as_ref()
                    .map(|m| m.clamp_speed_at_pose(linear, &pose2d))
                    .unwrap_or(linear);
                self.backend.execute_motion(MotionCommand::Drive {
                    linear: max_speed,
                    angular,
                    actuator: name.to_string(),
                });
            }
            "follow" => {
                let path_val = self.get_named_arg_value(named_args, "path")?;
                let waypoints = get_trajectory_waypoints(&path_val).unwrap_or_default();
                self.backend.execute_motion(MotionCommand::Follow {
                    waypoints,
                    actuator: name.to_string(),
                });
            }
            "move_to" => {
                let x = get_number(&self.get_named_arg_value(named_args, "x")?, 0.0);
                let y = get_number(&self.get_named_arg_value(named_args, "y")?, 0.0);
                let z = get_number(&self.get_named_arg_value(named_args, "z")?, 0.0);
                self.backend.execute_motion(MotionCommand::MoveTo {
                    x,
                    y,
                    z,
                    actuator: name.to_string(),
                });
            }
            "grip" => {
                self.backend.execute_motion(MotionCommand::Grip {
                    actuator: name.to_string(),
                });
            }
            "release" => {
                self.backend.execute_motion(MotionCommand::Release {
                    actuator: name.to_string(),
                });
            }
            "open" => {
                self.backend.execute_motion(MotionCommand::Open {
                    actuator: name.to_string(),
                });
            }
            "set_thrust" => {
                let thrust = get_number(&self.get_named_arg_value(named_args, "thrust")?, 0.0);
                self.backend.execute_motion(MotionCommand::SetThrust {
                    thrust,
                    actuator: name.to_string(),
                });
            }
            "hover" => {
                self.backend.execute_motion(MotionCommand::Hover {
                    actuator: name.to_string(),
                });
            }
            "execute" => {
                // Emit output when as deref provides a agent.
                if let Some(agent) = self.current_agent.as_deref() {
                    self.check_agent_capability(agent, "propose_motion", None, line)?;
                }
                let action_val = if let Some(first) = args.first() {
                    self.eval_expr(first)?
                } else {
                    self.get_named_arg_value(named_args, "action")?
                };

                // Take the branch when is safe action is false.
                if !is_safe_action(&action_val) {
                    // Take this path when is action proposal(&action val).
                    if is_action_proposal(&action_val) {
                        return Err(RuntimeError::new(
                            "Unsafe AI action: ActionProposal cannot execute actuators — call safety.validate() first",
                            line,
                        )
                        .into_spanda());
                    }
                    return Err(RuntimeError::new(
                        "Actuator execute() requires SafeAction from safety.validate()",
                        line,
                    )
                    .into_spanda());
                }

                // Take the branch when check safety before motion is false.
                if !self.check_safety_before_motion() {
                    // Emit output when on motion blocked provides a cb.
                    if let Some(cb) = &self.options.on_motion_blocked {
                        cb("Safety rule triggered — motion blocked".into());
                    }
                    self.backend.execute_motion(MotionCommand::Stop {
                        actuator: name.to_string(),
                    });
                    return Ok(RuntimeValue::Void);
                }

                // Take this path when let RuntimeValue::SafeAction { linear, angular } = action val.
                if let RuntimeValue::SafeAction { linear, angular } = action_val {
                    self.backend.execute_motion(MotionCommand::Drive {
                        linear,
                        angular,
                        actuator: name.to_string(),
                    });
                }
            }
            _ => {}
        }
        self.log(format!("{name}.{method}()"));
        Ok(RuntimeValue::Void)
    }

}

//! Navigation and SLAM adapter method dispatch for the interpreter.
//!

use super::{
    pose_from_state, runtime_pose, runtime_velocity, IntoSpandaError, Interpreter, RobotBackend,
    RuntimeError, RuntimeValue,
};
use spanda_ast::nodes::{Expr, UnitKind};
use spanda_error::SpandaError;
use std::collections::HashMap;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn eval_navigation_method(
        &mut self,
        goal: &mut Option<String>,
        property: &str,
        args: &[Expr],
        named_args: &[spanda_ast::nodes::NamedArg],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        match property {
            "goal" => {
                let text = if !args.is_empty() {
                    match self.eval_expr(&args[0])? {
                        RuntimeValue::String { value } => value,
                        RuntimeValue::Number { value, .. } => value.to_string(),
                        _ => String::new(),
                    }
                } else if let Ok(RuntimeValue::String { value }) =
                    self.get_named_arg_value(named_args, "text")
                {
                    value
                } else {
                    return Err(RuntimeError::new(
                        "navigation.goal() requires a text argument",
                        line,
                    )
                    .into_spanda());
                };
                *goal = Some(text.clone());
                Ok(RuntimeValue::Object {
                    type_name: "NavigationGoal".into(),
                    fields: HashMap::from([("text".into(), RuntimeValue::String { value: text })]),
                })
            }
            "path" => {
                let state = self.backend.get_state();
                let _waypoints = [
                    pose_from_state(&state.pose),
                    runtime_pose(state.pose.x + 1.0, state.pose.y, state.pose.theta, 0.0),
                ];
                Ok(RuntimeValue::Object {
                    type_name: "Path".into(),
                    fields: HashMap::from([(
                        "waypoints".into(),
                        RuntimeValue::Number {
                            value: 2.0,
                            unit: UnitKind::None,
                        },
                    )]),
                })
            }
            "cost_map" => Ok(RuntimeValue::Object {
                type_name: "CostMap".into(),
                fields: HashMap::from([(
                    "resolution".into(),
                    RuntimeValue::Number {
                        value: 0.05,
                        unit: UnitKind::M,
                    },
                )]),
            }),
            "navigate" => {
                let goal_label = goal.as_deref().unwrap_or("none");
                self.log(format!("navigation: executing goal '{goal_label}'"));
                if let Some(output) = self.host.invoke_nav2_bridge(goal_label) {
                    self.log(format!("navigation: Nav2 bridge output: {output}"));
                }
                if self.nav2_enabled || self.topic_path_to_message_type.contains_key("/cmd_vel")
                {
                    if let Some(message_type) =
                        self.topic_path_to_message_type.get("/cmd_vel").cloned()
                    {
                        let velocity = runtime_velocity(0.2, 0.0);
                        self.backend
                            .publish_topic("/cmd_vel", &message_type, velocity);
                        let prefix = if self.nav2_enabled {
                            "Nav2Adapter"
                        } else {
                            "Nav2 bridge"
                        };
                        self.log(format!(
                            "navigation: {prefix} publish /cmd_vel goal='{}'",
                            goal.as_deref().unwrap_or("none")
                        ));
                    }
                }
                Ok(RuntimeValue::Object {
                    type_name: "Trajectory".into(),
                    fields: HashMap::from([(
                        "status".into(),
                        RuntimeValue::String {
                            value: "executing".into(),
                        },
                    )]),
                })
            }
            _ => Err(
                RuntimeError::new(format!("Unknown navigation method '{property}'"), line)
                    .into_spanda(),
            ),
        }
    }

    pub(super) fn eval_slam_method(
        &mut self,
        property: &str,
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        match property {
            "localize" => {
                let state = self.backend.get_state();
                if let Some(output) = self.host.invoke_slam_bridge("localize") {
                    self.log(format!("slam: bridge localize output: {output}"));
                } else {
                    self.log("slam: localize (stub adapter)".into());
                }
                Ok(RuntimeValue::Object {
                    type_name: "LocalizationEstimate".into(),
                    fields: HashMap::from([
                        (
                            "pose".into(),
                            runtime_pose(state.pose.x, state.pose.y, state.pose.theta, 0.0),
                        ),
                        (
                            "confidence".into(),
                            RuntimeValue::Number {
                                value: 0.85,
                                unit: UnitKind::None,
                            },
                        ),
                    ]),
                })
            }
            "map" => {
                if let Some(output) = self.host.invoke_slam_bridge("map") {
                    self.log(format!("slam: bridge map output: {output}"));
                } else {
                    self.log("slam: map snapshot (stub adapter)".into());
                }
                Ok(RuntimeValue::Object {
                    type_name: "OccupancyGrid".into(),
                    fields: HashMap::from([
                        (
                            "resolution".into(),
                            RuntimeValue::Number {
                                value: 0.05,
                                unit: UnitKind::M,
                            },
                        ),
                        (
                            "width".into(),
                            RuntimeValue::Number {
                                value: 100.0,
                                unit: UnitKind::None,
                            },
                        ),
                    ]),
                })
            }
            _ => Err(
                RuntimeError::new(format!("Unknown slam method '{property}'"), line).into_spanda(),
            ),
        }
    }
}

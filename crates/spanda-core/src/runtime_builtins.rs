//! Builtin function dispatch for the interpreter.
//!

use super::{
    get_number, get_pose_fields, get_string, pose_value_to_state, runtime_pose, runtime_trajectory,
    runtime_velocity, IntoSpandaError, Interpreter, PoseValue, RobotBackend,
    RuntimeError, RuntimeValue,
};
use crate::ast::{Expr, UnitKind};
use crate::audit::{sha256 as audit_sha256, sign as audit_sign, verify_signature};
use crate::error::SpandaError;
use crate::safety::interpolate_poses;
use std::collections::HashMap;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn eval_builtin_function(
        &mut self,
        name: &str,
        args: &[Expr],
        named_args: &[crate::ast::NamedArg],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Eval builtin function.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
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
        // let result = instance.eval_builtin_function(name, args, named_args, line);

        // Match on name and handle each case.
        match name {
            "geo" => {
                let lat = if !args.is_empty() {
                    get_number(&self.eval_expr(&args[0])?, 0.0)
                } else {
                    0.0
                };
                let lon = if args.len() >= 2 {
                    get_number(&self.eval_expr(&args[1])?, 0.0)
                } else {
                    0.0
                };
                Ok(RuntimeValue::Object {
                    type_name: "GeoPoint".into(),
                    fields: HashMap::from([
                        (
                            "lat".into(),
                            RuntimeValue::Number {
                                value: lat,
                                unit: UnitKind::None,
                            },
                        ),
                        (
                            "lon".into(),
                            RuntimeValue::Number {
                                value: lon,
                                unit: UnitKind::None,
                            },
                        ),
                    ]),
                })
            }
            "pose" => Ok(runtime_pose(
                get_number(&self.get_named_arg_value(named_args, "x")?, 0.0),
                get_number(&self.get_named_arg_value(named_args, "y")?, 0.0),
                get_number(&self.get_named_arg_value(named_args, "theta")?, 0.0),
                get_number(&self.get_named_arg_value(named_args, "z")?, 0.0),
            )),
            "velocity" => Ok(runtime_velocity(
                get_number(&self.get_named_arg_value(named_args, "linear")?, 0.0),
                get_number(&self.get_named_arg_value(named_args, "angular")?, 0.0),
            )),
            "trajectory" => {
                let from_val = self.get_named_arg_value(named_args, "from")?;
                let to_val = self.get_named_arg_value(named_args, "to")?;
                let steps = get_number(&self.get_named_arg_value(named_args, "steps")?, 5.0);
                let from = get_pose_fields(&from_val).unwrap_or_default();
                let to = get_pose_fields(&to_val).unwrap_or_default();
                let waypoints: Vec<PoseValue> = interpolate_poses(
                    &pose_value_to_state(&from),
                    &pose_value_to_state(&to),
                    steps,
                )
                .into_iter()
                .map(|p| PoseValue {
                    x: p.x,
                    y: p.y,
                    theta: p.theta,
                    z: p.z,
                })
                .collect();
                Ok(runtime_trajectory(waypoints))
            }
            "transform" => {
                let from_frame = get_string(&self.get_named_arg_value(named_args, "from")?, "base");
                let to_frame = get_string(&self.get_named_arg_value(named_args, "to")?, "map");
                let pose = get_pose_fields(&self.get_named_arg_value(named_args, "pose")?)
                    .unwrap_or_default();
                Ok(RuntimeValue::Transform {
                    from_frame,
                    to_frame,
                    pose,
                })
            }
            "goal" => {
                let text = if let Some(arg) = args.first() {
                    // Match on eval expr and handle each case.
                    match self.eval_expr(arg)? {
                        RuntimeValue::String { value } => value,
                        RuntimeValue::Goal { text } => text,
                        _ => String::new(),
                    }
                } else {
                    get_string(&self.get_named_arg_value(named_args, "text")?, "")
                };
                Ok(RuntimeValue::Goal { text })
            }
            "recall" => {
                let key = if let Some(arg) = args.first() {
                    // Match on eval expr and handle each case.
                    match self.eval_expr(arg)? {
                        RuntimeValue::String { value } => value,
                        _ => String::new(),
                    }
                } else {
                    get_string(&self.get_named_arg_value(named_args, "key")?, "")
                };
                let agent_name = self.current_agent.clone().ok_or_else(|| {
                    RuntimeError::new(
                        "recall requires active agent context (run inside agent plan)",
                        line,
                    )
                    .into_spanda()
                })?;
                let agent = self.agents.get(&agent_name).ok_or_else(|| {
                    RuntimeError::new(format!("Unknown agent '{agent_name}'"), line).into_spanda()
                })?;
                let memory = agent.memory.as_ref().ok_or_else(|| {
                    RuntimeError::new(
                        "Agent has no memory — declare memory short_term or long_term on the agent",
                        line,
                    )
                    .into_spanda()
                })?;
                Ok(memory.recall(&key).cloned().unwrap_or(RuntimeValue::Void))
            }
            "sha256" => {
                let data = if let Some(arg) = args.first() {
                    get_string(&self.eval_expr(arg)?, "")
                } else {
                    get_string(&self.get_named_arg_value(named_args, "data")?, "")
                };
                let hash = audit_sha256(&data);
                Ok(RuntimeValue::Object {
                    type_name: "Hash".into(),
                    fields: HashMap::from([("hex".into(), RuntimeValue::String { value: hash.0 })]),
                })
            }
            "sign" => {
                self.security
                    .require_operation("identity.sign")
                    .map_err(|e| self.security_error(e, line))?;
                let data = if let Some(arg) = args.first() {
                    get_string(&self.eval_expr(arg)?, "")
                } else {
                    get_string(&self.get_named_arg_value(named_args, "data")?, "")
                };
                let key_raw = if args.len() > 1 {
                    get_string(&self.eval_expr(&args[1])?, "")
                } else {
                    get_string(&self.get_named_arg_value(named_args, "key")?, "")
                };
                let key = self.resolve_signing_key(&key_raw)?;
                Ok(RuntimeValue::Object {
                    type_name: "Signature".into(),
                    fields: HashMap::from([(
                        "value".into(),
                        RuntimeValue::String {
                            value: audit_sign(&data, &key),
                        },
                    )]),
                })
            }
            "verify_signature" => {
                self.security
                    .require_operation("identity.verify")
                    .map_err(|e| self.security_error(e, line))?;
                let data = if let Some(arg) = args.first() {
                    get_string(&self.eval_expr(arg)?, "")
                } else {
                    get_string(&self.get_named_arg_value(named_args, "data")?, "")
                };
                let signature = if args.len() > 1 {
                    get_string(&self.eval_expr(&args[1])?, "")
                } else {
                    get_string(&self.get_named_arg_value(named_args, "signature")?, "")
                };
                let key_raw = if args.len() > 2 {
                    get_string(&self.eval_expr(&args[2])?, "")
                } else {
                    get_string(&self.get_named_arg_value(named_args, "key")?, "")
                };
                let key = self.resolve_signing_key(&key_raw)?;
                Ok(RuntimeValue::Bool {
                    value: verify_signature(&data, &signature, &key),
                })
            }
            "Ok" => {
                let val = if let Some(arg) = args.first() {
                    self.eval_expr(arg)?
                } else {
                    RuntimeValue::Void
                };
                Ok(RuntimeValue::Result {
                    ok: true,
                    value: Box::new(val),
                })
            }
            "Err" => {
                let val = if let Some(arg) = args.first() {
                    self.eval_expr(arg)?
                } else {
                    RuntimeValue::Object {
                        type_name: "Error".into(),
                        fields: HashMap::new(),
                    }
                };
                Ok(RuntimeValue::Result {
                    ok: false,
                    value: Box::new(val),
                })
            }
            "Some" => {
                let val = if let Some(arg) = args.first() {
                    self.eval_expr(arg)?
                } else {
                    RuntimeValue::Void
                };
                Ok(RuntimeValue::Option {
                    present: true,
                    value: Some(Box::new(val)),
                })
            }
            "None" => Ok(RuntimeValue::Option {
                present: false,
                value: None,
            }),
            "channel" => Ok(self.concurrency.create_channel()),
            "send" => {
                let channel = args
                    .first()
                    .map(|a| self.eval_expr(a))
                    .transpose()?
                    .ok_or_else(|| {
                        RuntimeError::new("send requires channel", line).into_spanda()
                    })?;
                let value = if args.len() > 1 {
                    self.eval_expr(&args[1])?
                } else {
                    self.get_named_arg_value(named_args, "value")?
                };
                self.concurrency.bind_channel_type(&channel, &value, line)?;
                self.concurrency.send(&channel, value, line)?;
                Ok(RuntimeValue::Void)
            }
            "recv" => {
                let channel = args
                    .first()
                    .map(|a| self.eval_expr(a))
                    .transpose()?
                    .ok_or_else(|| {
                        RuntimeError::new("recv requires channel", line).into_spanda()
                    })?;

                // Match on try recv and handle each case.
                match self.concurrency.try_recv(&channel, line)? {
                    Some(value) => Ok(value),
                    None => Ok(RuntimeValue::Void),
                }
            }
            "join" => {
                let value = args
                    .first()
                    .map(|a| self.eval_expr(a))
                    .transpose()?
                    .ok_or_else(|| RuntimeError::new("join requires handle", line).into_spanda())?;

                // Match on value and handle each case.
                match value {
                    RuntimeValue::Future { .. } => {
                        self.telemetry.record_join();
                        self.trace_task_log("join future");
                        self.resolve_future(value, line)
                    }
                    RuntimeValue::TaskHandle { id } => self.resolve_task_handle(id, line),
                    _ => Err(
                        RuntimeError::new("join requires a Future or TaskHandle value", line)
                            .into_spanda(),
                    ),
                }
            }
            "send_agent" => {
                let from = self.current_agent.clone().ok_or_else(|| {
                    RuntimeError::new("send_agent requires active agent context", line)
                        .into_spanda()
                })?;
                let to = if let Some(arg) = args.first() {
                    get_string(&self.eval_expr(arg)?, "")
                } else {
                    get_string(&self.get_named_arg_value(named_args, "to")?, "")
                };

                // Skip further work when to is empty.
                if to.is_empty() {
                    return Err(
                        RuntimeError::new("send_agent requires target agent name", line)
                            .into_spanda(),
                    );
                }
                let value = if args.len() > 1 {
                    self.eval_expr(&args[1])?
                } else {
                    self.get_named_arg_value(named_args, "value")?
                };
                self.concurrency.send_agent(&from, &to, value, line)?;
                self.log(format!("send_agent {from} -> {to}"));
                Ok(RuntimeValue::Void)
            }
            "recv_agent" => {
                let agent = self.current_agent.clone().ok_or_else(|| {
                    RuntimeError::new("recv_agent requires active agent context", line)
                        .into_spanda()
                })?;

                // Match on try recv agent and handle each case.
                match self.concurrency.try_recv_agent(&agent, line) {
                    Some(value) => Ok(value),
                    None => Ok(RuntimeValue::Void),
                }
            }
            "peer_send" => {
                let peer = if let Some(arg) = args.first() {
                    get_string(&self.eval_expr(arg)?, "")
                } else {
                    get_string(&self.get_named_arg_value(named_args, "peer")?, "")
                };
                let topic = if args.len() > 1 {
                    get_string(&self.eval_expr(&args[1])?, "")
                } else {
                    get_string(&self.get_named_arg_value(named_args, "topic")?, "")
                };
                let value = if args.len() > 2 {
                    self.eval_expr(&args[2])?
                } else {
                    self.get_named_arg_value(named_args, "value")?
                };

                // Skip further work when peer is empty.
                if peer.is_empty() || topic.is_empty() {
                    return Err(
                        RuntimeError::new("peer_send requires (peer, topic, value)", line)
                            .into_spanda(),
                    );
                }
                let source = self.publish_source_id();
                self.comm_bus.publish_peer(
                    &peer,
                    &topic,
                    value,
                    self.default_transport,
                    Some(&source),
                );
                self.log(format!("peer_send {peer}.{topic}"));
                Ok(RuntimeValue::Void)
            }
            "serialize" => {
                let value = args
                    .first()
                    .map(|a| self.eval_expr(a))
                    .transpose()?
                    .unwrap_or(RuntimeValue::Void);
                let format = if args.len() > 1 {
                    get_string(&self.eval_expr(&args[1])?, "json")
                } else {
                    get_string(&self.get_named_arg_value(named_args, "format")?, "json")
                };
                crate::serialize::serialize_value(&value, &format)
            }
            "deserialize" => {
                let data = args
                    .first()
                    .map(|a| self.eval_expr(a))
                    .transpose()?
                    .ok_or_else(|| {
                        RuntimeError::new("deserialize requires data", line).into_spanda()
                    })?;
                let format = if args.len() > 1 {
                    get_string(&self.eval_expr(&args[1])?, "json")
                } else {
                    get_string(&self.get_named_arg_value(named_args, "format")?, "json")
                };
                crate::serialize::deserialize_value(&data, &format)
            }
            "assert" => {
                let condition = args
                    .first()
                    .map(|a| self.eval_expr(a))
                    .transpose()?
                    .ok_or_else(|| {
                        RuntimeError::new("assert requires a boolean condition", line).into_spanda()
                    })?;

                // Match on condition and handle each case.
                match condition {
                    RuntimeValue::Bool { value: true, .. } => Ok(RuntimeValue::Void),
                    RuntimeValue::Bool { value: false, .. } => {
                        Err(RuntimeError::new("Assertion failed", line).into_spanda())
                    }
                    _ => Err(
                        RuntimeError::new("assert requires a boolean condition", line)
                            .into_spanda(),
                    ),
                }
            }
            _ => Ok(RuntimeValue::Void),
        }
    }

}

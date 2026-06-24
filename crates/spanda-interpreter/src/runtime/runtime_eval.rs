//! Expression evaluation, member/call dispatch, and binary operators.
//!

use super::{Interpreter, IntoSpandaError, RobotBackend, RuntimeError, RuntimeValue};
use spanda_ai::{execute_agent_plan, mock_analyze_frame, mock_camera_frame, PlanExecutor};
use spanda_ast::comm_decl::DiscoverFilter;
use spanda_ast::nodes::{AgentDecl, BinaryOp, Expr, LiteralValue, Stmt, UnaryOp, UnitKind};
use spanda_comm::CommBus;
use spanda_error::SpandaError;
use spanda_regex_lang::{regex_capture, regex_find, regex_matches, regex_replace, regex_split};
use spanda_runtime::triggers::SystemTriggerCategory;
use spanda_typecheck::units::align_for_binary;
use std::collections::HashMap;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn eval_expr(&mut self, expr: &Expr) -> Result<RuntimeValue, SpandaError> {
        // Eval expr.
        //
        // Parameters:
        // - `self` — method receiver
        // - `expr` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.eval_expr(expr);

        // Match on expr and handle each case.
        match expr {
            Expr::LiteralExpr { value, .. } => Ok(match value {
                LiteralValue::Bool(b) => RuntimeValue::Bool { value: *b },
                LiteralValue::Number(n) => RuntimeValue::Number {
                    value: *n,
                    unit: UnitKind::None,
                },
                LiteralValue::String(s) => RuntimeValue::String { value: s.clone() },
                LiteralValue::Null => RuntimeValue::Void,
                LiteralValue::Regex(pattern) => RuntimeValue::Regex {
                    pattern: pattern.clone(),
                },
            }),
            Expr::UnitLiteralExpr { value, unit, .. } => Ok(RuntimeValue::Number {
                value: *value,
                unit: *unit,
            }),
            Expr::IdentExpr { name, span } => {
                // Emit output when get provides a enum name.
                if let Some(enum_name) = self.variant_owner.get(name) {
                    return Ok(RuntimeValue::Enum {
                        enum_name: enum_name.clone(),
                        variant: name.clone(),
                        payloads: Vec::new(),
                    });
                }
                self.env.get(name).cloned().ok_or_else(|| {
                    RuntimeError::new(format!("Undefined variable '{name}'"), span.start.line)
                        .into_spanda()
                })
            }
            Expr::BinaryExpr {
                op,
                left,
                right,
                span,
            } => {
                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;
                self.eval_binary(*op, left_val, right_val, span.start.line)
            }
            Expr::UnaryExpr {
                op,
                operand,
                span: _,
            } => {
                let operand_val = self.eval_expr(operand)?;

                // Match on op and handle each case.
                match op {
                    UnaryOp::Not => Ok(RuntimeValue::Bool {
                        value: matches!(operand_val, RuntimeValue::Bool { value, .. } if !value),
                    }),
                    UnaryOp::Neg => {
                        // Take this path when let RuntimeValue::Number { value, unit } = operand val.
                        if let RuntimeValue::Number { value, unit } = operand_val {
                            Ok(RuntimeValue::Number {
                                value: -value,
                                unit,
                            })
                        } else {
                            Ok(RuntimeValue::Void)
                        }
                    }
                }
            }
            Expr::MemberExpr {
                object,
                property,
                span: _,
            } => {
                // Take this path when let Expr::IdentExpr { name, .. } = object.as ref().
                if let Expr::IdentExpr { name, .. } = object.as_ref() {
                    // Emit output when get provides a variants.
                    if let Some(variants) = self.enum_variants.get(name) {
                        // Take the branch when any equals property).
                        if variants.iter().any(|v| v == property) {
                            return Ok(RuntimeValue::Enum {
                                enum_name: name.clone(),
                                variant: property.clone(),
                                payloads: Vec::new(),
                            });
                        }
                    }
                }
                let obj = self.eval_expr(object)?;
                self.eval_member(&obj, property)
            }
            Expr::CallExpr {
                callee,
                args,
                named_args,
                span,
            } => self.eval_call(callee, args, named_args, span.start.line),
            Expr::AwaitExpr { operand, span } => {
                let value = self.eval_expr(operand)?;
                self.resolve_future(value, span.start.line)
            }
            Expr::SpawnExpr { callee, args, span } => {
                let (name, arg_values) = self.eval_spawn_target(callee, args, span.start.line)?;
                self.telemetry.record_spawn();
                self.trace_task_log(format!("spawn handle {name}"));
                Ok(self.concurrency.create_task_handle(name, arg_values))
            }
            Expr::MatchExpr {
                scrutinee, arms, ..
            } => {
                let value = self.eval_expr(scrutinee)?;
                let variant = match &value {
                    RuntimeValue::Enum { variant, .. } => variant.clone(),
                    RuntimeValue::Result { ok: true, .. } => "Ok".into(),
                    RuntimeValue::Result { ok: false, .. } => "Err".into(),
                    RuntimeValue::Option { present: true, .. } => "Some".into(),
                    RuntimeValue::Option { present: false, .. } => "None".into(),
                    RuntimeValue::String { value } => value.clone(),
                    RuntimeValue::Object { type_name, .. } => type_name.clone(),
                    _ => String::new(),
                };

                // Process each arm.
                for arm in arms {
                    // Take the branch when variant equals variant.
                    if arm.variant == variant {
                        // Skip further work when bindings is empty.
                        if !arm.bindings.is_empty() {
                            // Take this path when let RuntimeValue::Enum { payloads, .. } = &value.
                            if let RuntimeValue::Enum { payloads, .. } = &value {
                                // Iterate over iter with destructured elements.
                                for (binding, payload) in arm.bindings.iter().zip(payloads.iter()) {
                                    self.env.set(binding.clone(), payload.clone());
                                }
                            }
                        }

                        // Execute each statement in sequence.
                        for stmt in &arm.body {
                            self.execute_stmt(stmt)?;
                        }

                        // Process each binding.
                        for binding in &arm.bindings {
                            self.env.remove(binding);
                        }
                        break;
                    }
                }
                Ok(RuntimeValue::Void)
            }
            Expr::StructLiteralExpr {
                type_name,
                fields,
                span,
            } => self.eval_struct_literal(type_name, fields, span.start.line),
            Expr::ServiceCallExpr { service_name, .. } => {
                // Take this path when let Some(RuntimeValue::Service { name, service type }) =.
                if let Some(RuntimeValue::Service { name, service_type }) =
                    self.env.get(service_name).cloned()
                {
                    let result = self.comm_bus.call_service(&name, &service_type, None);
                    self.backend.call_service(&name, &service_type);
                    self.log(format!("call {name}()"));
                    Ok(result)
                } else {
                    Ok(RuntimeValue::Void)
                }
            }
            Expr::ExecuteExpr {
                action_name, goal, ..
            } => {
                // Take this path when let Some(RuntimeValue::Action { name, action type }) =.
                if let Some(RuntimeValue::Action { name, action_type }) =
                    self.env.get(action_name).cloned()
                {
                    let goal_val = self.eval_expr(goal)?;
                    let result = self
                        .comm_bus
                        .send_action(&name, &action_type, goal_val.clone());
                    self.backend.send_action(&name, &action_type, goal_val);
                    self.log(format!("execute {name}"));
                    Ok(result)
                } else {
                    Ok(RuntimeValue::Void)
                }
            }
            Expr::DiscoverExpr { target, filter, .. } => {
                let results = self.comm_bus.discover(
                    *target,
                    filter
                        .as_ref()
                        .unwrap_or(&DiscoverFilter { capability: None }),
                );
                Ok(RuntimeValue::Object {
                    type_name: "DiscoveryResult".into(),
                    fields: HashMap::from([(
                        "count".into(),
                        RuntimeValue::Number {
                            value: results.len() as f64,
                            unit: UnitKind::None,
                        },
                    )]),
                })
            }
        }
    }

    fn eval_struct_literal(
        &mut self,
        type_name: &str,
        fields: &[spanda_ast::nodes::StructFieldInit],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Eval struct literal.
        //
        // Parameters:
        // - `self` — method receiver
        // - `type_name` — input value
        // - `fields` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.eval_struct_literal(type_name, fields, line);

        // Create mutable values for accumulating results.
        let mut values = HashMap::new();

        // Check each struct field.
        for field in fields {
            values.insert(field.name.clone(), self.eval_expr(&field.value)?);
        }

        // Take the branch when type name equals "Pose".
        if type_name == "Pose" {
            let x = values
                .get("x")
                .and_then(|v| v.as_number())
                .ok_or_else(|| RuntimeError::new("Pose.x must be numeric", line).into_spanda())?;
            let y = values
                .get("y")
                .and_then(|v| v.as_number())
                .ok_or_else(|| RuntimeError::new("Pose.y must be numeric", line).into_spanda())?;
            let heading = values
                .get("heading")
                .or_else(|| values.get("theta"))
                .and_then(|v| v.as_number())
                .unwrap_or(0.0);
            let z = values.get("z").and_then(|v| v.as_number()).unwrap_or(0.0);
            return Ok(RuntimeValue::Pose {
                x,
                y,
                theta: heading,
                z,
            });
        }
        Ok(RuntimeValue::Object {
            type_name: type_name.to_string(),
            fields: values,
        })
    }

    fn eval_member(
        &mut self,
        obj: &RuntimeValue,
        property: &str,
    ) -> Result<RuntimeValue, SpandaError> {
        // Eval member.
        //
        // Parameters:
        // - `self` — method receiver
        // - `obj` — input value
        // - `property` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.eval_member(obj, property);

        // Match on obj and handle each case.
        match obj {
            RuntimeValue::Scan { nearest_distance } if property == "nearest_distance" => {
                Ok(RuntimeValue::Number {
                    value: *nearest_distance,
                    unit: UnitKind::M,
                })
            }
            RuntimeValue::Pose { x, y, theta, z } => match property {
                "x" => Ok(RuntimeValue::Number {
                    value: *x,
                    unit: UnitKind::M,
                }),
                "y" => Ok(RuntimeValue::Number {
                    value: *y,
                    unit: UnitKind::M,
                }),
                "theta" => Ok(RuntimeValue::Number {
                    value: *theta,
                    unit: UnitKind::Rad,
                }),
                "z" => Ok(RuntimeValue::Number {
                    value: *z,
                    unit: UnitKind::M,
                }),
                _ => Ok(RuntimeValue::Void),
            },
            RuntimeValue::Velocity { linear, angular } => match property {
                "linear" => Ok(RuntimeValue::Number {
                    value: *linear,
                    unit: UnitKind::MPerS,
                }),
                "angular" => Ok(RuntimeValue::Number {
                    value: *angular,
                    unit: UnitKind::RadPerS,
                }),
                _ => Ok(RuntimeValue::Void),
            },
            RuntimeValue::Sensor { .. } if property == "nearest_distance" => {
                // Take this path when let RuntimeValue::Scan { nearest distance } = self.read sensor value(o.
                if let RuntimeValue::Scan { nearest_distance } = self.read_sensor_value(obj)? {
                    Ok(RuntimeValue::Number {
                        value: nearest_distance,
                        unit: UnitKind::M,
                    })
                } else {
                    Ok(RuntimeValue::Void)
                }
            }
            RuntimeValue::ActionProposal {
                linear,
                angular,
                source,
                trace,
            } => match property {
                "linear" => Ok(RuntimeValue::Number {
                    value: *linear,
                    unit: UnitKind::MPerS,
                }),
                "angular" => Ok(RuntimeValue::Number {
                    value: *angular,
                    unit: UnitKind::RadPerS,
                }),
                "trace" => {
                    let mut fields = HashMap::new();
                    fields.insert("source".to_string(), RuntimeValue::string(source.clone()));
                    fields.insert("steps".to_string(), RuntimeValue::string(trace.join("\n")));
                    fields.insert(
                        "step_count".to_string(),
                        RuntimeValue::Number {
                            value: trace.len() as f64,
                            unit: UnitKind::None,
                        },
                    );
                    Ok(RuntimeValue::object("ReasoningTrace", fields))
                }
                _ => Ok(RuntimeValue::Void),
            },
            RuntimeValue::SafeAction { linear, angular } => match property {
                "linear" => Ok(RuntimeValue::Number {
                    value: *linear,
                    unit: UnitKind::MPerS,
                }),
                "angular" => Ok(RuntimeValue::Number {
                    value: *angular,
                    unit: UnitKind::RadPerS,
                }),
                _ => Ok(RuntimeValue::Void),
            },
            RuntimeValue::Goal { text } if property == "text" => {
                Ok(RuntimeValue::string(text.clone()))
            }
            RuntimeValue::Agent { name } if property == "goal" => {
                let text = self
                    .agents
                    .get(name)
                    .map(|agent| match &agent.decl {
                        AgentDecl::AgentDecl { goal, .. } => goal.clone(),
                    })
                    .unwrap_or_default();
                Ok(RuntimeValue::Goal { text })
            }
            RuntimeValue::Completion { text, .. } if property == "text" => {
                Ok(RuntimeValue::String {
                    value: text.clone(),
                })
            }
            RuntimeValue::Object { fields, .. } => {
                Ok(fields.get(property).cloned().unwrap_or(RuntimeValue::Void))
            }
            _ => Ok(RuntimeValue::Void),
        }
    }

    fn eval_string_regex_method(
        &mut self,
        method: &str,
        text: &str,
        args: &[Expr],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Evaluate string regex helper methods: matches, find, replace, split, capture.
        let pattern_val = args.first().ok_or_else(|| {
            RuntimeError::new("Regex method requires pattern argument", line).into_spanda()
        })?;
        let pattern = match self.eval_expr(pattern_val)? {
            RuntimeValue::Regex { pattern } => pattern,
            _ => {
                return Err(
                    RuntimeError::new("Regex method requires Regex pattern argument", line)
                        .into_spanda(),
                )
            }
        };
        match method {
            "matches" => Ok(RuntimeValue::Bool {
                value: regex_matches(&pattern, text)?,
            }),
            "find" => Ok(match regex_find(&pattern, text)? {
                Some(found) => RuntimeValue::String { value: found },
                None => RuntimeValue::Null,
            }),
            "replace" => {
                let replacement = args.get(1).ok_or_else(|| {
                    RuntimeError::new("replace requires replacement argument", line).into_spanda()
                })?;
                let replacement = match self.eval_expr(replacement)? {
                    RuntimeValue::String { value } => value,
                    _ => {
                        return Err(
                            RuntimeError::new("replace replacement must be string", line)
                                .into_spanda(),
                        )
                    }
                };
                Ok(RuntimeValue::String {
                    value: regex_replace(&pattern, text, &replacement)?,
                })
            }
            "split" => {
                let parts = regex_split(&pattern, text)?;
                Ok(RuntimeValue::Object {
                    type_name: "StringList".into(),
                    fields: parts
                        .into_iter()
                        .enumerate()
                        .map(|(i, part)| (i.to_string(), RuntimeValue::String { value: part }))
                        .collect(),
                })
            }
            "capture" => Ok(match regex_capture(&pattern, text)? {
                Some(result) => RuntimeValue::Capture { result },
                None => RuntimeValue::Null,
            }),
            _ => Ok(RuntimeValue::Void),
        }
    }

    fn eval_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        named_args: &[spanda_ast::nodes::NamedArg],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Eval call.
        //
        // Parameters:
        // - `self` — method receiver
        // - `callee` — input value
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
        // let result = instance.eval_call(callee, args, named_args, line);

        if let Expr::IdentExpr { name, .. } = callee {
            if let Some(ext) = self.extern_functions.get(name).cloned() {
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.eval_expr(arg)?);
                }
                return self.options.ffi_registry.call(&ext, &arg_values);
            }
            if let Some(func) = self
                .module_functions
                .get(name)
                .or_else(|| self.imported_functions.get(name))
                .cloned()
            {
                if func.is_async {
                    let mut arg_values = Vec::new();
                    for arg in args {
                        arg_values.push(self.eval_expr(arg)?);
                    }
                    return Ok(RuntimeValue::Future {
                        func_name: func.name.clone(),
                        args: arg_values,
                        resolved: None,
                    });
                }
                return self.call_module_function(&func, args, line);
            }
            if let Some(enum_name) = self.variant_owner.get(name).cloned() {
                let mut payloads = Vec::new();
                for arg in args {
                    payloads.push(self.eval_expr(arg)?);
                }
                return Ok(RuntimeValue::Enum {
                    enum_name,
                    variant: name.clone(),
                    payloads,
                });
            }
            return self.eval_builtin_function(name, args, named_args, line);
        }

        let Expr::MemberExpr {
            object, property, ..
        } = callee
        else {
            return Ok(RuntimeValue::Void);
        };

        // Handle string regex methods on arbitrary receiver expressions.
        if let Ok(RuntimeValue::String { value: text }) = self.eval_expr(object) {
            return self.eval_string_regex_method(property, &text, args, line);
        }

        let Expr::IdentExpr {
            name: target_name, ..
        } = object.as_ref()
        else {
            return Ok(RuntimeValue::Void);
        };

        let target = self.env.get(target_name).cloned().ok_or_else(|| {
            RuntimeError::new(format!("Undefined '{target_name}'"), line).into_spanda()
        })?;

        if matches!(target, RuntimeValue::Robot) || target_name == "robot" {
            return self.eval_robot_method(property, args, named_args);
        }

        if matches!(target, RuntimeValue::Twin { .. }) {
            return self.eval_twin_method(property, args, named_args, line);
        }

        if let RuntimeValue::SensorFusion {
            ref sensors,
            ref estimator,
        } = target
        {
            if property == "read" {
                return self.read_fused_observation(sensors, estimator.as_deref());
            }
        }

        if let RuntimeValue::MissionControl { mut runtime } = target {
            let result = self.eval_mission_method(&mut runtime, property, line)?;
            self.env.define(
                target_name.clone(),
                RuntimeValue::MissionControl { runtime },
            );
            return Ok(result);
        }

        if let RuntimeValue::NavigationControl { mut goal } = target {
            let result =
                self.eval_navigation_method(&mut goal, property, args, named_args, line)?;
            self.env.define(
                target_name.clone(),
                RuntimeValue::NavigationControl { goal },
            );
            return Ok(result);
        }

        if matches!(target, RuntimeValue::SlamControl) {
            return self.eval_slam_method(property, line);
        }

        if let RuntimeValue::FleetControl { registry } = target {
            return self.eval_fleet_method(&registry, property, args, line);
        }

        if let RuntimeValue::Sensor { sensor_type, .. } = &target {
            if property == "read" {
                if let Some(agent) = self.current_agent.clone() {
                    self.check_agent_capability(&agent, "read", Some(target_name), line)?;
                }
                return self.read_sensor_value(&target);
            }
            if sensor_type == "Camera" {
                if property == "frame" {
                    return Ok(mock_camera_frame());
                }
                if property == "analyze" {
                    let frame = mock_camera_frame();
                    return Ok(mock_analyze_frame(Some(&frame), target_name));
                }
            }
        }

        let target = match target {
            RuntimeValue::TraitObject { agent, .. } => RuntimeValue::Agent { name: agent },
            other => other,
        };

        if let RuntimeValue::Agent { name } = &target {
            if let Some((params, body)) = self
                .agent_trait_impls
                .get(name)
                .and_then(|methods| methods.get(property))
                .cloned()
            {
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.eval_expr(arg)?);
                }
                let saved = self.env.clone();
                for (param, value) in params.iter().zip(arg_values) {
                    self.env.define(param.name.clone(), value);
                }
                self.current_agent = Some(name.clone());
                self.execute_block(&body)?;
                self.current_agent = None;
                self.env = saved;
                self.log(format!("agent {name}.{property}()"));
                return Ok(RuntimeValue::Void);
            }
            if property == "plan" {
                self.check_agent_capability(name, "plan", None, line)?;
                let agent = self.agents.get(name).ok_or_else(|| {
                    RuntimeError::new(format!("Unknown agent '{name}'"), line).into_spanda()
                })?;
                let agent = agent.clone();
                struct PlanRunner<'a, B: RobotBackend> {
                    interp: &'a mut Interpreter<B>,
                    agent_name: String,
                }
                impl<B: RobotBackend> PlanExecutor for PlanRunner<'_, B> {
                    fn execute_block(&mut self, stmts: &[Stmt]) -> Result<(), SpandaError> {
                        // Execute block.
                        //
                        // Parameters:
                        // - `self` — method receiver
                        // - `stmts` — input value
                        //
                        // Returns:
                        // Success value on completion, or an error.
                        //
                        // Options:
                        // None.
                        //
                        // Example:
                        // let result = instance.execute_block(stmts);

                        // Call current agent = Some on the current instance.
                        self.interp.current_agent = Some(self.agent_name.clone());
                        let result = self.interp.execute_block(stmts);
                        self.interp.current_agent = None;
                        result
                    }
                }
                let mut runner = PlanRunner {
                    interp: self,
                    agent_name: name.clone(),
                };
                execute_agent_plan(&agent, &mut runner)?;
                let _ = self.dispatch_system_trigger(SystemTriggerCategory::Ai, "GoalCompleted");
                self.log(format!("agent {name}.plan()"));
                return Ok(RuntimeValue::Void);
            }
        }

        if matches!(target, RuntimeValue::SafetyCtx) && property == "validate" {
            return self.eval_safety_validate(args, named_args, line);
        }

        if matches!(target, RuntimeValue::AuditCtx) {
            return self.eval_audit_method(property, args, named_args, line);
        }

        if matches!(target, RuntimeValue::LedgerCtx) {
            return self.eval_ledger_method(property, args, named_args, line);
        }

        if matches!(target, RuntimeValue::WorldModelCtx) {
            return self.eval_world_model_method(property, args, named_args, line);
        }

        if self.ai_models.contains_key(target_name)
            || matches!(target, RuntimeValue::AiModel { .. })
        {
            return self.eval_ai_method(target_name, property, args, named_args, line);
        }

        if let RuntimeValue::Actuator {
            name,
            actuator_type,
        } = target
        {
            return self.execute_actuator_method(
                &name,
                &actuator_type,
                property,
                args,
                named_args,
                line,
            );
        }

        Ok(RuntimeValue::Void)
    }

    pub(super) fn get_named_arg_value(
        &mut self,
        named_args: &[spanda_ast::nodes::NamedArg],
        name: &str,
    ) -> Result<RuntimeValue, SpandaError> {
        //
        // Parameters:
        // - `self` — method receiver
        // - `named_args` — input value
        // - `name` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.get_named_arg_value(named_args, name);

        // Apply each command-line argument.
        for arg in named_args {
            // Take the branch when name equals name.
            if arg.name == name {
                return self.eval_expr(&arg.value);
            }
        }
        Ok(RuntimeValue::Void)
    }

    fn eval_binary(
        &self,
        op: BinaryOp,
        left: RuntimeValue,
        right: RuntimeValue,
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Eval binary.
        //
        // Parameters:
        // - `self` — method receiver
        // - `op` — input value
        // - `left` — input value
        // - `right` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.eval_binary(op, left, right, line);

        // Match on op and handle each case.
        match op {
            BinaryOp::And => Ok(RuntimeValue::Bool {
                value: matches!(left, RuntimeValue::Bool { value: true, .. })
                    && matches!(right, RuntimeValue::Bool { value: true, .. }),
            }),
            BinaryOp::Or => Ok(RuntimeValue::Bool {
                value: matches!(left, RuntimeValue::Bool { value: true, .. })
                    || matches!(right, RuntimeValue::Bool { value: true, .. }),
            }),
            _ => {
                // Keep entries that match the expected pattern.
                if matches!(op, BinaryOp::Eq | BinaryOp::Neq)
                    && matches!(left, RuntimeValue::Enum { .. })
                    && matches!(right, RuntimeValue::Enum { .. })
                {
                    let RuntimeValue::Enum {
                        enum_name: e1,
                        variant: v1,
                        payloads: p1,
                    } = left
                    // Handle any remaining cases.
                    else {
                        unreachable!()
                    };
                    let RuntimeValue::Enum {
                        enum_name: e2,
                        variant: v2,
                        payloads: p2,
                    } = right
                    // Handle any remaining cases.
                    else {
                        unreachable!()
                    };
                    let equal = e1 == e2 && v1 == v2 && p1 == p2;
                    return Ok(RuntimeValue::Bool {
                        value: if op == BinaryOp::Eq { equal } else { !equal },
                    });
                }

                // Keep entries that match the expected pattern.
                if matches!(op, BinaryOp::Eq | BinaryOp::Neq)
                    && matches!(left, RuntimeValue::Bool { .. })
                    && matches!(right, RuntimeValue::Bool { .. })
                {
                    let RuntimeValue::Bool { value: l, .. } = left else {
                        unreachable!()
                    };
                    let RuntimeValue::Bool { value: r, .. } = right else {
                        unreachable!()
                    };
                    return Ok(RuntimeValue::Bool {
                        value: if op == BinaryOp::Eq { l == r } else { l != r },
                    });
                }

                // Take this path when let (.
                if let (
                    RuntimeValue::Number { value: l, unit: lu },
                    RuntimeValue::Number { value: r, unit: ru },
                ) = (left, right)
                {
                    let (l, r, result_unit) = align_for_binary(l, lu, r, ru).unwrap_or((l, r, lu));
                    return Ok(match op {
                        BinaryOp::Add => RuntimeValue::Number {
                            value: l + r,
                            unit: result_unit,
                        },
                        BinaryOp::Sub => RuntimeValue::Number {
                            value: l - r,
                            unit: result_unit,
                        },
                        BinaryOp::Mul => RuntimeValue::Number {
                            value: l * r,
                            unit: UnitKind::None,
                        },
                        BinaryOp::Div => RuntimeValue::Number {
                            value: l / r,
                            unit: UnitKind::None,
                        },
                        BinaryOp::Lt => RuntimeValue::Bool { value: l < r },
                        BinaryOp::Lte => RuntimeValue::Bool { value: l <= r },
                        BinaryOp::Gt => RuntimeValue::Bool { value: l > r },
                        BinaryOp::Gte => RuntimeValue::Bool { value: l >= r },
                        BinaryOp::Eq => RuntimeValue::Bool { value: l == r },
                        BinaryOp::Neq => RuntimeValue::Bool { value: l != r },
                        _ => RuntimeValue::Void,
                    });
                }
                let _ = line;
                Ok(RuntimeValue::Void)
            }
        }
    }
}

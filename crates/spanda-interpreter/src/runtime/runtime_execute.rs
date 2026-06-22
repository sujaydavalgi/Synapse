//! Statement and block execution for the interpreter.
//!

use super::{
    IntoSpandaError, Interpreter, MotionCommand, RobotBackend, RuntimeError, RuntimeValue,
};
use spanda_ast::nodes::{Expr, SpandaType, Stmt};
use crate::comm::CommBus;
use spanda_ast::comm_decl::DiscoverFilter;
use crate::error::SpandaError;
use spanda_runtime::triggers::SystemTriggerCategory;
use std::collections::HashMap;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn execute_block(&mut self, stmts: &[Stmt]) -> Result<(), SpandaError> {
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

        // Execute each statement in sequence.
        for stmt in stmts {
            self.execute_stmt(stmt)?;
        }
        Ok(())
    }

    pub(super) fn execute_stmt(&mut self, stmt: &Stmt) -> Result<(), SpandaError> {
        // Execute stmt.
        //
        // Parameters:
        // - `self` — method receiver
        // - `stmt` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_stmt(stmt);

        // use debug when debug is present.

        // Emit output when debug provides a debug.
        if let Some(debug) = &self.options.debug {
            let line = crate::debug::stmt_line(stmt);

            // Take this path when debug.should pause(line).
            if debug.should_pause(line) {
                let variables = self.env.snapshot_display();
                debug.record_pause(line, "breakpoint", variables);
                return Err(SpandaError::DebugPause {
                    line,
                    reason: "breakpoint".into(),
                });
            }
        }

        // Match on stmt and handle each case.
        match stmt {
            Stmt::VarDecl {
                name,
                type_annotation,
                init,
                ..
            } => {
                // Emit output when init provides a expr.
                if let Some(expr) = init {
                    let value = if matches!(type_annotation, Some(SpandaType::TraitObject { .. })) {
                        // Take this path when let Expr::IdentExpr { name: agent, .. } = expr.
                        if let Expr::IdentExpr { name: agent, .. } = expr {
                            // Take this path when let Some(SpandaType::TraitObject { trait name }) = type annotation.
                            if let Some(SpandaType::TraitObject { trait_name }) = type_annotation {
                                RuntimeValue::TraitObject {
                                    trait_name: trait_name.clone(),
                                    agent: agent.clone(),
                                }
                            } else {
                                self.eval_expr(expr)?
                            }
                        } else {
                            self.eval_expr(expr)?
                        }
                    } else {
                        self.eval_expr(expr)?
                    };
                    self.env.define(name.clone(), value);
                } else {
                    self.env.define(name.clone(), RuntimeValue::Void);
                }
            }
            Stmt::IfStmt {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let cond = self.eval_expr(condition)?;

                // Keep entries that match the expected pattern.
                if matches!(cond, RuntimeValue::Bool { value: true, .. }) {
                    self.execute_block(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute_block(else_branch)?;
                }
            }
            Stmt::LoopStmt {
                interval_ms, body, ..
            } => {
                // Process each max loop iteration.
                for _ in 0..self.options.max_loop_iterations {
                    self.backend.tick(*interval_ms);
                    self.execute_block(body)?;

                    // Take this path when self.
                    if self
                        .safety_monitor
                        .as_ref()
                        .map(|m| m.is_emergency_stop())
                        .unwrap_or(false)
                    {
                        break;
                    }
                }
            }
            Stmt::PublishStmt {
                topic_name,
                value,
                span,
            } => {
                let line = span.start.line;

                // Emit output when clone provides a agent.
                if let Some(agent) = self.current_agent.clone() {
                    self.check_agent_capability(&agent, "publish", Some(topic_name), line)?;
                }
                let topic = self.env.get(topic_name).cloned();
                let val = self.eval_expr(value)?;

                // Take this path when let Some(RuntimeValue::Topic.
                if let Some(RuntimeValue::Topic {
                    topic_path,
                    message_type,
                    ..
                }) = topic
                {
                    let payload = Self::runtime_value_payload(&val);
                    let source_id = self.publish_source_id();

                    if let Err(e) = self.security.prepare_publish(
                        &topic_path,
                        &payload,
                        &source_id,
                        &message_type,
                    ) {
                        if let Some(rt) = self.audit_runtime.as_mut() {
                            let _ = self.security.audit_security_event(
                                rt,
                                "publish_denied",
                                &format!("topic={topic_path} source={source_id} reason={e}"),
                            );
                        }
                        return Err(self.security_error(e, line));
                    }
                    if let Some(rt) = self.audit_runtime.as_mut() {
                        let _ = self.security.audit_security_event(
                            rt,
                            "encryption_enabled",
                            &format!("topic={topic_path}"),
                        );
                    }
                    self.comm_bus.publish(
                        &topic_path,
                        &message_type,
                        val.clone(),
                        self.default_transport,
                        Some(&source_id),
                    );
                    self.backend.publish_topic(&topic_path, &message_type, val);
                    self.topic_last_publish_ms
                        .insert(topic_path.clone(), self.sim_time_ms);
                    self.record_mission_event(
                        "topic_publish",
                        serde_json::json!({ "topic": topic_path }),
                    );

                    // Emit output when as mut provides a rt.
                    if let Some(rt) = self.audit_runtime.as_mut() {
                        let _ = self.security.audit_event(
                            rt,
                            "publish",
                            &format!("topic={topic_path}"),
                        );
                    }
                    self.log(format!("publish {topic_path}"));
                    let _ = self.dispatch_message_triggers(topic_name, &topic_path);
                }
            }
            Stmt::ServiceCallStmt {
                service_name, span, ..
            } => {
                let line = span.start.line;

                // Emit output when clone provides a agent.
                if let Some(agent) = self.current_agent.clone() {
                    self.check_agent_capability(&agent, "call", Some(service_name), line)?;
                }

                // Take this path when let Some(RuntimeValue::Service { name, service type }) =.
                if let Some(RuntimeValue::Service { name, service_type }) =
                    self.env.get(service_name).cloned()
                {
                    let endpoint = format!("/service/{name}");

                    // Handle the error returned from verify inbound.
                    if let Err(e) = self.security.verify_inbound(&endpoint, None, None) {
                        return Err(self.security_error(e, line));
                    }
                    self.backend.call_service(&name, &service_type);
                    self.log(format!("call {name}()"));
                }
            }
            Stmt::ActionSendStmt {
                action_name,
                goal,
                span,
                ..
            } => {
                let line = span.start.line;

                // Emit output when clone provides a agent.
                if let Some(agent) = self.current_agent.clone() {
                    self.check_agent_capability(&agent, "execute", Some(action_name), line)?;
                }

                // Take this path when let Some(RuntimeValue::Action { name, action type }) =.
                if let Some(RuntimeValue::Action { name, action_type }) =
                    self.env.get(action_name).cloned()
                {
                    let endpoint = format!("/action/{name}");
                    let goal_val = self.eval_expr(goal)?;
                    let payload = Self::runtime_value_payload(&goal_val);

                    // Handle the error returned from sign outbound.
                    if let Err(e) = self.security.sign_outbound(&endpoint, &payload) {
                        return Err(self.security_error(e, line));
                    }
                    self.comm_bus
                        .send_action(&name, &action_type, goal_val.clone());
                    self.backend.send_action(&name, &action_type, goal_val);
                    self.log(format!("send_goal {name}"));
                }
            }
            Stmt::SubscribeStmt { target, span, .. } => {
                let line = span.start.line;

                // Emit output when clone provides a agent.
                if let Some(agent) = self.current_agent.clone() {
                    self.check_agent_capability(&agent, "subscribe", Some(target), line)?;
                }
                let path = if target.contains('.') {
                    format!("/{}", target.replace('.', "/"))
                } else if let Some(RuntimeValue::Topic { topic_path, .. }) = self.env.get(target) {
                    topic_path.clone()
                } else {
                    format!("/{target}")
                };

                // Handle the error returned from verify inbound.
                if let Err(e) = self.security.authorize_subscribe(&path) {
                    return Err(self.security_error(e, line));
                }
                self.comm_bus.subscribe(&path, target);
                self.log(format!("subscribe {target}"));
            }
            Stmt::ExecuteStmt {
                action_name,
                goal,
                span,
                ..
            } => {
                let line = span.start.line;

                // Emit output when clone provides a agent.
                if let Some(agent) = self.current_agent.clone() {
                    self.check_agent_capability(&agent, "execute", Some(action_name), line)?;
                }

                // Take this path when let Some(RuntimeValue::Action { name, action type }) =.
                if let Some(RuntimeValue::Action { name, action_type }) =
                    self.env.get(action_name).cloned()
                {
                    let endpoint = format!("/action/{name}");
                    let goal_val = self.eval_expr(goal)?;
                    let payload = Self::runtime_value_payload(&goal_val);

                    // Handle the error returned from sign outbound.
                    if let Err(e) = self.security.sign_outbound(&endpoint, &payload) {
                        return Err(self.security_error(e, line));
                    }
                    self.comm_bus
                        .send_action(&name, &action_type, goal_val.clone());
                    self.backend.send_action(&name, &action_type, goal_val);
                    self.log(format!("execute {name}"));
                }
            }
            Stmt::DiscoverStmt {
                target,
                filter,
                span,
                ..
            } => {
                let line = span.start.line;

                // Emit output when clone provides a agent.
                if let Some(agent) = self.current_agent.clone() {
                    self.check_agent_capability(&agent, "discover", None, line)?;
                }
                let results = self.comm_bus.discover(
                    *target,
                    filter
                        .as_ref()
                        .unwrap_or(&DiscoverFilter { capability: None }),
                );
                self.log(format!("discover {:?}: {:?}", target, results));
                let _ = line;
            }
            Stmt::ReceiveStmt {
                topic_name,
                var_name,
                span,
                ..
            } => {
                let line = span.start.line;
                let path = if topic_name.contains('.') {
                    format!("/{}", topic_name.replace('.', "/"))
                } else if let Some(RuntimeValue::Topic { topic_path, .. }) =
                    self.env.get(topic_name)
                {
                    topic_path.clone()
                } else {
                    format!("/{topic_name}")
                };

                // Emit output when receive provides a val.
                if let Some(envelope) = self.comm_bus.receive_envelope(&path) {
                    let payload = Self::runtime_value_payload(&envelope.value);
                    if let Err(e) = self.security.verify_inbound_message(
                        &path,
                        &payload,
                        envelope.source_id.as_deref(),
                        None,
                        self.topic_path_to_message_type
                            .get(&path)
                            .map(String::as_str)
                            .unwrap_or("Unknown"),
                    ) {
                        return Err(self.security_error(e, line));
                    }
                    self.env.define(var_name.clone(), envelope.value);
                    self.log(format!("receive {topic_name} to {var_name}"));
                }
            }
            Stmt::EmergencyStopStmt { .. } => {
                // Emit output when safety monitor provides a monitor.
                if let Some(monitor) = &mut self.safety_monitor {
                    monitor.set_emergency_stop(true);
                }
                self.backend.set_emergency_stop(true);
                self.backend.execute_motion(MotionCommand::Stop {
                    actuator: "all".into(),
                });
                self.log("EMERGENCY STOP triggered".into());
                let _ =
                    self.dispatch_system_trigger(SystemTriggerCategory::Safety, "EmergencyStop");
            }
            Stmt::ResetEmergencyStopStmt { .. } => {
                // Emit output when safety monitor provides a monitor.
                if let Some(monitor) = &mut self.safety_monitor {
                    monitor.reset();
                }
                self.backend.set_emergency_stop(false);
                self.log("Emergency stop reset".into());
            }
            Stmt::EmitStmt { event_name, .. } => {
                self.dispatch_event(event_name)?;
            }
            Stmt::EnterStmt { state_name, span } => {
                self.execute_enter(state_name, span.start.line)?;
            }
            Stmt::RememberStmt { key, value, span } => {
                let stored = self.eval_expr(value)?;
                let agent_name = self.current_agent.clone().ok_or_else(|| {
                    RuntimeError::new(
                        "remember requires active agent context (run inside agent plan)",
                        span.start.line,
                    )
                    .into_spanda()
                })?;
                let agent = self.agents.get_mut(&agent_name).ok_or_else(|| {
                    RuntimeError::new(format!("Unknown agent '{agent_name}'"), span.start.line)
                        .into_spanda()
                })?;
                let memory = agent.memory.as_mut().ok_or_else(|| {
                    RuntimeError::new(
                        "Agent has no memory — declare memory short_term or long_term on the agent",
                        span.start.line,
                    )
                    .into_spanda()
                })?;
                memory.remember(key.clone(), stored);
                self.log(format!("remember '{key}'"));
            }
            Stmt::ExprStmt { expr, .. } => {
                self.eval_expr(expr)?;
            }
            Stmt::SpawnStmt { callee, args, span } => {
                let (name, arg_values) = self.eval_spawn_target(callee, args, span.start.line)?;
                self.telemetry.record_fire_and_forget_spawn();
                self.trace_task_log(format!("spawn fire-and-forget {name}"));
                self.concurrency.queue_fire_and_forget(name, arg_values);
            }
            Stmt::SelectStmt { arms, span } => {
                'select: for arm in arms {
                    let channel_val = self.eval_expr(&arm.channel)?;

                    // Emit output when line)? provides a msg.
                    if let Some(msg) = self.concurrency.try_recv(&channel_val, span.start.line)? {
                        self.env.define("_msg", msg);
                        self.execute_block(&arm.body)?;
                        break 'select;
                    }
                }
            }
            Stmt::ParallelStmt { body, span } => {
                self.telemetry.record_parallel_block();
                self.trace_task_log(format!("parallel block {} branch(es)", body.len()));
                let saved = self.env.clone_bindings();
                let mut pending_handles: Vec<(Option<String>, u64)> = Vec::new();
                let mut results = HashMap::new();

                self.log(format!(
                    "parallel: executing {} branch(es) cooperatively",
                    body.len()
                ));

                for stmt in body {
                    self.env = saved.clone_bindings();
                    match stmt {
                        Stmt::VarDecl {
                            name,
                            init: Some(init),
                            ..
                        } => {
                            let val = self.eval_expr(init)?;
                            if let RuntimeValue::TaskHandle { id } = val {
                                pending_handles.push((Some(name.clone()), id));
                            } else {
                                results.insert(name.clone(), val);
                            }
                        }
                        Stmt::ExprStmt { expr, .. } => {
                            let val = self.eval_expr(expr)?;
                            if let RuntimeValue::TaskHandle { id } = val {
                                pending_handles.push((None, id));
                            }
                        }
                        Stmt::SpawnStmt { callee, args, .. } => {
                            let (func_name, arg_values) =
                                self.eval_spawn_target(callee, args, span.start.line)?;
                            let handle = self.concurrency.create_task_handle(func_name, arg_values);
                            if let RuntimeValue::TaskHandle { id } = handle {
                                pending_handles.push((None, id));
                            }
                        }
                        _ => self.execute_stmt(stmt)?,
                    }
                }

                self.env = saved;

                for (name, id) in pending_handles {
                    let result = self.resolve_task_handle(id, span.start.line)?;
                    if let Some(binding) = name {
                        results.insert(binding, result);
                    }
                }

                if !results.is_empty() {
                    let count = results.len();
                    self.env.define(
                        "_parallel",
                        RuntimeValue::object("ParallelResults", results),
                    );
                    self.log(format!("parallel: aggregated {count} result(s)"));
                }
            }
            Stmt::ReturnStmt { .. } => {}
            Stmt::EnterModeStmt { mode, .. } => {
                self.enter_mode(mode)?;
            }
            Stmt::UseFallbackStmt { resource, .. } => {
                self.log(format!("fault: using fallback resource '{resource}'"));
            }
            Stmt::StopAllActuatorsStmt { .. } => {
                if let Some(monitor) = &mut self.safety_monitor {
                    monitor.set_emergency_stop(true);
                }
                self.backend.set_emergency_stop(true);
                self.backend.execute_motion(MotionCommand::Stop {
                    actuator: "all".into(),
                });
                self.log("safety: stop_all_actuators invoked".into());
            }
            Stmt::RunPipelineStmt { name, .. } => {
                self.execute_pipeline(name)?;
            }
            Stmt::NavigateStmt {
                goal,
                linear,
                angular,
                span,
            } => {
                self.execute_navigate_stmt(goal, linear.as_deref(), angular.as_deref(), span.start.line)?;
            }
        }
        Ok(())
    }

    pub(super) fn execute_block_with_return(
        &mut self,
        stmts: &[Stmt],
    ) -> Result<Option<RuntimeValue>, SpandaError> {
        // Execute block with return.
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
        // let result = instance.execute_block_with_return(stmts);

        // Execute each statement in sequence.
        for stmt in stmts {
            // Emit output when execute stmt with return provides a val.
            if let Some(val) = self.execute_stmt_with_return(stmt)? {
                return Ok(Some(val));
            }
        }
        Ok(None)
    }

    pub(super) fn execute_stmt_with_return(
        &mut self,
        stmt: &Stmt,
    ) -> Result<Option<RuntimeValue>, SpandaError> {
        // Execute stmt with return.
        //
        // Parameters:
        // - `self` — method receiver
        // - `stmt` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_stmt_with_return(stmt);

        // Match on stmt and handle each case.
        match stmt {
            Stmt::ReturnStmt { value, .. } => {
                let val = if let Some(expr) = value {
                    self.eval_expr(expr)?
                } else {
                    RuntimeValue::Void
                };
                Ok(Some(val))
            }
            Stmt::IfStmt {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let cond = self.eval_expr(condition)?;

                // Keep entries that match the expected pattern.
                if matches!(cond, RuntimeValue::Bool { value: true, .. }) {
                    // Emit output when execute block with return provides a v.
                    if let Some(v) = self.execute_block_with_return(then_branch)? {
                        return Ok(Some(v));
                    }
                } else if let Some(else_branch) = else_branch {
                    // Emit output when execute block with return provides a v.
                    if let Some(v) = self.execute_block_with_return(else_branch)? {
                        return Ok(Some(v));
                    }
                }
                Ok(None)
            }
            _ => {
                self.execute_stmt(stmt)?;
                Ok(None)
            }
        }
    }

}

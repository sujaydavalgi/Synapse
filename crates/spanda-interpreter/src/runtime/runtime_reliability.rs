//! Reliability modes, pipelines, retries, and watchdog helpers.
//!

use super::{
    runtime_velocity, Environment, Interpreter, IntoSpandaError, RobotBackend, RuntimeError,
    RuntimeValue, RUNTIME_TASK_COST_MS,
};
use crate::platform_events::emit_degraded_mode_entered;
use spanda_runtime::{RecoveryContext, RecoveryLevel, RecoveryStatus};
use spanda_ast::nodes::{Expr, RobotDecl};
use spanda_error::SpandaError;
use spanda_runtime::reliability_runtime::{
    recover_handlers_from_decls, ModeRuntime, PipelineRuntime, RetryRuntime, WatchdogRuntime,
};
use spanda_runtime::replay::MissionTrace;
use spanda_runtime::scheduler::SchedulerClock;

impl<B: RobotBackend> Interpreter<B> {
    /// Validate recovery through the assurance framework before executing handlers.
    fn recovery_allowed_for_issue(&self, issue: &str) -> bool {
        // Description:
        //     Recovery allowed for issue.
        //
        // Inputs:
        //     &self: value
        //         Caller-supplied &self.
        //     issue: &str
        //         Caller-supplied issue.
        //
        // Outputs:
        //     result: bool
        //         Return value from `recovery_allowed_for_issue`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_reliability::recovery_allowed_for_issue(&self, issue);

        let Some(program) = self.health_program.as_ref() else {
            return true;
        };
        let context = RecoveryContext {
            issue: issue.into(),
            diagnosis: None,
            classification: Some(self.assurance().classify_failure(issue)),
            level: RecoveryLevel::Level3AutomaticWithValidation,
        };
        let plan = self.assurance().plan_recovery(program, &context);
        let result = self.assurance().build_recovery_result_from_plan(program, &plan);
        !matches!(result.status, RecoveryStatus::Unsafe | RecoveryStatus::Failed)
    }

    pub(super) fn load_reliability_config(&mut self, robot: &RobotDecl) -> Result<(), SpandaError> {
        // Description:
        //     Load reliability config.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     robo: &RobotDecl
        //         Caller-supplied robo.
        //
        // Outputs:
        //     result: Result<(), SpandaError>
        //         Return value from `load_reliability_config`.
        //
        // Example:
        //     let result = spanda_interpreter::runtime_reliability::load_reliability_config(&mut self, robo);

        // Reset reliability runtime containers for this robot.
        self.watchdogs.clear();
        self.pipelines.clear();
        self.retries.clear();
        self.recovers.clear();
        self.modes.clear();
        self.task_heartbeats.clear();
        self.active_mode = "normal".into();
        self.init_recovery_runtime();

        // Start mission trace recording when enabled in interpreter options.
        if self.options.record_trace {
            let source = self
                .options
                .trace_source
                .clone()
                .unwrap_or_else(|| "program.sd".into());
            self.mission_trace = Some(MissionTrace::new(source));
        } else {
            self.mission_trace = None;
        }

        // Copy parsed reliability declarations into runtime form.
        let RobotDecl::RobotDecl {
            pipelines,
            watchdogs,
            retries,
            recovers,
            modes,
            ..
        } = robot;
        for pipeline in pipelines {
            let runtime = PipelineRuntime::from_decl(pipeline);
            self.pipelines.insert(runtime.name.clone(), runtime);
        }
        for watchdog in watchdogs {
            self.watchdogs.push(WatchdogRuntime::from_decl(watchdog));
        }
        for retry in retries {
            self.retries.push(RetryRuntime::from_decl(retry));
        }
        self.recovers = recover_handlers_from_decls(recovers);
        for mode in modes {
            let runtime = ModeRuntime::from_decl(mode);
            self.modes.insert(runtime.name.clone(), runtime);
        }

        // Enter the default normal mode when declared.
        if self.modes.contains_key("normal") {
            self.enter_mode("normal")?;
        }
        Ok(())
    }

    pub(super) fn enter_mode(&mut self, mode: &str) -> Result<(), SpandaError> {
        // Description:
        //     Enter mode.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     ode: &str
        //         Caller-supplied ode.
        //
        // Outputs:
        //     result: Result<(), SpandaError>
        //         Return value from `enter_mode`.
        //
        // Example:
        //     let result = spanda_interpreter::runtime_reliability::enter_mode(&mut self, ode);

        // Update active mode and execute the declared body when present.
        let previous_mode = self.active_mode.clone();
        self.active_mode = mode.to_string();
        if is_degraded_operating_mode(mode) && !is_degraded_operating_mode(&previous_mode) {
            emit_degraded_mode_entered(mode, "runtime_mode_transition", "runtime/mission");
        }
        if let Some(body) = self.modes.get(mode).map(|m| m.body.clone()) {
            self.execute_block(&body)?;
        } else {
            self.log(format!("mode: entered '{mode}' (no body declared)"));
            return Ok(());
        }
        self.record_mission_event("mode_enter", serde_json::json!({ "mode": mode }));
        self.log(format!("mode: entered '{mode}'"));
        Ok(())
    }

    pub(super) fn check_topic_qos_deadlines(&mut self) {
        // Description:
        //     Check topic qos deadlines.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //
        // Outputs:
        //     None.
        //
        // Example:
        //     let result = spanda_interpreter::runtime_reliability::check_topic_qos_deadlines(&mut self);

        // Compare elapsed sim time since the last publish for each topic.
        let snapshots: Vec<(String, f64, f64)> = self
            .topic_qos
            .iter()
            .filter_map(|(path, qos)| {
                let deadline_ms = qos.deadline_ms?;
                let last = self.topic_last_publish_ms.get(path).copied().unwrap_or(0.0);
                if last <= 0.0 {
                    return None;
                }
                let elapsed = self.sim_time_ms - last;
                if elapsed <= deadline_ms {
                    return None;
                }
                Some((path.clone(), elapsed, deadline_ms))
            })
            .collect();
        for (path, elapsed, deadline_ms) in snapshots {
            let misses = self.topic_deadline_misses.entry(path.clone()).or_insert(0);
            if *misses == 0 || elapsed > deadline_ms * (*misses as f64 + 1.0) {
                *misses += 1;
                self.telemetry
                    .record_topic_deadline_miss(&path, elapsed, deadline_ms);
                self.record_mission_event(
                    "topic_deadline_miss",
                    serde_json::json!({
                        "topic": path,
                        "elapsed_ms": elapsed,
                        "deadline_ms": deadline_ms,
                    }),
                );
                self.log(format!(
                    "topic '{path}': deadline miss ({elapsed:.1}ms > {deadline_ms:.1}ms)"
                ));
            }
        }
    }

    fn capture_replay_state(&self) -> spanda_runtime::replay::ReplayStateSnapshot {
        // Description:
        //     Capture replay state.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: spanda_runtime::replay::ReplayStateSnapshot
        //         Return value from `capture_replay_state`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_reliability::capture_replay_state(&self);

        let state = self.backend.get_state();
        spanda_runtime::replay::ReplayStateSnapshot {
            pose: state.pose,
            velocity: state.velocity,
            emergency_stop: state.emergency_stop,
            active_mode: Some(self.active_mode.clone()),
        }
    }

    pub(super) fn record_mission_event(
        &mut self,
        event: impl Into<String>,
        payload: serde_json::Value,
    ) {
        // Description:
        //     Record mission event.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     even: impl Into<String>
        //         Caller-supplied even.
        //     payload: serde_json::Value
        //         Caller-supplied payload.
        //
        // Outputs:
        //     None.
        //
        // Example:
        //     let result = spanda_interpreter::runtime_reliability::record_mission_event(&mut self, even, payload);

        // Skip when trace recording is disabled.
        if self.mission_trace.is_some() {
            let state = self.capture_replay_state();
            let sim_time = self.sim_time_ms;
            if let Some(trace) = self.mission_trace.as_mut() {
                trace.record_with_state(sim_time, event, payload, Some(state));
            }
        }
    }

    pub(super) fn uses_wall_scheduler(&self) -> bool {
        // Description:

        //     Uses wall scheduler.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //

        // Outputs:

        //     result: bool

        //         Return value from `uses_wall_scheduler`.

        //

        // Example:

        //     let result = spanda_interpreter::runtime_reliability::uses_wall_scheduler(&self);
        self.options.scheduler_clock == SchedulerClock::Wall && !self.options.replay_deterministic
    }

    pub(super) fn touch_task_heartbeat(&mut self, task_name: &str) {
        // Description:
        //     Touch task heartbeat.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     ask_name: &str
        //         Caller-supplied ask name.
        //
        // Outputs:
        //     None.
        //
        // Example:
        //     let result = spanda_interpreter::runtime_reliability::touch_task_heartbeat(&mut self, ask_name);

        // Store the current simulation time as the task heartbeat.
        self.task_heartbeats
            .insert(task_name.to_string(), self.sim_time_ms);
        self.telemetry_sink().record_task_heartbeat(
            task_name,
            self.sim_time_ms,
            self.telemetry_robot_id(),
            5000.0,
        );
    }

    pub(super) fn check_watchdogs(&mut self) -> Result<(), SpandaError> {
        // Description:
        //     Check watchdogs.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //
        // Outputs:
        //     result: Result<(), SpandaError>
        //         Return value from `check_watchdogs`.
        //
        // Example:
        //     let result = spanda_interpreter::runtime_reliability::check_watchdogs(&mut self);

        // Evaluate each declared watchdog handler.
        for index in 0..self.watchdogs.len() {
            let reference_ms = if let Some(target) = &self.watchdogs[index].target {
                *self.task_heartbeats.get(target).unwrap_or(&0.0)
            } else {
                0.0
            };
            let elapsed = self.sim_time_ms - reference_ms;
            let timeout_ms = self.watchdogs[index].timeout_ms;
            let should_fire = elapsed >= timeout_ms
                && self.watchdogs[index]
                    .last_fired_at_ms
                    .map(|last| self.sim_time_ms - last >= timeout_ms)
                    .unwrap_or(true);
            if !should_fire {
                continue;
            }
            let name = self.watchdogs[index].name.clone();
            let body = self.watchdogs[index].body.clone();
            self.watchdogs[index].last_fired_at_ms = Some(self.sim_time_ms);
            self.telemetry
                .record_watchdog_timeout(&name, self.sim_time_ms);
            self.record_mission_event(
                "watchdog_timeout",
                serde_json::json!({ "watchdog": name, "elapsed_ms": elapsed }),
            );
            self.log(format!(
                "watchdog '{name}': timeout after {elapsed:.1}ms (limit {timeout_ms:.1}ms)"
            ));
            self.execute_block(&body)?;
        }
        Ok(())
    }

    pub(super) fn execute_navigate_stmt(
        &mut self,
        goal: &Expr,
        linear: Option<&Expr>,
        angular: Option<&Expr>,
        line: u32,
    ) -> Result<(), SpandaError> {
        // Description:

        //     Execute navigate stmt.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     goal: &Expr

        //         Caller-supplied goal.

        //     linear: Option<&Expr>

        //         Caller-supplied linear.

        //     angular: Option<&Expr>

        //         Caller-supplied angular.

        //     line: u32

        //         Caller-supplied line.

        //

        // Outputs:

        //     result: Result<(), SpandaError>

        //         Return value from `execute_navigate_stmt`.

        //

        // Example:

        //     let result = spanda_interpreter::runtime_reliability::execute_navigate_stmt(&mut self, goal, linear, angular, line);
        let goal_text = match self.eval_expr(goal)? {
            RuntimeValue::String { value } => value,
            RuntimeValue::Number { value, .. } => value.to_string(),
            _ => {
                return Err(RuntimeError::new(
                    "navigate.goal requires a text or numeric expression",
                    line,
                )
                .into_spanda());
            }
        };

        // Require a mission-scoped navigation controller in the active robot env.
        match self.env.get("navigation") {
            Some(RuntimeValue::NavigationControl { goal: _ }) => {
                self.env.set(
                    "navigation",
                    RuntimeValue::NavigationControl {
                        goal: Some(goal_text.clone()),
                    },
                );
            }
            _ => {
                return Err(RuntimeError::new(
                    "navigate statement requires a robot with a declared mission",
                    line,
                )
                .into_spanda());
            }
        }

        let linear_mps = if let Some(expr) = linear {
            match self.eval_expr(expr)? {
                RuntimeValue::Number { value, .. } => value,
                _ => 0.2,
            }
        } else {
            0.2
        };
        let angular_rad = if let Some(expr) = angular {
            match self.eval_expr(expr)? {
                RuntimeValue::Number { value, .. } => value,
                _ => 0.0,
            }
        } else {
            0.0
        };

        self.log(format!("navigation: executing goal '{goal_text}'"));
        if self.nav2_enabled || self.topic_path_to_message_type.contains_key("/cmd_vel") {
            if let Some(message_type) = self.topic_path_to_message_type.get("/cmd_vel").cloned() {
                let velocity = runtime_velocity(linear_mps, angular_rad);
                self.backend
                    .publish_topic("/cmd_vel", &message_type, velocity);
                let prefix = if self.nav2_enabled {
                    "Nav2Adapter"
                } else {
                    "Nav2 bridge"
                };
                self.log(format!(
                    "navigation: {prefix} publish /cmd_vel goal='{goal_text}'"
                ));
            }
        }
        Ok(())
    }

    pub(super) fn execute_pipeline(&mut self, name: &str) -> Result<(), SpandaError> {
        // Description:
        //     Execute pipeline.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     name: &str
        //         Caller-supplied name.
        //
        // Outputs:
        //     result: Result<(), SpandaError>
        //         Return value from `execute_pipeline`.
        //
        // Example:
        //     let result = spanda_interpreter::runtime_reliability::execute_pipeline(&mut self, name);

        // Resolve the pipeline body and budget from runtime state.
        let Some(pipeline) = self.pipelines.get(name).cloned() else {
            return Err(RuntimeError::new(format!("unknown pipeline '{name}'"), 0).into_spanda());
        };
        let started = std::time::Instant::now();
        self.execute_block(&pipeline.body)?;
        let duration_ms = started.elapsed().as_secs_f64() * 1000.0;
        let duration_ms = duration_ms.max(RUNTIME_TASK_COST_MS);
        let slow = duration_ms > pipeline.budget_ms;
        self.telemetry
            .record_pipeline_execution(name, pipeline.budget_ms, duration_ms, slow);
        if slow {
            self.log(format!(
                "pipeline '{name}': budget {:.1}ms exceeded ({duration_ms:.2}ms)",
                pipeline.budget_ms
            ));
        } else {
            self.log(format!(
                "pipeline '{name}': completed in {duration_ms:.2}ms (budget {:.1}ms)",
                pipeline.budget_ms
            ));
        }
        self.record_mission_event(
            "pipeline_run",
            serde_json::json!({
                "pipeline": name,
                "duration_ms": duration_ms,
                "budget_ms": pipeline.budget_ms,
            }),
        );
        Ok(())
    }

    pub(super) fn run_retry_policies(&mut self) -> Result<(), SpandaError> {
        // Description:
        //     Run retry policies.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //
        // Outputs:
        //     result: Result<(), SpandaError>
        //         Return value from `run_retry_policies`.
        //
        // Example:
        //     let result = spanda_interpreter::runtime_reliability::run_retry_policies(&mut self);

        // Skip when no retry policies or faults are present.
        if self.retries.is_empty() || !self.hardware_monitor.has_injected_faults() {
            return Ok(());
        }

        // Execute each retry policy until success or fallback.
        for index in 0..self.retries.len() {
            if self.retries[index].exhausted {
                continue;
            }
            while self.retries[index].attempt < self.retries[index].attempts {
                self.retries[index].attempt += 1;
                let attempt = self.retries[index].attempt;
                let attempts = self.retries[index].attempts;
                let backoff_ms = self.retries[index].backoff_ms;
                self.log(format!(
                    "retry: attempt {attempt}/{attempts} (backoff {backoff_ms:.0}ms)"
                ));
                self.record_mission_event(
                    "retry_attempt",
                    serde_json::json!({
                        "attempt": attempt,
                        "max_attempts": attempts,
                    }),
                );
                let body = self.retries[index].body.clone();
                self.execute_block(&body)?;
                if !self.hardware_monitor.has_injected_faults() {
                    self.retries[index].attempt = 0;
                    break;
                }
            }
            if self.retries[index].attempt >= self.retries[index].attempts
                && !self.retries[index].exhausted
            {
                self.retries[index].exhausted = true;
                self.log("retry: exhausted attempts — running fallback".into());
                self.record_mission_event("retry_fallback", serde_json::json!({}));
                let fallback = self.retries[index].fallback.clone();
                self.execute_block(&fallback)?;
            }
        }
        Ok(())
    }

    pub(super) fn try_invoke_recovery(&mut self, err: &SpandaError) -> Result<bool, SpandaError> {
        // Description:
        //     Try invoke recovery.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     err: &SpandaError
        //         Caller-supplied err.
        //
        // Outputs:
        //     result: Result<bool, SpandaError>
        //         Return value from `try_invoke_recovery`.
        //
        // Example:
        //     let result = spanda_interpreter::runtime_reliability::try_invoke_recovery(&mut self, err);

        // Only runtime errors participate in recovery dispatch.
        let SpandaError::Runtime { message, .. } = err else {
            return Ok(false);
        };

        // Match recovery handlers by declared error name or substring.
        for (error_name, body) in self.recovers.clone() {
            if message.contains(&error_name)
                || (error_name == "RuntimeError" && matches!(err, SpandaError::Runtime { .. }))
            {
                if !self.recovery_allowed_for_issue(&error_name) {
                    self.log(format!(
                        "recover: blocked handler for '{error_name}' — recovery validation failed"
                    ));
                    self.record_mission_event(
                        "recover_blocked",
                        serde_json::json!({ "error": error_name, "message": message }),
                    );
                    return Ok(false);
                }
                let _ = self.execute_recovery_runtime(&error_name);
                self.log(format!("recover: invoking handler for '{error_name}'"));
                self.record_mission_event(
                    "recover",
                    serde_json::json!({ "error": error_name, "message": message }),
                );
                self.execute_block(&body)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(super) fn invoke_recovery_for_event(&mut self, event: &str) -> Result<(), SpandaError> {
        // Description:
        //     Invoke recovery for event.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     even: &str
        //         Caller-supplied even.
        //
        // Outputs:
        //     result: Result<(), SpandaError>
        //         Return value from `invoke_recovery_for_event`.
        //
        // Example:
        //     let result = spanda_interpreter::runtime_reliability::invoke_recovery_for_event(&mut self, even);

        // Prefer an exact event match, then generic sensor failure handlers.
        let handler_key = if self.recovers.contains_key(event) {
            Some(event.to_string())
        } else if event.ends_with("Failure") && self.recovers.contains_key("SensorFailure") {
            Some("SensorFailure".into())
        } else {
            None
        };
        if let Some(key) = handler_key {
            if !self.recovery_allowed_for_issue(event) {
                self.log(format!(
                    "recover: blocked hardware event '{event}' — recovery validation failed"
                ));
                self.record_mission_event(
                    "recover_blocked",
                    serde_json::json!({ "event": event, "handler": key }),
                );
                return Ok(());
            }
            let _ = self.execute_recovery_runtime(event);
            if let Some(body) = self.recovers.get(&key).cloned() {
                self.log(format!("recover: hardware event '{event}' -> '{key}'"));
                self.record_mission_event(
                    "recover_hardware",
                    serde_json::json!({ "event": event, "handler": key }),
                );
                self.execute_block(&body)?;
            }
        }
        Ok(())
    }

    pub(super) fn evaluate_stop_if(&mut self, env: &Environment) -> bool {
        // Description:
        //     Evaluate stop if.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     env: &Environment
        //         Caller-supplied env.
        //
        // Outputs:
        //     result: bool
        //         Return value from `evaluate_stop_if`.
        //
        // Example:
        //     let result = spanda_interpreter::runtime_reliability::evaluate_stop_if(&mut self, env);

        // Iterate over clone.
        for condition in &self.stop_if_conditions.clone() {
            let saved = self.env.clone_bindings();
            self.env = env.clone_bindings();
            let result = self.eval_expr(condition);
            self.env = saved;

            // Take this path when let Ok(RuntimeValue::Bool { value: true, .. }) = result.
            if let Ok(RuntimeValue::Bool { value: true, .. }) = result {
                return true;
            }
        }
        false
    }
}

fn is_degraded_operating_mode(mode: &str) -> bool {
    matches!(
        mode.to_ascii_lowercase().as_str(),
        "degraded" | "degradedmode" | "safe" | "safemode" | "emergency" | "emergencymode"
            | "recovery" | "recoverymode"
    )
}

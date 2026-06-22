//! Reliability modes, pipelines, retries, and watchdog helpers.
//!

use super::{
    Environment, IntoSpandaError, Interpreter, RobotBackend, RuntimeError, RuntimeValue,
    RUNTIME_TASK_COST_MS, runtime_velocity,
};
use spanda_ast::nodes::{Expr, RobotDecl};
use crate::error::SpandaError;
use crate::reliability_runtime::{
    recover_handlers_from_decls, ModeRuntime, PipelineRuntime, RetryRuntime, WatchdogRuntime,
};
use crate::replay::MissionTrace;
use spanda_runtime::scheduler::SchedulerClock;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn load_reliability_config(&mut self, robot: &RobotDecl) -> Result<(), SpandaError> {
        // Load watchdog, pipeline, retry, and recovery runtime state from a robot block.
        //
        // Parameters:
        // - `self` — method receiver
        // - `robot` — parsed robot declaration
        //
        // Returns:
        // Ok when configuration is loaded.
        //
        // Options:
        // None.
        //
        // Example:
        // interp.load_reliability_config(robot)?;

        // Reset reliability runtime containers for this robot.
        self.watchdogs.clear();
        self.pipelines.clear();
        self.retries.clear();
        self.recovers.clear();
        self.modes.clear();
        self.task_heartbeats.clear();
        self.active_mode = "normal".into();

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
        // Switch the active operating mode and run its configuration body.
        //
        // Parameters:
        // - `self` — method receiver
        // - `mode` — mode name without `_mode` suffix
        //
        // Returns:
        // Ok when the mode body completes.
        //
        // Options:
        // None.
        //
        // Example:
        // interp.enter_mode("degraded")?;

        // Update active mode and execute the declared body when present.
        self.active_mode = mode.to_string();
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
        // Detect topic publish deadline misses against declared QoS.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // interp.check_topic_qos_deadlines();

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

    fn capture_replay_state(&self) -> crate::replay::ReplayStateSnapshot {
        // Capture the current robot snapshot for mission trace playback.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Pose, velocity, safety, and mode snapshot.
        //
        // Options:
        // None.
        //
        // Example:
        // let snapshot = interp.capture_replay_state();

        let state = self.backend.get_state();
        crate::replay::ReplayStateSnapshot {
            pose: state.pose,
            velocity: state.velocity,
            emergency_stop: state.emergency_stop,
            active_mode: Some(self.active_mode.clone()),
        }
    }

    pub(super) fn record_mission_event(&mut self, event: impl Into<String>, payload: serde_json::Value) {
        // Append one frame to the mission trace when recording is enabled.
        //
        // Parameters:
        // - `self` — method receiver
        // - `event` — event label
        // - `payload` — structured payload
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // interp.record_mission_event("task_tick", json!({"task":"sense"}));

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
        // Report whether the scheduler should sleep on wall-clock deadlines.
        self.options.scheduler_clock == SchedulerClock::Wall && !self.options.replay_deterministic
    }

    pub(super) fn touch_task_heartbeat(&mut self, task_name: &str) {
        // Record the latest heartbeat time for watchdog evaluation.
        //
        // Parameters:
        // - `self` — method receiver
        // - `task_name` — watched task name
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // interp.touch_task_heartbeat("SafetyMonitor");

        // Store the current simulation time as the task heartbeat.
        self.task_heartbeats
            .insert(task_name.to_string(), self.sim_time_ms);
    }

    pub(super) fn check_watchdogs(&mut self) -> Result<(), SpandaError> {
        // Evaluate watchdog timeouts against task heartbeats at the current sim time.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Ok when watchdog bodies finish, or an execution error.
        //
        // Options:
        // None.
        //
        // Example:
        // interp.check_watchdogs()?;

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
        // Execute navigate { goal: ... } sugar over navigation.goal/navigate.
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
        // Execute a named pipeline and record latency-budget telemetry.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — pipeline name
        //
        // Returns:
        // Ok when the pipeline body completes.
        //
        // Options:
        // None.
        //
        // Example:
        // interp.execute_pipeline("obstacle_avoidance")?;

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
        // Run robot-level retry policies when injected hardware faults are active.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Ok when retry and fallback blocks finish.
        //
        // Options:
        // None.
        //
        // Example:
        // interp.run_retry_policies()?;

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
        // Attempt a declared recovery handler for a runtime error.
        //
        // Parameters:
        // - `self` — method receiver
        // - `err` — runtime error to match
        //
        // Returns:
        // true when a recovery handler ran successfully.
        //
        // Options:
        // None.
        //
        // Example:
        // if interp.try_invoke_recovery(&err)? { ... }

        // Only runtime errors participate in recovery dispatch.
        let SpandaError::Runtime { message, .. } = err else {
            return Ok(false);
        };

        // Match recovery handlers by declared error name or substring.
        for (error_name, body) in self.recovers.clone() {
            if message.contains(&error_name)
                || (error_name == "RuntimeError" && matches!(err, SpandaError::Runtime { .. }))
            {
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
        // Run a recovery handler keyed by hardware event name.
        //
        // Parameters:
        // - `self` — method receiver
        // - `event` — hardware event label
        //
        // Returns:
        // Ok when a handler completes or none matched.
        //
        // Options:
        // None.
        //
        // Example:
        // interp.invoke_recovery_for_event("LidarFailure")?;

        // Prefer an exact event match, then generic sensor failure handlers.
        let handler_key = if self.recovers.contains_key(event) {
            Some(event.to_string())
        } else if event.ends_with("Failure") && self.recovers.contains_key("SensorFailure") {
            Some("SensorFailure".into())
        } else {
            None
        };
        if let Some(key) = handler_key {
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
        // Evaluate stop if.
        //
        // Parameters:
        // - `self` — method receiver
        // - `env` — input value
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.evaluate_stop_if(env);

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

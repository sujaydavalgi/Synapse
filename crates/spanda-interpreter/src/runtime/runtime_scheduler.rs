//! Task scheduling, contracts, and multiplexed execution loops.
//!

use super::{
    priority_label, task_budget_violation_kind, IntoSpandaError,
    Interpreter, RobotBackend, RuntimeError, RuntimeValue, TaskSchedule, RUNTIME_TASK_COST_MS,
};
use spanda_ast::nodes::{Expr, Stmt};
use crate::error::SpandaError;
use spanda_ast::foundations::TaskPriority;
use spanda_runtime::scheduler;
use spanda_runtime::triggers::SystemTriggerCategory;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn run_scheduled_task(&mut self, schedule: &TaskSchedule) -> Result<bool, SpandaError> {
        // Run scheduled task.
        //
        // Parameters:
        // - `self` — method receiver
        // - `schedule` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.run_scheduled_task(schedule);

        // use budget when budget is present.

        // Emit output when budget provides a budget.
        if let Some(budget) = &schedule.budget {
            // Emit output when name) provides a metrics.
            if let Some(metrics) = self.telemetry.tasks.get(&schedule.name) {
                // Take this path when metrics.max duration ms > 0.0.
                if metrics.max_duration_ms > 0.0 {
                    // Emit output when task budget violation kind provides a kind.
                    if let Some(kind) = task_budget_violation_kind(
                        budget,
                        metrics.max_duration_ms,
                        schedule.interval_ms,
                    ) {
                        self.telemetry.record_budget_violation(
                            &schedule.name,
                            schedule.priority,
                            schedule.interval_ms,
                        );
                        self.telemetry.record_task_skip(
                            &schedule.name,
                            schedule.priority,
                            schedule.interval_ms,
                        );
                        self.log(format!(
                            "task '{}': {kind} budget exceeded — skipping tick",
                            schedule.name
                        ));
                        self.trace_task_log(format!("{} skipped ({kind} budget)", schedule.name));
                        return Ok(true);
                    }
                }
            }
        }
        let started = std::time::Instant::now();
        let continue_running = match self.execute_task_iteration(
            &schedule.body,
            &schedule.requires,
            &schedule.ensures,
            &schedule.invariant,
            Some(&schedule.name),
        ) {
            Ok(value) => value,
            Err(err) => {
                if self.try_invoke_recovery(&err)? {
                    true
                } else {
                    return Err(err);
                }
            }
        };
        self.touch_task_heartbeat(&schedule.name);
        let measured_ms = started.elapsed().as_secs_f64() * 1000.0;
        let duration_ms = measured_ms.max(RUNTIME_TASK_COST_MS);
        self.telemetry.record_task_duration(
            &schedule.name,
            schedule.priority,
            schedule.interval_ms,
            duration_ms,
        );

        // Emit output when budget provides a budget.
        if let Some(budget) = &schedule.budget {
            // Take this path when let Some(kind) =.
            if let Some(kind) =
                task_budget_violation_kind(budget, duration_ms, schedule.interval_ms)
            {
                self.telemetry.record_budget_violation(
                    &schedule.name,
                    schedule.priority,
                    schedule.interval_ms,
                );
                self.log(format!(
                    "task '{}': {kind} budget exceeded ({duration_ms:.2}ms)",
                    schedule.name
                ));
                self.trace_task_log(format!(
                    "{} budget violation {kind} duration={duration_ms:.2}ms",
                    schedule.name
                ));
            }
        }
        Ok(continue_running)
    }

    pub(super) fn eval_contract(&mut self, expr: &Expr) -> Result<bool, SpandaError> {
        // Eval contract.
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
        // let result = instance.eval_contract(expr);

        // Match on eval expr and handle each case.
        match self.eval_expr(expr)? {
            RuntimeValue::Bool { value, .. } => Ok(value),
            _ => Err(RuntimeError::new("Contract expression must be boolean", 0).into_spanda()),
        }
    }

    pub(super) fn execute_with_contracts(
        &mut self,
        body: &[Stmt],
        requires: &Option<Expr>,
        ensures: &Option<Expr>,
        invariant: &Option<Expr>,
    ) -> Result<(), SpandaError> {
        // Execute with contracts.
        //
        // Parameters:
        // - `self` — method receiver
        // - `body` — input value
        // - `requires` — input value
        // - `ensures` — input value
        // - `invariant` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_with_contracts(body, requires, ensures, invariant);

        // use req when requires is present.

        // Emit output when requires provides a req.
        if let Some(req) = requires {
            // Take the branch when eval contract is false.
            if !self.eval_contract(req)? {
                return Err(RuntimeError::new("requires contract failed", 0).into_spanda());
            }
        }
        self.execute_block(body)?;

        // Emit output when ensures provides a ens.
        if let Some(ens) = ensures {
            // Take the branch when eval contract is false.
            if !self.eval_contract(ens)? {
                return Err(RuntimeError::new("ensures contract failed", 0).into_spanda());
            }
        }

        // Emit output when invariant provides a inv.
        if let Some(inv) = invariant {
            // Take the branch when eval contract is false.
            if !self.eval_contract(inv)? {
                return Err(RuntimeError::new("invariant contract failed", 0).into_spanda());
            }
        }
        self.run_verify_rules()?;
        self.run_verify_warnings()?;
        Ok(())
    }

    fn run_verify_warnings(&mut self) -> Result<(), SpandaError> {
        // Run verify warnings.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.run_verify_warnings();

        // Compute warnings for the following logic.
        let warnings = self.verify_warning_rules.clone();

        // Skip further work when warnings is empty.
        if warnings.is_empty() {
            return Ok(());
        }

        // Iterate over enumerate with destructured elements.
        for (index, rule) in warnings.iter().enumerate() {
            // Match on eval expr and handle each case.
            match self.eval_expr(rule)? {
                RuntimeValue::Bool { value: false, .. } => {
                    let _ = self
                        .dispatch_system_trigger(SystemTriggerCategory::Verification, "Warning");
                    self.log(format!("verify warning {} triggered", index + 1));
                }
                RuntimeValue::Bool { value: true, .. } => {}
                _ => {
                    return Err(RuntimeError::new(
                        format!("verify warning {} must be boolean", index + 1),
                        0,
                    )
                    .into_spanda());
                }
            }
        }
        Ok(())
    }

    fn run_verify_rules(&mut self) -> Result<(), SpandaError> {
        // Run verify rules.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.run_verify_rules();

        // Compute rules for the following logic.
        let rules = self.verify_rules.clone();

        // Skip further work when rules is empty.
        if rules.is_empty() {
            return Ok(());
        }

        // Iterate over enumerate with destructured elements.
        for (index, rule) in rules.iter().enumerate() {
            // Match on eval expr and handle each case.
            match self.eval_expr(rule)? {
                RuntimeValue::Bool { value: true, .. } => {}
                RuntimeValue::Bool { value: false, .. } => {
                    let _ =
                        self.dispatch_system_trigger(SystemTriggerCategory::Verification, "Failed");
                    return Err(
                        RuntimeError::new(format!("verify rule {} failed", index + 1), 0)
                            .into_spanda(),
                    );
                }
                _ => {
                    return Err(RuntimeError::new(
                        format!("verify rule {} must be boolean", index + 1),
                        0,
                    )
                    .into_spanda());
                }
            }
        }
        self.log(format!("verify: all {} rule(s) passed", rules.len()));
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn execute_task_loop_with_contracts(
        &mut self,
        task_name: &str,
        priority: TaskPriority,
        body: &[Stmt],
        interval_ms: f64,
        requires: &Option<Expr>,
        ensures: &Option<Expr>,
        invariant: &Option<Expr>,
        budget: Option<spanda_ast::foundations::ResourceBudgetDecl>,
    ) -> Result<(), SpandaError> {
        // Execute task loop with contracts.
        //
        // Parameters:
        // - `self` — method receiver
        // - `task_name` — input value
        // - `priority` — input value
        // - `body` — input value
        // - `interval_ms` — input value
        // - `requires` — input value
        // - `ensures` — input value
        // - `invariant` — input value
        // - `budget` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_task_loop_with_contracts(task_name, priority, body, interval_ms, requires, ensures, invariant, budget);

        // Call record scheduler start on the current instance.
        self.telemetry.record_scheduler_start(1, interval_ms);
        self.trace_scheduler_log(format!(
            "single-task {task_name} interval={interval_ms}ms priority={}",
            priority_label(priority)
        ));
        let schedule = TaskSchedule {
            name: task_name.to_string(),
            priority,
            interval_ms,
            deadline_ms: None,
            jitter_ms_max: None,
            isolated: false,
            next_due_ms: 0.0,
            last_start_ms: None,
            body: body.to_vec(),
            requires: requires.clone(),
            ensures: ensures.clone(),
            invariant: invariant.clone(),
            budget,
        };

        let wall_mode = self.uses_wall_scheduler();
        let wall_start = std::time::Instant::now();
        let mut next_deadline = wall_start;

        // Process each max loop iteration.
        for iteration in 0..self.options.max_loop_iterations {
            let sim_time = if wall_mode {
                let deadline = scheduler::advance_wall_tick(&mut next_deadline, interval_ms);
                scheduler::sleep_until(deadline);
                scheduler::elapsed_ms(wall_start, std::time::Instant::now())
            } else {
                (iteration as f64 + 1.0) * interval_ms
            };
            self.backend.tick(interval_ms);
            self.sim_time_ms = sim_time;
            self.triggers_dispatched_this_tick = 0;
            self.telemetry.record_scheduler_tick();
            self.telemetry
                .record_task_tick(task_name, priority, interval_ms);
            self.trace_task_log(format!(
                "{task_name} tick={} priority={} interval={interval_ms}ms",
                iteration + 1,
                priority_label(priority)
            ));
            self.run_timer_triggers(sim_time)?;
            self.run_condition_triggers()?;
            self.run_trigger_maintenance()?;

            // Take the branch when run scheduled task is false.
            let continue_running = self.run_scheduled_task(&schedule)?;
            self.check_watchdogs()?;
            self.check_topic_qos_deadlines();
            self.record_mission_event(
                "scheduler_tick",
                serde_json::json!({ "sim_time_ms": sim_time, "task": task_name }),
            );
            if !continue_running {
                self.telemetry.record_emergency_stop();
                break;
            }
            self.run_verify_rules()?;
            self.update_twin_snapshot();
        }
        Ok(())
    }

    fn execute_task_iteration(
        &mut self,
        body: &[Stmt],
        requires: &Option<Expr>,
        ensures: &Option<Expr>,
        invariant: &Option<Expr>,
        task_name: Option<&str>,
    ) -> Result<bool, SpandaError> {
        // Execute task iteration.
        //
        // Parameters:
        // - `self` — method receiver
        // - `body` — input value
        // - `requires` — input value
        // - `ensures` — input value
        // - `invariant` — input value
        // - `task_name` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_task_iteration(body, requires, ensures, invariant, task_name);

        // use req when requires is present.

        // Emit output when requires provides a req.
        if let Some(req) = requires {
            // Take the branch when eval contract is false.
            if !self.eval_contract(req)? {
                let label = task_name
                    .map(|name| format!("task '{name}'"))
                    .unwrap_or_else(|| "task".into());

                // Emit output when task name provides a name.
                if let Some(name) = task_name {
                    self.telemetry
                        .record_task_skip(name, TaskPriority::Normal, 0.0);
                    self.trace_task_log(format!("{name} skipped (requires failed)"));
                }
                self.log(format!(
                    "{label} requires contract failed — skipping iteration"
                ));
                return Ok(true);
            }
        }
        self.execute_block(body).or_else(|err| {
            if self.try_invoke_recovery(&err)? {
                Ok(())
            } else {
                Err(err)
            }
        })?;

        // Emit output when ensures provides a ens.
        if let Some(ens) = ensures {
            // Take the branch when eval contract is false.
            if !self.eval_contract(ens)? {
                return Err(RuntimeError::new("task ensures contract failed", 0).into_spanda());
            }
        }

        // Emit output when invariant provides a inv.
        if let Some(inv) = invariant {
            // Take the branch when eval contract is false.
            if !self.eval_contract(inv)? {
                return Err(RuntimeError::new("task invariant contract failed", 0).into_spanda());
            }
        }
        let stop = self
            .safety_monitor
            .as_ref()
            .map(|m| m.is_emergency_stop())
            .unwrap_or(false);
        Ok(!stop)
    }

    pub(super) fn execute_multiplexed_tasks(&mut self, tasks: Vec<TaskSchedule>) -> Result<(), SpandaError> {
        // Execute multiplexed tasks.
        //
        // Parameters:
        // - `self` — method receiver
        // - `tasks` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_multiplexed_tasks(tasks);

        // skip further work when tasks is empty.
        if tasks.is_empty() {
            return Ok(());
        }
        let base_tick = tasks
            .iter()
            .map(|task| task.interval_ms)
            .fold(f64::INFINITY, f64::min)
            .max(1.0);
        let mut schedules = tasks;
        let mut sim_time = 0.0;
        self.telemetry
            .record_scheduler_start(schedules.len() as u64, base_tick);
        self.log(format!(
            "scheduler: multiplexing {} task(s) with base tick {}ms",
            schedules.len(),
            base_tick
        ));
        self.trace_scheduler_log(format!(
            "start tasks={} base_tick={base_tick}ms",
            schedules.len()
        ));
        let wall_mode = self.uses_wall_scheduler();
        let wall_start = std::time::Instant::now();
        let mut next_deadline = wall_start;

        // Process each max loop iteration.
        for iteration in 0..self.options.max_loop_iterations {
            let dt_ms = if wall_mode {
                let deadline = scheduler::advance_wall_tick(&mut next_deadline, base_tick);
                scheduler::sleep_until(deadline);
                sim_time = scheduler::elapsed_ms(wall_start, std::time::Instant::now());
                base_tick
            } else {
                sim_time += base_tick;
                base_tick
            };
            self.backend.tick(dt_ms);
            self.sim_time_ms = sim_time;
            self.triggers_dispatched_this_tick = 0;
            self.telemetry.record_scheduler_tick();
            self.trace_scheduler_log(format!("tick={} sim_time={sim_time}ms", iteration + 1));
            self.run_timer_triggers(sim_time)?;
            self.run_condition_triggers()?;
            self.run_trigger_maintenance()?;
            schedules.sort_by_key(|schedule| schedule.priority_rank());

            // Process each schedule.
            for schedule in &mut schedules {
                // Take this path when schedule.next due ms <= sim time.
                if schedule.next_due_ms <= sim_time {
                    // Record release jitter against the declared bound before running the task.
                    if let Some(max_jitter) = schedule.jitter_ms_max {
                        let lateness = (sim_time - schedule.next_due_ms).max(0.0);
                        self.telemetry.record_task_jitter(
                            &schedule.name,
                            schedule.priority,
                            schedule.interval_ms,
                            lateness,
                            max_jitter,
                        );
                    }
                    // Take this path when sim time > schedule.next due ms + declared deadline slack.
                    let deadline = schedule.deadline_ms.unwrap_or(schedule.interval_ms);
                    if sim_time > schedule.next_due_ms + deadline {
                        self.telemetry.record_missed_deadline(
                            &schedule.name,
                            schedule.priority,
                            schedule.interval_ms,
                        );
                        self.trace_task_log(format!(
                            "{} missed deadline at sim_time={sim_time}ms",
                            schedule.name
                        ));
                    }
                    self.telemetry.record_task_tick(
                        &schedule.name,
                        schedule.priority,
                        schedule.interval_ms,
                    );
                    self.log(format!("task '{}': tick", schedule.name));
                    self.trace_task_log(format!(
                        "{} tick priority={} interval={}ms",
                        schedule.name,
                        priority_label(schedule.priority),
                        schedule.interval_ms
                    ));
                    schedule.last_start_ms = Some(sim_time);

                    // Take the branch when run scheduled task is false.
                    if !self.run_scheduled_task(schedule)? {
                        self.telemetry.record_emergency_stop();
                        return Ok(());
                    }
                    schedule.next_due_ms = sim_time + schedule.interval_ms;
                }
            }
            self.check_watchdogs()?;
            self.check_topic_qos_deadlines();
            self.record_mission_event(
                "scheduler_tick",
                serde_json::json!({ "sim_time_ms": sim_time, "iteration": iteration + 1 }),
            );
            self.run_verify_rules()?;
            self.update_twin_snapshot();

            // Take this path when self.
            if self
                .safety_monitor
                .as_ref()
                .map(|m| m.is_emergency_stop())
                .unwrap_or(false)
            {
                self.telemetry.record_emergency_stop();
                break;
            }
        }
        Ok(())
    }

    pub(super) fn execute_trigger_only_loop(&mut self) -> Result<(), SpandaError> {
        // Execute trigger only loop.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_trigger_only_loop();

        // Compute base tick for the following logic.
        let base_tick = self
            .trigger_timers
            .iter()
            .map(|t| t.interval_ms)
            .fold(f64::INFINITY, f64::min)
            .max(1.0);
        let mut sim_time = 0.0;
        self.log(format!(
            "scheduler: trigger-only loop with base tick {base_tick}ms"
        ));
        self.trace_scheduler_log(format!("trigger-only base_tick={base_tick}ms"));
        let wall_mode = self.uses_wall_scheduler();
        let wall_start = std::time::Instant::now();
        let mut next_deadline = wall_start;

        // Process each max loop iteration.
        for iteration in 0..self.options.max_loop_iterations {
            let dt_ms = if wall_mode {
                let deadline = scheduler::advance_wall_tick(&mut next_deadline, base_tick);
                scheduler::sleep_until(deadline);
                sim_time = scheduler::elapsed_ms(wall_start, std::time::Instant::now());
                base_tick
            } else {
                sim_time += base_tick;
                base_tick
            };
            self.backend.tick(dt_ms);
            self.sim_time_ms = sim_time;
            self.triggers_dispatched_this_tick = 0;
            self.telemetry.record_scheduler_tick();
            self.run_timer_triggers(sim_time)?;
            self.run_condition_triggers()?;
            self.run_trigger_maintenance()?;
            self.run_verify_rules()?;
            self.run_verify_warnings()?;
            self.update_twin_snapshot();

            // Take this path when self.
            if self
                .safety_monitor
                .as_ref()
                .map(|m| m.is_emergency_stop())
                .unwrap_or(false)
            {
                break;
            }
            let _ = iteration;
        }
        Ok(())
    }

}

//! Runtime telemetry for deterministic scheduler and task execution.

use crate::foundations::TaskPriority;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TaskMetrics {
    pub name: String,
    pub priority: String,
    pub interval_ms: f64,
    pub ticks: u64,
    pub skipped: u64,
    pub missed_deadlines: u64,
    pub budget_violations: u64,
    pub last_duration_ms: f64,
    pub max_duration_ms: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SchedulerMetrics {
    pub multiplexed_tasks: u64,
    pub scheduler_ticks: u64,
    pub base_tick_ms: f64,
    pub emergency_stops: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ExecutionMetrics {
    pub spawns: u64,
    pub joins: u64,
    pub parallel_blocks: u64,
    pub fire_and_forget_spawns: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TriggerMetrics {
    pub name: String,
    pub category: String,
    pub priority: String,
    pub executions: u64,
    pub failures: u64,
    pub missed_deadlines: u64,
    pub last_duration_ms: f64,
    pub max_duration_ms: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RuntimeTelemetry {
    pub tasks: HashMap<String, TaskMetrics>,
    pub triggers: HashMap<String, TriggerMetrics>,
    pub scheduler: SchedulerMetrics,
    pub execution: ExecutionMetrics,
    pub replay_frames: u64,
}

impl RuntimeTelemetry {
    pub fn task_mut(
        &mut self,
        name: &str,
        priority: TaskPriority,
        interval_ms: f64,
    ) -> &mut TaskMetrics {
        // Task mut.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `priority` — input value
        // - `interval_ms` — input value
        //
        // Returns:
        // &mut TaskMetrics.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.task_mut(name, priority, interval_ms);

        // Call tasks on the current instance.
        self.tasks
            .entry(name.to_string())
            .or_insert_with(|| TaskMetrics {
                name: name.to_string(),
                priority: priority_label(priority),
                interval_ms,
                ..Default::default()
            })
    }

    pub fn record_task_tick(&mut self, name: &str, priority: TaskPriority, interval_ms: f64) {
        // Record task tick.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `priority` — input value
        // - `interval_ms` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.record_task_tick(name, priority, interval_ms);

        // Call task mut on the current instance.
        self.task_mut(name, priority, interval_ms).ticks += 1;
    }

    pub fn record_task_duration(
        &mut self,
        name: &str,
        priority: TaskPriority,
        interval_ms: f64,
        duration_ms: f64,
    ) {
        // Record task duration.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `priority` — input value
        // - `interval_ms` — input value
        // - `duration_ms` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.record_task_duration(name, priority, interval_ms, duration_ms);

        // Compute entry for the following logic.
        let entry = self.task_mut(name, priority, interval_ms);
        entry.last_duration_ms = duration_ms;

        // Take this path when duration ms > entry.max duration ms.
        if duration_ms > entry.max_duration_ms {
            entry.max_duration_ms = duration_ms;
        }
    }

    pub fn record_budget_violation(
        &mut self,
        name: &str,
        priority: TaskPriority,
        interval_ms: f64,
    ) {
        // Record budget violation.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `priority` — input value
        // - `interval_ms` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.record_budget_violation(name, priority, interval_ms);

        // Call task mut on the current instance.
        self.task_mut(name, priority, interval_ms).budget_violations += 1;
    }

    pub fn record_task_skip(&mut self, name: &str, priority: TaskPriority, interval_ms: f64) {
        // Record task skip.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `priority` — input value
        // - `interval_ms` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.record_task_skip(name, priority, interval_ms);

        // Call task mut on the current instance.
        self.task_mut(name, priority, interval_ms).skipped += 1;
    }

    pub fn record_missed_deadline(&mut self, name: &str, priority: TaskPriority, interval_ms: f64) {
        // Record missed deadline.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `priority` — input value
        // - `interval_ms` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.record_missed_deadline(name, priority, interval_ms);

        // Call task mut on the current instance.
        self.task_mut(name, priority, interval_ms).missed_deadlines += 1;
    }

    pub fn record_scheduler_start(&mut self, task_count: u64, base_tick_ms: f64) {
        // Record scheduler start.
        //
        // Parameters:
        // - `self` — method receiver
        // - `task_count` — input value
        // - `base_tick_ms` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.record_scheduler_start(task_count, base_tick_ms);

        // Call multiplexed tasks = task count; on the current instance.
        self.scheduler.multiplexed_tasks = task_count;
        self.scheduler.base_tick_ms = base_tick_ms;
    }

    pub fn record_scheduler_tick(&mut self) {
        // Record scheduler tick.
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
        // let result = instance.record_scheduler_tick();

        // Call scheduler ticks += 1; on the current instance.
        self.scheduler.scheduler_ticks += 1;
    }

    pub fn record_emergency_stop(&mut self) {
        // Record emergency stop.
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
        // let result = instance.record_emergency_stop();

        // Call emergency stops += 1; on the current instance.
        self.scheduler.emergency_stops += 1;
    }

    pub fn record_spawn(&mut self) {
        // Record spawn.
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
        // let result = instance.record_spawn();

        // Call spawns += 1; on the current instance.
        self.execution.spawns += 1;
    }

    pub fn record_fire_and_forget_spawn(&mut self) {
        // Record fire and forget spawn.
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
        // let result = instance.record_fire_and_forget_spawn();

        // Call fire and forget spawns += 1; on the current instance.
        self.execution.fire_and_forget_spawns += 1;
    }

    pub fn record_join(&mut self) {
        // Record join.
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
        // let result = instance.record_join();

        // Call joins += 1; on the current instance.
        self.execution.joins += 1;
    }

    pub fn record_parallel_block(&mut self) {
        // Record parallel block.
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
        // let result = instance.record_parallel_block();

        // Call parallel blocks += 1; on the current instance.
        self.execution.parallel_blocks += 1;
    }

    pub fn record_replay_frames(&mut self, count: u64) {
        // Record replay frames.
        //
        // Parameters:
        // - `self` — method receiver
        // - `count` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.record_replay_frames(count);

        // Call replay frames = count; on the current instance.
        self.replay_frames = count;
    }

    pub fn trigger_mut(
        &mut self,
        name: &str,
        category: &str,
        priority: TaskPriority,
    ) -> &mut TriggerMetrics {
        // Trigger mut.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `category` — input value
        // - `priority` — input value
        //
        // Returns:
        // &mut TriggerMetrics.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.trigger_mut(name, category, priority);

        // Call triggers on the current instance.
        self.triggers
            .entry(name.to_string())
            .or_insert_with(|| TriggerMetrics {
                name: name.to_string(),
                category: category.to_string(),
                priority: priority_label(priority),
                ..Default::default()
            })
    }

    pub fn record_trigger_execution(
        &mut self,
        name: &str,
        category: &str,
        priority: TaskPriority,
        duration_ms: f64,
        failed: bool,
    ) {
        // Record trigger execution.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `category` — input value
        // - `priority` — input value
        // - `duration_ms` — input value
        // - `failed` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.record_trigger_execution(name, category, priority, duration_ms, failed);

        // Compute entry for the following logic.
        let entry = self.trigger_mut(name, category, priority);
        entry.executions += 1;

        // Take this path when failed.
        if failed {
            entry.failures += 1;
        }
        entry.last_duration_ms = duration_ms;

        // Take this path when duration ms > entry.max duration ms.
        if duration_ms > entry.max_duration_ms {
            entry.max_duration_ms = duration_ms;
        }
    }

    pub fn record_trigger_missed_deadline(
        &mut self,
        name: &str,
        category: &str,
        priority: TaskPriority,
    ) {
        // Record trigger missed deadline.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `category` — input value
        // - `priority` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.record_trigger_missed_deadline(name, category, priority);

        // Call trigger mut on the current instance.
        self.trigger_mut(name, category, priority).missed_deadlines += 1;
    }
}

fn priority_label(priority: TaskPriority) -> String {
    // Priority label.
    //
    // Parameters:
    // - `priority` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::telemetry::priority_label(priority);

    // Match on priority and handle each case.
    match priority {
        TaskPriority::Critical => "critical".into(),
        TaskPriority::High => "high".into(),
        TaskPriority::Normal => "normal".into(),
        TaskPriority::Low => "low".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregates_task_and_scheduler_metrics() {
        // Aggregates task and scheduler metrics.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::telemetry::aggregates_task_and_scheduler_metrics();

        let mut telemetry = RuntimeTelemetry::default();
        telemetry.record_scheduler_start(2, 50.0);
        telemetry.record_scheduler_tick();
        telemetry.record_task_tick("sense", TaskPriority::High, 50.0);
        telemetry.record_missed_deadline("sense", TaskPriority::High, 50.0);
        telemetry.record_spawn();
        telemetry.record_join();

        assert_eq!(telemetry.scheduler.scheduler_ticks, 1);
        assert_eq!(telemetry.tasks["sense"].ticks, 1);
        assert_eq!(telemetry.tasks["sense"].missed_deadlines, 1);
        assert_eq!(telemetry.execution.spawns, 1);
        assert_eq!(telemetry.execution.joins, 1);
    }
}

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
        self.task_mut(name, priority, interval_ms).ticks += 1;
    }

    pub fn record_task_duration(
        &mut self,
        name: &str,
        priority: TaskPriority,
        interval_ms: f64,
        duration_ms: f64,
    ) {
        let entry = self.task_mut(name, priority, interval_ms);
        entry.last_duration_ms = duration_ms;
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
        self.task_mut(name, priority, interval_ms).budget_violations += 1;
    }

    pub fn record_task_skip(&mut self, name: &str, priority: TaskPriority, interval_ms: f64) {
        self.task_mut(name, priority, interval_ms).skipped += 1;
    }

    pub fn record_missed_deadline(&mut self, name: &str, priority: TaskPriority, interval_ms: f64) {
        self.task_mut(name, priority, interval_ms).missed_deadlines += 1;
    }

    pub fn record_scheduler_start(&mut self, task_count: u64, base_tick_ms: f64) {
        self.scheduler.multiplexed_tasks = task_count;
        self.scheduler.base_tick_ms = base_tick_ms;
    }

    pub fn record_scheduler_tick(&mut self) {
        self.scheduler.scheduler_ticks += 1;
    }

    pub fn record_emergency_stop(&mut self) {
        self.scheduler.emergency_stops += 1;
    }

    pub fn record_spawn(&mut self) {
        self.execution.spawns += 1;
    }

    pub fn record_fire_and_forget_spawn(&mut self) {
        self.execution.fire_and_forget_spawns += 1;
    }

    pub fn record_join(&mut self) {
        self.execution.joins += 1;
    }

    pub fn record_parallel_block(&mut self) {
        self.execution.parallel_blocks += 1;
    }

    pub fn record_replay_frames(&mut self, count: u64) {
        self.replay_frames = count;
    }

    pub fn trigger_mut(
        &mut self,
        name: &str,
        category: &str,
        priority: TaskPriority,
    ) -> &mut TriggerMetrics {
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
        let entry = self.trigger_mut(name, category, priority);
        entry.executions += 1;
        if failed {
            entry.failures += 1;
        }
        entry.last_duration_ms = duration_ms;
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
        self.trigger_mut(name, category, priority).missed_deadlines += 1;
    }
}

fn priority_label(priority: TaskPriority) -> String {
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

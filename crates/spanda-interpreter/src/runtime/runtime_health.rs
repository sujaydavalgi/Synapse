//! Runtime health polling wired to hardware monitor state.

use super::{Interpreter, RobotBackend};
use spanda_ast::nodes::{Program, RobotDecl};
use spanda_capability::{evaluate_runtime_health, HealthStatus};
use std::collections::HashMap;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn cache_health_program(&mut self, program: &Program) {
        // Cache the program when it declares health checks for runtime polling.
        //
        // Parameters:
        // - `self` — method receiver
        // - `program` — parsed program
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.cache_health_program(program);

        let Program::Program {
            health_checks,
            robots,
            ..
        } = program;
        let has_health = !health_checks.is_empty()
            || robots.iter().any(|robot| {
                let RobotDecl::RobotDecl {
                    health_checks: robot_checks,
                    ..
                } = robot;
                !robot_checks.is_empty()
            });
        self.health_program = has_health.then(|| program.clone());
        self.last_health_overall = None;
    }

    pub(super) fn poll_runtime_health_changes(&mut self) {
        // Re-evaluate health checks when monitor faults or events change.
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
        // let result = instance.poll_runtime_health_changes();

        let Some(program) = self.health_program.clone() else {
            return;
        };
        let faults = self.hardware_monitor.runtime_faults();
        let events = self.hardware_monitor.runtime_events();
        let report = evaluate_runtime_health(&faults, &events, &program);
        let label = format!("{:?}", report.overall);
        if self.last_health_overall.as_deref() == Some(label.as_str()) {
            return;
        }
        self.last_health_overall = Some(label.clone());
        self.log(format!(
            "health: overall {label} (monitor={})",
            self.hardware_monitor.overall_health_label()
        ));
        for check in &report.checks {
            if !matches!(check.status, HealthStatus::Healthy | HealthStatus::Unknown) {
                self.log(format!(
                    "health: {} on {} {:?}",
                    check.name, check.target, check.status
                ));
            }
        }
        if matches!(
            report.overall,
            HealthStatus::Critical | HealthStatus::Unsafe | HealthStatus::Failed
        ) {
            self.record_debug_event(
                1,
                "health_critical",
                &[("overall", label)],
            );
        }
    }

    pub(super) fn record_debug_event(&self, line: u32, reason: &str, vars: &[(&str, String)]) {
        // Record a debugger pause/event when a debug session is attached.
        //
        // Parameters:
        // - `self` — method receiver
        // - `line` — source line for the event
        // - `reason` — pause reason label
        // - `vars` — variable snapshot entries
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.record_debug_event(line, reason, vars);

        let Some(debug) = &self.options.debug else {
            return;
        };
        let mut variables = HashMap::new();
        for (key, value) in vars {
            variables.insert((*key).into(), value.clone());
        }
        debug.record_pause(line, reason, variables);
    }
}

//! Runtime health polling wired to hardware monitor state.

use super::{Interpreter, RobotBackend};
use spanda_ast::foundations::{HealthPolicyDecl, HealthPolicyReaction};
use spanda_ast::nodes::{Program, RobotDecl};
use spanda_capability::{evaluate_runtime_health, HealthReport, HealthStatus};
use std::collections::HashMap;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn cache_health_program(&mut self, program: &Program) {
        // Description:
        //     Cache health program.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     progra: &Program
        //         Caller-supplied progra.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_health::cache_health_program(&mut self, progra);

        let Program::Program {
            health_checks,
            health_policies,
            anomaly_handlers,
            state_estimators,
            recovery_policies,
            mitigations,
            robots,
            ..
        } = program;
        let has_health = !health_checks.is_empty()
            || !health_policies.is_empty()
            || !anomaly_handlers.is_empty()
            || !state_estimators.is_empty()
            || !recovery_policies.is_empty()
            || !mitigations.is_empty()
            || robots.iter().any(|robot| {
                let RobotDecl::RobotDecl {
                    health_checks: robot_checks,
                    ..
                } = robot;
                !robot_checks.is_empty()
            });
        self.health_program = has_health.then(|| program.clone());
        self.last_health_overall = None;
        self.last_health_checks.clear();
        self.applied_health_reactions.clear();
    }

    pub(super) fn poll_runtime_health_changes(&mut self) {
        // Description:
        //     Poll runtime health changes.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_health::poll_runtime_health_changes(&mut self);

        let Some(program) = self.health_program.clone() else {
            return;
        };
        let faults = self.hardware_monitor.runtime_faults();
        let events = self.hardware_monitor.runtime_events();
        let mut report = evaluate_runtime_health(&faults, &events, &program);
        spanda_capability::apply_fleet_health_checks(&mut report, &program, &self.fleets, &faults);
        let label = format!("{:?}", report.overall);
        let mut any_change = false;

        if self.last_health_overall.as_deref() != Some(label.as_str()) {
            any_change = true;
            let _ = spanda_telemetry_store::record_health_event(
                "overall",
                &label,
                self.sim_time_ms,
            );
            self.last_health_overall = Some(label.clone());
        }

        for check in &report.checks {
            let status = format!("{:?}", check.status);
            let key = format!("{}:{}", check.target, check.name);
            if self.last_health_checks.get(&key) != Some(&status) {
                any_change = true;
                let _ = spanda_telemetry_store::record_health_event(
                    format!("{}:{}", check.target, check.name),
                    &status,
                    self.sim_time_ms,
                );
                self.last_health_checks.insert(key, status);
            }
        }

        if !any_change {
            return;
        }

        self.poll_recovery_approvals();
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
            self.record_debug_event(1, "health_critical", &[("overall", label.clone())]);
        }
        self.apply_health_policy_reactions(&report);
        self.poll_learned_anomaly_detectors(&report);
        self.apply_anomaly_handlers(&report);
        self.apply_swarm_health_coordination(&report);
    }

    fn apply_swarm_health_coordination(&mut self, report: &HealthReport) {
        // Description:
        //     Apply swarm health coordination.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     repor: &HealthReport
        //         Caller-supplied repor.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_health::apply_swarm_health_coordination(&mut self, repor);

        if !matches!(
            report.overall,
            HealthStatus::Critical | HealthStatus::Unsafe | HealthStatus::Failed
        ) {
            return;
        }
        for swarm in &self.program_swarms {
            let spanda_ast::robotics_decl::SwarmDecl::SwarmDecl {
                name,
                fleet_name,
                policy,
                ..
            } = swarm;
            if self.fleets.members(fleet_name).is_some() {
                self.log(format!(
                    "swarm: {name} applying {:?} coordination for fleet {fleet_name} on {:?}",
                    policy, report.overall
                ));
                self.record_debug_event(
                    1,
                    "swarm_health_coordination",
                    &[
                        ("swarm", name.clone()),
                        ("fleet", fleet_name.clone()),
                        ("overall", format!("{:?}", report.overall)),
                    ],
                );
            }
        }
    }

    fn apply_health_policy_reactions(&mut self, report: &HealthReport) {
        // Description:
        //     Apply health policy reactions.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     repor: &HealthReport
        //         Caller-supplied repor.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_health::apply_health_policy_reactions(&mut self, repor);

        let Some(program) = self.health_program.clone() else {
            return;
        };
        let Program::Program {
            health_policies, ..
        } = program;
        if health_policies.is_empty() {
            return;
        }

        // Collect active status labels from overall and per-check results.
        let mut active_statuses = vec![format!("{:?}", report.overall)];
        for check in &report.checks {
            if !matches!(check.status, HealthStatus::Healthy | HealthStatus::Unknown) {
                active_statuses.push(format!("{:?}", check.status));
            }
        }

        // Run each policy reaction that matches an active status.
        for policy in &health_policies {
            let HealthPolicyDecl::HealthPolicyDecl {
                name, reactions, ..
            } = policy;
            for HealthPolicyReaction { status, body } in reactions {
                if !active_statuses
                    .iter()
                    .any(|active| active.eq_ignore_ascii_case(status))
                {
                    continue;
                }
                let key = format!("{name}:{status}");
                if !self.applied_health_reactions.insert(key) {
                    continue;
                }
                self.log(format!("health_policy: applying {name} on {status}"));
                for stmt in body {
                    if let Err(err) = self.execute_stmt(stmt) {
                        self.log(format!("health_policy: action failed: {err}"));
                    }
                }
                self.record_debug_event(
                    1,
                    "health_policy_applied",
                    &[("policy", name.clone()), ("status", status.clone())],
                );
            }
        }

        // Reset reaction latches when health returns to healthy.
        if matches!(report.overall, HealthStatus::Healthy) {
            self.applied_health_reactions.clear();
        }
    }

    pub(super) fn record_debug_event(&self, line: u32, reason: &str, vars: &[(&str, String)]) {
        // Description:
        //     Record debug event.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //     line: u32
        //         Caller-supplied line.
        //     reason: &str
        //         Caller-supplied reason.
        //     vars: &[(&str, String)]
        //         Caller-supplied vars.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_health::record_debug_event(&self, line, reason, vars);

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

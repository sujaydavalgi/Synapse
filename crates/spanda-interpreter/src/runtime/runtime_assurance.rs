//! Runtime anomaly handler reactions wired to health polling.

use super::{Interpreter, RobotBackend};
use spanda_ast::assurance_decl::AnomalyHandlerDecl;
use spanda_ast::nodes::Program;
use spanda_capability::{HealthReport, HealthStatus};

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn apply_anomaly_handlers(&mut self, report: &HealthReport) {
        let Some(program) = self.health_program.clone() else {
            return;
        };
        let Program::Program {
            anomaly_handlers,
            anomaly_detectors,
            ..
        } = program;
        if anomaly_handlers.is_empty() {
            return;
        }

        let health_degraded = !matches!(
            report.overall,
            HealthStatus::Healthy | HealthStatus::Unknown
        );
        if !health_degraded {
            self.applied_anomaly_handlers.clear();
            return;
        }

        let mut triggered: std::collections::HashSet<String> = anomaly_detectors
            .iter()
            .map(|det| {
                let spanda_ast::assurance_decl::AnomalyDetectorDecl::AnomalyDetectorDecl {
                    name,
                    ..
                } = det;
                name.clone()
            })
            .collect();

        for check in &report.checks {
            if matches!(check.status, HealthStatus::Healthy | HealthStatus::Unknown) {
                continue;
            }
            for det in &anomaly_detectors {
                let spanda_ast::assurance_decl::AnomalyDetectorDecl::AnomalyDetectorDecl {
                    name,
                    expected,
                    ..
                } = det;
                if check.name == *name || expected.iter().any(|e| check.metric.contains(&e.metric))
                {
                    triggered.insert(name.clone());
                }
            }
        }

        for handler in anomaly_handlers {
            let AnomalyHandlerDecl::AnomalyHandlerDecl {
                detector,
                severity,
                actions,
                ..
            } = handler;
            if !triggered.contains(detector.as_str()) {
                continue;
            }
            let key = format!("{detector}:{severity}");
            if !self.applied_anomaly_handlers.insert(key) {
                continue;
            }
            self.log(format!(
                "anomaly: applying handler for {detector} ({severity})"
            ));
            for action in actions {
                self.log(format!("anomaly: action {action}"));
                if action.contains("audit.record") {
                    self.record_debug_event(1, "audit_record", &[("event", action.clone())]);
                }
            }
            self.record_debug_event(
                1,
                "anomaly_handler_applied",
                &[
                    ("detector", detector.clone()),
                    ("severity", severity.clone()),
                ],
            );
        }
    }
}

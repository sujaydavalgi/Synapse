//! Runtime anomaly handler reactions and state estimator fusion wiring.

use super::{Interpreter, RobotBackend, RuntimeValue};
use spanda_ast::assurance_decl::{AnomalyHandlerDecl, StateEstimatorDecl};
use spanda_ast::nodes::Program;
use spanda_capability::{HealthReport, HealthStatus};

fn sensor_name_from_input(input: &str) -> String {
    input.split('.').next().unwrap_or(input).to_string()
}

impl<B: RobotBackend> Interpreter<B> {
    /// Register `state_estimator` declarations as fusion runtime bindings.
    pub(super) fn setup_state_estimators(&mut self) {
        let Some(program) = self.health_program.clone() else {
            return;
        };
        let Program::Program {
            state_estimators, ..
        } = program;
        if state_estimators.is_empty() {
            return;
        }

        let fusion_already_bound = self.env.get("fusion").is_some();
        for decl in &state_estimators {
            let StateEstimatorDecl::StateEstimatorDecl {
                name,
                inputs,
                output_type,
                ..
            } = decl;
            let sensors: Vec<String> = inputs.iter().map(|i| sensor_name_from_input(i)).collect();
            let fusion = RuntimeValue::SensorFusion {
                sensors: sensors.clone(),
                estimator: Some(name.clone()),
            };
            self.env.define(name.clone(), fusion);
            self.log(format!(
                "state_estimator '{name}': {} input(s) -> {output_type}",
                inputs.len()
            ));
        }

        if !fusion_already_bound && state_estimators.len() == 1 {
            let StateEstimatorDecl::StateEstimatorDecl { inputs, .. } = &state_estimators[0];
            let sensors: Vec<String> = inputs.iter().map(|i| sensor_name_from_input(i)).collect();
            self.fusion_sensors = sensors.clone();
            self.env.define(
                "fusion",
                RuntimeValue::SensorFusion {
                    sensors,
                    estimator: state_estimators.first().map(|decl| {
                        let StateEstimatorDecl::StateEstimatorDecl { name, .. } = decl;
                        name.clone()
                    }),
                },
            );
            self.log("state_estimator: aliased fusion binding".into());
        }
    }

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

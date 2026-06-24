//! Runtime anomaly handler reactions, learned backends, and state estimator fusion wiring.

use super::{Interpreter, RobotBackend, RuntimeValue};
use spanda_ast::assurance_decl::{AnomalyDetectorDecl, AnomalyHandlerDecl, StateEstimatorDecl};
use spanda_ast::nodes::Program;
use spanda_capability::{HealthReport, HealthStatus};
use spanda_providers::dispatch_official_package_call;
use std::time::Instant;

fn sensor_name_from_input(input: &str) -> String {
    input.split('.').next().unwrap_or(input).to_string()
}

fn collect_learned_detectors(program: &Program) -> Vec<(String, String)> {
    let Program::Program {
        imports,
        anomaly_detectors,
        ..
    } = program;
    let import_backend = imports.iter().find_map(|imp| {
        let spanda_ast::nodes::ImportDecl::ImportDecl { path, .. } = imp;
        if path.contains("assurance.anomaly") || path.ends_with("anomaly") {
            Some(path.clone())
        } else {
            None
        }
    });
    anomaly_detectors
        .iter()
        .filter_map(|decl| {
            let AnomalyDetectorDecl::AnomalyDetectorDecl {
                name,
                learned_backend,
                ..
            } = decl;
            let backend = learned_backend
                .clone()
                .or_else(|| import_backend.clone())?;
            Some((name.clone(), backend))
        })
        .collect()
}

fn observed_confidence(report: &HealthReport, detector: &str) -> f64 {
    for check in &report.checks {
        if check.name == detector
            || check.metric.contains("confidence")
            || check.metric.contains("localization")
        {
            if !matches!(check.status, HealthStatus::Healthy | HealthStatus::Unknown) {
                return 0.5;
            }
        }
    }
    if !matches!(
        report.overall,
        HealthStatus::Healthy | HealthStatus::Unknown
    ) {
        return 0.6;
    }
    0.95
}

impl<B: RobotBackend> Interpreter<B> {
    /// Ensure the learned anomaly package is registered when detectors declare backends.
    pub(super) fn ensure_learned_anomaly_package(&self) {
        let mut registry = self.provider_registry.borrow_mut();
        if registry.has_official_package("spanda-anomaly") {
            registry.grant_capability("assurance.anomaly.scan");
            return;
        }
        let mut names = registry.official_packages().to_vec();
        names.push("spanda-anomaly".into());
        registry.set_official_packages(names);
        registry.grant_capability("assurance.anomaly.scan");
    }

    /// Poll optional learned anomaly backends during health transitions.
    pub(super) fn poll_learned_anomaly_detectors(&mut self, report: &HealthReport) {
        let Some(program) = self.health_program.clone() else {
            return;
        };
        let learned = collect_learned_detectors(&program);
        if learned.is_empty() {
            return;
        }
        self.ensure_learned_anomaly_package();
        self.learned_anomaly_triggers.clear();

        for (detector, backend) in learned {
            let observed = observed_confidence(report, &detector);
            let args = vec![
                RuntimeValue::String {
                    value: detector.clone(),
                },
                RuntimeValue::Number {
                    value: observed,
                    unit: spanda_ast::nodes::UnitKind::None,
                },
            ];
            let trace_providers = self.options.trace_providers;
            let record_trace = self.options.record_trace;
            let sim_time_ms = self.sim_time_ms;
            let started = Instant::now();
            let score = {
                let mut registry = self.provider_registry.borrow_mut();
                dispatch_official_package_call(
                    &mut registry,
                    &backend,
                    "scan_learned",
                    &args,
                    if trace_providers {
                        Some(&mut self.telemetry)
                    } else {
                        None
                    },
                    if record_trace {
                        self.mission_trace.as_mut()
                    } else {
                        None
                    },
                    sim_time_ms,
                )
            };
            let Some(RuntimeValue::Number { value, .. }) = score else {
                continue;
            };
            if value > 0.0 {
                self.learned_anomaly_triggers.insert(detector.clone());
                self.log(format!(
                    "learned anomaly: {detector} via {backend} (observed={observed:.2})"
                ));
                self.record_debug_event(
                    1,
                    "learned_anomaly_detected",
                    &[
                        ("detector", detector),
                        ("backend", backend),
                        ("score", format!("{value:.2}")),
                    ],
                );
                let _ = started;
            }
        }
    }

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
        if !health_degraded && self.learned_anomaly_triggers.is_empty() {
            self.applied_anomaly_handlers.clear();
            return;
        }

        let mut triggered: std::collections::HashSet<String> = anomaly_detectors
            .iter()
            .map(|det| {
                let AnomalyDetectorDecl::AnomalyDetectorDecl { name, .. } = det;
                name.clone()
            })
            .collect();
        triggered.extend(self.learned_anomaly_triggers.iter().cloned());

        for check in &report.checks {
            if matches!(check.status, HealthStatus::Healthy | HealthStatus::Unknown) {
                continue;
            }
            for det in &anomaly_detectors {
                let AnomalyDetectorDecl::AnomalyDetectorDecl {
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

//! Anomaly detection static analysis.
//!
use crate::types::{Anomaly, AnomalySeverity, ExpectedBehaviorModel, LearnedBehaviorModel};
use spanda_ast::assurance_decl::{AnomalyDetectorDecl, AnomalyHandlerDecl, ExpectedBehavior};
use spanda_ast::nodes::Program;
use spanda_capability::evaluate_health_checks;

/// Anomaly scan report.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AnomalyReport {
    pub detectors: Vec<ExpectedBehaviorModel>,
    pub handlers: Vec<String>,
    pub anomalies: Vec<Anomaly>,
    pub passed: bool,
}

fn parse_severity(raw: &str) -> AnomalySeverity {
    match raw.to_lowercase().as_str() {
        "critical" => AnomalySeverity::Critical,
        "high" => AnomalySeverity::High,
        "medium" => AnomalySeverity::Medium,
        _ => AnomalySeverity::Low,
    }
}

/// Scan program for anomaly detector coverage and static violations.
pub fn scan_anomalies(program: &Program) -> AnomalyReport {
    let Program::Program {
        anomaly_detectors,
        anomaly_handlers,
        ..
    } = program;

    let detectors: Vec<ExpectedBehaviorModel> = anomaly_detectors
        .iter()
        .map(|decl| {
            let AnomalyDetectorDecl::AnomalyDetectorDecl { name, expected, .. } = decl;
            ExpectedBehaviorModel {
                detector: name.clone(),
                rules: expected
                    .iter()
                    .map(|e| format!("{} {} {}", e.metric, e.operator, e.threshold))
                    .collect(),
            }
        })
        .collect();

    let handlers: Vec<String> = anomaly_handlers
        .iter()
        .map(|h| {
            let AnomalyHandlerDecl::AnomalyHandlerDecl {
                detector,
                severity,
                actions,
                ..
            } = h;
            format!("{detector}@{severity}: {}", actions.join(", "))
        })
        .collect();

    let mut anomalies = Vec::new();
    let health = evaluate_health_checks(program);
    for check in &health.checks {
        if matches!(
            check.status,
            spanda_capability::HealthStatus::Failed
                | spanda_capability::HealthStatus::Critical
                | spanda_capability::HealthStatus::Unsafe
        ) {
            anomalies.push(Anomaly {
                detector: check.name.clone(),
                metric: check.metric.clone(),
                expected: check.threshold.clone(),
                observed: check
                    .message
                    .clone()
                    .unwrap_or_else(|| format!("{:?}", check.status)),
                severity: AnomalySeverity::High,
            });
        }
    }

    for decl in anomaly_detectors {
        let AnomalyDetectorDecl::AnomalyDetectorDecl { name, expected, .. } = decl;
        if expected.is_empty() {
            anomalies.push(Anomaly {
                detector: name.clone(),
                metric: "expected".into(),
                expected: "at least one rule".into(),
                observed: "none".into(),
                severity: AnomalySeverity::Medium,
            });
        }
    }

    let handler_names: std::collections::HashSet<_> = anomaly_handlers
        .iter()
        .map(|h| {
            let AnomalyHandlerDecl::AnomalyHandlerDecl { detector, .. } = h;
            detector.clone()
        })
        .collect();
    for det in anomaly_detectors {
        let AnomalyDetectorDecl::AnomalyDetectorDecl { name, .. } = det;
        if !handler_names.contains(name) {
            anomalies.push(Anomaly {
                detector: name.clone(),
                metric: "handler".into(),
                expected: "on anomaly handler".into(),
                observed: "missing".into(),
                severity: AnomalySeverity::Low,
            });
        }
    }

    let passed = anomalies.iter().all(|a| {
        !matches!(
            a.severity,
            AnomalySeverity::Critical | AnomalySeverity::High
        )
    });

    AnomalyReport {
        detectors,
        handlers,
        anomalies,
        passed,
    }
}

/// List learned behavior model placeholders (optional package backends).
pub fn learned_models(_program: &Program) -> Vec<LearnedBehaviorModel> {
    Vec::new()
}

/// Parse severity from handler declaration.
pub fn handler_severity(raw: &str) -> AnomalySeverity {
    parse_severity(raw)
}

/// Format expected behavior for reports.
pub fn format_expected(e: &ExpectedBehavior) -> String {
    format!("{} {} {}", e.metric, e.operator, e.threshold)
}

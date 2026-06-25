//! Deployment gate evaluation before rollout.
//!
use crate::engine::evaluate_readiness;
use crate::types::{ReadinessOptions, ReadinessSeverity};
use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;
use spanda_capability::capability_traceability;

/// Named deployment gate with pass/fail outcome.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeploymentGate {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

/// Deployment gate evaluation report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeploymentGateReport {
    pub passed: bool,
    pub gates: Vec<DeploymentGate>,
}

/// Threshold policy for deployment gates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeploymentGatePolicy {
    pub minimum_readiness_score: u32,
    pub require_safety_audit: bool,
    pub require_capability_traceability: bool,
}

impl Default for DeploymentGatePolicy {
    fn default() -> Self {
        Self {
            minimum_readiness_score: 80,
            require_safety_audit: true,
            require_capability_traceability: true,
        }
    }
}

impl DeploymentGatePolicy {
    pub fn production() -> Self {
        Self {
            minimum_readiness_score: 90,
            require_safety_audit: true,
            require_capability_traceability: true,
        }
    }
}

/// Evaluate deployment gates for a program before rollout.
pub fn evaluate_deployment_gates(
    program: &Program,
    source: &str,
    options: &ReadinessOptions,
    policy: &DeploymentGatePolicy,
) -> DeploymentGateReport {
    // Run readiness, safety, and capability gates for deploy blocking.
    //
    // Parameters:
    // - `program` — parsed `.sd` program
    // - `source` — program source for safety audit
    // - `options` — readiness evaluation options
    // - `policy` — gate thresholds and required checks
    //
    // Returns:
    // Deployment gate report with per-gate pass/fail.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = evaluate_deployment_gates(&program, source, &options, &policy);

    let mut gates = Vec::new();
    let readiness = evaluate_readiness(program, options);
    gates.push(DeploymentGate {
        name: "readiness".into(),
        passed: readiness.mission_ready && readiness.score.total >= policy.minimum_readiness_score,
        message: format!(
            "score {} (min {}), mission_ready={}",
            readiness.score.total, policy.minimum_readiness_score, readiness.mission_ready
        ),
    });
    if policy.require_safety_audit {
        let audit = crate::auditor::audit_program(program, source);
        let passed = audit.critical_count == 0 && audit.high_count == 0;
        gates.push(DeploymentGate {
            name: "safety".into(),
            passed,
            message: format!(
                "critical={}, high={}, medium={}, low={}",
                audit.critical_count, audit.high_count, audit.medium_count, audit.low_count
            ),
        });
    }
    if policy.require_capability_traceability {
        let trace = capability_traceability(program);
        let failed = trace
            .capability_rows
            .iter()
            .filter(|row| row.status == "FAIL")
            .count();
        let passed = trace.errors.is_empty() && failed == 0;
        gates.push(DeploymentGate {
            name: "capability".into(),
            passed,
            message: if passed {
                "capability traceability passed".into()
            } else {
                format!("{failed} capability rows failed, {} trace errors", trace.errors.len())
            },
        });
    }
    let health_issues = readiness
        .issues
        .iter()
        .filter(|issue| issue.factor == "Health" && issue.severity >= ReadinessSeverity::High)
        .count();
    gates.push(DeploymentGate {
        name: "health".into(),
        passed: health_issues == 0,
        message: if health_issues == 0 {
            "no high-severity health issues".into()
        } else {
            format!("{health_issues} high-severity health issues")
        },
    });
    let passed = gates.iter().all(|gate| gate.passed);
    DeploymentGateReport { passed, gates }
}

//! Extended readiness traceability matrix.

use serde::{Deserialize, Serialize};
use spanda_ast::foundations::MissionDecl;
use spanda_ast::nodes::Program;
use spanda_capability::{capability_traceability, health_traceability};

/// Readiness traceability row linking requirement to verification status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadinessTraceRow {
    pub mission_requirement: String,
    pub capability: String,
    pub hardware: String,
    pub provider: String,
    pub package: String,
    pub health_check: String,
    pub safety_rule: String,
    pub verification_status: String,
    pub readiness_impact: String,
}

/// Build readiness traceability matrix for a program.
pub fn readiness_traceability(program: &Program) -> Vec<ReadinessTraceRow> {
    let trace = capability_traceability(program);
    let health = health_traceability(program);
    let mut rows = Vec::new();

    let Program::Program { robots, .. } = program;
    for robot in robots {
        let spanda_ast::nodes::RobotDecl::RobotDecl {
            name,
            mission,
            exposes_capabilities,
            ..
        } = robot;
        let mission_caps: Vec<String> = mission
            .as_ref()
            .map(|m| {
                let MissionDecl::MissionDecl {
                    required_capabilities,
                    ..
                } = m;
                required_capabilities.clone()
            })
            .unwrap_or_default();

        let caps: Vec<String> = if mission_caps.is_empty() {
            exposes_capabilities.clone()
        } else {
            mission_caps
        };

        for cap in caps {
            let cap_row = trace.capability_rows.iter().find(|r| r.capability == cap);
            let health_row = health.iter().find(|h| h.component == *name);
            rows.push(ReadinessTraceRow {
                mission_requirement: cap.clone(),
                capability: cap.clone(),
                hardware: cap_row.map(|r| r.hardware.clone()).unwrap_or_default(),
                provider: cap_row.map(|r| r.provider.clone()).unwrap_or_default(),
                package: cap_row.map(|r| r.package.clone()).unwrap_or_default(),
                health_check: health_row
                    .map(|h| h.health_check.clone())
                    .unwrap_or_else(|| "—".into()),
                safety_rule: cap_row
                    .and_then(|r| r.safety_rule.clone())
                    .unwrap_or_else(|| "—".into()),
                verification_status: cap_row
                    .map(|r| r.status.clone())
                    .unwrap_or_else(|| "UNKNOWN".into()),
                readiness_impact: match cap_row.map(|r| r.status.as_str()) {
                    Some("OK") => "None".into(),
                    Some("FAIL") => "Blocks mission".into(),
                    Some("PARTIAL") => "Degraded readiness".into(),
                    _ => "Review required".into(),
                },
            });
        }
    }

    rows
}

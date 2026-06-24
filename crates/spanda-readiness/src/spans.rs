//! Span lookup for readiness diagnostics in the IDE.

use crate::types::ReadinessIssue;
use spanda_ast::assurance_decl::{
    AnomalyDetectorDecl, AssuranceCaseDecl, KnowledgeModelDecl, StateEstimatorDecl,
};
use spanda_ast::foundations::{DeployDecl, HealthCheckDecl};
use spanda_ast::nodes::Program;

/// Resolve a display line/column for a readiness issue factor.
pub fn line_column_for_factor(program: &Program, factor: &str) -> (u32, u32) {
    match factor {
        "Hardware" | "Battery" | "Connectivity" | "Storage" | "Compute" | "Packages"
        | "Providers" => deploy_span(program).unwrap_or_else(|| first_robot_span(program)),
        "Health" => first_health_check_span(program).unwrap_or_else(|| first_robot_span(program)),
        "Capabilities" | "Mission Requirements" => {
            mission_span(program).unwrap_or_else(|| first_robot_span(program))
        }
        "Safety" => first_robot_safety_span(program).unwrap_or_else(|| first_robot_span(program)),
        "Fleet" => first_fleet_span(program).unwrap_or((1, 1)),
        "Assurance" => assurance_span(program).unwrap_or((1, 1)),
        _ => (1, 1),
    }
}

/// Resolve a precise line/column for a readiness issue using message context.
pub fn line_column_for_issue(program: &Program, issue: &ReadinessIssue) -> (u32, u32) {
    if issue.factor == "Assurance" {
        if let Some(name) = extract_quoted_name(&issue.message, "Anomaly detector '") {
            if let Some(span) = anomaly_detector_span(program, &name) {
                return span;
            }
        }
        if issue.message.contains("Assurance case") {
            if let Some(span) = first_assurance_case_without_evidence(program) {
                return span;
            }
        }
        if issue.message.contains("Knowledge model") {
            if let Some(span) = first_empty_knowledge_model(program) {
                return span;
            }
        }
        if let Some(name) = extract_quoted_name(&issue.message, "State estimator '") {
            if let Some(span) = state_estimator_span(program, &name) {
                return span;
            }
        }
        if issue.message.contains("State estimator") {
            if let Some(span) = first_empty_state_estimator(program) {
                return span;
            }
        }
        if let Some(span) = assurance_span(program) {
            return span;
        }
    }
    line_column_for_factor(program, &issue.factor)
}

fn extract_quoted_name(message: &str, prefix: &str) -> Option<String> {
    let rest = message.strip_prefix(prefix)?;
    let end = rest.find('\'')?;
    Some(rest[..end].to_string())
}

fn assurance_span(program: &Program) -> Option<(u32, u32)> {
    first_assurance_case_span(program)
        .or_else(|| first_knowledge_model_span(program))
        .or_else(|| first_state_estimator_span(program))
        .or_else(|| first_anomaly_detector_span(program))
        .or_else(|| first_mitigation_span(program))
}

fn span_coords(span: &spanda_ast::nodes::Span) -> (u32, u32) {
    (span.start.line, span.start.column)
}

fn first_assurance_case_span(program: &Program) -> Option<(u32, u32)> {
    let Program::Program {
        assurance_cases, ..
    } = program;
    assurance_cases.first().map(|decl| {
        let AssuranceCaseDecl::AssuranceCaseDecl { span, .. } = decl;
        span_coords(span)
    })
}

fn first_assurance_case_without_evidence(program: &Program) -> Option<(u32, u32)> {
    let Program::Program {
        assurance_cases, ..
    } = program;
    assurance_cases.iter().find_map(|decl| {
        let AssuranceCaseDecl::AssuranceCaseDecl { evidence, span, .. } = decl;
        if evidence.is_empty() {
            Some(span_coords(span))
        } else {
            None
        }
    })
}

fn first_knowledge_model_span(program: &Program) -> Option<(u32, u32)> {
    let Program::Program {
        knowledge_models, ..
    } = program;
    knowledge_models.first().map(|decl| {
        let KnowledgeModelDecl::KnowledgeModelDecl { span, .. } = decl;
        span_coords(span)
    })
}

fn first_empty_knowledge_model(program: &Program) -> Option<(u32, u32)> {
    let Program::Program {
        knowledge_models, ..
    } = program;
    knowledge_models.iter().find_map(|decl| {
        let KnowledgeModelDecl::KnowledgeModelDecl {
            components, span, ..
        } = decl;
        if components.is_empty() {
            Some(span_coords(span))
        } else {
            None
        }
    })
}

fn first_anomaly_detector_span(program: &Program) -> Option<(u32, u32)> {
    let Program::Program {
        anomaly_detectors, ..
    } = program;
    anomaly_detectors.first().map(|decl| {
        let AnomalyDetectorDecl::AnomalyDetectorDecl { span, .. } = decl;
        span_coords(span)
    })
}

fn anomaly_detector_span(program: &Program, name: &str) -> Option<(u32, u32)> {
    let Program::Program {
        anomaly_detectors, ..
    } = program;
    anomaly_detectors.iter().find_map(|decl| {
        let AnomalyDetectorDecl::AnomalyDetectorDecl {
            span, name: det, ..
        } = decl;
        if det == name {
            Some(span_coords(span))
        } else {
            None
        }
    })
}

fn first_mitigation_span(program: &Program) -> Option<(u32, u32)> {
    let Program::Program { mitigations, .. } = program;
    mitigations.first().map(|decl| {
        let spanda_ast::assurance_decl::MitigationDecl::MitigationDecl { span, .. } = decl;
        span_coords(span)
    })
}

fn first_state_estimator_span(program: &Program) -> Option<(u32, u32)> {
    let Program::Program {
        state_estimators, ..
    } = program;
    state_estimators.first().map(|decl| {
        let StateEstimatorDecl::StateEstimatorDecl { span, .. } = decl;
        span_coords(span)
    })
}

fn first_empty_state_estimator(program: &Program) -> Option<(u32, u32)> {
    let Program::Program {
        state_estimators, ..
    } = program;
    state_estimators.iter().find_map(|decl| {
        let StateEstimatorDecl::StateEstimatorDecl { inputs, span, .. } = decl;
        if inputs.is_empty() {
            Some(span_coords(span))
        } else {
            None
        }
    })
}

fn state_estimator_span(program: &Program, name: &str) -> Option<(u32, u32)> {
    let Program::Program {
        state_estimators, ..
    } = program;
    state_estimators.iter().find_map(|decl| {
        let StateEstimatorDecl::StateEstimatorDecl {
            span, name: est, ..
        } = decl;
        if est == name {
            Some(span_coords(span))
        } else {
            None
        }
    })
}

fn deploy_span(program: &Program) -> Option<(u32, u32)> {
    let Program::Program { deployments, .. } = program;
    deployments.first().map(|deploy| {
        let DeployDecl::DeployDecl { span, .. } = deploy;
        (span.start.line, span.start.column)
    })
}

fn first_robot_span(program: &Program) -> (u32, u32) {
    let Program::Program { robots, .. } = program;
    robots
        .first()
        .map(|robot| {
            let spanda_ast::nodes::RobotDecl::RobotDecl { span, .. } = robot;
            (span.start.line, span.start.column)
        })
        .unwrap_or((1, 1))
}

fn first_health_check_span(program: &Program) -> Option<(u32, u32)> {
    let Program::Program { health_checks, .. } = program;
    health_checks.first().map(|hc| {
        let HealthCheckDecl::HealthCheckDecl { span, .. } = hc;
        (span.start.line, span.start.column)
    })
}

fn mission_span(program: &Program) -> Option<(u32, u32)> {
    let Program::Program { robots, .. } = program;
    for robot in robots {
        let spanda_ast::nodes::RobotDecl::RobotDecl { mission, .. } = robot;
        if let Some(mission) = mission {
            let spanda_ast::foundations::MissionDecl::MissionDecl { span, .. } = mission;
            return Some((span.start.line, span.start.column));
        }
    }
    None
}

fn first_robot_safety_span(program: &Program) -> Option<(u32, u32)> {
    let Program::Program { robots, .. } = program;
    for robot in robots {
        let spanda_ast::nodes::RobotDecl::RobotDecl { safety, .. } = robot;
        if let Some(spanda_ast::nodes::SafetyBlock::SafetyBlock { span, .. }) = safety {
            return Some((span.start.line, span.start.column));
        }
    }
    None
}

fn first_fleet_span(program: &Program) -> Option<(u32, u32)> {
    let Program::Program { fleets, .. } = program;
    fleets.first().map(|fleet| {
        let spanda_ast::robotics_decl::FleetDecl::FleetDecl { span, .. } = fleet;
        (span.start.line, span.start.column)
    })
}

//! Fleet, swarm, group, and crowd readiness evaluation.

use crate::engine::evaluate_readiness;
use crate::types::{
    FleetReadinessReport, ReadinessIssue, ReadinessOptions, ReadinessReport, ReadinessSeverity,
    ReadinessStatus,
};
use spanda_ast::nodes::Program;
use spanda_capability::{evaluate_health_checks, HealthStatus};

/// Evaluate readiness across all robots in a fleet program.
pub fn evaluate_fleet_readiness(
    program: &Program,
    options: &ReadinessOptions,
) -> FleetReadinessReport {
    let Program::Program { robots, fleets, .. } = program;
    let health = evaluate_health_checks(program);

    let mut robot_reports = Vec::new();
    let mut healthy = 0u32;
    let mut degraded = 0u32;
    let mut not_ready = 0u32;
    let mut issues = Vec::new();

    if robots.is_empty() {
        issues.push(ReadinessIssue {
            factor: "Fleet".into(),
            severity: ReadinessSeverity::High,
            message: "No robots declared in fleet program".into(),
            suggested_action: Some("Add robot declarations".into()),
        });
    }

    for robot in robots {
        let spanda_ast::nodes::RobotDecl::RobotDecl { name, .. } = robot;
        let report = evaluate_readiness(program, options);
        let status = report.status;
        match status {
            ReadinessStatus::Ready => healthy += 1,
            ReadinessStatus::Degraded => degraded += 1,
            _ => not_ready += 1,
        }
        robot_reports.push(ReadinessReport {
            robots: vec![name.clone()],
            ..report
        });
    }

    for fleet in fleets {
        let spanda_ast::robotics_decl::FleetDecl::FleetDecl { name, members, .. } = fleet;
        issues.push(ReadinessIssue {
            factor: "Fleet".into(),
            severity: ReadinessSeverity::Info,
            message: format!("Fleet '{name}' has {} members", members.len()),
            suggested_action: None,
        });
    }

    for check in &health.checks {
        if check.metric.starts_with("require:")
            && (check.status == HealthStatus::Critical || check.status == HealthStatus::Failed)
        {
            issues.push(ReadinessIssue {
                factor: "Fleet".into(),
                severity: ReadinessSeverity::High,
                message: check
                    .message
                    .clone()
                    .unwrap_or_else(|| check.metric.clone()),
                suggested_action: Some("Review fleet health policy".into()),
            });
        }
    }

    let total = robots.len() as u32;
    let mission_capacity_percent = if total == 0 {
        0
    } else {
        (healthy + degraded)
            .saturating_mul(100)
            .checked_div(total)
            .unwrap_or(0)
            .min(100)
    };
    let fleet_score = if robot_reports.is_empty() {
        0
    } else {
        robot_reports.iter().map(|r| r.score.total).sum::<u32>() / robot_reports.len() as u32
    };

    FleetReadinessReport {
        fleet_score,
        healthy_robots: healthy,
        degraded_robots: degraded,
        not_ready_robots: not_ready,
        mission_capacity_percent,
        robot_reports,
        issues,
    }
}

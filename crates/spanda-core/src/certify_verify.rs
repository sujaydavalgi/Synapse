//! Certification proof checklist for `spanda verify --strict-certify`.

use crate::ast::{Program, RobotDecl};
use crate::foundations::DeployDecl;
use crate::hardware::{CompatItem, CompatSeverity};
use crate::robotics_platform::{CertifyDecl, CertificationStandard};

fn pass(category: &str, message: impl Into<String>, line: u32, column: u32) -> CompatItem {
    CompatItem {
        category: category.into(),
        message: message.into(),
        severity: CompatSeverity::Pass,
        line,
        column,
    }
}

fn warn(category: &str, message: impl Into<String>, line: u32, column: u32) -> CompatItem {
    CompatItem {
        category: category.into(),
        message: message.into(),
        severity: CompatSeverity::Warning,
        line,
        column,
    }
}

fn error(category: &str, message: impl Into<String>, line: u32, column: u32) -> CompatItem {
    CompatItem {
        category: category.into(),
        message: message.into(),
        severity: CompatSeverity::Error,
        line,
        column,
    }
}

fn robot_by_name<'a>(robots: &'a [RobotDecl], name: &str) -> Option<&'a RobotDecl> {
    robots.iter().find(|r| r.name() == name)
}

/// Build certification proof checklist items for verify reporting.
pub fn verify_certification_proof(program: &Program, strict: bool) -> Vec<CompatItem> {
    // Evaluate deploy/certify/safety/mission coverage for CI gates.
    //
    // Parameters:
    // - `program` — parsed Spanda program
    // - `strict` — treat gaps as errors instead of warnings
    //
    // Returns:
    // Compatibility items for the verify report.
    //
    // Options:
    // None.
    //
    // Example:
    // let items = verify_certification_proof(&program, true);

    let Program::Program {
        deployments,
        certifications,
        robots,
        program_safety_zones,
        ..
    } = program;

    let mut items = Vec::new();
    let has_deploy = !deployments.is_empty();
    let has_certify = !certifications.is_empty();

    if has_deploy && !has_certify {
        let item = if strict {
            error(
                "certify",
                "Deploy targets require certification metadata — add certify ISO13849 (or IEC61508 / ISO26262)",
                1,
                1,
            )
        } else {
            warn(
                "certify",
                "Deploy targets declared without certification metadata — add certify ISO13849 (or IEC61508 / ISO26262)",
                1,
                1,
            )
        };
        items.push(item);
    }

    if has_certify && !has_deploy {
        items.push(pass(
            "certify",
            "Certification metadata recorded — no deploy targets declared",
            1,
            1,
        ));
    }

    for cert in certifications {
        let CertifyDecl::CertifyDecl {
            standard,
            level,
            span,
        } = cert;
        if strict && matches!(standard, CertificationStandard::Iso13849) && level.is_none() {
            items.push(error(
                "certify",
                "ISO13849 certification should declare a performance level (e.g. PLd) under strict verify",
                span.start.line,
                span.start.column,
            ));
        }
        if strict && matches!(standard, CertificationStandard::Iso26262) && level.is_none() {
            items.push(warn(
                "certify",
                "ISO26262 certification should declare ASIL level under strict verify",
                span.start.line,
                span.start.column,
            ));
        }
    }

    for deploy in deployments {
        let DeployDecl::DeployDecl {
            robot_name,
            targets,
            span,
        } = deploy;
        let Some(robot) = robot_by_name(robots, robot_name) else {
            if strict {
                items.push(error(
                    "certify",
                    format!("Deploy robot '{robot_name}' not found for certification proof"),
                    span.start.line,
                    span.start.column,
                ));
            }
            continue;
        };
        let RobotDecl::RobotDecl {
            mission,
            safety,
            ..
        } = robot;
        if mission.is_none() {
            let item = if strict {
                error(
                    "certify",
                    format!("Deployed robot '{robot_name}' should declare a mission under strict verify"),
                    span.start.line,
                    span.start.column,
                )
            } else {
                warn(
                    "certify",
                    format!("Deployed robot '{robot_name}' has no mission metadata"),
                    span.start.line,
                    span.start.column,
                )
            };
            items.push(item);
        }
        if safety.is_none() {
            let item = if strict {
                error(
                    "certify",
                    format!("Deployed robot '{robot_name}' should declare safety rules under strict verify"),
                    span.start.line,
                    span.start.column,
                )
            } else {
                warn(
                    "certify",
                    format!("Deployed robot '{robot_name}' has no safety block"),
                    span.start.line,
                    span.start.column,
                )
            };
            items.push(item);
        }
        for hardware in targets {
            items.push(pass(
                "certify",
                format!(
                    "Deploy target {robot_name}@{hardware} covered by certification proof checklist"
                ),
                span.start.line,
                span.start.column,
            ));
        }
    }

    if strict && has_deploy && program_safety_zones.is_empty() {
        items.push(warn(
            "certify",
            "Strict certification verify recommends program-level safety_zone declarations for deployed fleets",
            1,
            1,
        ));
    }

    if has_certify && has_deploy {
        items.push(pass(
            "certify",
            "Certification proof checklist satisfied for declared deploy targets",
            1,
            1,
        ));
    }

    items
}

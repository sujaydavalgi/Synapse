//! Multi-robot fleet verification for collisions, deadlocks, and conflicts.

use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;
use std::collections::HashSet;

/// Fleet verification finding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetVerifyFinding {
    pub category: String,
    pub severity: String,
    pub message: String,
}

/// Fleet-level verification report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetVerifyReport {
    pub compatible: bool,
    pub findings: Vec<FleetVerifyFinding>,
}

/// Verify fleet programs for inter-robot conflicts.
pub fn verify_fleet(program: &Program) -> FleetVerifyReport {
    let Program::Program {
        robots,
        fleets,
        program_safety_zones,
        ..
    } = program;

    let mut findings = Vec::new();
    let robot_names: HashSet<String> = robots
        .iter()
        .map(|r| {
            let spanda_ast::nodes::RobotDecl::RobotDecl { name, .. } = r;
            name.clone()
        })
        .collect();

    if robots.len() > 1 && program_safety_zones.is_empty() {
        findings.push(FleetVerifyFinding {
            category: "collision".into(),
            severity: "warning".into(),
            message: "Multiple robots without shared safety zones — collision risk".into(),
        });
    }

    for fleet in fleets {
        let spanda_ast::robotics_decl::FleetDecl::FleetDecl { name, members, .. } = fleet;
        for member in members {
            if !robot_names.contains(member) {
                findings.push(FleetVerifyFinding {
                    category: "communication".into(),
                    severity: "error".into(),
                    message: format!("Fleet '{name}' references unknown robot '{member}'"),
                });
            }
        }
        if members.len() > 1 {
            let behaviors: Vec<&str> = robots
                .iter()
                .filter(|r| {
                    let spanda_ast::nodes::RobotDecl::RobotDecl { name, .. } = r;
                    members.contains(name)
                })
                .filter_map(|r| {
                    let spanda_ast::nodes::RobotDecl::RobotDecl { behaviors, .. } = r;
                    behaviors.first().map(|b| {
                        let spanda_ast::nodes::BehaviorDecl::BehaviorDecl { name, .. } = b;
                        name.as_str()
                    })
                })
                .collect();
            if behaviors.len() > 1 && behaviors.windows(2).any(|w| w[0] == w[1]) {
                findings.push(FleetVerifyFinding {
                    category: "resource-contention".into(),
                    severity: "warning".into(),
                    message: format!(
                        "Fleet '{name}' robots share identical behavior — resource contention likely"
                    ),
                });
            }
        }
    }

    let topics: HashSet<String> = robots
        .iter()
        .flat_map(|r| {
            let spanda_ast::nodes::RobotDecl::RobotDecl { topics, .. } = r;
            topics.iter().map(|t| {
                let spanda_ast::nodes::TopicDecl::TopicDecl { name, .. } = t;
                name.clone()
            })
        })
        .collect();
    if robots.len() > 2 && topics.len() < robots.len() / 2 {
        findings.push(FleetVerifyFinding {
            category: "communication".into(),
            severity: "medium".into(),
            message: "Limited inter-robot topics — communication failures may go undetected".into(),
        });
    }

    let compatible = !findings.iter().any(|f| f.severity == "error");
    FleetVerifyReport {
        compatible,
        findings,
    }
}

/// Verify fleet from source.
pub fn verify_fleet_source(source: &str) -> Result<FleetVerifyReport, spanda_error::SpandaError> {
    let tokens = spanda_lexer::tokenize(source)?;
    let program = spanda_parser::parse(tokens)?;
    Ok(verify_fleet(&program))
}

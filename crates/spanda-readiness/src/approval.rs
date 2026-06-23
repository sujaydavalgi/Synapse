//! Human-in-the-loop approval path verification.

use serde::{Deserialize, Serialize};
use spanda_ast::foundations::MissionDecl;
use spanda_ast::nodes::Program;

/// Approval verification row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovalVerifyRow {
    pub actor: String,
    pub action: String,
    pub approval_path_exists: bool,
    pub actor_exists: bool,
    pub fallback_exists: bool,
    pub status: String,
}

/// Approval verification report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovalVerifyReport {
    pub compatible: bool,
    pub rows: Vec<ApprovalVerifyRow>,
}

/// Verify human approval requirements on missions.
pub fn verify_approvals(program: &Program) -> ApprovalVerifyReport {
    let Program::Program { robots, .. } = program;
    let mut rows = Vec::new();

    let actor_robots: std::collections::HashSet<String> = robots
        .iter()
        .map(|r| {
            let spanda_ast::nodes::RobotDecl::RobotDecl { name, .. } = r;
            name.clone()
        })
        .collect();

    for robot in robots {
        let spanda_ast::nodes::RobotDecl::RobotDecl {
            name,
            mission,
            modes,
            topics,
            ..
        } = robot;
        let Some(mission) = mission.as_ref() else {
            continue;
        };
        let MissionDecl::MissionDecl {
            required_approvals, ..
        } = mission;

        for approval in required_approvals {
            let actor_exists = actor_robots.contains(&approval.actor)
                || approval.actor == "Operator"
                || topics.iter().any(|t| {
                    let spanda_ast::nodes::TopicDecl::TopicDecl { message_type, .. } = t;
                    message_type == "Approval"
                });
            let fallback_exists = !modes.is_empty();
            for action in &approval.actions {
                let approval_path = topics.iter().any(|t| {
                    let spanda_ast::nodes::TopicDecl::TopicDecl {
                        name: topic_name,
                        message_type,
                        ..
                    } = t;
                    message_type == "Approval"
                        || topic_name.to_lowercase().contains("approval")
                        || topic_name.to_lowercase().contains(&action.to_lowercase())
                }) || approval.actor == "Operator";
                rows.push(ApprovalVerifyRow {
                    actor: approval.actor.clone(),
                    action: action.clone(),
                    approval_path_exists: approval_path,
                    actor_exists,
                    fallback_exists,
                    status: if approval_path && actor_exists && fallback_exists {
                        "PASS".into()
                    } else {
                        "FAIL".into()
                    },
                });
            }
        }
        let _ = name;
    }

    let compatible = rows.iter().all(|r| r.status == "PASS");
    ApprovalVerifyReport { compatible, rows }
}

/// Verify approvals from source.
pub fn verify_approvals_source(
    source: &str,
) -> Result<ApprovalVerifyReport, spanda_error::SpandaError> {
    let tokens = spanda_lexer::tokenize(source)?;
    let program = spanda_parser::parse(tokens)?;
    Ok(verify_approvals(&program))
}

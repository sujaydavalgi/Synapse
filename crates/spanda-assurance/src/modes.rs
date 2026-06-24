//! Operating mode management analysis.
//!
use crate::types::{ModeKind, OperatingMode};
use spanda_ast::assurance_decl::OperatingModeDecl;
use spanda_ast::nodes::Program;

fn parse_mode_kind(raw: &str) -> ModeKind {
    match raw.to_lowercase().as_str() {
        "degraded" | "degraded_mode" => ModeKind::Degraded,
        "safe" | "safe_mode" => ModeKind::Safe,
        "emergency" | "emergency_mode" => ModeKind::Emergency,
        "recovery" | "recovery_mode" => ModeKind::Recovery,
        _ => ModeKind::Normal,
    }
}

/// Extract operating modes from declarations and robot mode blocks.
pub fn extract_operating_modes(program: &Program) -> Vec<OperatingMode> {
    let Program::Program {
        operating_modes,
        robots,
        ..
    } = program;

    let mut modes: Vec<OperatingMode> = operating_modes
        .iter()
        .map(|decl| {
            let OperatingModeDecl::OperatingModeDecl {
                name, mode_kind, ..
            } = decl;
            OperatingMode {
                name: name.clone(),
                kind: parse_mode_kind(mode_kind),
            }
        })
        .collect();

    for robot in robots {
        let spanda_ast::nodes::RobotDecl::RobotDecl {
            modes: robot_modes, ..
        } = robot;
        for m in robot_modes {
            let spanda_ast::foundations::ModeDecl::ModeDecl { name, .. } = m;
            modes.push(OperatingMode {
                name: name.clone(),
                kind: ModeKind::Normal,
            });
        }
    }

    modes
}

/// Validate mode coverage (safe and degraded modes recommended).
pub fn validate_modes(program: &Program) -> Vec<String> {
    let modes = extract_operating_modes(program);
    let mut issues = Vec::new();
    let has_safe = modes.iter().any(|m| m.kind == ModeKind::Safe);
    let has_degraded = modes.iter().any(|m| m.kind == ModeKind::Degraded);
    if !modes.is_empty() && !has_safe {
        issues.push("No safe mode declared".into());
    }
    if !modes.is_empty() && !has_degraded {
        issues.push("No degraded mode declared".into());
    }
    issues
}

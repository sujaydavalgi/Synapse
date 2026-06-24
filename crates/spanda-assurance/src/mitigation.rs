//! Mitigation planning analysis.
//!
use crate::types::{FallbackMode, MitigationPlan, RecoveryAction, SafeModeTransition};
use spanda_ast::assurance_decl::MitigationDecl;
use spanda_ast::nodes::Program;

/// Mitigation plan report.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MitigationReport {
    pub plans: Vec<MitigationPlan>,
    pub transitions: Vec<SafeModeTransition>,
    pub passed: bool,
}

/// Extract mitigation plans from program declarations.
pub fn extract_mitigations(program: &Program) -> MitigationReport {
    let Program::Program {
        mitigations,
        operating_modes,
        ..
    } = program;
    let plans: Vec<MitigationPlan> = mitigations
        .iter()
        .map(|decl| {
            let MitigationDecl::MitigationDecl { name, branches, .. } = decl;
            let actions: Vec<RecoveryAction> = branches
                .iter()
                .flat_map(|b| {
                    b.actions.iter().map(|a| RecoveryAction {
                        description: a.clone(),
                        condition: Some(b.condition.clone()),
                    })
                })
                .collect();
            let fallback = branches.iter().find_map(|b| {
                b.actions
                    .iter()
                    .find(|a| a.contains("degraded") || a.contains("safe"))
                    .map(|a| FallbackMode { mode: a.clone() })
            });
            MitigationPlan {
                name: name.clone(),
                actions,
                fallback,
            }
        })
        .collect();

    let transitions: Vec<SafeModeTransition> = operating_modes
        .iter()
        .map(|m| {
            let spanda_ast::assurance_decl::OperatingModeDecl::OperatingModeDecl {
                name,
                mode_kind,
                ..
            } = m;
            SafeModeTransition {
                from_mode: "normal".into(),
                to_mode: name.clone(),
                trigger: mode_kind.clone(),
            }
        })
        .collect();

    let passed = !plans.is_empty() || mitigations.is_empty();

    MitigationReport {
        plans,
        transitions,
        passed,
    }
}

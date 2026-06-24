//! Mission planning and execution assurance.
//!
use crate::types::{AnomalySeverity, MissionAbortReason, MissionExecutionState, MissionPlan};
use spanda_ast::assurance_decl::MissionPlanDecl;
use spanda_ast::nodes::Program;
use spanda_readiness::{verify_mission, MissionVerificationReport};

/// Mission assurance report.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MissionAssuranceReport {
    pub plans: Vec<MissionPlan>,
    pub execution: MissionExecutionState,
    pub verification: spanda_readiness::MissionVerificationReport,
    pub abort_reasons: Vec<MissionAbortReason>,
    pub passed: bool,
}

/// Verify mission plans against readiness mission verification.
pub fn verify_mission_assurance(program: &Program) -> MissionAssuranceReport {
    let Program::Program { mission_plans, .. } = program;
    let plans: Vec<MissionPlan> = mission_plans
        .iter()
        .map(|decl| {
            let MissionPlanDecl::MissionPlanDecl {
                name,
                steps,
                constraints,
                ..
            } = decl;
            MissionPlan {
                name: name.clone(),
                steps: steps.iter().map(|s| s.name.clone()).collect(),
                constraints: constraints.iter().map(|c| c.constraint.clone()).collect(),
            }
        })
        .collect();

    let verifications = verify_mission(program, None);
    let passed =
        verifications.iter().all(|v| v.achievable) && !plans.is_empty() || mission_plans.is_empty();

    let verification = verifications
        .into_iter()
        .next()
        .unwrap_or(MissionVerificationReport {
            achievable: true,
            mission_name: None,
            robot: None,
            required_capabilities: Vec::new(),
            hardware_satisfied: true,
            capabilities_satisfied: true,
            connectivity_satisfied: true,
            battery_sufficient: true,
            compute_sufficient: true,
            safety_satisfied: true,
            issues: Vec::new(),
        });

    let execution = MissionExecutionState {
        plan: plans.first().map(|p| p.name.clone()).unwrap_or_default(),
        current_step: plans.first().and_then(|p| p.steps.first().cloned()),
        status: if passed {
            "ready".into()
        } else {
            "blocked".into()
        },
    };

    let abort_reasons: Vec<MissionAbortReason> = verification
        .issues
        .iter()
        .map(|issue| MissionAbortReason {
            reason: issue.clone(),
            severity: AnomalySeverity::High,
        })
        .collect();

    MissionAssuranceReport {
        plans,
        execution,
        verification,
        abort_reasons,
        passed,
    }
}

//! Resilience and recovery policy analysis.
//!
use crate::types::{FaultToleranceStrategy, RecoveryPolicy, RedundancyModel, ResiliencePolicy};
use spanda_ast::assurance_decl::ResiliencePolicyDecl;
use spanda_ast::nodes::Program;
use spanda_readiness::{default_deploy_target, evaluate_readiness, ReadinessOptions};

/// Resilience check report.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ResilienceReport {
    pub policies: Vec<ResiliencePolicy>,
    pub recovery: Vec<RecoveryPolicy>,
    pub redundancy: Vec<RedundancyModel>,
    pub readiness_score: u32,
    pub passed: bool,
}

/// Check resilience policies and readiness integration.
pub fn check_resilience(program: &Program) -> ResilienceReport {
    let Program::Program {
        resilience_policies,
        mitigations,
        ..
    } = program;

    let policies: Vec<ResiliencePolicy> = resilience_policies
        .iter()
        .map(|decl| {
            let ResiliencePolicyDecl::ResiliencePolicyDecl {
                name, strategies, ..
            } = decl;
            ResiliencePolicy {
                name: name.clone(),
                strategies: strategies
                    .iter()
                    .map(|s| FaultToleranceStrategy {
                        name: s.clone(),
                        description: format!("Strategy: {s}"),
                    })
                    .collect(),
            }
        })
        .collect();

    let recovery: Vec<RecoveryPolicy> = mitigations
        .iter()
        .map(|m| {
            let spanda_ast::assurance_decl::MitigationDecl::MitigationDecl {
                name, branches, ..
            } = m;
            RecoveryPolicy {
                name: name.clone(),
                actions: branches.iter().flat_map(|b| b.actions.clone()).collect(),
            }
        })
        .collect();

    let options = ReadinessOptions {
        target: default_deploy_target(program),
        ..Default::default()
    };
    let readiness = evaluate_readiness(program, &options);
    let passed = readiness.mission_ready && !policies.is_empty() || resilience_policies.is_empty();

    ResilienceReport {
        policies,
        recovery,
        redundancy: Vec::new(),
        readiness_score: readiness.score.total,
        passed,
    }
}

//! Prognostics analysis from prognostics declarations.
//!
use crate::types::{
    Confidence, DegradationTrend, FailurePrediction, PrognosticModel, RemainingUsefulLife,
};
use spanda_ast::assurance_decl::PrognosticsDecl;
use spanda_ast::nodes::Program;

/// Prognostics report.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PrognosticsReport {
    pub models: Vec<PrognosticModel>,
    pub rul_predictions: Vec<RemainingUsefulLife>,
    pub failure_predictions: Vec<FailurePrediction>,
    pub trends: Vec<DegradationTrend>,
    pub warnings: Vec<String>,
    pub passed: bool,
}

/// Evaluate prognostics declarations and emit warnings.
pub fn evaluate_prognostics(program: &Program) -> PrognosticsReport {
    let Program::Program { prognostics, .. } = program;
    let mut models = Vec::new();
    let mut rul_predictions = Vec::new();
    let mut warnings = Vec::new();

    for decl in prognostics {
        let PrognosticsDecl::PrognosticsDecl { name, rules, .. } = decl;
        let rule_strs: Vec<String> = rules
            .iter()
            .map(|r| {
                if let Some(th) = &r.threshold {
                    format!("{} {} {}", r.kind, r.target, th)
                } else {
                    format!("{} {}", r.kind, r.target)
                }
            })
            .collect();
        models.push(PrognosticModel {
            name: name.clone(),
            target: rules.first().map(|r| r.target.clone()).unwrap_or_default(),
            rules: rule_strs,
        });

        for rule in rules {
            if rule.kind == "predict" {
                rul_predictions.push(RemainingUsefulLife {
                    component: rule.target.clone(),
                    estimate: rule.threshold.clone().unwrap_or_else(|| "unknown".into()),
                    confidence: Confidence(0.75),
                });
            }
            if rule.kind == "warn_if" {
                if let Some(th) = &rule.threshold {
                    warnings.push(format!(
                        "Prognostics '{name}': warn if {} < {th}",
                        rule.target
                    ));
                }
            }
        }
    }

    let passed = warnings.is_empty() || !prognostics.is_empty();

    PrognosticsReport {
        models,
        rul_predictions,
        failure_predictions: Vec::new(),
        trends: Vec::new(),
        warnings,
        passed,
    }
}

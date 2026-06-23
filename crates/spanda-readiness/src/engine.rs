//! Readiness evaluation engine composing verification subsystems.

use crate::types::{
    ReadinessFactorScore, ReadinessIssue, ReadinessOptions, ReadinessReport, ReadinessScore,
    ReadinessSeverity, ReadinessStatus,
};
use spanda_ast::nodes::Program;
use spanda_capability::{
    capability_traceability, check_minimum_capabilities, evaluate_health_checks,
    infer_robot_capabilities,
};
use spanda_hardware::{verify_program_compatibility, CompatSeverity, VerifyOptions};
use spanda_security::validate::{security_check, SecuritySeverity};

/// Evaluate operational readiness for a parsed program.
pub fn evaluate_readiness(program: &Program, options: &ReadinessOptions) -> ReadinessReport {
    let policy = options.policy.clone().unwrap_or_default();
    let mut issues = Vec::new();
    let mut factor_scores = Vec::new();

    let Program::Program { robots, .. } = program;
    let robot_names: Vec<String> = robots
        .iter()
        .map(|r| {
            let spanda_ast::nodes::RobotDecl::RobotDecl { name, .. } = r;
            name.clone()
        })
        .collect();

    let verify_opts = VerifyOptions {
        target: options.target.clone(),
        all_targets: options.target.is_none(),
        simulate: options.simulate,
        strict_certify: options.strict,
    };
    let hw_report = verify_program_compatibility(program, &verify_opts);
    let hw_score = score_from_compatible(hw_report.compatible, hw_report.errors().count());
    factor_scores.push(factor_row("Hardware", hw_score, policy.weights.hardware));
    for item in hw_report.errors() {
        issues.push(ReadinessIssue {
            factor: "Hardware".into(),
            severity: ReadinessSeverity::High,
            message: item.message.clone(),
            suggested_action: None,
        });
    }
    for item in hw_report.items.iter().filter(|i| {
        i.message.to_lowercase().contains("battery")
            || i.message.to_lowercase().contains("lte")
            || i.message.to_lowercase().contains("signal")
    }) {
        let sev = if item.message.to_lowercase().contains("below") {
            ReadinessSeverity::Medium
        } else {
            ReadinessSeverity::Low
        };
        issues.push(ReadinessIssue {
            factor: if item.message.to_lowercase().contains("battery") {
                "Battery".into()
            } else {
                "Connectivity".into()
            },
            severity: sev,
            message: item.message.clone(),
            suggested_action: None,
        });
    }

    let cap_report = check_minimum_capabilities(program);
    let cap_score = score_from_compatible(cap_report.compatible, cap_report.errors.len());
    factor_scores.push(factor_row(
        "Capabilities",
        cap_score,
        policy.weights.capabilities,
    ));
    for err in &cap_report.errors {
        issues.push(ReadinessIssue {
            factor: "Capabilities".into(),
            severity: ReadinessSeverity::High,
            message: err.clone(),
            suggested_action: None,
        });
    }

    let health_report = evaluate_health_checks(program);
    let health_score = match health_report.overall {
        spanda_capability::HealthStatus::Healthy => 100,
        spanda_capability::HealthStatus::Degraded | spanda_capability::HealthStatus::Warning => 70,
        spanda_capability::HealthStatus::Critical | spanda_capability::HealthStatus::Failed => 30,
        spanda_capability::HealthStatus::Unsafe => 0,
        _ => 85,
    };
    factor_scores.push(factor_row("Health", health_score, policy.weights.health));
    for check in &health_report.checks {
        if check
            .message
            .as_deref()
            .unwrap_or("")
            .contains("calibration")
        {
            issues.push(ReadinessIssue {
                factor: "Health".into(),
                severity: ReadinessSeverity::Low,
                message: check
                    .message
                    .clone()
                    .unwrap_or_else(|| format!("{} check pending", check.name)),
                suggested_action: Some("Schedule calibration".into()),
            });
        }
    }

    let trace = capability_traceability(program);
    let connectivity_score = score_from_errors(trace.errors.len(), trace.warnings.len());
    factor_scores.push(factor_row(
        "Connectivity",
        connectivity_score,
        policy.weights.connectivity,
    ));
    for warn in &trace.warnings {
        if warn.to_lowercase().contains("lte")
            || warn.to_lowercase().contains("connectivity")
            || warn.to_lowercase().contains("signal")
        {
            issues.push(ReadinessIssue {
                factor: "Connectivity".into(),
                severity: ReadinessSeverity::Medium,
                message: warn.clone(),
                suggested_action: Some("Check antenna and network coverage".into()),
            });
        }
    }

    let safety_score = if cap_report.compatible { 95 } else { 40 };
    factor_scores.push(factor_row("Safety", safety_score, policy.weights.safety));

    let battery_score = hw_report
        .items
        .iter()
        .find(|i| i.message.to_lowercase().contains("battery"))
        .map(|i| {
            if i.severity == CompatSeverity::Error {
                30
            } else if i.severity == CompatSeverity::Warning {
                65
            } else {
                90
            }
        })
        .unwrap_or(90);
    factor_scores.push(factor_row("Battery", battery_score, policy.weights.battery));
    if battery_score < 80 {
        issues.push(ReadinessIssue {
            factor: "Battery".into(),
            severity: ReadinessSeverity::Medium,
            message: "Battery below recommended threshold".into(),
            suggested_action: Some("Charge or swap battery before mission".into()),
        });
    }

    factor_scores.push(factor_row("Storage", 90, policy.weights.storage));
    factor_scores.push(factor_row("Compute", 88, policy.weights.compute));

    let pkg_missing = trace
        .capability_rows
        .iter()
        .filter(|r| r.package.is_empty() && r.status == "FAIL")
        .count();
    let pkg_score = score_from_compatible(pkg_missing == 0, pkg_missing);
    factor_scores.push(factor_row("Packages", pkg_score, policy.weights.packages));

    let provider_missing = trace
        .capability_rows
        .iter()
        .filter(|r| r.provider.is_empty() && !r.capability.is_empty())
        .count();
    let provider_score = score_from_compatible(provider_missing == 0, provider_missing);
    factor_scores.push(factor_row(
        "Providers",
        provider_score,
        policy.weights.providers,
    ));

    let mission_score = infer_robot_capabilities(program)
        .iter()
        .map(|r| {
            if r.rows.iter().any(|row| row.status == "FAIL") {
                50
            } else if r.rows.iter().any(|row| row.status == "PARTIAL") {
                75
            } else {
                100
            }
        })
        .min()
        .unwrap_or(100);
    factor_scores.push(factor_row(
        "Mission Requirements",
        mission_score,
        policy.weights.mission,
    ));

    let total = compute_weighted_total(&factor_scores);
    let has_critical = issues.iter().any(|i| {
        matches!(
            i.severity,
            ReadinessSeverity::Critical | ReadinessSeverity::High
        )
    });
    let mission_ready = total >= policy.minimum_score && hw_report.compatible && !has_critical;
    let status = if mission_ready {
        if issues.iter().any(|i| {
            matches!(
                i.severity,
                ReadinessSeverity::Medium | ReadinessSeverity::Low
            )
        }) {
            ReadinessStatus::Degraded
        } else {
            ReadinessStatus::Ready
        }
    } else if total >= policy.minimum_score.saturating_sub(15) {
        ReadinessStatus::Degraded
    } else {
        ReadinessStatus::NotReady
    };

    ReadinessReport {
        status,
        mission_ready,
        score: ReadinessScore {
            total,
            maximum: 100,
            factors: factor_scores,
        },
        issues,
        policy,
        target: options.target.clone(),
        robots: robot_names,
    }
}

/// Evaluate readiness from source text.
pub fn evaluate_readiness_source(
    source: &str,
    options: &ReadinessOptions,
) -> Result<ReadinessReport, spanda_error::SpandaError> {
    let tokens = spanda_lexer::tokenize(source)?;
    let program = spanda_parser::parse(tokens)?;
    Ok(evaluate_readiness(&program, options))
}

fn factor_row(factor: &str, score: u32, weight: u32) -> ReadinessFactorScore {
    ReadinessFactorScore {
        factor: factor.into(),
        score,
        weight,
        weighted: (score as f64) * (weight as f64) / 100.0,
    }
}

fn compute_weighted_total(factors: &[ReadinessFactorScore]) -> u32 {
    let weight_sum: u32 = factors.iter().map(|f| f.weight).sum();
    if weight_sum == 0 {
        return 0;
    }
    let weighted: f64 = factors.iter().map(|f| f.weighted).sum();
    (weighted * 100.0 / weight_sum as f64).round() as u32
}

fn score_from_compatible(compatible: bool, error_count: usize) -> u32 {
    if compatible && error_count == 0 {
        100
    } else if compatible {
        85
    } else if error_count == 1 {
        60
    } else {
        30
    }
}

fn score_from_errors(errors: usize, warnings: usize) -> u32 {
    let base = 100u32;
    base.saturating_sub((errors * 25) as u32)
        .saturating_sub((warnings * 5) as u32)
        .max(20)
}

/// Add security findings as readiness issues when evaluating with security context.
pub fn append_security_issues(source: &str, issues: &mut Vec<ReadinessIssue>) {
    if let Ok(report) = security_check(source) {
        for finding in report.findings {
            let sev = match finding.severity {
                SecuritySeverity::Error => ReadinessSeverity::High,
                SecuritySeverity::Warning => ReadinessSeverity::Medium,
                SecuritySeverity::Info => ReadinessSeverity::Low,
            };
            issues.push(ReadinessIssue {
                factor: "Safety".into(),
                severity: sev,
                message: finding.message,
                suggested_action: None,
            });
        }
    }
}

//! Readiness evaluation engine composing verification subsystems.

use crate::runtime::{build_runtime_context_with_config, RuntimeReadinessContext};
use crate::types::{
    ReadinessFactorScore, ReadinessIssue, ReadinessOptions, ReadinessReport, ReadinessScore,
    ReadinessSeverity, ReadinessStatus,
};
use spanda_ast::nodes::Program;
use spanda_capability::{
    apply_fleet_health_checks, capability_traceability, check_minimum_capabilities,
    evaluate_health_checks, evaluate_runtime_health, infer_robot_capabilities, HealthStatus,
};
use spanda_hardware::{verify_program_compatibility, CompatSeverity, VerifyOptions};
use spanda_security::validate::{security_check, SecuritySeverity};

/// Evaluate operational readiness for a parsed program.
pub fn evaluate_readiness(program: &Program, options: &ReadinessOptions) -> ReadinessReport {
    // Description:
    //     Evaluate readiness.
    //
    // Inputs:
    //     progra: &Program
    //         Caller-supplied progra.
    //     options: &ReadinessOptions
    //         Caller-supplied options.
    //
    // Outputs:
    //     result: ReadinessReport
    //         Return value from `evaluate_readiness`.
    //
    // Example:

    //     let result = spanda_readiness::engine::evaluate_readiness(progra, options);

    evaluate_readiness_with_runtime(program, options, None)
}

/// Evaluate readiness with optional live runtime health context.
pub fn evaluate_readiness_with_runtime(
    program: &Program,
    options: &ReadinessOptions,
    runtime: Option<&RuntimeReadinessContext>,
) -> ReadinessReport {
    // Description:
    //     Evaluate readiness with runtime.
    //
    // Inputs:
    //     progra: &Program
    //         Caller-supplied progra.
    //     options: &ReadinessOptions
    //         Caller-supplied options.
    //     runtime: Option<&RuntimeReadinessContext>
    //         Caller-supplied runtime.
    //
    // Outputs:
    //     result: ReadinessReport
    //         Return value from `evaluate_readiness_with_runtime`.
    //
    // Example:

    //     let result = spanda_readiness::engine::evaluate_readiness_with_runtime(progra, options, runtime);

    let policy = options
        .policy
        .clone()
        .or_else(|| {
            options
                .system_config
                .as_ref()
                .and_then(|cfg| crate::config::policy_from_system_config(cfg))
        })
        .unwrap_or_default();
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
        target: options.target.clone().or_else(|| {
            options
                .system_config
                .as_ref()
                .and_then(|cfg| spanda_config::default_verify_target(cfg))
        }),
        all_targets: options.target.is_none()
            && options
                .system_config
                .as_ref()
                .and_then(|cfg| spanda_config::default_verify_target(cfg))
                .is_none(),
        simulate: options.simulate,
        strict_certify: options.strict,
    };
    let hw_report = if let Some(ref cfg) = options.system_config {
        spanda_config::verify_with_system_config(program, Some(cfg), verify_opts)
    } else {
        verify_program_compatibility(program, &verify_opts)
    };
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

    if let Some(ref cfg) = options.system_config {
        for (severity, message) in crate::config::config_robot_alignment_issues(cfg, &robot_names) {
            issues.push(ReadinessIssue {
                factor: "Configuration".into(),
                severity,
                message,
                suggested_action: Some("Add robot to spanda.devices.toml fleet config".into()),
            });
        }
        for (severity, message) in crate::config::config_device_identity_issues(cfg) {
            issues.push(ReadinessIssue {
                factor: "Connectivity".into(),
                severity,
                message,
                suggested_action: Some("Update device identity in spanda.toml fragments".into()),
            });
        }
        if let Some(ref baseline) = options.baseline_config {
            for (severity, message) in
                crate::config::config_drift_issues(baseline.as_ref(), cfg, Some(program))
            {
                issues.push(ReadinessIssue {
                    factor: "Configuration".into(),
                    severity,
                    message,
                    suggested_action: Some(
                        "Reconcile live config with approved baseline or update baseline".into(),
                    ),
                });
            }
        }
        if !cfg.validation.passed {
            for finding in cfg
                .validation
                .findings
                .iter()
                .filter(|f| matches!(f.severity, spanda_config::ValidationSeverity::Error))
            {
                issues.push(ReadinessIssue {
                    factor: "Configuration".into(),
                    severity: ReadinessSeverity::High,
                    message: finding.message.clone(),
                    suggested_action: Some("Run spanda config validate".into()),
                });
            }
        }
    }

    for (expected, actual) in &options.agent_drift {
        for (factor, severity, message) in crate::config::agent_drift_issues(expected, actual) {
            issues.push(ReadinessIssue {
                factor,
                severity,
                message,
                suggested_action: Some(
                    "Run spanda drift --agent or verify deploy agent attestation status".into(),
                ),
            });
        }
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

    let health_report = if let Some(ctx) = runtime {
        evaluate_runtime_health(&ctx.faults, &ctx.events, program)
    } else {
        evaluate_health_checks(program)
    };
    let mut health_report = health_report;
    if runtime.is_some() {
        let Program::Program { fleets, .. } = program;
        if !fleets.is_empty() {
            let mut fleet_registry = spanda_runtime::robotics::FleetRegistry::default();
            for fleet in fleets {
                let spanda_ast::robotics_decl::FleetDecl::FleetDecl { name, members, .. } = fleet;
                fleet_registry.register(name, members.clone());
            }
            let faults = runtime.map(|c| c.faults.as_slice()).unwrap_or(&[]);
            apply_fleet_health_checks(&mut health_report, program, &fleet_registry, faults);
        }
    }
    let health_score = match health_report.overall {
        HealthStatus::Healthy => 100,
        HealthStatus::Degraded | HealthStatus::Warning => 70,
        HealthStatus::Critical | HealthStatus::Failed => 30,
        HealthStatus::Unsafe => 0,
        _ => 85,
    };
    factor_scores.push(factor_row("Health", health_score, policy.weights.health));
    for check in &health_report.checks {
        if matches!(
            check.status,
            HealthStatus::Degraded
                | HealthStatus::Warning
                | HealthStatus::Critical
                | HealthStatus::Failed
                | HealthStatus::Unsafe
                | HealthStatus::Offline
        ) {
            issues.push(ReadinessIssue {
                factor: "Health".into(),
                severity: match check.status {
                    HealthStatus::Critical | HealthStatus::Failed | HealthStatus::Unsafe => {
                        ReadinessSeverity::High
                    }
                    HealthStatus::Degraded | HealthStatus::Warning | HealthStatus::Offline => {
                        ReadinessSeverity::Medium
                    }
                    _ => ReadinessSeverity::Low,
                },
                message: check.message.clone().unwrap_or_else(|| {
                    format!("{} {} {}", check.metric, check.operator, check.threshold)
                }),
                suggested_action: Some("Review health policy reactions".into()),
            });
        } else if check
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

    let Program::Program {
        assurance_cases,
        knowledge_models,
        state_estimators,
        anomaly_detectors,
        anomaly_handlers,
        mitigations,
        ..
    } = program;
    let mut assurance_score = 100u32;
    if !assurance_cases.is_empty() {
        if assurance_cases.iter().all(|c| {
            let spanda_ast::assurance_decl::AssuranceCaseDecl::AssuranceCaseDecl {
                evidence, ..
            } = c;
            !evidence.is_empty()
        }) {
            assurance_score = assurance_score.saturating_sub(0);
        } else {
            assurance_score = assurance_score.saturating_sub(40);
            issues.push(ReadinessIssue {
                factor: "Assurance".into(),
                severity: ReadinessSeverity::High,
                message: "Assurance case missing evidence links".into(),
                suggested_action: Some("Add evidence to assurance_case declarations".into()),
            });
        }
    }
    if !knowledge_models.is_empty()
        && knowledge_models.iter().any(|m| {
            let spanda_ast::assurance_decl::KnowledgeModelDecl::KnowledgeModelDecl {
                components,
                ..
            } = m;
            components.is_empty()
        })
    {
        assurance_score = assurance_score.saturating_sub(20);
        issues.push(ReadinessIssue {
            factor: "Assurance".into(),
            severity: ReadinessSeverity::Medium,
            message: "Knowledge model has empty components".into(),
            suggested_action: None,
        });
    }
    for est in state_estimators {
        let spanda_ast::assurance_decl::StateEstimatorDecl::StateEstimatorDecl {
            name, inputs, ..
        } = est;
        if inputs.is_empty() {
            assurance_score = assurance_score.saturating_sub(15);
            issues.push(ReadinessIssue {
                factor: "Assurance".into(),
                severity: ReadinessSeverity::Medium,
                message: format!("State estimator '{name}' has no inputs"),
                suggested_action: Some("Add sensor inputs to state_estimator".into()),
            });
        }
    }
    if !anomaly_detectors.is_empty() {
        let handler_names: std::collections::HashSet<_> = anomaly_handlers
            .iter()
            .map(|h| {
                let spanda_ast::assurance_decl::AnomalyHandlerDecl::AnomalyHandlerDecl {
                    detector,
                    ..
                } = h;
                detector.clone()
            })
            .collect();
        for det in anomaly_detectors {
            let spanda_ast::assurance_decl::AnomalyDetectorDecl::AnomalyDetectorDecl {
                name, ..
            } = det;
            if !handler_names.contains(name) {
                assurance_score = assurance_score.saturating_sub(10);
                issues.push(ReadinessIssue {
                    factor: "Assurance".into(),
                    severity: ReadinessSeverity::Low,
                    message: format!("Anomaly detector '{name}' has no on anomaly handler"),
                    suggested_action: Some("Add on anomaly handler".into()),
                });
            }
        }
    }
    if mitigations.is_empty() && !anomaly_detectors.is_empty() {
        assurance_score = assurance_score.saturating_sub(10);
    }
    factor_scores.push(factor_row(
        "Assurance",
        assurance_score,
        policy.weights.assurance,
    ));

    let mut human_mission_ok = true;
    if let Some(ref cfg) = options.system_config {
        if cfg.human_registry.has_operators() {
            let human_report = crate::human::evaluate_human_collaboration(cfg.as_ref(), program);
            factor_scores.push(factor_row(
                "Operator",
                human_report.total_score,
                policy.weights.mission.min(15),
            ));
            issues.extend(human_report.issues);
            human_mission_ok = human_report.mission_ready;
        }
    }

    let total = compute_weighted_total(&factor_scores);
    let has_critical = issues.iter().any(|i| {
        matches!(
            i.severity,
            ReadinessSeverity::Critical | ReadinessSeverity::High
        )
    });
    let mission_ready =
        total >= policy.minimum_score && hw_report.compatible && !has_critical && human_mission_ok;
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
    // Description:
    //     Evaluate readiness source.
    //
    // Inputs:
    //     source: &str
    //         Caller-supplied source.
    //     options: &ReadinessOptions
    //         Caller-supplied options.
    //
    // Outputs:
    //     result: Result<ReadinessReport, spanda_error::SpandaError>
    //         Return value from `evaluate_readiness_source`.
    //
    // Example:

    //     let result = spanda_readiness::engine::evaluate_readiness_source(source, options);

    let tokens = spanda_lexer::tokenize(source)?;
    let program = spanda_parser::parse(tokens)?;
    let runtime = options.include_runtime.then(|| {
        build_runtime_context_with_config(
            &program,
            options.inject_health_faults,
            options.system_config.as_deref(),
        )
    });
    Ok(evaluate_readiness_with_runtime(
        &program,
        options,
        runtime.as_ref(),
    ))
}

fn factor_row(factor: &str, score: u32, weight: u32) -> ReadinessFactorScore {
    // Description:
    //     Factor row.
    //
    // Inputs:
    //     factor: &str
    //         Caller-supplied factor.
    //     score: u32
    //         Caller-supplied score.
    //     weigh: u32
    //         Caller-supplied weigh.
    //
    // Outputs:
    //     result: ReadinessFactorScore
    //         Return value from `factor_row`.
    //
    // Example:

    //     let result = spanda_readiness::engine::factor_row(factor, score, weigh);

    ReadinessFactorScore {
        factor: factor.into(),
        score,
        weight,
        weighted: (score as f64) * (weight as f64) / 100.0,
    }
}

fn compute_weighted_total(factors: &[ReadinessFactorScore]) -> u32 {
    // Description:
    //     Compute weighted total.
    //
    // Inputs:
    //     factors: &[ReadinessFactorScore]
    //         Caller-supplied factors.
    //
    // Outputs:
    //     result: u32
    //         Return value from `compute_weighted_total`.
    //
    // Example:

    //     let result = spanda_readiness::engine::compute_weighted_total(factors);

    let weight_sum: u32 = factors.iter().map(|f| f.weight).sum();
    if weight_sum == 0 {
        return 0;
    }
    let weighted: f64 = factors.iter().map(|f| f.weighted).sum();
    (weighted * 100.0 / weight_sum as f64).round() as u32
}

fn score_from_compatible(compatible: bool, error_count: usize) -> u32 {
    // Description:
    //     Score from compatible.
    //
    // Inputs:
    //     compatible: bool
    //         Caller-supplied compatible.
    //     error_coun: usize
    //         Caller-supplied error coun.
    //
    // Outputs:
    //     result: u32
    //         Return value from `score_from_compatible`.
    //
    // Example:

    //     let result = spanda_readiness::engine::score_from_compatible(compatible, error_coun);

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
    // Description:
    //     Score from errors.
    //
    // Inputs:
    //     errors: usize
    //         Caller-supplied errors.
    //     warnings: usize
    //         Caller-supplied warnings.
    //
    // Outputs:
    //     result: u32
    //         Return value from `score_from_errors`.
    //
    // Example:

    //     let result = spanda_readiness::engine::score_from_errors(errors, warnings);

    let base = 100u32;
    base.saturating_sub((errors * 25) as u32)
        .saturating_sub((warnings * 5) as u32)
        .max(20)
}

/// Add security findings as readiness issues when evaluating with security context.
pub fn append_security_issues(source: &str, issues: &mut Vec<ReadinessIssue>) {
    // Description:
    //     Append security issues.
    //
    // Inputs:
    //     source: &str
    //         Caller-supplied source.
    //     issues: &mut Vec<ReadinessIssue>
    //         Caller-supplied issues.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_readiness::engine::append_security_issues(source, issues);

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
